//! MVP specification tests for the link-native symbolic assistant.
//!
//! Every test in this module pins down a single requirement from `VISION.md`,
//! `GOALS.md`, `NON-GOALS.md`, or `REQUIREMENTS.md`. Tests that describe
//! the current behavior of the prototype are active. Tests that describe MVP
//! behavior that has not been implemented yet are marked with `#[ignore]` and
//! a `MVP-target:` reason, so they document the failing expectations without
//! blocking CI. Run them locally with `cargo test -- --include-ignored`.
//!
//! The suite is split by surface so future PRs can grow each area:
//!
//! - `chat_surface`: bounded chat, identity, greeting, diagnostics defaults.
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
//! - `transparent_state`: querying the network through chat without leaking
//!   internal state by default.

mod agent_isolation;
mod calculator_delegation;
mod chat_surface;
mod code_generation;
mod links_network;
mod multilingual;
mod network_visualization;
mod openai_compatibility;
mod prompt_variations;
mod reasoning_loop;
mod reasoning_paths;
mod source_cache;
mod telegram_surface;
mod translation_via_links;
mod transparent_state;
