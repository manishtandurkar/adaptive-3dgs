//! Resource-pressure monitor for adaptive 3DGS rendering.
//!
//! A background thread samples device state (GPU memory headroom, system memory
//! pressure, thermal state, power state) at a fixed interval. The renderer reports
//! per-frame values (render time, UI frame time, splat count, allocator usage)
//! through a [`MonitorHandle`]. Each sample can be appended to a CSV log.
//!
//! Every signal is an `Option`: a value the platform cannot provide is logged as an
//! empty cell, never as a guessed number.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as backend;

#[cfg(not(target_os = "macos"))]
mod fallback;
#[cfg(not(target_os = "macos"))]
use fallback as backend;

/// Device state read from OS APIs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OsReadings {
    /// Bytes currently allocated by this process on the GPU device.
    pub gpu_alloc_bytes: Option<u64>,
    /// Bytes the GPU can use before performance degrades (macOS: recommendedMaxWorkingSetSize).
    pub gpu_budget_bytes: Option<u64>,
    /// System memory pressure level: 1 normal, 2 warning, 4 critical (macOS kernel levels).
    pub sys_mem_pressure: Option<u32>,
    /// Percentage of system memory the kernel considers available (macOS: kern.memorystatus_level).
    pub sys_mem_free_pct: Option<u32>,
    /// Thermal state: 0 nominal, 1 fair, 2 serious, 3 critical.
    pub thermal_state: Option<u32>,
    pub on_battery: Option<bool>,
    pub battery_pct: Option<u32>,
    pub low_power_mode: Option<bool>,
}

impl OsReadings {
    /// Budget minus current allocation. Negative means over budget.
    pub fn gpu_headroom_bytes(&self) -> Option<i64> {
        Some(self.gpu_budget_bytes? as i64 - self.gpu_alloc_bytes? as i64)
    }
}

/// One row of the monitor log.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sample {
    /// Milliseconds since the monitor started.
    pub t_ms: f64,
    pub os: OsReadings,
    /// Mean splat render time (render + readback) over renders since the previous sample.
    pub render_ms: Option<f64>,
    /// Number of splat renders since the previous sample.
    pub render_count: u32,
    /// Mean UI frame time over frames since the previous sample.
    pub ui_frame_ms: Option<f64>,
    pub num_splats: Option<u32>,
    pub burn_bytes_in_use: Option<u64>,
    pub burn_bytes_reserved: Option<u64>,
}

pub const CSV_HEADER: &str = "t_ms,gpu_alloc_bytes,gpu_budget_bytes,gpu_headroom_bytes,\
sys_mem_pressure,sys_mem_free_pct,thermal_state,on_battery,battery_pct,low_power_mode,\
render_ms,render_count,ui_frame_ms,num_splats,burn_bytes_in_use,burn_bytes_reserved";

fn cell<T: ToString>(v: Option<T>) -> String {
    v.map_or_else(String::new, |v| v.to_string())
}

impl Sample {
    pub fn csv_row(&self) -> String {
        let o = &self.os;
        [
            format!("{:.1}", self.t_ms),
            cell(o.gpu_alloc_bytes),
            cell(o.gpu_budget_bytes),
            cell(o.gpu_headroom_bytes()),
            cell(o.sys_mem_pressure),
            cell(o.sys_mem_free_pct),
            cell(o.thermal_state),
            cell(o.on_battery.map(u8::from)),
            cell(o.battery_pct),
            cell(o.low_power_mode.map(u8::from)),
            cell(self.render_ms.map(|v| format!("{v:.3}"))),
            self.render_count.to_string(),
            cell(self.ui_frame_ms.map(|v| format!("{v:.3}"))),
            cell(self.num_splats),
            cell(self.burn_bytes_in_use),
            cell(self.burn_bytes_reserved),
        ]
        .join(",")
    }
}

#[derive(Default)]
struct Accum {
    sum: f64,
    count: u32,
}

impl Accum {
    fn push(&mut self, v: f64) {
        self.sum += v;
        self.count += 1;
    }

    fn take(&mut self) -> (Option<f64>, u32) {
        let out = (self.count > 0).then(|| self.sum / self.count as f64);
        let count = self.count;
        *self = Self::default();
        (out, count)
    }
}

#[derive(Default)]
struct AppState {
    render: Accum,
    ui_frame: Accum,
    num_splats: Option<u32>,
    burn_bytes_in_use: Option<u64>,
    burn_bytes_reserved: Option<u64>,
}

struct Shared {
    app: Mutex<AppState>,
    latest: Mutex<Sample>,
    stop: AtomicBool,
}

/// Cheap, cloneable handle the renderer uses to report per-frame values.
#[derive(Clone)]
pub struct MonitorHandle {
    shared: Arc<Shared>,
}

impl MonitorHandle {
    pub fn report_render_ms(&self, ms: f64) {
        self.shared.app.lock().unwrap().render.push(ms);
    }

    pub fn report_ui_frame_ms(&self, ms: f64) {
        self.shared.app.lock().unwrap().ui_frame.push(ms);
    }

    pub fn set_num_splats(&self, n: u32) {
        self.shared.app.lock().unwrap().num_splats = Some(n);
    }

    pub fn set_burn_memory(&self, bytes_in_use: u64, bytes_reserved: u64) {
        let mut app = self.shared.app.lock().unwrap();
        app.burn_bytes_in_use = Some(bytes_in_use);
        app.burn_bytes_reserved = Some(bytes_reserved);
    }

    /// Most recent sample taken by the background thread.
    pub fn latest(&self) -> Sample {
        *self.shared.latest.lock().unwrap()
    }
}

/// Owns the sampling thread. Dropping it stops the thread and flushes the log.
pub struct Monitor {
    handle: MonitorHandle,
    thread: Option<JoinHandle<()>>,
}

impl Monitor {
    /// Starts sampling every `interval`. If `log_path` is set, each sample is appended as a CSV row.
    pub fn start(interval: Duration, log_path: Option<&Path>) -> io::Result<Self> {
        let mut log = match log_path {
            Some(path) => {
                if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
                    std::fs::create_dir_all(dir)?;
                }
                let mut w = BufWriter::new(File::create(path)?);
                writeln!(w, "{CSV_HEADER}")?;
                Some(w)
            }
            None => None,
        };

        let shared = Arc::new(Shared {
            app: Mutex::new(AppState::default()),
            latest: Mutex::new(Sample::default()),
            stop: AtomicBool::new(false),
        });
        let thread_shared = shared.clone();

        let thread = std::thread::Builder::new()
            .name("pressure-monitor".into())
            .spawn(move || {
                let start = Instant::now();
                let mut reader = backend::Reader::new();
                let mut next = start;
                while !thread_shared.stop.load(Ordering::Relaxed) {
                    let os = reader.read();
                    let sample = {
                        let mut app = thread_shared.app.lock().unwrap();
                        let (render_ms, render_count) = app.render.take();
                        let (ui_frame_ms, _) = app.ui_frame.take();
                        Sample {
                            t_ms: start.elapsed().as_secs_f64() * 1000.0,
                            os,
                            render_ms,
                            render_count,
                            ui_frame_ms,
                            num_splats: app.num_splats,
                            burn_bytes_in_use: app.burn_bytes_in_use,
                            burn_bytes_reserved: app.burn_bytes_reserved,
                        }
                    };
                    *thread_shared.latest.lock().unwrap() = sample;
                    if let Some(w) = log.as_mut() {
                        let _ = writeln!(w, "{}", sample.csv_row()).and_then(|_| w.flush());
                    }
                    next += interval;
                    let now = Instant::now();
                    if next > now {
                        std::thread::sleep(next - now);
                    } else {
                        next = now;
                    }
                }
            })?;

        Ok(Self {
            handle: MonitorHandle { shared },
            thread: Some(thread),
        })
    }

    /// Starts a monitor configured from the environment:
    /// `ADAPTIVE_LOG` (CSV path, optional) and `ADAPTIVE_INTERVAL_MS` (default 100).
    pub fn from_env() -> io::Result<Self> {
        let log_path = std::env::var_os("ADAPTIVE_LOG").map(PathBuf::from);
        let interval_ms = std::env::var("ADAPTIVE_INTERVAL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);
        Self::start(Duration::from_millis(interval_ms), log_path.as_deref())
    }

    pub fn handle(&self) -> MonitorHandle {
        self.handle.clone()
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.handle.shared.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

static GLOBAL: OnceLock<Monitor> = OnceLock::new();

/// Starts the process-wide monitor from the environment (see [`Monitor::from_env`]).
/// Calling it again is a no-op.
pub fn init_global_from_env() -> io::Result<()> {
    if GLOBAL.get().is_none() {
        let monitor = Monitor::from_env()?;
        let _ = GLOBAL.set(monitor);
    }
    Ok(())
}

/// Handle to the process-wide monitor, if [`init_global_from_env`] was called.
pub fn global() -> Option<&'static MonitorHandle> {
    GLOBAL.get().map(|m| &m.handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headroom_is_budget_minus_alloc() {
        let os = OsReadings {
            gpu_alloc_bytes: Some(3),
            gpu_budget_bytes: Some(10),
            ..Default::default()
        };
        assert_eq!(os.gpu_headroom_bytes(), Some(7));
        assert_eq!(OsReadings::default().gpu_headroom_bytes(), None);
    }

    #[test]
    fn csv_row_matches_header_width() {
        let cols = CSV_HEADER.split(',').count();
        assert_eq!(Sample::default().csv_row().split(',').count(), cols);
    }

    #[test]
    fn accum_averages_and_resets() {
        let mut a = Accum::default();
        a.push(2.0);
        a.push(4.0);
        assert_eq!(a.take(), (Some(3.0), 2));
        assert_eq!(a.take(), (None, 0));
    }

    #[test]
    fn reported_values_reach_samples() {
        let m = Monitor::start(Duration::from_millis(10), None).unwrap();
        let h = m.handle();
        h.set_num_splats(42);
        h.report_render_ms(5.0);
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(h.latest().num_splats, Some(42));
    }
}
