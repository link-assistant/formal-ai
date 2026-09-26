//! Agent CLI-authored red regression for GitHub issue #707.

use std::fs;

fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate root parent")
        .to_path_buf()
}

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
    let seed = fs::read_to_string(root().join("data/seed/tools.lino")).expect("tool registry");
    for primitive in REQUIRED {
        assert!(
            seed.contains(&format!("name {primitive}")),
            "missing primitive {primitive}"
        );
    }
}
