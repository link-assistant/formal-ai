#!/usr/bin/env python3
"""Issue #1021 — report which declared dependency is behind its newest stable release.

Written after a miss, not before one. The first pass of this branch's dependency
refresh took `browser-commander` 0.16.0 and called it current; 0.16.1 had been on
the registry since 2026-08-02T14:39Z, six and a half hours after 0.16.0. Nothing
caught it, because nothing was looking: the gates check that the tree *audits*
and *builds*, which both 0.16.0 and 0.16.1 do. "Newest stable" is a claim about
the registry at the moment of the bump, and it has to be read from the registry.

This is a tool, not a gate, and it is deliberately not wired into
`data/meta/ci-gates/`. A check that fails whenever an unrelated maintainer
publishes would turn every pull request red for a reason no pull request caused.
Run it when you refresh dependencies; read the OLD rows as a work list.

Two traps it exists to encode, both of which produced a wrong answer by hand:

  * Cargo renames. `mem = { package = "platform-mem", ... }` must be looked up as
    `platform-mem`. Querying the manifest key finds an unrelated crate called
    `mem` whose 0.5.0 makes a current dependency look three minors stale.
  * npm dist-tags. `latest` is not "the highest version" — it is wherever the
    maintainer pointed it. electron-builder publishes 27.0.0-alpha on `next`,
    26.15.7 on `v26`, and points `latest` at 26.15.3, so the manifest's
    `^26.15.7` reads as *behind* `latest` while being ahead of it. This script
    compares against the highest non-prerelease version instead. Reading the
    version list rather than the tag needs SemVer's definition of a pre-release
    and not a list of the usual words: `@vscode/vsce` numbers its `next` line
    `3.9.3-0` through `3.9.3-5`, which no alpha/beta/rc pattern would catch.

Usage (needs network; no arguments):

    python3 experiments/issue-1021-dependency-freshness/check.py

Recorded result on 2026-09-01, run from the repository root:
    31 crates checked, 0 behind newest stable
    32 npm specs checked, 0 behind newest stable
    (2 registry-verified holds; 5 floating @link-assistant/ specs skipped)
"""

import json
import re
import subprocess
import sys
import tomllib
import urllib.request
from functools import lru_cache

CRATES_UA = {"User-Agent": "formal-ai-dep-check (link.assistant.team@proton.me)"}
# SemVer's own rule, not a keyword list: everything after the first `-` is a
# pre-release identifier. A list of names misses `@vscode/vsce@3.9.3-5`, whose
# stable line stops at 3.9.2 and whose `next` tag counts up in bare integers.
PRERELEASE = re.compile(r"^\d+\.\d+\.\d+-")

# Packages in the `@link-assistant/` scope stay floating, per the rule this
# repository already wrote down: they are released from sibling repositories and
# pinning them here would freeze the very integration this crate exists to test.
FLOATING_SCOPE = "@link-assistant/"

# Fail-closed holds for broken upstream releases. A listed version is not
# accepted on trust: `verified_npm_hold` checks both that the held release has a
# complete optional package set and that the newest stable release still names
# packages which the registry does not contain. Once upstream publishes those
# artifacts, the row becomes OLD and this hold has to be removed.
NPM_UPSTREAM_HOLDS = {
    ("@kreuzberg/html-to-markdown-node", "3.5.5"),
}

NPM_MANIFESTS = [
    "desktop/package.json",
    "experiments/agent_cli_e2e/issue_819_tui/package.json",
    "experiments/opencode_vscode_e2e/driver/package.json",
    "package.json",
    "tests/e2e/package.json",
    "vscode/package.json",
]
# `dev/log/` and `docs/case-studies/` also contain package.json files. Those are
# archived copies kept as evidence of what a run saw at the time, and refreshing
# them would destroy the record. They are not listed above on purpose.

SPEC_SECTIONS = ("dependencies", "devDependencies", "optionalDependencies",
                 "peerDependencies", "overrides", "resolutions")


def satisfied_by(spec: str, latest: str) -> bool:
    """True when `spec` already names `latest`, at whatever precision it pins to."""
    pinned = spec.lstrip("^~=>< ")
    # A manifest may pin `tokio = "1"` against a latest of `1.53.1`, or
    # `sha2 = "0.11"` against `0.11.0`; either direction of prefix counts.
    return latest.startswith(pinned) or pinned.startswith(latest)


def cargo_specs(path: str = "Cargo.toml") -> dict[str, tuple[str, str]]:
    """Map manifest key -> (real crate name, version spec), following `package =`."""
    manifest = tomllib.load(open(path, "rb"))
    tables = [manifest, *manifest.get("target", {}).values()]
    found: dict[str, tuple[str, str]] = {}
    for table in tables:
        for section in ("dependencies", "dev-dependencies", "build-dependencies"):
            for key, value in table.get(section, {}).items():
                if isinstance(value, str):
                    found[key] = (key, value)
                elif isinstance(value, dict) and "version" in value:
                    found[key] = (value.get("package", key), value["version"])
                # A path or git dependency has no registry version to compare.
    return found


def crates_latest(crate: str) -> str:
    request = urllib.request.Request(
        f"https://crates.io/api/v1/crates/{crate}", headers=CRATES_UA
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        payload = json.load(response)
    return payload["crate"].get("max_stable_version") or "?"


def npm_specs() -> dict[str, list[tuple[str, str]]]:
    """Map package name -> [(manifest, spec)], flattening nested override blocks."""
    found: dict[str, list[tuple[str, str]]] = {}

    def walk(node: dict, manifest: str) -> None:
        for name, value in node.items():
            if isinstance(value, dict):
                walk(value, manifest)  # `overrides` nests one level per parent
            elif isinstance(value, str) and re.match(r"^[\^~=]?\d", value):
                found.setdefault(name, []).append((manifest, value))

    for manifest in NPM_MANIFESTS:
        document = json.load(open(manifest))
        for section in SPEC_SECTIONS:
            if section in document:
                walk(document[section], manifest)
    return found


def npm_latest(package: str) -> str:
    """Highest non-prerelease version, which is not necessarily the `latest` tag."""
    output = subprocess.run(
        ["npm", "view", package, "versions", "--json"],
        capture_output=True, text=True, timeout=120,
    ).stdout.strip()
    if not output:
        return "?"
    versions = json.loads(output)
    if isinstance(versions, str):
        versions = [versions]
    stable = [version for version in versions if not PRERELEASE.search(version)]
    return stable[-1] if stable else "?"


@lru_cache(maxsize=None)
def unpublished_optional_dependencies(package: str, version: str) -> tuple[str, ...]:
    """Return optional package specs declared by a release but absent from npm."""
    metadata = subprocess.run(
        ["npm", "view", f"{package}@{version}", "optionalDependencies", "--json"],
        capture_output=True, text=True,
    )
    if metadata.returncode != 0 or not metadata.stdout.strip():
        return ()
    dependencies = json.loads(metadata.stdout)
    if not isinstance(dependencies, dict):
        return ()
    missing = []
    for dependency, spec in sorted(dependencies.items()):
        published = subprocess.run(
            ["npm", "view", f"{dependency}@{spec}", "version", "--json"],
            capture_output=True, text=True,
        )
        if published.returncode != 0:
            missing.append(f"{dependency}@{spec}")
    return tuple(missing)


def verified_npm_hold(package: str, spec: str, latest: str) -> tuple[str, ...]:
    """Return the registry-proven missing artifacts that justify a known hold."""
    pinned = spec.lstrip("^~=>< ")
    if (package, pinned) not in NPM_UPSTREAM_HOLDS:
        return ()
    if unpublished_optional_dependencies(package, pinned):
        return ()
    return unpublished_optional_dependencies(package, latest)


def main() -> int:
    stale = 0

    print("== Cargo.toml")
    cargo = cargo_specs()
    for key in sorted(cargo):
        crate, spec = cargo[key]
        latest = crates_latest(crate)
        current = satisfied_by(spec, latest)
        stale += not current
        label = f"{key} (package = {crate})" if crate != key else key
        print(f"{'OK ' if current else 'OLD'} {label:<44} {spec:<12} latest={latest}")
    cargo_stale = stale
    print(f"\n{len(cargo)} crates checked, {cargo_stale} behind newest stable\n")

    print("== package.json")
    npm = npm_specs()
    skipped = 0
    checked = 0
    held = 0
    for name in sorted(npm):
        if name.startswith(FLOATING_SCOPE):
            for manifest, spec in npm[name]:
                skipped += 1
                print(f"SCP {name:<44} {spec:<12} floating by rule  {manifest}")
            continue
        latest = npm_latest(name)
        for manifest, spec in npm[name]:
            checked += 1
            current = satisfied_by(spec, latest)
            hold = () if current else verified_npm_hold(name, spec, latest)
            if hold:
                held += 1
                print(f"HLD {name:<44} {spec:<12} latest={latest:<12} "
                      f"missing={','.join(hold)}  {manifest}")
                continue
            stale += not current
            print(f"{'OK ' if current else 'OLD'} {name:<44} {spec:<12} "
                  f"latest={latest:<12} {manifest}")
    print(f"\n{checked} npm specs checked, {stale - cargo_stale} behind newest stable "
          f"({held} registry-verified holds; "
          f"{skipped} floating {FLOATING_SCOPE} specs skipped)")

    return 1 if stale else 0


if __name__ == "__main__":
    sys.exit(main())
