//! Regenerate the committed workspace self-AST census (issue #673).
//!
//! ```bash
//! cargo run --example regenerate_self_ast_census
//! ```
//!
//! The census is a pure function of the owned sources, so this only ever rewrites
//! the documents whose modules actually changed — the incremental property the
//! issue asks for — and deletes documents whose module is gone.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use formal_ai::self_ast_census::{workspace, CENSUS_DIR};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let documents = workspace().documents();
    let expected: BTreeSet<String> = documents.iter().map(|(path, _)| path.clone()).collect();

    let mut written = 0_usize;
    for (path, contents) in &documents {
        let absolute = root.join(path);
        if fs::read_to_string(&absolute).is_ok_and(|current| &current == contents) {
            continue;
        }
        if let Some(parent) = absolute.parent() {
            fs::create_dir_all(parent).expect("create census directory");
        }
        fs::write(&absolute, contents).expect("write census document");
        written += 1;
    }

    let mut removed = 0_usize;
    let mut orphans = Vec::new();
    collect_lino(&root.join(CENSUS_DIR), &mut orphans);
    for absolute in orphans {
        let relative = absolute
            .strip_prefix(root)
            .expect("census document lives in the repository")
            .to_string_lossy()
            .replace('\\', "/");
        if !expected.contains(&relative) {
            fs::remove_file(&absolute).expect("remove orphaned census document");
            removed += 1;
        }
    }

    eprintln!(
        "self-AST census: {} documents ({written} rewritten, {removed} removed)",
        documents.len()
    );
}

fn collect_lino(directory: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_lino(&path, out);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("lino") {
            out.push(path);
        }
    }
}
