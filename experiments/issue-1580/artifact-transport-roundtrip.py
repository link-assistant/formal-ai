#!/usr/bin/env python3
"""Create and verify a tiny real Actions artifact without repository build output."""
import argparse
from pathlib import Path

PAYLOAD = bytes(range(256)) * 4


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("prepare", "verify"))
    parser.add_argument("directory", type=Path)
    options = parser.parse_args()
    root = options.directory
    if options.mode == "prepare":
        (root / "source/nested").mkdir(parents=True)
        (root / "source/nested/payload.bin").write_bytes(PAYLOAD)
        (root / "download/nested").mkdir(parents=True)
        (root / "download/nested/payload.bin").write_bytes(b"old cached executable")
        (root / "download/retained.txt").write_text("retained", encoding="utf-8")
    else:
        assert (root / "download/nested/payload.bin").read_bytes() == PAYLOAD
        assert (root / "download/retained.txt").read_text(encoding="utf-8") == "retained"
        assert not list((root / "download").rglob(".artifact-*"))
        print("Actual artifact transport preserved every byte and refreshed the cached file.")


if __name__ == "__main__":
    main()
