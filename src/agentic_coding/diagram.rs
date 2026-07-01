//! Third agentic recipe — driving Formal AI to *generate the mermaid diagrams of
//! its own agentic recipes* (issue #538).
//!
//! The maintainer asked, among the issue's broader axes, for a *"generated mermaid
//! diagram split into parts"* giving a high-level visual overview, and a diagram of
//! *"what exactly happens when the input to the system [comes] from different
//! places"*. This module is the smallest real, tested slice of that axis: the
//! diagrams are **generated from the planner's own recipe definitions**
//! ([`RECIPES`]) rather than hand-drawn, so they cannot drift from what the code
//! actually does, and the *same* in-repo Agent CLI that enriched the tomato and
//! potato meanings writes them.
//!
//! It is a non-lexeme axis on purpose: it proves the "drive the Agent-CLI recipe"
//! method generalises beyond editing meaning data to producing *architecture
//! documentation*, from a differently worded natural-language request
//! ([`DIAGRAM_TASK`]). Neural inference stays a NON-GOAL — the document is a pure
//! function of the recipe table below.

use std::fmt::Write as _;

/// One step of a recipe, as the planner's state machine executes it.
#[derive(Debug, Clone, Copy)]
struct RecipeStep {
    /// The tool the planner emits for this step (`web_search`, `write_file`, …).
    tool: &'static str,
    /// What the step accomplishes, shown in the diagram node.
    note: &'static str,
}

/// One agentic recipe the deterministic planner knows how to walk.
#[derive(Debug, Clone, Copy)]
struct Recipe {
    /// Stable key used to build mermaid node ids (must be a valid id fragment).
    key: &'static str,
    /// Human-readable recipe title.
    title: &'static str,
    /// The issue the recipe was built for.
    issue: &'static str,
    /// The keyword hint the task router (`plan_chat_step`) matches to reach it.
    route: &'static str,
    /// The tool steps, in execution order (the terminal "final answer" is implied).
    steps: &'static [RecipeStep],
}

/// The recipes the planner can walk, mirroring `planner::plan_chat_step`. Adding a
/// recipe here changes the generated diagrams, so the document stays a faithful
/// picture of the planner's actual behaviour.
const RECIPES: &[Recipe] = &[
    Recipe {
        key: "formalize",
        title: "Formalize a text into a knowledge base",
        issue: "#468",
        route: "formaliz / knowledge base / fisherman",
        steps: &[
            RecipeStep {
                tool: "web_search",
                note: "find the source text",
            },
            RecipeStep {
                tool: "web_fetch",
                note: "read it; a tool error falls back to the canonical synopsis",
            },
            RecipeStep {
                tool: "write_file",
                note: "formalize prose into Links Notation",
            },
            RecipeStep {
                tool: "run_command",
                note: "cat the file to verify the write landed",
            },
        ],
    },
    Recipe {
        key: "meaning",
        title: "Make a meaning more detailed",
        issue: "#538",
        route: "more detailed / grammatical number / a known concept",
        steps: &[
            RecipeStep {
                tool: "web_search",
                note: "find the concept's Wikidata lexemes",
            },
            RecipeStep {
                tool: "web_fetch",
                note: "read the forms; recover the missing plural",
            },
            RecipeStep {
                tool: "write_file",
                note: "re-derive the grounded block, byte-for-byte the seed",
            },
            RecipeStep {
                tool: "run_command",
                note: "cat the block to verify",
            },
        ],
    },
    Recipe {
        key: "diagram",
        title: "Generate these diagrams",
        issue: "#538",
        route: "mermaid / diagram / visual overview",
        steps: &[
            RecipeStep {
                tool: "write_file",
                note: "render the mermaid parts from the recipe table",
            },
            RecipeStep {
                tool: "run_command",
                note: "cat the document to verify",
            },
        ],
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

/// Whether `prompt` asks to generate the agentic-recipe diagrams (issue #538).
#[must_use]
pub fn is_diagram_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
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
            route = recipe.route,
            key = recipe.key,
            title = recipe.title,
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
        title = recipe.title,
        issue = recipe.issue,
    );
    let _ = writeln!(out, "```mermaid");
    let _ = writeln!(out, "flowchart LR");
    let mut previous = format!("{}_task", recipe.key);
    let _ = writeln!(
        out,
        "    {previous}[\"user task<br/>({route})\"]",
        route = recipe.route
    );
    for (index, step) in recipe.steps.iter().enumerate() {
        let node = format!("{}_{}", recipe.key, index);
        let _ = writeln!(
            out,
            "    {previous} --> {node}[\"{number}. {tool}<br/>{note}\"]",
            number = index + 1,
            tool = step.tool,
            note = step.note,
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
        "     recipe table (src/agentic_coding/diagram.rs). Do not hand-edit; regenerate. -->\n"
    );
    let _ = writeln!(
        out,
        "A high-level, split-into-parts visual overview of how Formal AI drives its own agentic\n\
         CLI to complete a task. Part 1 shows how a request is routed to a recipe; each later part\n\
         details what happens for input handled by that recipe — the deterministic\n\
         `search -> fetch -> write -> verify -> final` state machine in `src/agentic_coding/`.\n"
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
