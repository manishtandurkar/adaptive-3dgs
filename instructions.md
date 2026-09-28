# Windows Setup Instructions

Step-by-step guide to clone, build, and run this project on a Windows laptop. Follow the
steps in order. Commands are for PowerShell unless marked otherwise.

Disk space needed: about 10 GB free (Rust toolchain ~1.5 GB, Brush build ~4-5 GB,
Python environment ~1-2 GB, scene ~0.3 GB, Visual Studio Build Tools ~2-3 GB).

---

## Part 0 (optional): Quick GPU test before installing anything

To check that Brush runs on your GPU in 5 minutes, before the full setup:

1. Download the latest Windows build from https://github.com/ArthurBrussee/brush/releases
   (the file with "windows" in its name), and unzip it.
2. Download the bonsai scene (~309 MB):
   https://huggingface.co/datasets/dylanebert/3dgs/resolve/main/bonsai/point_cloud/iteration_30000/point_cloud.ply
3. Open Brush, then drag the .ply file into its window (or use its open/load button).

If bonsai renders, your GPU works with Brush. Still do the full setup below, because we
build our own modified version of Brush.

---

## Part 1: Install prerequisites

Install these in this order.

### 1.1 GPU driver

Update to the latest driver for your graphics card:

- NVIDIA: https://www.nvidia.com/Download/index.aspx
- AMD: https://www.amd.com/en/support
- Intel: https://www.intel.com/content/www/us/en/download-center/home.html

Restart after installing.

### 1.2 Git for Windows

Download: https://git-scm.com/download/win

Use the default options. This also installs Git Bash, which is needed in Part 4.

### 1.3 Visual Studio 2022 Build Tools

Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/

1. Run the installer.
2. Tick the workload "Desktop development with C++".
3. On the right side, make sure these are ticked: "MSVC v143 build tools" and
   "Windows 11 SDK" (or "Windows 10 SDK").
4. Click Install. Restart when finished.

This must be installed before Rust. Rust uses its linker, and our GPU memory monitor uses
the Windows SDK.

### 1.4 Rust

Download: https://rustup.rs (click "rustup-init.exe (64-bit)")

1. Run rustup-init.exe.
2. When asked, choose option 1 (default installation).
3. Close and reopen PowerShell after it finishes.

### 1.5 CMake

Download: https://cmake.org/download/ (Windows x64 Installer, .msi)

During install, select "Add CMake to the system PATH for all users".

### 1.6 Python 3.13

Download: https://www.python.org/downloads/

On the first installer screen, tick "Add python.exe to PATH" before clicking Install.

If typing `python` opens the Microsoft Store instead of running Python: open Settings, then
Apps, then Advanced app settings, then App execution aliases, and turn off both
"python.exe" and "python3.exe" (App Installer). Then reopen PowerShell.

### 1.7 (Optional) VS Code

Download: https://code.visualstudio.com/

Recommended extensions: rust-analyzer, C/C++ (Microsoft), Python (Microsoft), CMake Tools.

### 1.8 Verify everything

Close and reopen PowerShell, then run:

```powershell
git --version
rustc --version
cargo --version
cmake --version
python --version
```

Expected:

- git 2.x
- rustc 1.88 or newer
- cargo 1.88 or newer
- cmake 3.20 or newer
- Python 3.13.x

If any command says "not recognized", that tool is not on PATH. Reinstall it with the PATH
option ticked, or restart the laptop.

### 1.9 Configure Git

Run once (use the same email as your GitHub account):

```powershell
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
git config --global core.longpaths true
git config --global core.autocrlf true
```

`core.longpaths` prevents path-too-long errors in the Brush code. `core.autocrlf` keeps line
endings consistent between Windows and Mac teammates.

### 1.10 GitHub access

1. Create a GitHub account if you do not have one, and send your username to Manish.
2. Manish adds you as a collaborator on `manishtandurkar/adaptive-3dgs` (and on
   `manishtandurkar/brush` if you will change renderer code). Accept the email invite.
3. The first time you push, Git opens a browser window to log in to GitHub. Sign in and
   approve. Git remembers it after that.

---

## Part 2: Clone the repository

Pick a folder with a short path, for example `C:\dev`. Long paths can break the Rust build
on Windows.

```powershell
mkdir C:\dev
cd C:\dev
git clone --recursive https://github.com/manishtandurkar/adaptive-3dgs
cd adaptive-3dgs
```

`--recursive` also downloads the Brush renderer into `external\brush`. Check it worked:

```powershell
dir external\brush
```

You should see files like `Cargo.toml` and `README.md`. If the folder is empty, run:

```powershell
git submodule update --init
```

---

## Part 3: Python environment

From the project folder (`C:\dev\adaptive-3dgs`):

```powershell
python -m venv .venv
.venv\Scripts\activate
```

Your prompt should now start with `(.venv)`.

If activation fails with "running scripts is disabled on this system", run this once and
try again:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

Then install the packages:

```powershell
python -m pip install --upgrade pip
pip install numpy pandas matplotlib scipy pillow imageio opencv-python plyfile torch torchvision torchmetrics lpips scikit-image psutil tqdm
```

This downloads about 1 GB and takes a few minutes. Verify:

```powershell
python -c "import torch, numpy, pandas, lpips, torchmetrics, skimage, psutil; print('OK', torch.__version__)"
```

Every time you open a new terminal to work on the project, activate the environment again
with `.venv\Scripts\activate`.

---

## Part 4: Download the test scene

Open Git Bash (Start menu, then "Git Bash"), then run:

```bash
cd /c/dev/adaptive-3dgs
bash scripts/get_scenes.sh
```

This downloads the bonsai scene (~309 MB) into `scenes\bonsai\point_cloud.ply`.

If you prefer PowerShell instead of Git Bash:

```powershell
mkdir scenes\bonsai -Force
curl.exe -L -o scenes\bonsai\point_cloud.ply https://huggingface.co/datasets/dylanebert/3dgs/resolve/main/bonsai/point_cloud/iteration_30000/point_cloud.ply
```

---

## Part 5: Build Brush

Back in PowerShell:

```powershell
cd C:\dev\adaptive-3dgs\external\brush
cargo build --release
```

The first build downloads and compiles about 900 packages. Expect 10-30 minutes depending
on the laptop, and the laptop will get warm. Keep it plugged in. Later builds only
recompile what changed and are much faster.

The build is done when you see:

```
Finished `release` profile [optimized] target(s) in ...
```

Tip: Windows Defender scanning can slow the build a lot. If it is very slow, add
`C:\dev\adaptive-3dgs` as an exclusion in Windows Security, then Virus & threat
protection, then Manage settings, then Exclusions.

---

## Part 6: Run Brush with the bonsai scene

From `C:\dev\adaptive-3dgs\external\brush`:

```powershell
.\target\release\brush.exe ..\..\scenes\bonsai\point_cloud.ply --with-viewer
```

A window should open showing the bonsai scene.

- Drag with the mouse to rotate, scroll to zoom.
- The starting view may look smeared or have floating blobs. This is normal: the scene was
  captured by cameras circling a table in the middle of the room. Zoom out and orbit until
  you are looking at the bonsai on the central table, and it should look photorealistic.
- Close the window to exit.

### Laptops with two GPUs (integrated + NVIDIA/AMD)

Windows may run Brush on the weaker integrated GPU. To force the dedicated GPU:

1. Settings, then System, then Display, then Graphics.
2. Click "Browse" and select
   `C:\dev\adaptive-3dgs\external\brush\target\release\brush.exe`.
3. Click it, then Options, then choose "High performance", then Save.
4. Restart Brush.

To confirm which GPU is used: open Task Manager, then the Performance tab. The dedicated
GPU's graph should rise while Brush is running.

---

## Part 7: Report back to the team

Send the following in the team chat:

1. GPU model and dedicated memory: Task Manager, then Performance, then GPU. Note the name
   (e.g. "NVIDIA GeForce GTX 1050") and "Dedicated GPU memory" (e.g. 4.0 GB).
2. Did Brush build successfully? If not, paste the last 20 lines of the error.
3. Does bonsai render, and does it feel smooth or choppy when rotating?
4. While Brush is open: "Dedicated GPU memory" usage in Task Manager.
5. How long the first build took.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| `link.exe not found` or `linker 'link.exe' not found` during build | Visual Studio Build Tools missing or "Desktop development with C++" not ticked. Re-run the VS installer, tick it, restart, then rebuild. |
| `cargo` or `rustc` not recognized | Close and reopen PowerShell. If still missing, re-run rustup-init.exe. |
| `external\brush` is empty | `git submodule update --init` from the project folder. |
| Build fails with path-too-long errors | Make sure you did Part 1.9 and the project is at a short path like `C:\dev`. |
| `python` opens the Microsoft Store | Turn off the python App execution aliases (Part 1.6). |
| `git push` rejected / permission denied | Accept the collaborator invite (Part 1.10), then push again. |
| `.venv\Scripts\activate` blocked | `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned` |
| Brush window opens but is black or crashes | Update the GPU driver (Part 1.1). Then report the GPU model and error to the team. |
| Brush runs very slowly | Check it is using the dedicated GPU (Part 6). |
| Scene file not found | Make sure Part 4 finished and `scenes\bonsai\point_cloud.ply` exists (~309 MB). |

---

## Updating later

To pull the latest project changes (including updates to our Brush fork):

```powershell
cd C:\dev\adaptive-3dgs
git pull
git submodule update --init --recursive
cd external\brush
cargo build --release
```

---

## Not needed

Do not install these; the project does not use them:

- CUDA Toolkit
- Anaconda / conda
- WSL
