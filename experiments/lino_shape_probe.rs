//! Probe which Links Notation line shapes the canonical parser accepts, so the
//! workspace self-AST census (issue #673) emits a document that
//! `tests/unit/data_files.rs` can parse.
//!
//! Run with: `cargo run --example lino_shape_probe`

fn main() {
    let candidates = [
        ("two tokens", "root\n  key value\n"),
        ("three tokens", "root\n  src/agentic_coding/planner.rs full_ast 31\n"),
        (
            "nested three levels",
            "root\n  modules\n    src/agentic_coding/planner.rs\n      fidelity full_ast\n",
        ),
        (
            "symbol span row",
            "root\n  symbols\n    function\n      name plan_chat_step\n      lines 120 210\n",
        ),
        ("two integers", "root\n  span 120 210\n"),
    ];
    for (label, text) in candidates {
        match links_notation::parse_lino(text.trim()) {
            Ok(links) => println!("OK   {label}: {links:?}"),
            Err(error) => println!("FAIL {label}: {error}"),
        }
    }
}
