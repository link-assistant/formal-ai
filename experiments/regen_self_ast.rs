// One-off regeneration of the self-AST fixtures after planner.rs changed (#680).
use formal_ai::agentic_coding::{self_ast, run_agentic_task, AST_TASK};

fn main() {
    // 1) The committed census document.
    std::fs::write("data/meta/self-ast.lino", self_ast::render_document()).unwrap();
    // 2) The committed Agent CLI session for the self-AST recipe.
    let outcome = run_agentic_task(AST_TASK).unwrap();
    let rendered = serde_json::to_string_pretty(&outcome.session_json()).unwrap();
    std::fs::write(
        "docs/case-studies/issue-538/agent-cli-session-self-ast.json",
        format!("{rendered}\n"),
    )
    .unwrap();
    eprintln!("regenerated self-ast census + session");
}
