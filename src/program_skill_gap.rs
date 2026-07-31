//! Honest failure for `write_program` requests no synthesis route derives.
//!
//! Issue #699 batch 3 migrates the `write_program` dead end. The meta-builder
//! already synthesizes *outside* the curated catalogue — a request that the
//! verified template catalogue cannot serve is retried against the composite
//! blueprint recipes, the cached coding oracle, and the seed-driven idiom
//! composer (`data/seed/coding-idioms.lino`), each of which derives code the
//! catalogue never stored. What was missing is the other half of the issue's
//! requirement: when *every* route misses, the engine must fail with a named
//! skill gap rather than answering with something else.
//!
//! Before this module the miss rendered a catalogue recitation — "I do not have
//! a template for language `rust` and task `missing`. Supported tasks:
//! `hello_world`, `count_to_three`, …". That answer is doubly wrong under the
//! issue's generality-first rule: it advertises memorized specifics as if they
//! were the system's capability surface, and it names no gap a human or the
//! self-improvement loop can act on.
//!
//! The replacement follows the `skill_gap` protocol already established for
//! procedure compilation (issue #674, [`crate::skill_procedure`]): a stable,
//! quotable English gap *name* that travels in the event log, and a localized
//! reply rendered from seed data
//! (`data/seed/multilingual-responses-synthesis.lino`). Nothing in this module
//! enumerates what the catalogue happens to hold.

use crate::language::Language;
use crate::seed::response_for;

/// Placeholders the seed records carry.
const TASK_PLACEHOLDER: &str = "{task}";
const LANGUAGE_PLACEHOLDER: &str = "{language}";
const GAP_PLACEHOLDER: &str = "{gap}";
const ROUTES_PLACEHOLDER: &str = "{routes}";

/// Parameter value reported when the formalizer extracted none.
///
/// A slug, not prose: it is the same token the event log and the diagnostics
/// chips already carry for an unfilled `write_program` parameter.
pub const MISSING_PARAMETER: &str = "missing";

/// The synthesis routes tried, in dispatch order, before this gap is named.
///
/// Slugs rather than sentences, so the list is language-neutral and matches the
/// module names a reader can open. Keeping them here — instead of in the
/// localized text — means adding a route updates every language at once.
pub const SYNTHESIS_ROUTES: &[&str] = &[
    "catalog",
    "blueprint_recipes",
    "coding_oracle",
    "seed_idiom_composer",
];

/// Name the gap in seeded wording (R379), always in English.
///
/// The name is an identity: it travels in the `skill_gap` event and in the
/// migration ledger, so it must not vary with the language the request was
/// written in. The user-facing reply is localized separately by [`render`].
#[must_use]
pub fn gap_name(task: Option<&str>, language: Option<&str>) -> String {
    response_for("write_program_skill_gap_name", Language::English.slug())
        .unwrap_or_default()
        .replace(TASK_PLACEHOLDER, task.unwrap_or(MISSING_PARAMETER))
        .replace(LANGUAGE_PLACEHOLDER, language.unwrap_or(MISSING_PARAMETER))
}

/// Render the localized skill-gap reply for an underivable program request.
///
/// The *displayed* gap is localized so the reply reads as one sentence in the
/// requester's language; the identity [`gap_name`] returns stays English.
#[must_use]
pub fn render(task: Option<&str>, language: Option<&str>, response_language: Language) -> String {
    let gap = response_for("write_program_skill_gap_name", response_language.slug())
        .unwrap_or_else(|| gap_name(task, language))
        .replace(TASK_PLACEHOLDER, task.unwrap_or(MISSING_PARAMETER))
        .replace(LANGUAGE_PLACEHOLDER, language.unwrap_or(MISSING_PARAMETER));
    let template = response_for("write_program_skill_gap", response_language.slug())
        .or_else(|| response_for("write_program_skill_gap", Language::English.slug()))
        .unwrap_or_default();
    template
        .replace(GAP_PLACEHOLDER, &gap)
        .replace(ROUTES_PLACEHOLDER, &SYNTHESIS_ROUTES.join(", "))
        .replace(TASK_PLACEHOLDER, task.unwrap_or(MISSING_PARAMETER))
        .replace(LANGUAGE_PLACEHOLDER, language.unwrap_or(MISSING_PARAMETER))
}
