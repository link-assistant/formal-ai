//! Source-only compiler operation proof; compiler execution remains in GitHub CI.
use std::path::PathBuf;
use std::process::Command;

pub fn require_compiler_contract() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let result = Command::new("node")
        .args(["--test", "rust/tests/web/selected-source-wasm.test.mjs"])
        .current_dir(root)
        .output()
        .expect("selected-source compiler operation controls must run");
    assert!(
        result.status.success(),
        "selected-source compiler operands, receipts and operation ordering must hold: {} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
