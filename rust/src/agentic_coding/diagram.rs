//! Third agentic recipe — driving Formal AI to *generate the mermaid diagrams of
//! its own agentic recipes* (issue #538).
//!
//! The maintainer asked, among the issue's broader axes, for a *"generated mermaid
//! diagram split into parts"* giving a high-level visual overview, and a diagram of
//! *"what exactly happens when the input to the system \[comes\] from different
//! places"*. This module is the smallest real, tested slice of that axis: the
//! diagrams are **generated from the planner's own recipe definitions**
//! (`RECIPES`) rather than hand-drawn, so they cannot drift from what the code
//! actually does, and the *same* in-repo Agent CLI that enriched the tomato and
//! potato meanings writes them.
//!
//! It is a non-lexeme axis on purpose: it proves the "drive the Agent-CLI recipe"
//! method generalises beyond editing meaning data to producing *architecture
//! documentation*, from a differently worded natural-language request
//! ([`DIAGRAM_TASK`]). Neural inference stays a NON-GOAL — the document is a pure
//! function of the recipe table below.

use std::fmt::Write as _;

use super::wording::agentic_message;

struct Recipe {
    key: &'static str,
    issue: &'static str,
    tools: &'static [&'static str],
}

impl Recipe {
    fn wording(&self, field: &str) -> String {
        agentic_message(&format!("diagram_{}_{field}", self.key), &[])
    }
}

/// The diagrams describe the first four recipes from the issue #468/#538
/// slice of `planner::plan_chat_step`.
const RECIPES: &[Recipe] = &[
    Recipe {
        key: "formalize",
        issue: "#468",
        tools: &["web_search", "web_fetch", "write_file", "run_command"],
    },
    Recipe {
        key: "meaning",
        issue: "#538",
        tools: &["web_search", "web_fetch", "write_file", "run_command"],
    },
    Recipe {
        key: "diagram",
        issue: "#538",
        tools: &["write_file", "run_command"],
    },
    Recipe {
        key: "self_ast",
        issue: "#538",
        tools: &["write_file", "run_command"],
    },
];

/// A *differently worded* request for the diagram recipe.
///
/// Distinct natural language from the tomato/potato tasks is the maintainer's
/// generality check: the router must recognise the intent from the words, not from
/// a hardcoded string.
pub const DIAGRAM_TASK: &str = "Generate the mermaid diagrams of our agentic recipes, split into \
                                parts, as a visual overview of how Formal AI drives its own tools.";

/// The workspace path the planner writes the generated diagram document to.
pub const DIAGRAM_PATH: &str = "agentic-recipes.md";

/// Keywords that mark a user turn as the diagram-generation task.
const DIAGRAM_KEYWORDS: [&str; 4] = ["mermaid", "diagram", "visual overview", "flowchart"];

/// Whether `prompt` asks to generate the agentic-recipe diagrams (issue #538);
/// a keyword inside a quoted literal is payload and does not count.
#[must_use]
pub fn is_diagram_task(prompt: &str) -> bool {
    // A cue inside a quoted literal is payload, not the request: "Insert the
    // line 'js/mermaid.bundle.js' after …" is an edit (PR #1188 dogfood T70).
    let mut unquoted = String::new();
    let mut cursor = 0;
    for segment in crate::normal_markov::quoted_segment_spans(prompt) {
        unquoted.push_str(&prompt[cursor..segment.start]);
        unquoted.push(' ');
        cursor = segment.end;
    }
    unquoted.push_str(&prompt[cursor..]);
    let lower = unquoted.to_lowercase();
    DIAGRAM_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
}

/// Render the overview part: how one task is routed to a recipe by keywords.
fn render_overview(out: &mut String) {
    let _ = writeln!(out, "## Part 1 — Overview: how a task is routed\n");
    let _ = writeln!(out, "```mermaid");
    let _ = writeln!(out, "flowchart TD");
    let _ = writeln!(
        out,
        "    user[\"user task\"] --> router{{\"plan_chat_step<br/>routes by keywords\"}}"
    );
    for recipe in RECIPES {
        let _ = writeln!(
            out,
            "    router -->|\"{route}\"| {key}[\"{title} ({issue})\"]",
            route = recipe.wording("route"),
            key = recipe.key,
            title = recipe.wording("title"),
            issue = recipe.issue,
        );
    }
    let _ = writeln!(
        out,
        "    router -->|\"otherwise\"| solver[\"ordinary solver text\"]"
    );
    let _ = writeln!(out, "```\n");
}

/// Render one recipe part: its tool steps as a left-to-right state machine.
fn render_recipe(out: &mut String, part: usize, recipe: &Recipe) {
    let _ = writeln!(
        out,
        "## Part {part} — Recipe: {title} ({issue})\n",
        title = recipe.wording("title"),
        issue = recipe.issue,
    );
    let _ = writeln!(out, "```mermaid");
    let _ = writeln!(out, "flowchart LR");
    let mut previous = format!("{}_task", recipe.key);
    let _ = writeln!(
        out,
        "    {previous}[\"user task<br/>({route})\"]",
        route = recipe.wording("route")
    );
    for (index, tool) in recipe.tools.iter().enumerate() {
        let node = format!("{}_{}", recipe.key, index);
        let _ = writeln!(
            out,
            "    {previous} --> {node}[\"{number}. {tool}<br/>{note}\"]",
            number = index + 1,
            note = recipe.wording(&format!("step_{index}")),
        );
        previous = node;
    }
    let final_node = format!("{}_final", recipe.key);
    let _ = writeln!(
        out,
        "    {previous} --> {final_node}([\"final answer<br/>the artifact inline\"])"
    );
    let _ = writeln!(out, "```\n");
}

/// Render the whole generated diagram document (Markdown with mermaid parts).
///
/// The document is deterministic and ends with a single trailing newline. It is
/// what the Agent CLI writes to [`DIAGRAM_PATH`] and what `docs/diagrams/agentic-recipes.md`
/// is committed as, asserted byte-for-byte in the issue-#538 tests.
#[must_use]
pub fn render_document() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# Formal AI agentic recipes (generated)\n");
    let _ = writeln!(
        out,
        "<!-- Generated by the Formal AI Agent CLI (`formal-ai agent`) from the planner's own"
    );
    let _ = writeln!(
        out,
        "     recipe table (rust/src/agentic_coding/diagram.rs). Do not hand-edit; regenerate. -->\n"
    );
    let _ = writeln!(
        out,
        "A high-level, split-into-parts visual overview of how Formal AI drives its own agentic\n\
         CLI to complete a task. Part 1 shows how a request is routed to a recipe; each later part\n\
         details what happens for input handled by that recipe — the deterministic\n\
         `search -> fetch -> write -> verify -> final` state machine in `rust/src/agentic_coding/`.\n\n\
         Scope: these diagrams cover the issue #468/#538 recipe slice. The planner also routes\n\
         self-heal, dreaming-audit, source-graph, ledger, explain, change-request, repair-strategy,\n\
         rebuild, Google-Trends learning/catalog, question-catalog, and the general capability\n\
         router — the `otherwise` edge in Part 1 stands for all of them.\n"
    );
    render_overview(&mut out);
    for (offset, recipe) in RECIPES.iter().enumerate() {
        render_recipe(&mut out, offset + 2, recipe);
    }
    // Exactly one trailing newline (the last writeln! above already left a blank
    // line); trim any extra so the file satisfies the end-of-file-fixer hook.
    format!("{}\n", out.trim_end())
}

/// The self-contained final answer: a natural-language summary plus the generated
/// document inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    format!(
        "Generated the agentic-recipe mermaid diagrams, split into {parts} parts, from the \
         planner's own recipe table — a visual overview of how Formal AI drives its own tools.\n\n\
         Generated document ({DIAGRAM_PATH}):\n\n{document}",
        parts = RECIPES.len() + 1,
        document = document.trim_end(),
    )
}
