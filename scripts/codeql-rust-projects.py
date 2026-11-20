#!/usr/bin/env python3
"""Give CodeQL's Rust analyzer the real standalone-script and worker contexts.

CodeQL discovers only ancestor manifests. Independent rust-script dependencies
cannot be put in one Cargo workspace: compatible, conflicting exact pins then
fail resolution. Build each distinct declared dependency context separately and
combine its resolved crate graph in scripts/rust-project.json. Cargo compiles
only dependency collectors, never the scripts or the main application here.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib


FENCE = re.compile(r"^//!\s*```cargo[^\S\n]*\n(.*?)^//!\s*```", re.M | re.S)


def inline_manifest(path):
    source = path.read_text()
    match = FENCE.search(source)
    if match:
        text = "\n".join(re.sub(r"^//! ?", "", line) for line in match[1].splitlines())
        manifest = tomllib.loads(text)
    elif re.match(r"^#![ \t]*/", source) and "rust-script" in source.splitlines()[0]:
        manifest = {}
    else:
        return None  # A module: its declaring script supplies its dependencies.
    unexpected = set(manifest) - {"package", "dependencies"}
    if unexpected or set(manifest.get("package", {})) - {"edition"}:
        raise ValueError(f"{path}: unsupported inline manifest fields; extend the context adapter: {manifest}")
    for dependency in manifest.get("dependencies", {}).values():
        if isinstance(dependency, dict) and any(k in dependency for k in ("path", "workspace", "optional")):
            raise ValueError(f"{path}: dependency requires an explicit context adapter: {dependency}")
    return {"edition": manifest.get("package", {}).get("edition", "2021"),
            "dependencies": manifest.get("dependencies", {})}


def contexts(root):
    groups = {}
    files = sorted((root / "scripts").rglob("*.rs"))
    for path in files:
        context = inline_manifest(path)
        if context is None:
            continue
        key = json.dumps(context, sort_keys=True, separators=(",", ":"))
        group = groups.setdefault(key, {**context, "sources": []})
        group["sources"].append(str(path.resolve()))
    if not groups:
        raise ValueError("No standalone Rust script contexts found")
    return list(groups.values()), [str(path.resolve()) for path in files]


def toml_value(value):
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return str(value)
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(v) for v in value) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(toml_value(k) + " = " + toml_value(v) for k, v in value.items()) + " }"
    raise ValueError(f"Unsupported manifest value: {value!r}")


def collector_manifest(group, name):
    return (f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = {toml_value(group["edition"])}\n'
            'publish = false\n[workspace]\n[lib]\npath = "collector.rs"\n[dependencies]\n' +
            "".join(toml_value(k) + " = " + toml_value(v) + "\n" for k, v in group["dependencies"].items()))


def rustc_environment(package, target_name):
    version = package["version"].split("-", 1)
    numbers = version[0].split(".")
    return {"CARGO_MANIFEST_DIR": str(Path(package["manifest_path"]).parent),
            "CARGO_PKG_NAME": package["name"], "CARGO_PKG_VERSION": package["version"],
            "CARGO_PKG_VERSION_MAJOR": numbers[0], "CARGO_PKG_VERSION_MINOR": numbers[1],
            "CARGO_PKG_VERSION_PATCH": numbers[2], "CARGO_PKG_VERSION_PRE": version[1] if len(version) > 1 else "",
            "CARGO_PKG_AUTHORS": ":".join(package.get("authors", [])),
            "CARGO_PKG_DESCRIPTION": package.get("description") or "",
            "CARGO_PKG_HOMEPAGE": package.get("homepage") or "",
            "CARGO_PKG_REPOSITORY": package.get("repository") or "",
            "CARGO_PKG_LICENSE": package.get("license") or "",
            "CARGO_CRATE_NAME": target_name.replace("-", "_")}


def dependency_graph(metadata, messages, host):
    """Use Cargo's resolved versions, dependency aliases, features and build data."""
    packages = {p["id"]: p for p in metadata["packages"]}
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    root_id = metadata["resolve"]["root"]
    build_data, macros = {}, {}
    for message in messages:
        pid = message.get("package_id")
        if message.get("reason") == "build-script-executed":
            build_data[pid] = message
        if message.get("reason") == "compiler-artifact" and "proc-macro" in message["target"]["kind"]:
            dylibs = [p for p in message["filenames"] if Path(p).suffix in {".so", ".dylib", ".dll"}]
            if len(dylibs) != 1 or not Path(dylibs[0]).is_file():
                raise ValueError(f"Missing or ambiguous compiled proc-macro: {pid}")
            macros[pid] = dylibs[0]

    def normal_deps(pid):
        # --filter-platform already discards dependencies for other platforms.
        return [d for d in nodes[pid]["deps"] if any(k["kind"] is None for k in d["dep_kinds"])]

    reachable = set()

    def visit(pid):
        if pid in reachable:
            return
        reachable.add(pid)
        for dep in normal_deps(pid):
            visit(dep["pkg"])

    for dep in normal_deps(root_id):
        visit(dep["pkg"])
    order = sorted(reachable)
    indices = {pid: i for i, pid in enumerate(order)}

    def dependencies(pid):
        return [{"crate": indices[d["pkg"]], "name": d["name"].replace("-", "_")} for d in normal_deps(pid)]

    crates = []
    for pid in order:
        package, node = packages[pid], nodes[pid]
        libs = [t for t in package["targets"] if any(k in {"lib", "rlib", "dylib", "staticlib", "cdylib", "proc-macro"} for k in t["kind"])]
        if len(libs) != 1:
            raise ValueError(f"Expected exactly one library target for {pid}")
        target = libs[0]
        build = build_data.get(pid, {})
        has_build_script = any("custom-build" in t["kind"] for t in package["targets"])
        if has_build_script and not build:
            raise ValueError(f"Build script output absent: {pid}")
        env = rustc_environment(package, target["name"])
        env.update(dict(build.get("env", [])))
        if build.get("out_dir"):
            env["OUT_DIR"] = build["out_dir"]
        crate = {"display_name": target["name"], "root_module": target["src_path"],
                 "edition": target["edition"], "version": package["version"],
                 "deps": dependencies(pid), "cfg": [f'feature="{f}"' for f in node["features"]] + build.get("cfgs", []),
                 "env": env, "target": host, "is_workspace_member": False,
                 "source": {"include_dirs": [str(Path(package["manifest_path"]).parent)], "exclude_dirs": []}}
        if build.get("out_dir"):
            crate["source"]["include_dirs"].append(build["out_dir"])
        if "proc-macro" in target["kind"]:
            if pid not in macros:
                raise ValueError(f"Compiled proc-macro absent: {pid}")
            crate.update(is_proc_macro=True, proc_macro_dylib_path=macros[pid])
        crates.append(crate)
    return crates, dependencies(root_id)


def append_graph(crates, dependencies, graph):
    offset = len(graph)
    for crate in crates:
        crate["deps"] = [{**dep, "crate": dep["crate"] + offset} for dep in crate["deps"]]
    graph.extend(crates)
    return [{**dep, "crate": dep["crate"] + offset} for dep in dependencies]


def harmonize_sources(graph):
    # Different Cargo contexts can use the same library with distinct features.
    # rust-analyzer requires identical source roots for any shared source files.
    sources = {}
    for crate in graph:
        source = crate["source"]
        key = source["include_dirs"][0]
        sources.setdefault(key, set()).update(source["include_dirs"])
    for crate in graph:
        crate["source"]["include_dirs"] = sorted(sources[crate["source"]["include_dirs"][0]])


def wasm_project(root, sysroot, sysroot_src):
    worker = root / "js/wasm-worker"
    build = (worker / "build.sh").read_text()
    edition = re.findall(r"--edition[= ](\d+)", build)
    target = re.findall(r"--target[= ]([a-z0-9-]+)", build)
    if edition != ["2024"] or target != ["wasm32-unknown-unknown"] or "--crate-type cdylib" not in build:
        raise ValueError("WASM worker build changed; update its analysis-context adapter")
    return {"sysroot": sysroot, "sysroot_src": sysroot_src, "crates": [{
        "display_name": "formal_ai_worker", "root_module": str(worker / "src/lib.rs"),
        "edition": edition[0], "target": target[0], "deps": [], "cfg": [],
        "env": {}, "is_workspace_member": True,
        "source": {"include_dirs": [str(worker / "src"), str(root / "rust/src")], "exclude_dirs": []}}]}


def command(argv, output, env):
    with output.open("wb") as log:
        result = subprocess.run(argv, stdout=log, stderr=subprocess.PIPE, env=env)
    output.with_suffix(output.suffix + ".stderr").write_bytes(result.stderr)
    if result.returncode:
        raise RuntimeError(f"Command failed ({result.returncode}); inspect {output} and its .stderr: {argv}")
    return output.read_text()


def prepare(root, output):
    root, output = root.resolve(), output.resolve()
    toolchain = os.environ["CODEQL_RUST_SYSROOT_TOOLCHAIN"]
    sysroot = os.environ["CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT"]
    sysroot_src = os.environ["CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT_SRC"]
    if not Path(sysroot_src).is_dir():
        raise ValueError("Verified rust-src directory is absent")
    targets = [root / "scripts/rust-project.json", root / "js/wasm-worker/rust-project.json"]
    if any(p.exists() for p in targets):
        raise ValueError("Refusing to overwrite existing Rust analysis manifests")
    output.mkdir(parents=True, exist_ok=False)
    env = {**os.environ, "CARGO_BUILD_JOBS": "1", "CARGO_INCREMENTAL": "0", "CARGO_PROFILE_DEV_DEBUG": "0"}
    prefix = ["rustup", "run", toolchain]
    info = command(prefix + ["rustc", "-vV"], output / "rustc-version.txt", env)
    host = re.search(r"^host: (\S+)$", info, re.M).group(1)
    groups, all_sources = contexts(root)
    graph, receipt = [], {"toolchain": toolchain, "host": host, "groups": [], "sourceFiles": all_sources}
    for group in groups:
        identity = hashlib.sha256(json.dumps(group["dependencies"], sort_keys=True).encode() + group["edition"].encode()).hexdigest()[:16]
        directory = output / identity
        directory.mkdir()
        manifest = directory / "Cargo.toml"
        manifest.write_text(collector_manifest(group, "codeql-context-" + identity))
        (directory / "collector.rs").write_text("// Dependency build context only; maintained sources remain at their original paths.\n")
        if group["dependencies"]:
            metadata = json.loads(command(prefix + ["cargo", "metadata", "--format-version=1", "--filter-platform", host,
                                                    "--manifest-path", str(manifest)], directory / "metadata.json", env))
            built = command(prefix + ["cargo", "check", "--locked", "--lib", "--message-format=json", "--manifest-path", str(manifest),
                                      "--target", host, "--target-dir", str(output / "target")], directory / "build.jsonl", env)
            messages = [json.loads(line) for line in built.splitlines() if line.strip()]
            libraries, dependencies = dependency_graph(metadata, messages, host)
            dependencies = append_graph(libraries, dependencies, graph)
        else:
            dependencies = []
        root_package = {"name": "codeql-context-" + identity, "version": "0.1.0", "manifest_path": str(manifest)}
        for source in group["sources"]:
            graph.append({"display_name": Path(source).stem.replace("-", "_"), "root_module": source,
                          "edition": group["edition"], "deps": dependencies, "cfg": ["test"],
                          "env": rustc_environment(root_package, Path(source).stem), "target": host, "is_workspace_member": True,
                          "source": {"include_dirs": [str(root / "scripts")], "exclude_dirs": []}})
        receipt["groups"].append({"identity": identity, **group, "manifest": str(manifest)})
    harmonize_sources(graph)
    project = {"sysroot": sysroot, "sysroot_src": sysroot_src, "crates": graph}
    worker = wasm_project(root, sysroot, sysroot_src)
    # Publish only after all dependency contexts have been resolved and built.
    targets[0].write_text(json.dumps(project, indent=2) + "\n")
    targets[1].write_text(json.dumps(worker, indent=2) + "\n")
    receipt["projects"] = [{"path": str(p), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in targets]
    (output / "contexts.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"standaloneScriptRoots": sum(len(g["sources"]) for g in groups),
                      "dependencyContexts": len(groups), "crates": len(graph), "workerTarget": "wasm32-unknown-unknown"}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        prepare(args.root, args.output)
    except (ValueError, RuntimeError, OSError, KeyError, AttributeError) as error:
        print(f"Rust analysis context unavailable: {error}", file=sys.stderr)
        sys.exit(1)
