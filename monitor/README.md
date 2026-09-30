# pressure-monitor

Samples device resource pressure for resource-pressure-aware 3DGS rendering. The adaptive policy uses these readings. It is not network-adaptive or visibility-adaptive.

A background thread reads OS signals at a fixed interval (100 ms by default). The renderer reports per-frame values through a `MonitorHandle`. Each sample can be appended to a CSV file.

## Usage

Standalone check without Brush:

```
cd monitor
cargo run --example probe          # one line per second, Ctrl-C to stop
cargo run --example probe 10       # stop after 10 s
cargo test
```

Inside Brush, `brush-app` starts the monitor at launch. Environment variables:

| Variable | Effect |
|---|---|
| `ADAPTIVE_LOG=logs/run.csv` | Write every sample to this CSV (the directory is created if needed). Without it, nothing is written. |
| `ADAPTIVE_INTERVAL_MS=100` | Sampling interval. |
| `ADAPTIVE_CONTINUOUS=1` | Re-render splats every UI frame. By default Brush only re-renders when the camera moves, so a still camera would log no render times. |

```
cd external/brush
ADAPTIVE_LOG=../../logs/run.csv ADAPTIVE_CONTINUOUS=1 \
  cargo run --release -- --with-viewer ../../scenes/bonsai/point_cloud.ply
```

## CSV columns

An empty cell means the platform cannot provide that value. The monitor never fills in a guess.

| Column | Meaning | macOS source |
|---|---|---|
| `t_ms` | Milliseconds since the monitor started | |
| `gpu_alloc_bytes` | Bytes this process has allocated on the GPU | `MTLDevice.currentAllocatedSize` |
| `gpu_budget_bytes` | GPU working-set budget | `MTLDevice.recommendedMaxWorkingSetSize` |
| `gpu_headroom_bytes` | `budget - alloc` (see the definition below) | |
| `sys_mem_pressure` | Kernel pressure level: 1 normal, 2 warning, 4 critical | `sysctl kern.memorystatus_vm_pressure_level` |
| `sys_mem_free_pct` | System memory the kernel considers available, in % | `sysctl kern.memorystatus_level` |
| `thermal_state` | 0 nominal, 1 fair, 2 serious, 3 critical | `NSProcessInfo.thermalState` |
| `on_battery` | 1 when running on battery | IOKit `IOPSGetProvidingPowerSourceType` |
| `battery_pct` | Battery charge in % | IOKit power source description |
| `low_power_mode` | 1 when Low Power Mode is on | `NSProcessInfo.isLowPowerModeEnabled` |
| `render_ms` | Mean splat render time (render plus GPU→CPU readback) since the previous sample | Brush `splat_backbuffer.rs` |
| `render_count` | Number of splat renders since the previous sample | Brush |
| `ui_frame_ms` | Mean UI frame time since the previous sample | Brush `scene.rs` |
| `num_splats` | Gaussians in the last rendered frame | Brush |
| `burn_bytes_in_use`, `burn_bytes_reserved` | Burn allocator pool usage (the renderer's own tensors) | `Device::memory_pool_usage` |

## GPU memory headroom on Apple Silicon

Apple Silicon has unified memory, so the GPU has no dedicated VRAM. We define:

GPU memory headroom = `recommendedMaxWorkingSetSize - currentAllocatedSize`

This is how much more this process can allocate on the GPU before Metal warns that performance will degrade. It counts only this process's GPU allocations. Other apps using shared memory don't change it, so the system-wide signals `sys_mem_free_pct` and `sys_mem_pressure` are logged next to it.

`MTLCreateSystemDefaultDevice()` returns the same `MTLDevice` that wgpu/Burn render with, so its `currentAllocatedSize` includes Brush's allocations. Checked on 2026-09-30 on an M5 with 16 GB: loading bonsai (1.24M Gaussians) raised `gpu_alloc_bytes` from 1 MB to 588 MB, of which Burn reserved 457 MB. Later growth in Burn's pool showed up in both columns.

## Measured on the M5 (2026-09-30)

- `gpu_budget_bytes` = 12.71 GB on the 16 GB machine.
- bonsai at about 1450x1200: `render_ms` about 3.4 ms, UI frame 16.7 ms (vsync).
- With `ADAPTIVE_CONTINUOUS=1` Brush completes only about 35-40 renders/s against 60 UI frames/s. The render request and readback are pipelined through an `AsyncMap`, so renders cap below the display rate. Keep this in mind before using `render_count` as an FPS number.
- A 6 GB allocation of `np.ones` lowered `sys_mem_free_pct` by only about 5-7% and did not change `sys_mem_pressure`. macOS compresses memory, so a memory-pressure generator has to write random (incompressible) data.

## Platforms

- macOS: implemented (`src/macos.rs`).
- Windows: planned, using DXGI `QueryVideoMemoryInfo` for memory. Until then `src/fallback.rs` returns empty values for every OS signal, and the Brush-reported columns still work.
