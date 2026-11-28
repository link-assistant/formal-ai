"""Replay the first pinned GSM8K request with the v0.347.0 desktop CLI."""

import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile


ASSET_NAME = "formal-ai-desktop-linux-x64-0.347.0.tar.gz"
ASSET_URL = (
    "https://github.com/link-assistant/formal-ai/releases/download/v0.347.0/"
    + ASSET_NAME
)
ASSET_BYTES = 390_021_614
ASSET_SHA256 = "e1313271276c48aad4bdd4f9071d34e5548cf3bb32bf127ff9701a383c15ff01"


class AssetStream:
    def __init__(self, stream):
        self.stream = stream
        self.bytes = 0
        self.digest = hashlib.sha256()

    def read(self, size):
        chunk = self.stream.read(size)
        self.bytes += len(chunk)
        self.digest.update(chunk)
        if self.bytes > ASSET_BYTES:
            raise ValueError("desktop release exceeds its published size")
        return chunk


def extract_cli(binary):
    # Stream the release so its browsers and other desktop resources need no disk.
    with subprocess.Popen(
        ["curl", "--proto", "=https", "-fsSL", "--max-filesize", str(ASSET_BYTES), ASSET_URL],
        stdout=subprocess.PIPE,
    ) as download:
        asset = AssetStream(download.stdout)
        with gzip.GzipFile(fileobj=asset) as decompressed:
            with tarfile.open(fileobj=decompressed, mode="r|") as archive:
                for member in archive:
                    if not member.name.endswith("/resources/bin/formal-ai"):
                        continue
                    if not member.isfile() or not 0 < member.size <= 200_000_000:
                        raise ValueError("unexpected CLI member in desktop release")
                    with archive.extractfile(member) as source, binary.open("xb") as target:
                        shutil.copyfileobj(source, target, length=128 * 1024)
        while asset.read(128 * 1024):
            pass
        if download.wait() != 0:
            raise RuntimeError("desktop release download failed")
    if asset.bytes != ASSET_BYTES or asset.digest.hexdigest() != ASSET_SHA256:
        raise ValueError("desktop release did not match its published bytes")
    binary.chmod(0o700)


def main():
    dataset = Path(sys.argv[1])
    with dataset.open(encoding="utf-8") as rows:
        question = json.loads(next(rows))["question"]
    with tempfile.TemporaryDirectory() as directory:
        binary = Path(directory) / "formal-ai"
        extract_cli(binary)
        subprocess.run([binary, "--version"], check=True)
        subprocess.run(
            [binary, "chat", "--thinking", "--prompt", question],
            cwd=directory,
            env={**os.environ, "FORMAL_AI_OFFLINE": "true"},
            check=True,
        )


if __name__ == "__main__":
    main()
