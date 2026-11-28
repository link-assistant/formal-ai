#!/usr/bin/env python3
"""Recover exact source bytes for retained native AST histograms from Git.

This preserves historical parser regression inputs. It generates no native
counts and proves no current SDK native/JavaScript parser equivalence.
Run from any directory with Python 3 and the repository's full Git history.
"""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]
DESTINATION = ROOT / "experiments/issue-1580/fixtures/retained-ast-census"


def git(*arguments):
    return subprocess.check_output(["git", *arguments], cwd=ROOT)


def source_id(source):
    value = 0xcbf29ce484222325
    for byte in source:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def capture_sources():
    captures = []
    for document in sorted((ROOT / "data/meta/self-ast/src").rglob("*.lino")):
        raw = document.read_bytes()
        text = raw.decode("utf-8")
        if "    node_kinds\n" not in text:
            continue
        target = re.search(r"^  target (\S+)$", text, re.M)[1]
        expected = re.search(r"content_id source_module_([0-9a-f]{16})", text)[1]
        relative = document.relative_to(ROOT).as_posix()
        # Check document revisions in order; retain the first actual source
        # whose bytes match the source identity recorded in this document.
        candidates = git("log", "--format=%H", "--", relative).decode().splitlines()
        for commit in candidates:
            result = subprocess.run(["git", "show", f"{commit}:rust/{target}"],
                                    cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if result.returncode == 0 and source_id(result.stdout) == expected:
                source = result.stdout
                break
        else:
            raise RuntimeError(f"No captured source matches {relative}: {expected}")
        blob = git("rev-parse", f"{commit}:rust/{target}").decode().strip()
        captures.append({
            "document": relative, "target": target, "captureCommit": commit,
            "sourceBlob": blob, "sourceId": expected,
            "documentSha256": hashlib.sha256(raw).hexdigest(),
            "sourceSha256": hashlib.sha256(source).hexdigest(),
            "source": source.decode("utf-8"),
        })
    if len(captures) < 80:
        raise RuntimeError(f"Only {len(captures)} retained histograms; expected at least 80")
    DESTINATION.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(captures, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    archive = gzip.compress(payload, mtime=0)
    (DESTINATION / "sources.json.gz").write_bytes(archive)
    manifest = {
        "schemaVersion": 1,
        "scope": "Retained historical native histograms; not current SDK parser parity",
        "source": "Exact Git blobs matching committed source_module FNV-1a identities",
        "generator": "experiments/issue-1580/capture-retained-census-sources.py",
        "nativeCountsRegenerated": False, "captureCount": len(captures),
        "archive": "sources.json.gz", "archiveBytes": len(archive),
        "uncompressedBytes": len(payload), "archiveSha256": hashlib.sha256(archive).hexdigest(),
    }
    (DESTINATION / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    capture_sources()
