//! The shared *generate → verify → final* recipe, and the self-inspection
//! recipes expressed through it.
//!
//! Every self-referential recipe Formal AI runs — the self-AST, the self-healing
//! case, the source-links map, the learning ledger, the grounded explanation, the
//! change request, the repair strategy, the rebuild plan, the question catalog and
//! the two Google-Trends recipes — has the same three-step shape: write a
//! generated document, read it back to verify it, then answer with what was
//! generated. They differ only in the document, so they share one planner
//! ([`plan_document_recipe`]) and one description of a recipe
//! ([`DocumentRecipe`]).
//!
//! These live beside [`super::planner`] rather than inside it: the planner is the
//! router that decides *which* recipe a request is, and this module is *what each
//! recipe does*. Keeping the two apart means adding a recipe never lengthens the
//! router.

use serde_json::json;

use super::change_request;
use super::diagram;
use super::dreaming_audit;
use super::explain;
use super::google_trends_catalog;
use super::google_trends_learning;
use super::ledger;
use super::meaning_detail;
use super::planner::{
    AgenticPlan, Capability, Progress, fetch_arguments, plan_one, tool_for, write_arguments,
};
use super::question_catalog;
use super::rebuild_plan;
use super::repair_strategy;
use super::self_ast;
use super::self_heal;
use super::source_links;
use crate::protocol::ChatMessage;

/// A self-referential *generate → verify → final* recipe expressed as data.
///
/// Every self-inspection recipe (diagram, self-AST, self-heal, source-links,
/// ledger, explain, change-request, repair-strategy, rebuild, question-catalog,
/// Google-Trends catalog, Google-Trends learning) has the *same* three-step shape:
/// write a generated document to `path`, verify it by running `verify_command`,
/// then answer with `final_answer`. They differ only in the document they generate,
/// so they are modelled as one struct and one planner
/// ([`plan_document_recipe`]) rather than a dozen copy-pasted functions — the exact
/// generalization the meta-algorithm is meant to embody.
pub(super) struct DocumentRecipe {
    /// The workspace-relative path the generated document is written to.
    pub(super) path: &'static str,
    /// The generated Links Notation document (a pure function of committed state).
    pub(super) document: String,
    /// The sandbox-allowlisted command that reads the document back for verification.
    pub(super) verify_command: String,
    /// The inline final answer returned once the write and verify steps are done.
    pub(super) final_answer: String,
}

/// Plan the next step of a [`DocumentRecipe`]: `write → verify → final`. Steps whose
/// capability the CLI did not advertise (or the conversation already satisfied) are
/// skipped, so the loop adapts to whatever subset of tools a given CLI exposes.
pub(super) fn plan_document_recipe(
    messages: &[ChatMessage],
    tool_names: &[&str],
    recipe: DocumentRecipe,
) -> AgenticPlan {
    let progress = Progress::scan(messages);

    // Step 1: write the generated document.
    if let Some(tool) =
        tool_for(tool_names, Capability::Write).filter(|_| !progress.done(Capability::Write))
    {
        return plan_one(tool, write_arguments(recipe.path, &recipe.document));
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) =
        tool_for(tool_names, Capability::Run).filter(|_| !progress.done(Capability::Run))
    {
        return plan_one(
            tool,
            json!({ "command": recipe.verify_command }).to_string(),
        );
    }
    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(recipe.final_answer)
}

/// The issue-#538 recipe: search → fetch (Wikidata lexemes) → write the enriched
/// meaning block → verify → final. Mirrors the formalization recipe but re-derives
/// the enriched meaning block from the fetched lexeme facts instead of formalizing
/// prose. The concept to enrich is routed from the request itself
/// ([`meaning_detail::concept_for_task`]), so the *same* recipe makes tomato,
/// potato, or any registered concept more detailed. Steps whose tool the CLI did
/// not advertise are skipped.
pub(super) fn plan_meaning_detail_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> AgenticPlan {
    // Route to the concept the request names (default: tomato — the canonical task).
    let concept = meaning_detail::concept_for_task(task).unwrap_or(&meaning_detail::TOMATO);

    let search_tool = tool_for(tool_names, Capability::Search);
    let fetch_tool = tool_for(tool_names, Capability::Fetch);
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);

    // Step 1: search for the Wikidata lexeme data.
    if let Some(tool) = search_tool
        && !progress.done(Capability::Search) {
            return plan_one(tool, json!({ "query": concept.search_query }).to_string());
        }
    // Step 2: fetch the lexeme forms (where the missing plural is recovered).
    if let Some(tool) = fetch_tool
        && !progress.done(Capability::Fetch) {
            return plan_one(tool, fetch_arguments(concept.source_url));
        }

    // Re-derive the enriched block from the fetched lexeme facts (or the canonical
    // fallback when the fetch errored), exactly as the formalization recipe does.
    let block = meaning_detail::enrich_block(concept, progress.fetched_text.as_deref());

    // Step 3: write the enriched meaning block.
    if let Some(tool) = write_tool
        && !progress.done(Capability::Write) {
            return plan_one(tool, write_arguments(concept.kb_path, &block));
        }
    // Step 4: verify by reading the enriched block back (mirrors the formalization
    // recipe; `cat` is the allowlisted read the sandbox workspace supports).
    if let Some(tool) = run_tool
        && !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {}", concept.kb_path) });
            return plan_one(tool, arguments.to_string());
        }

    // Step 5: nothing left to do — answer with the enriched block inline.
    AgenticPlan::Final(meaning_detail::final_answer_for(concept, &block))
}

/// The issue-#538 diagram recipe: write the generated mermaid document → verify →
/// final. Unlike the other two recipes it needs no web step — the diagrams are a
/// pure function of the planner's own recipe table ([`diagram::render_document`]),
/// so the loop *documents itself*. Steps whose tool the CLI did not advertise are
/// skipped.
pub(super) fn plan_diagram_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = diagram::render_document();
    let final_answer = diagram::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: diagram::DIAGRAM_PATH,
            verify_command: format!("cat {}", diagram::DIAGRAM_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#538 self-AST recipe: write the generated CST/AST-in-data document →
/// verify → final. Like the diagram recipe it needs no web step — the document is a
/// pure function of the planner's own source parsed through the meta-language links
/// network ([`self_ast::render_document`]), so the loop *inspects itself*. Steps
/// whose tool the CLI did not advertise are skipped.
pub(super) fn plan_self_ast_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = self_ast::render_document();
    let final_answer = self_ast::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: self_ast::AST_PATH,
            verify_command: format!("cat {}", self_ast::AST_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 self-healing recipe: write the generated repair-case document →
/// verify → final. Like the diagram and self-AST recipes it needs no web step — the
/// document is a pure function of the canonical self-healing case
/// ([`self_heal::render_document`]), so the loop *repairs itself*. Steps whose tool
/// the CLI did not advertise are skipped.
pub(super) fn plan_self_heal_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = self_heal::render_document();
    let final_answer = self_heal::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: self_heal::SELF_HEAL_PATH,
            verify_command: format!("cat {}", self_heal::SELF_HEAL_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 source-links recipe: write the generated whole-repository
/// source ↔ links projection document → verify → final. Like the diagram, self-AST,
/// and self-healing recipes it needs no web step — the document is a pure function
/// of the system's own embedded source projected through the meta-language links
/// network ([`source_links::render_document`]), so the loop *translates itself*.
/// Steps whose tool the CLI did not advertise are skipped.
pub(super) fn plan_source_links_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = source_links::render_document();
    let final_answer = source_links::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: source_links::SOURCE_LINKS_PATH,
            verify_command: format!("cat {}", source_links::SOURCE_LINKS_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 learning-ledger recipe: write the generated approved-lesson ledger
/// document → verify → final. Like the other self-inspection recipes it needs no web
/// step — the document is a pure function of the canonical, human-approved ledger
/// ([`ledger::render_document`]). Steps whose tool the CLI did not advertise are
/// skipped.
pub(super) fn plan_ledger_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = ledger::render_document();
    let final_answer = ledger::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: ledger::LEDGER_PATH,
            verify_command: format!("cat {}", ledger::LEDGER_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 self-explanation recipe: write the generated grounded-explanation
/// document → verify → final. Like the other self-inspection recipes it needs no web
/// step — the document is a pure function of the system's own embedded source cited
/// through the owned manifest ([`explain::render_document`]), so the loop *explains
/// itself*. Steps whose tool the CLI did not advertise are skipped.
pub(super) fn plan_explain_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = explain::render_document();
    let final_answer = explain::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: explain::EXPLAIN_PATH,
            verify_command: format!("cat {}", explain::EXPLAIN_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 self-change recipe: write the generated reviewable pull-request
/// document → verify → final. Like the other self-referential recipes it needs no web
/// step — the document is a deterministic function of the request and its grounded
/// target ([`change_request::render_document`]), so the loop turns a user's request to
/// *change Formal AI itself* into a reviewable PR. Steps whose tool the CLI did not
/// advertise are skipped.
pub(super) fn plan_change_request_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = change_request::render_document();
    let final_answer = change_request::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: change_request::CHANGE_PATH,
            verify_command: format!("cat {}", change_request::CHANGE_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 general repair-classification recipe: write the generated
/// repair-strategies document → verify → final. Like the other self-referential recipes
/// it needs no web step — the document is a deterministic function of the three
/// self-contained canonical failure traces ([`repair_strategy::render_document`]), so
/// the loop decides *which part* of itself to repair for every failure class. Steps
/// whose tool the CLI did not advertise are skipped.
pub(super) fn plan_repair_strategy_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = repair_strategy::render_document();
    let final_answer = repair_strategy::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: repair_strategy::REPAIR_STRATEGY_PATH,
            verify_command: format!("cat {}", repair_strategy::REPAIR_STRATEGY_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 rebuild-and-reattach recipe: write the generated
/// rebuild-and-reattach plan → verify → final. Like the change-request and source-links
/// recipes it needs no web step — the plan is a deterministic function of the accepted
/// change and the grounded UI artifacts ([`rebuild_plan::render_document`]), so the loop
/// turns an accepted change into the ordered, reversible plan to recompile Formal AI and
/// reattach the improved worker to the UI. Steps whose tool the CLI did not advertise are
/// skipped.
pub(super) fn plan_rebuild_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = rebuild_plan::render_document();
    let final_answer = rebuild_plan::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: rebuild_plan::REBUILD_PATH,
            verify_command: format!("cat {}", rebuild_plan::REBUILD_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#527 question-catalog recipe: write the generated question-catalog
/// document → verify → final. Like the other self-referential recipes it needs no web
/// step — the document is a deterministic function of the seed lexicon and the
/// deterministic engine ([`question_catalog::render_document`]), so the loop *generates
/// every possible question and answers it*. Steps whose tool the CLI did not advertise
/// are skipped.
pub(super) fn plan_question_catalog_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = question_catalog::render_document();
    let final_answer = question_catalog::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: question_catalog::QUESTION_CATALOG_PATH,
            verify_command: format!("cat {}", question_catalog::QUESTION_CATALOG_PATH),
            final_answer,
            document,
        },
    )
}

pub(super) fn plan_dreaming_audit_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = dreaming_audit::render_document();
    let final_answer = dreaming_audit::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: dreaming_audit::DREAMING_AUDIT_PATH,
            verify_command: format!("cat {}", dreaming_audit::DREAMING_AUDIT_PATH),
            final_answer,
            document,
        },
    )
}

/// The issues-#498 + #558 learning-frontier recipe: write the generated
/// learning-frontier report → verify → final. Like the other self-referential recipes
/// it needs no web step — the report is a pure function of the committed Trends catalog
/// routed through the human-gated self-improvement loop
/// ([`google_trends_learning::render_document`]), so the loop maps its own coverage gap
/// and hands it to human triage. Steps whose tool the CLI did not advertise are skipped.
pub(super) fn plan_google_trends_learning_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = google_trends_learning::render_document();
    let final_answer = google_trends_learning::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: google_trends_learning::GOOGLE_TRENDS_LEARNING_PATH,
            verify_command: google_trends_learning::verification_command(),
            final_answer,
            document,
        },
    )
}

pub(super) fn plan_google_trends_catalog_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = google_trends_catalog::render_document();
    let final_answer = google_trends_catalog::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: google_trends_catalog::GOOGLE_TRENDS_CATALOG_PATH,
            verify_command: google_trends_catalog::verification_command(),
            final_answer,
            document,
        },
    )
}

