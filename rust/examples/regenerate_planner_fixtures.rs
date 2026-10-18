//! Regenerate the self-healing scenario; source CST is an on-demand artifact.

use formal_ai::agentic_coding::self_heal;

fn main() {
    std::fs::write(
        "data/meta/self-healing-case.lino",
        self_heal::render_document(),
    )
    .expect("write self-healing fixture");
}
