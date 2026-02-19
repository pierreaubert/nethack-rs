#!/usr/bin/env python3
"""
Resumable download of TripoSR model weights from Hugging Face.

Downloads in chunks with resume support. Safe to ctrl-C and restart.

Usage:
    python scripts/download_weights.py
    python scripts/download_weights.py --chunk-size 50  # 50MB chunks
"""

import argparse
import sys
import time
import urllib.request
import urllib.error
from pathlib import Path


HF_BASE = "https://huggingface.co"
REPO_ID = "stabilityai/TripoSR"

# Files to download with their expected locations
FILES = [
    ("config.yaml", "config.yaml"),
    ("model.ckpt", "model.ckpt"),
]

# The DINO ViT model is also needed (downloaded by transformers on first run)
# but it's small and handled automatically. The big one is model.ckpt (~5GB).


def hf_url(repo_id: str, filename: str) -> str:
    return f"{HF_BASE}/{repo_id}/resolve/main/{filename}"


def get_remote_size(url: str) -> int | None:
    """Get file size from server via HEAD request."""
    req = urllib.request.Request(url, method="HEAD")
    req.add_header("User-Agent", "img2char/1.0")
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            size = resp.headers.get("Content-Length")
            return int(size) if size else None
    except urllib.error.URLError:
        return None


def download_with_resume(url: str, dest: Path, chunk_size_mb: int = 10) -> bool:
    """
    Download a file with resume support.
    Returns True if download completed, False if interrupted.
    """
    partial = dest.with_suffix(dest.suffix + ".partial")
    chunk_bytes = chunk_size_mb * 1024 * 1024

    # Get total size
    remote_size = get_remote_size(url)
    if remote_size is None:
        print(f"    WARNING: Could not determine file size for {url}")

    # Check if already fully downloaded
    if dest.exists():
        if remote_size and dest.stat().st_size == remote_size:
            print(f"    Already downloaded: {dest.name} ({format_size(remote_size)})")
            return True
        # Size mismatch — re-download
        dest.unlink()

    # Resume from partial
    existing_size = partial.stat().st_size if partial.exists() else 0
    if existing_size > 0:
        if remote_size and existing_size >= remote_size:
            # Partial is complete, rename
            partial.rename(dest)
            print(f"    Completed (from partial): {dest.name}")
            return True
        print(f"    Resuming {dest.name} from {format_size(existing_size)}", end="")
        if remote_size:
            print(f" / {format_size(remote_size)} ({existing_size * 100 // remote_size}%)")
        else:
            print()
    else:
        size_str = format_size(remote_size) if remote_size else "unknown size"
        print(f"    Downloading {dest.name} ({size_str})")

    dest.parent.mkdir(parents=True, exist_ok=True)

    while True:
        current_size = partial.stat().st_size if partial.exists() else 0

        # Check if done
        if remote_size and current_size >= remote_size:
            break

        # Build range request
        range_end = current_size + chunk_bytes - 1
        req = urllib.request.Request(url)
        req.add_header("User-Agent", "img2char/1.0")
        req.add_header("Range", f"bytes={current_size}-{range_end}")

        try:
            t0 = time.monotonic()
            bytes_this_request = 0
            read_chunk = 1024 * 1024  # Stream 1MB at a time to avoid IncompleteRead

            with urllib.request.urlopen(req, timeout=60) as resp:
                with open(partial, "ab") as f:
                    while True:
                        piece = resp.read(read_chunk)
                        if not piece:
                            break
                        f.write(piece)
                        f.flush()
                        bytes_this_request += len(piece)

                        new_size = current_size + bytes_this_request
                        elapsed = time.monotonic() - t0
                        speed = bytes_this_request / elapsed if elapsed > 0 else 0

                        if remote_size:
                            pct = new_size * 100 // remote_size
                            print(
                                f"\r    {format_size(new_size)} / {format_size(remote_size)} "
                                f"({pct}%) - {format_size(speed)}/s   ",
                                end="",
                                flush=True,
                            )

            if bytes_this_request > 0:
                print()  # newline after progress

            if bytes_this_request == 0:
                break

            # If we got less than a full chunk, we're done
            if bytes_this_request < chunk_bytes:
                break

        except (urllib.error.URLError, TimeoutError, ConnectionError, OSError,
                Exception) as e:
            # Save whatever was written so far — it's already flushed to disk
            saved = partial.stat().st_size if partial.exists() else 0
            print(f"\n    Network error: {type(e).__name__}: {e}")
            print(f"    Progress saved at {format_size(saved)}. Retrying in 5s...")
            time.sleep(5)
            continue

    # Verify final size
    final_size = partial.stat().st_size if partial.exists() else 0
    if remote_size and final_size != remote_size:
        print(f"    WARNING: Size mismatch: got {final_size}, expected {remote_size}")
        print(f"    Run again to retry from {format_size(final_size)}")
        return False

    partial.rename(dest)
    print(f"    Done: {dest.name}")
    return True


def format_size(n: int | float) -> str:
    if n >= 1_000_000_000:
        return f"{n / 1_000_000_000:.2f} GB"
    if n >= 1_000_000:
        return f"{n / 1_000_000:.1f} MB"
    if n >= 1_000:
        return f"{n / 1_000:.0f} KB"
    return f"{n} B"


def main():
    parser = argparse.ArgumentParser(description="Download TripoSR weights (resumable)")
    parser.add_argument(
        "--chunk-size", type=int, default=10,
        help="Download chunk size in MB (default: 10)",
    )
    parser.add_argument(
        "--dest", type=str, default=None,
        help="Destination directory (default: TripoSR/weights/)",
    )
    args = parser.parse_args()

    script_dir = Path(__file__).parent.parent.resolve()
    dest_dir = Path(args.dest) if args.dest else script_dir / "weights"
    dest_dir.mkdir(parents=True, exist_ok=True)

    print(f"=== TripoSR weight download ===")
    print(f"  Destination: {dest_dir}")
    print(f"  Chunk size:  {args.chunk_size} MB")
    print()

    all_ok = True
    for filename, local_name in FILES:
        url = hf_url(REPO_ID, filename)
        dest = dest_dir / local_name
        ok = download_with_resume(url, dest, args.chunk_size)
        if not ok:
            all_ok = False

    if all_ok:
        print(f"\n  All weights downloaded to {dest_dir}")
        print(f"  Run pipeline with:")
        print(f"    python pipeline.py -i input/image.png -o output/ --skip-rig")
    else:
        print(f"\n  Some downloads incomplete. Run this script again to resume.")

    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
