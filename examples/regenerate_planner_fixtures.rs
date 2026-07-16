//! Regenerate every committed artifact derived from the agentic planner source.

use formal_ai::agentic_coding::{run_agentic_task, self_ast, self_heal, AST_TASK};

fn main() {
    std::fs::write("data/meta/self-ast.lino", self_ast::render_document())
        .expect("write self-AST fixture");
    std::fs::write(
        "data/meta/self-healing-case.lino",
        self_heal::render_document(),
    )
    .expect("write self-healing fixture");

    let outcome = run_agentic_task(AST_TASK).expect("run self-AST recipe");
    let rendered =
        serde_json::to_string_pretty(&outcome.session_json()).expect("serialize self-AST session");
    std::fs::write(
        "docs/case-studies/issue-538/agent-cli-session-self-ast.json",
        format!("{rendered}\n"),
    )
    .expect("write self-AST session");
}
