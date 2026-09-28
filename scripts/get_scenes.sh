#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BASE="https://huggingface.co/datasets/dylanebert/3dgs/resolve/main"

mkdir -p "$ROOT/scenes/bonsai"
if [ ! -f "$ROOT/scenes/bonsai/point_cloud.ply" ]; then
    echo "Downloading bonsai (~309 MB)..."
    curl -L --fail -o "$ROOT/scenes/bonsai/point_cloud.ply" \
        "$BASE/bonsai/point_cloud/iteration_30000/point_cloud.ply"
fi

echo "Scenes ready in $ROOT/scenes"
