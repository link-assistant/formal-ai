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


def observe(codeql, databases, output):
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
        query = Path(__file__).with_name("MacroContexts.ql")
        manifest.update({"database": str(database), "databaseMetadata": identity(database / "codeql-database.yml"),
                         "codeql": identity(codeql), "query": identity(query), "queryPack": identity(packs[0])})
        (output / "codeql-database.yml").write_bytes((database / "codeql-database.yml").read_bytes())
        (output / "qlpack.yml").write_bytes(packs[0].read_bytes())
        for name in ["codeql-pack.lock.yml", "qlpack.lock.yml"]:
            if (pack / name).is_file():
                manifest[name] = identity(pack / name)
                (output / name).write_bytes((pack / name).read_bytes())
        (output / "MacroContexts.ql").write_bytes(query.read_bytes())
        run("version", ["version", "--format=json"])
        # Inherit the producer's exact bundled dependencies without downloading
        # a different library. The temporary query is removed even on failure.
        with tempfile.TemporaryDirectory(prefix="formal-ai-macro-contexts-", dir=pack) as temporary:
            installed = Path(temporary) / query.name
            installed.write_bytes(query.read_bytes())
            run("query", ["query", "run", "--database=" + str(database),
                          "--output=" + str(output / "macro-contexts.bqrs"),
                          "--threads=2", "--ram=2048", "--max-disk-cache=512",
                          "--warnings=error", str(installed)])
        run("decode", ["bqrs", "decode", "--format=json", "--entities=all",
                       "--output=" + str(output / "macro-contexts.json"), str(output / "macro-contexts.bqrs")])
        decoded = json.loads((output / "macro-contexts.json").read_text())
        table = decoded["#select"]
        if len(table["columns"]) != 8 or not table["tuples"] or any(len(row) != 8 for row in table["tuples"]):
            raise ValueError("Missing or malformed macro observation table")
        manifest.update({"complete": True, "rows": len(table["tuples"]),
                         "result": identity(output / "macro-contexts.json"),
                         "bqrs": identity(output / "macro-contexts.bqrs")})
    finally:
        (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--codeql", type=Path, required=True)
    parser.add_argument("--databases", type=json.loads, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(observe(args.codeql, args.databases, args.output.resolve())))
