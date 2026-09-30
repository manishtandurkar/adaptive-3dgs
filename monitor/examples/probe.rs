//! Prints one monitor sample per second. Run: `cargo run --example probe`
//! Pass a number of seconds as the first argument to stop early (default: run forever).

use std::time::Duration;

use pressure_monitor::Monitor;

fn mb(v: Option<u64>) -> String {
    v.map_or("-".into(), |b| format!("{:.0} MB", b as f64 / 1e6))
}

fn main() -> std::io::Result<()> {
    let secs: Option<u64> = std::env::args().nth(1).and_then(|s| s.parse().ok());
    let monitor = Monitor::from_env()?;
    let h = monitor.handle();
    let mut elapsed = 0;
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let s = h.latest();
        let o = s.os;
        println!(
            "t={:>6.1}s  gpu alloc {:>9}  budget {:>9}  headroom {:>9}  mem_pressure {:?} free {:?}%  thermal {:?}  battery {:?} {:?}%  low_power {:?}",
            s.t_ms / 1000.0,
            mb(o.gpu_alloc_bytes),
            mb(o.gpu_budget_bytes),
            o.gpu_headroom_bytes()
                .map_or("-".into(), |b| format!("{:.0} MB", b as f64 / 1e6)),
            o.sys_mem_pressure,
            o.sys_mem_free_pct,
            o.thermal_state,
            o.on_battery,
            o.battery_pct,
            o.low_power_mode,
        );
        elapsed += 1;
        if secs.is_some_and(|n| elapsed >= n) {
            return Ok(());
        }
    }
}
