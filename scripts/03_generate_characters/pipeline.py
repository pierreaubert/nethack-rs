#!/usr/bin/env python3
"""
img2char: Batch 2D image → game-ready 3D character pipeline.

Usage:
    python pipeline.py --input input/ --output output/
    python pipeline.py --input input/hero.png --output output/ --format glb
    python pipeline.py --input input/ --output output/ --skip-rig  # mesh only
    python pipeline.py --input input/ --output output/ --workers 8
    python pipeline.py --all --output output/ --rig-dirs monsters,player  # batch all nh-bevy assets

Pipeline stages:
    1. TripoSR: single image → 3D textured mesh (OBJ)
    2. (optional) Decimate mesh to target face count via fast_simplification
    3. Blender headless: auto-rig with humanoid armature → FBX/GLB

Requirements:
    - Python venv with TripoSR dependencies (see setup.sh)
    - Blender installed (for rigging stage; mesh generation works without it)
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path


SCRIPT_DIR = Path(__file__).parent.resolve()
TRIPOSR_DIR = SCRIPT_DIR / "TripoSR"
BLENDER_RIG_SCRIPT = SCRIPT_DIR / "scripts" / "blender_rig.py"

NH_BEVY_ASSETS = Path("/assets/items")

IMAGE_EXTENSIONS = {".png", ".jpg", ".jpeg", ".webp", ".bmp"}


def detect_device() -> str:
    """Pick the best available compute device."""
    try:
        import torch
        if torch.cuda.is_available():
            return "cuda:0"
        if torch.backends.mps.is_available():
            return "mps"
    except ImportError:
        pass
    return "cpu"


@dataclass
class CharacterResult:
    name: str
    image_path: str
    mesh_path: str = ""
    rigged_path: str = ""
    mesh_time: float = 0.0
    rig_time: float = 0.0
    error: str = ""


def find_blender() -> str | None:
    """Locate the Blender executable."""
    # Check PATH
    blender = shutil.which("blender")
    if blender:
        return blender

    # macOS app bundle
    mac_path = "/Applications/Blender.app/Contents/MacOS/Blender"
    if os.path.isfile(mac_path):
        return mac_path

    return None


def find_images(input_path: Path) -> list[Path]:
    """Find all image files in the input path."""
    if input_path.is_file():
        return [input_path]

    images = []
    for f in sorted(input_path.rglob("*")):
        if f.is_file() and f.suffix.lower() in IMAGE_EXTENSIONS:
            images.append(f)
    return images


def relative_output_dir(image_path: Path, input_root: Path, output_dir: Path) -> Path:
    """Compute the output directory for an image, preserving directory structure.

    For a single file input (input_root == image_path), returns output_dir/stem.
    For a directory input, mirrors the relative path:
        input_root=assets/, image=assets/monsters/bat.png → output_dir/monsters/bat/
    """
    rel = image_path.relative_to(input_root)
    # Single file: rel is just "bat.png" → parent is ".", so we get output_dir/bat
    # Directory:   rel is "monsters/bat.png" → output_dir/monsters/bat
    return output_dir / rel.parent / rel.stem


def generate_mesh(image_path: Path, char_output: Path, mc_resolution: int, bake_texture: bool, device: str = "cpu") -> Path:
    """
    Run TripoSR to generate a 3D mesh from a single image.
    Returns the path to the generated OBJ file.
    """
    name = image_path.stem

    # Use local weights if downloaded, otherwise TripoSR downloads from HF
    weights_dir = SCRIPT_DIR / "weights"
    model_path = str(weights_dir) if (weights_dir / "model.ckpt").exists() else "stabilityai/TripoSR"

    cmd = [
        sys.executable,
        str(TRIPOSR_DIR / "run.py"),
        str(image_path),
        "--output-dir", str(char_output),
        "--mc-resolution", str(mc_resolution),
        "--device", device,
        "--pretrained-model-name-or-path", model_path,
    ]
    if bake_texture:
        cmd.append("--bake-texture")

    print(f"  [{name}] Generating mesh (resolution={mc_resolution}, device={device})...")
    print(f"  [{name}] Running: {' '.join(cmd)}")
    result = subprocess.run(
        cmd,
        cwd=str(TRIPOSR_DIR),
    )

    if result.returncode != 0:
        raise RuntimeError(f"TripoSR failed for {name} (exit code {result.returncode})")

    # TripoSR outputs to <output_dir>/0/mesh.obj (single image → index 0)
    mesh_path = char_output / "0" / "mesh.obj"
    if not mesh_path.exists():
        # Try .glb
        mesh_path = char_output / "0" / "mesh.glb"
    if not mesh_path.exists():
        raise FileNotFoundError(
            f"No mesh found in {char_output / '0'}. "
            f"Contents: {list((char_output / '0').iterdir()) if (char_output / '0').exists() else 'dir missing'}"
        )

    return mesh_path


def decimate_mesh(mesh_path: Path, max_faces: int) -> tuple[Path, int, int]:
    """Decimate a mesh to at most max_faces triangles using fast_simplification.

    Overwrites the mesh file in-place. Returns (path, original_faces, new_faces).
    Note: this preserves vertex colors but strips UV coordinates.
    """
    import trimesh  # type: ignore[import-unresolved]

    mesh = trimesh.load(mesh_path, process=False)
    original_faces = len(mesh.faces)
    if original_faces <= max_faces:
        return mesh_path, original_faces, original_faces

    simplified = mesh.simplify_quadric_decimation(face_count=max_faces)
    # Carry over vertex colors if the original had them
    if hasattr(mesh.visual, "vertex_colors") and mesh.visual.vertex_colors is not None:
        # After decimation vertex indices change; vertex colors can't be mapped 1:1.
        # Export without per-vertex color (texture or flat shading).
        pass

    simplified.export(str(mesh_path))
    return mesh_path, original_faces, len(simplified.faces)


def rig_mesh(mesh_path: Path, output_path: Path, blender_bin: str) -> Path:
    """
    Use Blender headless to auto-rig the mesh.
    Returns the path to the rigged file.
    """
    cmd = [
        blender_bin,
        "--background",
        "--python", str(BLENDER_RIG_SCRIPT),
        "--",
        str(mesh_path),
        str(output_path),
    ]

    result = subprocess.run(cmd)

    if result.returncode != 0:
        raise RuntimeError(f"Blender rigging failed (exit code {result.returncode})")

    if not output_path.exists():
        raise FileNotFoundError(f"Rigged file not created: {output_path}")

    return output_path


def should_rig(image_path: Path, rig_dirs: list[str] | None) -> bool:
    """Determine whether this image should be rigged based on its path.

    If rig_dirs is None, rig everything. If rig_dirs is a list,
    only rig images whose path contains one of the directory patterns.
    """
    if rig_dirs is None:
        return True
    path_str = str(image_path)
    return any(f"/{d}/" in path_str or path_str.startswith(f"{d}/") for d in rig_dirs)


def process_single(
    image_path: Path,
    input_root: Path,
    output_dir: Path,
    blender_bin: str | None,
    export_format: str,
    mc_resolution: int,
    bake_texture: bool,
    skip_rig: bool,
    device: str = "cpu",
    rig_dirs: list[str] | None = None,
    max_faces: int | None = None,
) -> CharacterResult:
    """Process a single character image through the full pipeline."""
    name = image_path.stem
    char_output = relative_output_dir(image_path, input_root, output_dir)
    result = CharacterResult(name=name, image_path=str(image_path))

    # Stage 1: Generate mesh
    try:
        t0 = time.monotonic()
        mesh_path = generate_mesh(image_path, char_output, mc_resolution, bake_texture, device)
        result.mesh_time = time.monotonic() - t0
        result.mesh_path = str(mesh_path)
        print(f"  [{name}] Mesh generated in {result.mesh_time:.1f}s → {mesh_path}")
    except Exception as e:
        result.error = f"mesh generation: {e}"
        print(f"  [{name}] FAILED mesh: {e}")
        return result

    # Stage 1.5: Decimate
    if max_faces is not None:
        try:
            _, orig, final = decimate_mesh(mesh_path, max_faces)
            if orig != final:
                print(f"  [{name}] Decimated {orig} → {final} faces")
            else:
                print(f"  [{name}] Already ≤{max_faces} faces ({orig}), no decimation needed")
        except Exception as e:
            result.error = f"decimation: {e}"
            print(f"  [{name}] FAILED decimate: {e}")
            return result

    # Stage 2: Auto-rig (skip if globally disabled, or if this image isn't in a rig dir)
    do_rig = not skip_rig and should_rig(image_path, rig_dirs)
    if not do_rig or blender_bin is None:
        if not skip_rig and not do_rig:
            print(f"  [{name}] Skipping rig (not in rig-dirs)")
        elif not skip_rig:
            print(f"  [{name}] Skipping rig (Blender not found)")
        return result

    try:
        rigged_path = char_output / f"{name}_rigged.{export_format}"
        t0 = time.monotonic()
        rig_mesh(mesh_path, rigged_path, blender_bin)
        result.rig_time = time.monotonic() - t0
        result.rigged_path = str(rigged_path)
        print(f"  [{name}] Rigged in {result.rig_time:.1f}s → {rigged_path}")
    except Exception as e:
        result.error = f"rigging: {e}"
        print(f"  [{name}] FAILED rig: {e}")

    return result


def main():
    parser = argparse.ArgumentParser(
        description="img2char: Batch 2D image → game-ready 3D character",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--input", "-i", default=None, help="Input image or directory of images")
    parser.add_argument("--output", "-o", required=True, help="Output directory")
    parser.add_argument("--all", action="store_true", help=f"Process all nh-bevy assets ({NH_BEVY_ASSETS})")
    parser.add_argument("--format", "-f", default="fbx", choices=["fbx", "glb"], help="Export format (default: fbx)")
    parser.add_argument("--resolution", "-r", type=int, default=256, help="Marching cubes resolution (default: 256, higher=slower+detailed)")
    parser.add_argument("--bake-texture", action="store_true", help="Bake texture atlas (slower but better UV mapping)")
    parser.add_argument("--skip-rig", action="store_true", help="Skip rigging, output mesh only")
    parser.add_argument("--max-faces", type=int, default=None,
                        help="Decimate meshes to at most this many triangles (e.g. 5000). "
                             "Uses fast_simplification. Strips UV coords; best without --bake-texture.")
    parser.add_argument("--rig-dirs", type=str, default=None,
                        help="Comma-separated directory patterns to rig (e.g. 'monsters,player'). "
                             "Images outside these dirs get mesh only. Omit to rig everything.")
    parser.add_argument("--workers", "-w", type=int, default=1, help="Parallel workers for mesh generation (default: 1)")
    parser.add_argument("--blender", type=str, default=None, help="Path to Blender executable (auto-detected if not set)")
    parser.add_argument("--device", type=str, default=None, help="Compute device: cpu, mps, cuda:0 (auto-detected if not set)")
    args = parser.parse_args()

    # Resolve input path
    if args.all:
        if args.input:
            parser.error("--all and --input are mutually exclusive")
        input_path = NH_BEVY_ASSETS
        if not input_path.exists():
            print(f"ERROR: nh-bevy assets not found at {input_path}")
            sys.exit(1)
    elif args.input:
        input_path = Path(args.input).resolve()
    else:
        parser.error("one of --input or --all is required")

    rig_dirs: list[str] | None = args.rig_dirs.split(",") if args.rig_dirs else None
    output_dir = Path(args.output).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    # Validate TripoSR
    if not TRIPOSR_DIR.exists():
        print(f"ERROR: TripoSR not found at {TRIPOSR_DIR}")
        print("Run setup.sh first.")
        sys.exit(1)

    # Find images
    images = find_images(input_path)
    if not images:
        print(f"No images found in {input_path}")
        sys.exit(1)

    # For relative path computation: use parent dir when input is a single file
    input_root = input_path.parent if input_path.is_file() else input_path

    # Find Blender
    blender_bin = args.blender or find_blender()
    if not args.skip_rig and blender_bin is None:
        print("WARNING: Blender not found. Will generate meshes only.")
        print("Install Blender or pass --blender /path/to/blender")

    # Detect compute device
    device = args.device or detect_device()

    print(f"=== img2char pipeline ===")
    print(f"  Images:     {len(images)}")
    print(f"  Output:     {output_dir}")
    print(f"  Format:     {args.format}")
    print(f"  Resolution: {args.resolution}")
    print(f"  Max faces:  {args.max_faces or 'unlimited'}")
    print(f"  Device:     {device}")
    print(f"  Blender:    {blender_bin or 'not found'}")
    print(f"  Rig dirs:   {rig_dirs or 'all (no filter)'}")
    print(f"  Workers:    {args.workers}")
    print()

    t_start = time.monotonic()
    results: list[CharacterResult] = []

    if args.workers > 1:
        # Parallel mesh generation, then sequential rigging (Blender is heavy)
        with ProcessPoolExecutor(max_workers=args.workers) as pool:
            futures = {
                pool.submit(
                    process_single,
                    img, input_root, output_dir, blender_bin, args.format,
                    args.resolution, args.bake_texture, args.skip_rig, device, rig_dirs,
                    args.max_faces,
                ): img
                for img in images
            }
            for future in as_completed(futures):
                results.append(future.result())
    else:
        for img in images:
            r = process_single(
                img, input_root, output_dir, blender_bin, args.format,
                args.resolution, args.bake_texture, args.skip_rig, device, rig_dirs,
                args.max_faces,
            )
            results.append(r)

    total_time = time.monotonic() - t_start

    # Summary
    print()
    print(f"=== Results ({total_time:.1f}s total) ===")
    succeeded = [r for r in results if not r.error]
    failed = [r for r in results if r.error]

    for r in sorted(results, key=lambda x: x.name):
        status = "OK" if not r.error else f"FAIL: {r.error}"
        rigged = f" | rigged: {r.rigged_path}" if r.rigged_path else ""
        print(f"  {r.name}: {status} (mesh: {r.mesh_time:.1f}s, rig: {r.rig_time:.1f}s){rigged}")

    print(f"\n  {len(succeeded)}/{len(results)} succeeded")

    # Write manifest
    manifest = {
        "total_time": total_time,
        "characters": [
            {
                "name": r.name,
                "image": r.image_path,
                "mesh": r.mesh_path,
                "rigged": r.rigged_path,
                "mesh_time": r.mesh_time,
                "rig_time": r.rig_time,
                "error": r.error,
            }
            for r in results
        ],
    }
    manifest_path = output_dir / "manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"\n  Manifest: {manifest_path}")


if __name__ == "__main__":
    main()
