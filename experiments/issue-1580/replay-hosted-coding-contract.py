#!/usr/bin/env python3
"""Read one producer-bound native executable without downloading its siblings."""

import argparse
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import urllib.error
import urllib.parse
import urllib.request
import zipfile


class KeepRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, location):
        return None


class ArtifactRanges(io.RawIOBase):
    """A seekable view over a GitHub Actions artifact via bounded HTTP ranges."""

    def __init__(self, location, size):
        self.location = location
        self.size = size
        self.offset = 0
        self.bytes_received = 0

    def tell(self):
        return self.offset

    def seek(self, offset, whence=io.SEEK_SET):
        origin = {io.SEEK_SET: 0, io.SEEK_CUR: self.offset, io.SEEK_END: self.size}[whence]
        self.offset = origin + offset
        if self.offset < 0:
            raise ValueError("negative artifact offset")
        return self.offset

    def read(self, size=-1):
        count = self.size - self.offset if size < 0 else min(size, self.size - self.offset)
        if count <= 0:
            return b""
        if count > 8 * 1024 * 1024:
            raise ValueError("artifact read exceeds the 8 MiB range limit")
        last = self.offset + count - 1
        requested = f"bytes={self.offset}-{last}"
        request = urllib.request.Request(self.location, headers={"Range": requested})
        with urllib.request.urlopen(request) as response:
            expected_range = f"bytes {self.offset}-{last}/{self.size}"
            if response.status != 206 or response.headers.get("Content-Range") != expected_range:
                raise ValueError("artifact server did not confirm the requested byte range")
            data = response.read(count + 1)
        if len(data) != count:
            raise ValueError("artifact range length differs from the requested length")
        self.offset += count
        self.bytes_received += count
        return data

    def seekable(self):
        return True


def signed_location(artifact_id):
    credential = subprocess.check_output(["gh", "auth", "token"], text=True).strip()
    url = f"https://api.github.com/repos/link-assistant/formal-ai/actions/artifacts/{artifact_id}/zip"
    request = urllib.request.Request(url, headers={"Authorization": f"Bearer {credential}"})
    try:
        urllib.request.build_opener(KeepRedirect()).open(request).close()
    except urllib.error.HTTPError as error:
        location = error.headers.get("Location", "")
        if error.code == 302 and urllib.parse.urlsplit(location).scheme == "https":
            return location
        if error.code != 302:
            raise RuntimeError(f"GitHub artifact request failed with HTTP {error.code}") from None
    raise RuntimeError("GitHub did not supply a signed HTTPS artifact location")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("metadata", type=Path)
    parser.add_argument("compiled_commit")
    parser.add_argument("destination", type=Path)
    parser.add_argument("--member", choices=("coding_contracts", "formal-ai"), default="coding_contracts")
    parser.add_argument("--inspect", action="store_true")
    args = parser.parse_args()
    metadata = json.loads(args.metadata.read_text())
    artifact_id = metadata["id"]
    ranges = ArtifactRanges(signed_location(artifact_id), metadata["size_in_bytes"])
    with zipfile.ZipFile(ranges) as archive:
        receipt = json.loads(archive.read("native-contract-receipt.json"))
        identity = receipt["identity"]
        if (
            receipt["schema"] != "native-contract-artifact/v1"
            or identity["commit"] != args.compiled_commit
            or str(identity["run"]) != str(metadata["workflow_run"]["id"])
        ):
            raise ValueError("native contract receipt does not identify the requested producer")
        name = args.member
        member = archive.getinfo(name)
        expected = receipt["files"][name]
        if not 0 < member.file_size <= 768 * 1024 * 1024 or member.file_size != expected["bytes"]:
            raise ValueError("native contract member differs from the bounded producer receipt")
        print(f"{name}: {member.file_size} executable bytes, {member.compress_size} transport bytes", flush=True)
        if args.inspect:
            return
        digest = hashlib.sha256()
        with archive.open(member) as source, tempfile.NamedTemporaryFile(
            dir=args.destination.parent
        ) as destination:
            for block in iter(lambda: source.read(4 * 1024 * 1024), b""):
                destination.write(block)
                digest.update(block)
            if digest.hexdigest() != expected["sha256"]:
                raise ValueError("native contract bytes differ from the producer receipt")
            destination.flush()
            temporary = Path(destination.name)
            temporary.chmod(0o700)
            args.destination.hardlink_to(temporary)
        print(f"verified {digest.hexdigest()} across {ranges.bytes_received} artifact bytes")


if __name__ == "__main__":
    main()
