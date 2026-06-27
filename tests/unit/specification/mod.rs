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
//! - `network_visualization`: optional link-graph view alongside chat.
//! - `desktop_surface`: packaged desktop shell around the shared HTTP/web
//!   boundary.
//! - `vscode_surface`: dual-host VS Code extension (Node + Web Worker) embedding
//!   the shared web chat around the same HTTP/web boundary.
//! - `transparent_state`: querying the network through chat without leaking
//!   internal state by default.

mod agent_isolation;
mod agentic_meta_algorithm;
mod associative_packages;
mod behavior_rules;
mod benchmarks;
mod calculator_delegation;
mod capabilities;
mod chat_surface;
mod code_generation;
mod code_generation_blueprint;
mod code_generation_coreference;
mod code_generation_program_modifiers;
mod coding_modification_benchmarks;
mod cue_lexicon;
mod definition_fusion;
mod desktop_surface;
mod formalization;
mod intent_formalization;
mod issue_146;
mod issue_402;
mod issue_435;
mod issue_436;
mod issue_462;
mod issue_465;
mod links_network;
mod meta_algorithm;
mod meta_construction;
mod meta_frame;
mod meta_reasoning;
mod meta_self_improvement;
mod method_registry;
mod multilingual;
mod natural_language_access;
mod natural_language_skill_compilation;
mod network_visualization;
mod openai_compatibility;
mod probabilistic_reasoning;
mod procedural_howto_benchmarks;
mod project_lookups;
mod prompt_variations;
mod reasoning_loop;
mod reasoning_paths;
mod reasoning_paths_procedures;
mod recipe_interpreter;
mod recursive_core_recipe;
mod route_method_alias;
mod selection;
mod self_improvement;
mod shared_dialog_replay;
mod skill_ledger;
mod solution_evidence;
mod source_cache;
mod substitution_rules;
mod summarization_pipeline;
mod synthesis;
mod telegram_surface;
mod text_manipulation;
mod text_manipulation_benchmarks;
mod translation_via_links;
mod transparent_state;
mod unit_incompatibility;
mod unknown_reasoning;
mod vscode_surface;
