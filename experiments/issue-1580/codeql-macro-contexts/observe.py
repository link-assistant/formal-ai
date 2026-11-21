#!/usr/bin/env python3
"""Observe the existing hosted Rust database; never substitute for security checks."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile


def identity(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


PROFILES = {"macro-contexts": ("MacroContexts.ql", 8),
            "included-string-payloads": ("IncludedStringPayloads.ql", 9)}


def observe(codeql, databases, output, profile="macro-contexts"):
    output.mkdir(parents=True, exist_ok=False)
    manifest = {"complete": False, "commands": [], "semanticCoverageProven": False,
                "independentAuthorshipCredit": 0}

    def run(name, arguments):
        with (output / (name + ".stdout")).open("wb") as stdout, (output / (name + ".stderr")).open("wb") as stderr:
            result = subprocess.run([str(codeql), *arguments], stdout=stdout, stderr=stderr)
        manifest["commands"].append({"name": name, "argv": [str(codeql), *arguments],
                                     "exitCode": result.returncode,
                                     "stdout": identity(output / (name + ".stdout")),
                                     "stderr": identity(output / (name + ".stderr"))})
        if result.returncode:
            raise RuntimeError(f"CodeQL {name} failed with exit {result.returncode}; see preserved streams")

    try:
        if profile not in PROFILES:
            raise ValueError("Unknown macro observation profile")
        query_name, columns = PROFILES[profile]
        manifest["profile"] = profile
        database = Path(databases["rust"])
        if not database.is_absolute() or not (database / "codeql-database.yml").is_file():
            raise ValueError("Expected the completed Rust database reported by the analyze action")
        codeql = codeql.resolve(strict=True)
        packs = sorted((codeql.parent / "qlpacks/codeql/rust-queries").glob("*/qlpack.yml"))
        if len(packs) != 1:
            raise ValueError(f"Expected one bundled Rust query pack, found {len(packs)}")
        pack = packs[0].parent
        if not re.search(r"^name: *codeql/rust-queries *$", packs[0].read_text(), re.M):
            raise ValueError("Unexpected bundled query pack identity")
        query = Path(__file__).with_name(query_name)
        manifest.update({"database": str(database), "databaseMetadata": identity(database / "codeql-database.yml"),
                         "codeql": identity(codeql), "query": identity(query), "queryPack": identity(packs[0])})
        (output / "codeql-database.yml").write_bytes((database / "codeql-database.yml").read_bytes())
        (output / "qlpack.yml").write_bytes(packs[0].read_bytes())
        for name in ["codeql-pack.lock.yml", "qlpack.lock.yml"]:
            if (pack / name).is_file():
                manifest[name] = identity(pack / name)
                (output / name).write_bytes((pack / name).read_bytes())
        (output / query_name).write_bytes(query.read_bytes())
        run("version", ["version", "--format=json"])
        # query run sizes a child evaluator again; a launcher -J heap is not inherited.
        # Resolve the same real dataset policy first, preserving its authoritative output.
        memory_options = ["--ram=4096", "--off-heap-ram=0"]
        dataset = database / "db-rust"
        if not dataset.is_dir():
            raise ValueError("Missing completed Rust dataset for memory resolution")
        run("memory", ["resolve", "ram", "--format=json", "--dataset=" + str(dataset), *memory_options])
        resolved = json.loads((output / "memory.stdout").read_text())
        if not isinstance(resolved, list) or not all(isinstance(value, str) for value in resolved):
            raise ValueError("Malformed CodeQL memory resolution")
        heaps = [re.fullmatch(r"-J=?-Xmx([1-9][0-9]*)([mMgG])", value) for value in resolved]
        heaps = [match for match in heaps if match]
        heap_mib = int(heaps[0][1]) * (1024 if heaps[0][2].lower() == "g" else 1) if len(heaps) == 1 else 0
        manifest["resolvedMemory"] = {"totalMiB": 4096, "offHeapMiB": 0,
                                      "heapMiB": heap_mib, "arguments": resolved}
        if not 2048 <= heap_mib <= 4096 or any(value.startswith("--off-heap-ram=") and
                                             value != "--off-heap-ram=0" for value in resolved):
            raise ValueError("CodeQL resolved heap cannot cache loading stages inside the requested policy")
        # Inherit the producer's exact bundled dependencies without downloading
        # a different library. The temporary query is removed even on failure.
        with tempfile.TemporaryDirectory(prefix="formal-ai-macro-contexts-", dir=pack) as temporary:
            installed = Path(temporary) / query.name
            installed.write_bytes(query.read_bytes())
            run("query", ["query", "run", "--database=" + str(database),
                          "--output=" + str(output / (profile + ".bqrs")),
                          "--threads=2", *memory_options, "--max-disk-cache=512",
                          "--warnings=error", str(installed)])
        diagnostics = [line for line in (output / "query.stderr").read_text().splitlines()
                       if re.match(r"^(?:WARNING|WARN|ERROR):", line)]
        manifest["queryRuntimeDiagnostics"] = diagnostics
        if diagnostics:
            raise RuntimeError("CodeQL query emitted runtime diagnostics; see preserved query.stderr")
        run("decode", ["bqrs", "decode", "--format=json", "--entities=all",
                       "--output=" + str(output / (profile + ".json")), str(output / (profile + ".bqrs"))])
        decoded = json.loads((output / (profile + ".json")).read_text())
        table = decoded["#select"]
        if len(table["columns"]) != columns or not table["tuples"] or any(len(row) != columns for row in table["tuples"]):
            raise ValueError("Missing or malformed macro observation table")
        manifest.update({"complete": True, "rows": len(table["tuples"]),
                         "result": identity(output / (profile + ".json")),
                         "bqrs": identity(output / (profile + ".bqrs"))})
    finally:
        (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--codeql", type=Path, required=True)
    parser.add_argument("--databases", type=json.loads, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--profile", choices=PROFILES, default="macro-contexts")
    args = parser.parse_args()
    print(json.dumps(observe(args.codeql, args.databases, args.output.resolve(), args.profile)))
