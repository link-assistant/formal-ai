#!/usr/bin/env python3
"""Compare the published and current Rust declaration functions in a tiny crate."""

import argparse
from pathlib import Path
import re
import subprocess
import tomllib


def extract(source, version):
    functions = []
    for name in ("declared_string_array", "valid_identifier"):
        function = re.search(rf"(?ms)^fn {name}\(.+?^\}}", source)[0]
        functions.append(
            function.replace("declared_string_array(", f"{version}_declaration(").replace(
                "valid_identifier(", f"{version}_valid_identifier("
            )
        )
    return "\n".join(functions)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("published_commit")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    output = args.output
    (output / "src").mkdir(parents=True)
    path = Path("rust/src/agentic_coding/structured_edit.rs")
    before = subprocess.check_output(["git", "show", f"{args.published_commit}:{path}"], text=True)
    (output / "src/main.rs").write_text(
        """
#[derive(Clone, Copy)]
struct Region { open: usize, close: usize }
"""
        + extract(before, "published")
        + "\n"
        + extract(path.read_text(), "current")
        + r'''

fn match_regions(source: &str) -> Vec<Region> {
    source.match_indices("[\"brave\"]").map(|(open, text)| Region {
        open,
        close: open + text.len() - 1,
    }).collect()
}

fn main() {
    let name = "WEB_SEARCH_PROVIDERS";
    let source = "pub const WEB_SEARCH_PROVIDERS: &[&str] = &[\"brave\"];\n";
    let regions = match_regions(source);
    assert!(published_declaration(source, &regions, name).is_none());
    println!("Published Rust matcher rejected the hosted fixture");
    let anchor = current_declaration(source, &regions, name).expect("declared initializer");
    assert_eq!(&source[anchor.open..=anchor.close], "[\"brave\"]");
    println!("Corrected Rust matcher identifies the hosted fixture initializer");
    for rejected in [
        source.replace("const", "prefixconst"),
        source.replace(name, "DIFFERENT_NAME"),
        source.repeat(2),
    ] {
        assert!(current_declaration(&rejected, &match_regions(&rejected), name).is_none());
    }
    let static_source = source.replace("const", "static");
    assert!(current_declaration(&static_source, &match_regions(&static_source), name).is_some());
    println!("Static, token-boundary, wrong-name and duplicate-declaration controls passed");
}
'''
    )
    lock = Path("rust/Cargo.lock").read_text()
    packages = {"regex", "regex-syntax", "regex-automata", "memchr", "aho-corasick"}
    regex_version = next(item["version"] for item in tomllib.loads(lock)["package"] if item["name"] == "regex")
    header, *blocks = lock.split("[[package]]")
    pinned = [block for block in blocks if re.search(r'^\s*name = "([^"]+)"', block)[1] in packages]
    (output / "Cargo.lock").write_text(
        header + '[[package]]\nname = "declaration-probe"\nversion = "0.1.0"\n'
        'dependencies = ["regex"]\n\n' + "".join(f"[[package]]{block}" for block in pinned)
    )
    (output / "Cargo.toml").write_text(
        '[package]\nname = "declaration-probe"\nversion = "0.1.0"\nedition = "2024"\n'
        f'\n[dependencies]\nregex = "={regex_version}"\n'
    )


if __name__ == "__main__":
    main()
