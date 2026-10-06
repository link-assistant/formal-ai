//! Requirement specification tests for the link-native symbolic assistant.
//!
//! Every test in this module pins down a single requirement from `VISION.md`,
//! `GOALS.md`, `NON-GOALS.md`, or `REQUIREMENTS.md`. Active tests describe
//! implemented behavior. Ignored tests are retained only for requirements that
//! already have an explicit tracking entry, so the expectation stays visible
//! without making unrelated CI jobs fail.
//!
//! The suite is split by surface so future PRs can grow each area:
//!
//! - `chat_surface`: bounded chat, identity, greeting, diagnostics defaults.
//! - `capabilities`: supported feature-status questions and availability.
//! - `code_generation`: top programming languages, execution evidence,
//!   unsupported-execution honesty.
//! - `multilingual`: English, Russian, Hindi, and Chinese conversations.
//! - `openai_compatibility`: Chat Completions, Responses, and HTTP routes.
//! - `telegram_surface`: private chats, public chats, code formatting.
//! - `links_network`: doublet links, dynamic types, add-only history,
//!   concept uniqueness, trace links.
//! - `reasoning_loop`: impulse, local search, external search, decomposition,
//!   candidate validation, smallest sufficient answer.
//! - `source_cache`: external source access caching with provenance and TTL.
//! - `agent_isolation`: chat vs agent autonomy and isolated execution.
//! - `translation_via_links`: links notation as the language of meaning.
//! - `network_visualization`: optional links network view alongside chat.
//! - `desktop_surface`: packaged desktop shell around the shared HTTP/web
//!   boundary.
//! - `vscode_surface`: dual-host VS Code extension (Node + Web Worker) embedding
//!   the shared web chat around the same HTTP/web boundary.
//! - `transparent_state`: querying the network through chat without leaking
//!   internal state by default.

mod agent_isolation;
mod agentic_meta_algorithm;
mod arbitrary_skill_compilation;
mod associative_packages;
mod behavior_delta;
mod behavior_rules;
mod benchmarks;
mod budget_search_meta_algorithm;
mod calculator_delegation;
mod capabilities;
mod capability_routing_table;
mod chat_surface;
mod code_generation;
mod code_generation_blueprint;
mod code_generation_coreference;
mod code_generation_program_modifiers;
mod coding_discovery_meta_algorithm;
mod coding_modification_benchmarks;
mod computer_use_meta_algorithm;
mod concept_lookup_meta_algorithm;
mod conversation_history;
mod cue_lexicon;
mod definition_fusion;
mod desktop_surface;
mod document_verification_meta_algorithm;
mod dreaming_meta_algorithm;
mod equation_corpus;
mod execution_evidence;
mod external_benchmarks;
mod forced_language_seam;
mod formalization;
mod formalization_depth_meta_algorithm;
mod github_repository_traffic;
mod grounded_action_meta_algorithm;
mod intent_formalization;
mod issue_146;
mod issue_402;
mod issue_435;
mod issue_436;
mod issue_462;
mod issue_465;
mod issue_467;
mod issue_595;
mod issue_682;
mod issue_710;
mod issue_892;
mod issue_893_summarization_validation;
mod links_network;
mod links_network_terminology_meta_algorithm;
mod market_price_verification_meta_algorithm;
mod memory_query;
mod meta_algorithm;
mod meta_construction;
mod meta_frame;
mod meta_reasoning;
mod meta_self_improvement;
mod method_registry;
mod multilingual;
mod natural_language_access;
mod natural_language_skill_compilation;
mod needs;
mod nemotron_training_samples;
mod network_visualization;
mod obligation_ledger;
mod openai_compatibility;
mod prerequisite_recipe;
mod probabilistic_reasoning;
mod procedural_howto_benchmarks;
mod project_lookups;
mod prompt_variations;
mod question_generation_lexicon;
mod reasoning_loop;
mod reasoning_paths;
mod reasoning_paths_procedures;
mod reasoning_standard_meta_algorithm;
mod recipe_interpreter;
mod recursive_core_recipe;
mod refutation_search;
mod repository_workspace_protocol;
mod response_language_followup;
mod response_language_meta_algorithm;
mod route_method_alias;
mod routing_precedence;
mod selection;
mod selection_heuristics;
mod self_hosting_metric;
mod self_improvement;
mod shared_dialog_replay;
mod skill_ledger;
mod solution_evidence;
mod source_cache;
mod source_reconstruction;
mod substitution_rules;
mod summarization_pipeline;
mod synthesis;
mod task_decomposition;
mod telegram_surface;
mod text_manipulation;
mod text_manipulation_benchmarks;
mod translation_round_trip;
mod translation_via_links;
mod transparent_state;
mod triz_contradictions;
mod unit_incompatibility;
mod unknown_reasoning;
mod vscode_surface;
mod world_state_benchmarks;

/// Every JSX module of the web front-end (`js/app/*.jsx`), concatenated. The
/// front-end is split into feature modules and bundled by bun into the served
/// `js/app.js`; source-level surface assertions read all of them so they hold
/// wherever a helper lives. `web_app_sources_cover_every_jsx_module` keeps
/// this list in step with the directory.
pub(crate) const WEB_APP_SOURCES: &str = concat!(
    include_str!("../../../../js/app/agent-plan.jsx"),
    include_str!("../../../../js/app/app-constants.jsx"),
    include_str!("../../../../js/app/app-conversation-hooks.jsx"),
    include_str!("../../../../js/app/app-desktop-hooks.jsx"),
    include_str!("../../../../js/app/app-layout-hooks.jsx"),
    include_str!("../../../../js/app/app-memory-hooks.jsx"),
    include_str!("../../../../js/app/app-worker-hooks.jsx"),
    include_str!("../../../../js/app/app.jsx"),
    include_str!("../../../../js/app/attachments.jsx"),
    include_str!("../../../../js/app/conversations.jsx"),
    include_str!("../../../../js/app/debugger-view.jsx"),
    include_str!("../../../../js/app/demo-mode.jsx"),
    include_str!("../../../../js/app/desktop-bridge.jsx"),
    include_str!("../../../../js/app/glyphs.jsx"),
    include_str!("../../../../js/app/interface-commands.jsx"),
    include_str!("../../../../js/app/issue-reporting.jsx"),
    include_str!("../../../../js/app/local-behavior-rules.jsx"),
    include_str!("../../../../js/app/local-fallback.jsx"),
    include_str!("../../../../js/app/local-prompts.jsx"),
    include_str!("../../../../js/app/local-self-knowledge.jsx"),
    include_str!("../../../../js/app/main.jsx"),
    include_str!("../../../../js/app/markdown-render.jsx"),
    include_str!("../../../../js/app/memory-events.jsx"),
    include_str!("../../../../js/app/message-view.jsx"),
    include_str!("../../../../js/app/preferences.jsx"),
    include_str!("../../../../js/app/recall-query.jsx"),
    include_str!("../../../../js/app/sidebar-section.jsx"),
    include_str!("../../../../js/app/thinking-steps.jsx"),
    include_str!("../../../../js/app/toolbar-icons.jsx"),
    include_str!("../../../../js/app/user-context.jsx"),
);

#[test]
fn web_app_sources_cover_every_jsx_module() {
    let app_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../js/app");
    for entry in std::fs::read_dir(app_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsx") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(
            WEB_APP_SOURCES.contains(&source),
            "{} must be listed in WEB_APP_SOURCES",
            path.display()
        );
    }
}
