//! Agent CLI-authored red regression for GitHub issue #707.

use std::fs;

const REQUIRED: [&str; 12] = [
    "fs.read",
    "fs.write",
    "fs.list",
    "fs.move",
    "shell.run",
    "http.fetch",
    "http.post",
    "dom.query",
    "dom.extract",
    "archive.pack",
    "archive.unpack",
    "process.status",
];

#[test]
fn computer_use_primitive_taxonomy_is_seeded() {
    let seed = fs::read_to_string("data/seed/tools.lino").expect("tool registry");
    for primitive in REQUIRED {
        assert!(
            seed.contains(&format!("name {primitive}")),
            "missing primitive {primitive}"
        );
    }
}