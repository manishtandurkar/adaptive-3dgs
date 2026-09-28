# adaptive-3dgs

Resource-pressure-aware adaptive rendering for 3D Gaussian Splatting (3DGS).

A 3DGS renderer that adjusts rendered detail live based on the device's own resource
pressure (GPU memory headroom, frame time, thermal/power state) rather than network
bandwidth or camera visibility. Final-year research project; the goal is a working system
plus a paper with benchmarked results.

## Status

Week 1 (setup) in progress.

- [x] Brush builds and renders the bonsai scene on macOS (Apple M5, Metal)
- [ ] Brush verified on the Windows test laptop
- [ ] Literature re-check (IEEE Xplore, ACM DL, arXiv)

## System overview

| Component | Description | Language |
|---|---|---|
| Base renderer | [Brush](https://github.com/ArthurBrussee/brush) (forked, in `external/brush`) | Rust |
| Resource monitor | Samples GPU memory (DXGI on Windows, Metal on macOS), frame time, thermal/power | Rust |
| Adaptive policy | Maps monitor readings to a detail level (Gaussian count / LOD tier) | C++ |
| Benchmark harness | Fixed-detail baseline vs. adaptive, same machine per pair; PSNR/SSIM/LPIPS | Python |
| Live demo | Fly-through with on-screen detail level, memory pressure, frame time | Rust |

## Repository layout

```
external/brush/        Brush renderer (git submodule, our fork)
scripts/get_scenes.sh  Downloads pretrained scenes into scenes/
scenes/                Scene files (not committed)
CLAUDE.md              Project brief and conventions
```

## Setup

### 1. Clone

```
git clone --recursive https://github.com/manishtandurkar/adaptive-3dgs
cd adaptive-3dgs
```

If you already cloned without `--recursive`: `git submodule update --init`

### 2. Install prerequisites

macOS:

- Xcode Command Line Tools, Homebrew, Git, CMake, Python 3.13
- Rust 1.88+: `brew install rustup && rustup default stable`

Windows:

- Latest GPU driver
- Git for Windows
- Visual Studio 2022 Build Tools with "Desktop development with C++" (install before Rust)
- Rust 1.88+ via [rustup](https://rustup.rs)
- CMake, Python 3.13 (add both to PATH)

No CUDA toolkit, conda, or WSL needed.

### 3. Python environment

```
python3 -m venv .venv
source .venv/bin/activate          # Windows: .venv\Scripts\activate
pip install numpy pandas matplotlib scipy pillow imageio opencv-python plyfile torch torchvision torchmetrics lpips scikit-image psutil tqdm
```

### 4. Download scenes

```
bash scripts/get_scenes.sh         # Windows: run from Git Bash
```

Downloads the Mip-NeRF 360 bonsai scene (pretrained 3DGS, 30k iterations, ~309 MB,
1.24M Gaussians) into `scenes/bonsai/`.

### 5. Build and run Brush

```
cd external/brush
cargo build --release              # first build takes ~10-15 min
```

```
./target/release/brush ../../scenes/bonsai/point_cloud.ply --with-viewer
```

On Windows the binary is `target\release\brush.exe`.

The scene looks best when viewed from around the central table; views far from the
original capture cameras show floaters and smearing, which is expected for 3DGS.

## Rigor rules

- Every paper claim traces to a logged benchmark run.
- Baseline and adaptive runs are always paired on the same machine.
- Quality loss is reported with PSNR, SSIM, and LPIPS, evaluated from the original
  capture camera poses.

## Acknowledgements

Built on [Brush](https://github.com/ArthurBrussee/brush) by Arthur Brussee. Bonsai scene
from Mip-NeRF 360 (Barron et al.), trained with the original 3DGS (Kerbl et al.), via
[dylanebert/3dgs](https://huggingface.co/datasets/dylanebert/3dgs).
