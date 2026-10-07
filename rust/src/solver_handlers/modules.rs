// Every handler module in this directory.
//
// Issue #991: this list is `merge=union` (see `data/meta/merge-conflict-policy.lino`),
// so two branches that each add a handler produce a superset instead of a conflict.
// It lives apart from `mod.rs` because that file also holds dispatch logic, and a
// union of two logic edits can compile while being wrong.
// Regenerate with `rust-script scripts/normalize-ordered-lists.rs --write`.

mod agent_workspace;
mod behavior_rule_followups;
mod behavior_rule_matching;
mod behavior_rules;
mod benchmark_prompts;
mod calculator_rate;
mod calendar;
mod calendar_create;
mod calendar_ics;
mod code_debugging;
mod code_explanation;
mod code_refactoring;
mod code_review;
mod compound_interest;
mod conversation_memory;
mod creative_composition;
mod creative_writing;
mod curated_project_fetch;
mod document_originality;
mod document_request;
mod fact_checking;
mod factual_qa;
mod feature_capability;
mod formalization_task;
mod format_conversion;
mod installation_conversion;
mod meta_explanation;
mod natural_language_tools;
mod pattern_inference;
mod playwright_script;
mod procedure_rules;
mod product_search;
mod program_blueprint;
mod program_synthesis;
mod prompt_text_question;
mod regex_synthesis;
mod research_table;
mod response_language_followup;
mod self_awareness;
mod shell_command_compose;
mod shell_command_transform;
mod software_project;
mod software_project_code;
mod software_project_followup;
mod software_project_phrases;
mod sql_synthesis;
mod statistics;
mod task_decomposition;
mod test_generation;
mod text_edit_ops;
mod text_manipulation;
mod text_request_framing;
mod unit_conversion;

pub mod numeric_list;
pub mod policy_gates;
pub mod summarization_request;
pub mod text_rewrite;
mod user_intent;
mod verifiable_task;
mod web_requests;
pub mod web_search_intent;
mod world_state;
