#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== img2char setup ==="

# 1. Python venv
if [ ! -d "venv" ]; then
    echo "Creating Python virtual environment..."
    python3 -m venv venv
fi
source venv/bin/activate

pip install --upgrade pip setuptools wheel
pip install -U -r scripts/requirements.txt

# 2. Detect hardware and install the right PyTorch
detect_backend() {
    # Check for NVIDIA GPU (CUDA)
    if command -v nvidia-smi &>/dev/null; then
        echo "cuda"
        return
    fi

    # Check for Apple Silicon (MPS)
    if [[ "$(uname -s)" == "Darwin" ]]; then
        local arch
        arch="$(uname -m)"
        if [[ "$arch" == "arm64" ]]; then
            echo "mps"
            return
        fi
    fi

    # Check for AMD GPU (ROCm) on Linux
    if [[ -d "/opt/rocm" ]] || command -v rocm-smi &>/dev/null; then
        echo "rocm"
        return
    fi

    echo "cpu"
}

BACKEND=$(detect_backend)
echo "Detected backend: $BACKEND"

case "$BACKEND" in
    cuda)
        echo "Installing PyTorch with CUDA support..."
        pip install torch torchvision
        ;;
    mps)
        echo "Installing PyTorch with MPS (Apple Silicon) support..."
        # Default PyPI torch includes MPS on arm64 macOS
        pip install torch torchvision
        ;;
    rocm)
        echo "Installing PyTorch with ROCm support..."
        pip install torch torchvision --index-url https://download.pytorch.org/whl/rocm6.2
        ;;
    cpu)
        echo "No GPU detected. Installing PyTorch (CPU-only)..."
        pip install torch torchvision --index-url https://download.pytorch.org/whl/cpu
        ;;
esac

# Verify what we got
python3 -c "
import torch
print(f'  PyTorch {torch.__version__}')
print(f'  CUDA available: {torch.cuda.is_available()}')
if hasattr(torch.backends, 'mps'):
    print(f'  MPS available:  {torch.backends.mps.is_available()}')
print(f'  Device: ', end='')
if torch.cuda.is_available():
    print(f'cuda ({torch.cuda.get_device_name(0)})')
elif hasattr(torch.backends, 'mps') and torch.backends.mps.is_available():
    print('mps (Apple Silicon)')
else:
    print('cpu')
"

# 3. Clone TripoSR if not present
if [ ! -d "TripoSR" ]; then
    echo "Cloning TripoSR..."
    cd tmp && git clone https://github.com/VAST-AI-Research/TripoSR.git && cd ..
fi

# 4. Install TripoSR dependencies
echo "Installing TripoSR dependencies..."
cd tmp/TripoSR && pip install -r requirements.txt && cd ..

# 5. Install additional pipeline dependencies
echo "Installing pipeline dependencies..."
pip install trimesh pygltflib numpy Pillow tqdm

# 6. Check Blender
if command -v blender &>/dev/null; then
    echo "Blender found: $(blender --version 2>&1 | head -1)"
elif [ -d "/Applications/Blender.app" ]; then
    echo "Blender found at /Applications/Blender.app"
    echo "Will use: /Applications/Blender.app/Contents/MacOS/Blender"
else
    echo ""
    echo "WARNING: Blender not found."
    echo "Install Blender for auto-rigging step: https://www.blender.org/download/"
    echo "The mesh generation step will still work without Blender."
fi

echo ""
echo "=== Setup complete ==="
echo "Usage:"
echo "  source venv/bin/activate"
echo "  python pipeline.py --input input/ --output output/"
echo ""
echo "Put your character images (PNG) in the input/ directory."
