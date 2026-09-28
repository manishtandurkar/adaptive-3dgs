# Project: On-Device Resource-Pressure-Aware Adaptive Rendering for 3D Gaussian Splatting

## Overview

This is a final-year (7th semester) research project, targeted at a rigorous, publishable
contribution — not a consumer product. The deliverable is a working system plus a paper
with a defensible, novel claim. Team of 3-4 students, 2-2.5 month timeline, worked on
alongside 5-6 other coursework projects.

## The problem

3D Gaussian Splatting (3DGS) renders photorealistic 3D scenes from millions of small
Gaussian primitives. Full-detail scenes require several GB of GPU memory, which does not
fit on resource-constrained devices (laptops, phones, VR headsets, older GPUs).

## What already exists (do not duplicate this)

- **Compression research** (LightGaussian, Compact 3DGS, SA-3DGS, CAT-3DGS, etc.) — shrinks
  scene storage size, offline, once, before the scene runs. Heavily published, ML/CUDA-heavy.
  Not our angle — we consume an existing compressed scene, we do not invent compression.
- **Network/visibility-adaptive streaming** (Voyager, LapisGS, CAGS, SplatStream) — adapts
  detail live, but reacts to bandwidth or camera-visibility, not on-device resource state.
- **Distance/view-based LOD** (Octree-GS, Hierarchical 3DGS, LODGE, Atlas) — picks detail
  from camera distance or projected size, not device state.
- **Fixed-budget / device-tier methods** (FLoD, MoQSplat, GaussAnything, Milef et al. CLoD)
  — the closest competitors. They cap Gaussian count or VRAM at a developer-set number
  (FLoD: level chosen once per device; MoQSplat: fixed M_max; GaussAnything: fixed budget,
  with frame-time slack pacing only update work). None reads live device state.
- **Outside 3DGS**: Unity Adaptive Performance changes LOD live from device thermal level;
  US patent 10,491,711 picks VR video quality from device temperature. So thermal-aware
  adaptive rendering in general is not new.

Full comparison with sources: `research.md` (literature check, 2026-09-28).

## The gap this project targets

No existing 3DGS system changes its Gaussian budget or LOD tier at runtime based on
measured device state — available GPU memory right now, thermal/throttle state, and
power/battery state — as opposed to viewpoint, network bandwidth, or a fixed developer-set
budget. The closest related paper ("Splats under Pressure", 2026) is an offline benchmark
with hand-picked LOD tiers and does not adapt anything.

Claim wording: "To our knowledge, this is the first 3DGS renderer that adapts its Gaussian
budget or LOD tier at runtime to measured device state (available GPU memory,
thermal/throttle status, and power state), rather than to viewpoint, network bandwidth, or
a fixed developer-set budget."

**Framing to keep consistent everywhere (code comments, docs, paper):** "resource-pressure-
aware", not "network-adaptive" or "visibility-adaptive" — that distinction is the entire
novelty claim and must not get blurred.

- Headline signals are GPU memory headroom and thermal/throttle state. Frame time is a
  supporting input, not the novelty (GaussAnything already uses frame-time slack at runtime).
- Never claim "first thermal-aware adaptive rendering/LOD" in general — Unity Adaptive
  Performance and the patent are counter-examples. The claim is specific to 3DGS.
- Don't narrow the claim to a platform (e.g. "laptops only"); the controller is
  device-agnostic.

## System components

1. **Base renderer** — Brush (https://github.com/ArthurBrussee/brush), an open-source Rust
   3DGS engine on wgpu/Burn, loading a pre-compressed `.ply` / `.compressed.ply` scene.
   Chosen over gsplat because gsplat is CUDA-only and the team hardware is an Apple M5 Mac
   (no CUDA) plus a Windows laptop with an older discrete GPU. Brush runs on Metal, Vulkan,
   and DX12 from one codebase. Status: pending week-1 test on both machines; fallback is
   gsplat on an NVIDIA machine. Not built from scratch. Treat as a dependency, not a target
   for modification unless strictly necessary for instrumentation hooks.
2. **Resource monitor** — continuously samples GPU memory headroom, thermal/throttle state,
   power/battery state, and frame time. Thermal and power are required, not optional — they
   are core to the claim. This is standard systems instrumentation, not ML.
   wgpu does not expose free VRAM, so memory is read from OS APIs: DXGI
   `QueryVideoMemoryInfo` on Windows, Metal `recommendedMaxWorkingSetSize` /
   `currentAllocatedSize` on macOS. Apple Silicon has unified memory, so "GPU memory
   headroom" there must be defined explicitly in the paper.
3. **Adaptive decision policy** — maps monitor readings to a detail-level decision
   (upgrade/downgrade rendered Gaussian count or LOD tier) each frame or on a sampling
   interval. Uses hysteresis so tiers don't oscillate, and prefers an importance-ordered
   Gaussian list so a lower tier is "render the first N".
4. **Benchmark harness** — runs the adaptive version against these baselines across
   devices, logging memory usage, frame time, and visual-quality delta (e.g. PSNR/SSIM
   against full-detail reference):
   - fixed detail, no adaptation
   - static per-device tier (FLoD-style)
   - distance-based LOD (Octree-GS/LODGE-style)
   - fixed VRAM budget with eviction (MoQSplat-style)
   - thermal-warning step-down (Unity Adaptive Performance-style)

   Beating the last two is what shows the contribution is more than engineering. Pressure
   must be induced without `nvidia-smi` where the GPU isn't NVIDIA (e.g. a separate
   memory-filling process, sustained-load runs until throttling).
5. **Live demo layer** — on-screen overlay showing current detail level, memory pressure,
   and frame time in real time during interactive fly-through.

## Correctness / rigor expectations

- Every claim in the eventual paper needs to trace to a benchmark run, not intuition.
- Baseline (fixed detail) vs. adaptive comparisons must run on the same machine per pair —
  never compare a baseline run on one laptop against an adaptive run on another.
- Visual-quality loss under adaptation must be reported with a real metric (PSNR/SSIM/LPIPS),
  not just described qualitatively.
- Before extending scope, re-check this exact niche against IEEE Xplore / ACM DL — the
  surrounding literature (Voyager, LapisGS, CAGS, "Splats under Pressure") is moving fast;
  re-verify novelty periodically, not just once at the start.

## Tech stack

- **Base 3DGS renderer**: Brush (Rust 1.88+, wgpu) — do not reimplement.
- **Resource monitor + adaptive policy**: policy in C++ as a standalone module, called from
  Brush via Rust FFI; instrumentation hooks inside Brush written in Rust.
- **Benchmark harness**: `std::chrono` for timing (C++ portions); Python + matplotlib/pandas
  for turning logged results into paper figures.
- **Build**: CMake for any standalone C++ components, kept cross-platform (team uses both Mac
  and Windows laptops).
- **Version control**: Git/GitHub, shared repo.
- **Testing**: doctest (header vendored in the repo) for correctness checks on the decision
  policy logic.
- **Python env**: project-local `.venv` (numpy, pandas, matplotlib, scipy, torch,
  torchmetrics, lpips, scikit-image, psutil, plyfile, opencv). No CUDA toolkit, conda, or WSL.

## Team devices

- Apple M5 MacBook, 16 GB unified memory (Metal).
- Windows laptop with an older discrete GPU (exact model/VRAM to be confirmed) — primary
  resource-constrained demo machine.

## Coding conventions (C++ portions)

- K&R brace style (opening brace on the same line).
- Braces required for all statements, even single-line ones — no brace-less `if`/`for`.
- No inline comments; keep code self-explanatory or explain in commit messages / docs instead.

## Milestones (2-2.5 month window)

- **Week 1**: Deep literature check (IEEE Xplore, ACM DL) confirming the gap still holds.
  Base renderer running locally on team laptops.
- **Weeks 2-3**: Resource monitor built and validated (accurate live memory/frame-time
  readings on both Mac and Windows).
- **Weeks 3-5**: Adaptive decision policy implemented and integrated with the renderer.
- **Weeks 5-6**: Benchmark harness + experiments across available devices.
- **Weeks 6-7**: Live demo polish (this is the centerpiece for presentation — prioritize it).
- **Weeks 7-8**: Paper write-up (problem, related work, method, results).

## What "done" looks like for the demo

A live fly-through on a resource-constrained laptop that stays smooth under an adaptive
policy while a fixed-detail baseline stutters or runs out of memory on the same hardware,
with on-screen live stats (memory, frame time, detail level) and a before/after visual-
quality comparison.

## Things to flag back to the user, not silently work around

- If the base renderer proves too hard to get running in week 1, say so immediately —
  don't burn weeks 2-3 still fighting setup.
- If a literature search turns up a closer match to this exact gap, surface it right away
  rather than continuing on the original framing.
- If the correctness/rigor of a benchmark claim is shaky, flag it rather than writing it up
  as solid.
- Brush's README claims it is "generally faster than gsplat" — do not cite this in the paper
  unless measured ourselves.
- Open literature checks before submitting (see `research.md`): read Milef et al. CLoD
  "budget-based rendering" in full (does frame time set the budget? highest priority), read
  Atlas §6 (runtime Gaussian management), and pull the Google Scholar "Cited by" list for
  "Splats under Pressure" (arXiv 2604.07177). Several comparators are unreviewed 2026
  preprints — verify each before citing.
