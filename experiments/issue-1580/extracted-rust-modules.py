#!/usr/bin/env python3
"""Bounded structural reproduction for extracted native modules.

Uses only rustc metadata on tiny dependency-free fixtures. Declaration text comes
from the maintained sources; test bodies and dependencies are deliberately not
compiled here. The real reasoning/workflow contracts remain hosted CI's gate.
No repository lint settings are changed.
"""
import argparse
import json
from pathlib import Path
import re
import resource
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
PARENTS = {
    "chat_surface": ("behavior_rule_grammar", "rule_details"),
    "debug_session": ("calculator_origins",),
}


def source(path, revision):
    if revision:
        return subprocess.check_output(
            ["git", "show", revision + ":" + path], cwd=ROOT, text=True
        )
    return (ROOT / path).read_text()


def limits():
    resource.setrlimit(resource.RLIMIT_AS, (1024**3, 1024**3))
    resource.setrlimit(resource.RLIMIT_STACK, (16 * 1024**2, 16 * 1024**2))
    resource.setrlimit(resource.RLIMIT_CPU, (15, 15))


def compile_fixture(directory):
    result = subprocess.run(
        ["rustc", "--edition=2024", "--crate-type=lib", "--emit=metadata",
         "--crate-name=extracted_module_probe", "-Dunused-imports", "-Adead-code",
         str(directory / "lib.rs"), "-o", str(directory / "probe.rmeta")],
        cwd=directory, text=True, capture_output=True, preexec_fn=limits,
    )
    return {"exitCode": result.returncode, "stdout": result.stdout, "stderr": result.stderr}


def module_case(directory, revision, focused, omit_child=False):
    directory.mkdir()
    specification = directory / "specification"
    specification.mkdir()
    for parent, children in PARENTS.items():
        original = source("rust/tests/unit/specification/" + parent + ".rs", revision)
        declarations = []
        for child in children:
            pattern = r'(?:#\[path = "[^"]+"\]\n)?mod ' + child + r";"
            matches = re.findall(pattern, original)
            assert len(matches) == 1, (parent, child, matches)
            declarations.append(matches[0])
            relative = "rust/tests/unit/specification/" + parent + "/" + child + ".rs"
            assert source(relative, revision), relative
            folder = specification / parent
            folder.mkdir(exist_ok=True)
            if not (omit_child and child == "rule_details"):
                (folder / (child + ".rs")).write_text("// Declaration-only fixture child.\n")
        (specification / (parent + ".rs")).write_text("\n".join(declarations) + "\n")
    (specification / "mod.rs").write_text("mod chat_surface;\nmod debug_session;\n")
    if focused:
        root = source("rust/tests/unit/reasoning_contracts.rs", revision)
        declarations = []
        for parent in PARENTS:
            pattern = r'#\[path = "specification/' + parent + r'\.rs"\]\nmod ' + parent + ";"
            matches = re.findall(pattern, root)
            assert len(matches) == 1, (parent, matches)
            declarations.append(matches[0])
        (directory / "lib.rs").write_text("\n".join(declarations) + "\n")
    else:
        (directory / "lib.rs").write_text("mod specification;\n")
    return compile_fixture(directory)


def renderer_case(directory, revision, omit_renderer=False):
    directory.mkdir()
    original = source("scripts/check-dependencies-latest.rs", revision)
    start = original.index('#[path = "check-dependencies-snapshot.rs"]')
    end = original.index("\nfn ecosystem_of(", start)
    declarations = original[start:end]
    (directory / "lib.rs").write_text(
        "mod dependency_gate {\npub struct Finding;\n" + declarations
        + "\npub fn exercise_io() { read_snapshot(); write_snapshot(); }\n}\n"
        + "pub fn exercise() { dependency_gate::exercise_io(); }\n"
    )
    (directory / "dependency_gate").mkdir()
    (directory / "dependency_gate/check-dependencies-snapshot.rs").write_text(
        "pub fn read_snapshot() {}\npub fn write_snapshot() {}\n"
        + ("" if omit_renderer else
           "pub fn render_lino(_: &[super::Finding], _: &[String]) -> String { String::new() }\n")
    )
    return compile_fixture(directory)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--before", required=True, help="Commit containing the observed CI failures.")
    options = parser.parse_args()
    observations = {}
    with tempfile.TemporaryDirectory(prefix="extracted-native-", dir=ROOT / "experiments") as temporary:
        directory = Path(temporary)
        for label, revision in [("before", options.before), ("current", None)]:
            for layout in ["ordinary", "focused"]:
                observations[label + "-" + layout] = module_case(
                    directory / (label + "-" + layout), revision, layout == "focused"
                )
            observations[label + "-renderer"] = renderer_case(directory / (label + "-renderer"), revision)
        observations["missing-child-control"] = module_case(
            directory / "missing-child-control", None, True, omit_child=True
        )
        observations["missing-renderer-control"] = renderer_case(
            directory / "missing-renderer-control", None, omit_renderer=True
        )
    print(json.dumps({
        "scope": "declaration-only structural fixtures; no full crate qualification",
        "before": options.before, "observations": observations,
        "addressSpaceLimitBytes": 1024**3, "stackLimitBytes": 16 * 1024**2,
        "cpuLimitSecondsPerCompiler": 15, "fullIssueComplete": False,
    }, indent=2), flush=True)
    assert observations["before-ordinary"]["exitCode"] == 0
    assert observations["before-focused"]["exitCode"] != 0
    assert "E0583" in observations["before-focused"]["stderr"]
    assert observations["before-renderer"]["exitCode"] != 0
    assert "unused import" in observations["before-renderer"]["stderr"]
    for label in ["current-ordinary", "current-focused", "current-renderer"]:
        assert observations[label]["exitCode"] == 0, (label, observations[label])
    assert observations["missing-child-control"]["exitCode"] != 0
    assert "rule_details" in observations["missing-child-control"]["stderr"]
    assert observations["missing-renderer-control"]["exitCode"] != 0
    assert "render_lino" in observations["missing-renderer-control"]["stderr"]


if __name__ == "__main__":
    main()
