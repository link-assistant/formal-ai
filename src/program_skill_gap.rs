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
///
/// Issue #906: this sentinel is an *internal* marker and must never reach the
/// requester. Substituting it into the localized wording produced answers like
/// "no synthesis route reaches task `hello_world` in language `missing`. …
/// Teach me the missing idiom for `missing`" — which reports a gap in a
/// language nobody named. Each unfilled parameter now selects its own seeded
/// wording via [`shape`]; the sentinel survives only as the last-resort filler
/// for a template that still carries a placeholder we have no value for.
pub const MISSING_PARAMETER: &str = "missing";

/// Which of the four `write_program` dead ends a request reached.
///
/// The distinction is the point of issue #906's second fix: "no synthesis route
/// derives this" and "you did not say which language" are different facts about
/// the request, and only the first is a gap in *our* skills.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    /// Both parameters are known; no route derives the pair.
    SkillGap,
    /// The language is known; the task did not resolve to a catalogued one.
    TaskUnspecified,
    /// A task is known, but the request names no implementation language.
    LanguageUnspecified,
    /// Neither parameter was named.
    RequestUnspecified,
}

impl Shape {
    /// The seed intent whose record carries the user-facing reply.
    #[must_use]
    const fn answer_intent(self) -> &'static str {
        match self {
            Self::SkillGap | Self::TaskUnspecified => "write_program_skill_gap",
            Self::LanguageUnspecified => "write_program_language_unspecified",
            Self::RequestUnspecified => "write_program_request_unspecified",
        }
    }

    /// The seed intent whose record names the dead end for the event log.
    #[must_use]
    const fn name_intent(self) -> &'static str {
        match self {
            Self::SkillGap => "write_program_skill_gap_name",
            Self::TaskUnspecified => "write_program_skill_gap_name_task_unspecified",
            Self::LanguageUnspecified => "write_program_language_unspecified_name",
            Self::RequestUnspecified => "write_program_request_unspecified_name",
        }
    }

    /// The engine intent this shape answers under.
    ///
    /// Only a genuine skill gap is reported as one. A request that named no
    /// language never reached a route at all, so calling it a "skill gap" would
    /// misreport the engine's own capability.
    #[must_use]
    pub const fn intent(self) -> &'static str {
        match self {
            Self::SkillGap | Self::TaskUnspecified => "write_program_skill_gap",
            Self::LanguageUnspecified => "write_program_language_unspecified",
            Self::RequestUnspecified => "write_program_request_unspecified",
        }
    }

    /// The evidence-trail event this shape appends.
    #[must_use]
    pub const fn event(self) -> &'static str {
        match self {
            Self::SkillGap | Self::TaskUnspecified => "skill_gap",
            Self::LanguageUnspecified | Self::RequestUnspecified => "unspecified_parameter",
        }
    }

    /// The response link this shape answers under.
    #[must_use]
    pub const fn response_link(self) -> &'static str {
        match self {
            Self::SkillGap | Self::TaskUnspecified => "response:write_program:skill_gap",
            Self::LanguageUnspecified => "response:write_program:language_unspecified",
            Self::RequestUnspecified => "response:write_program:request_unspecified",
        }
    }
}

/// Classify a `(task, language)` pair — see [`Shape`].
#[must_use]
pub const fn shape(task: Option<&str>, language: Option<&str>) -> Shape {
    match (task, language) {
        (Some(_), Some(_)) => Shape::SkillGap,
        (None, Some(_)) => Shape::TaskUnspecified,
        (Some(_), None) => Shape::LanguageUnspecified,
        (None, None) => Shape::RequestUnspecified,
    }
}

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
    named_in(task, language, Language::English)
}

/// [`gap_name`] rendered in `response_language`, falling back to the English
/// identity when that language has no record.
fn named_in(task: Option<&str>, language: Option<&str>, response_language: Language) -> String {
    let shape = shape(task, language);
    response_for(shape.name_intent(), response_language.slug())
        .or_else(|| response_for(shape.name_intent(), Language::English.slug()))
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
    let shape = shape(task, language);
    let gap = named_in(task, language, response_language);
    let template = response_for(shape.answer_intent(), response_language.slug())
        .or_else(|| response_for(shape.answer_intent(), Language::English.slug()))
        .unwrap_or_default();
    template
        .replace(GAP_PLACEHOLDER, &gap)
        .replace(ROUTES_PLACEHOLDER, &SYNTHESIS_ROUTES.join(", "))
        .replace(TASK_PLACEHOLDER, task.unwrap_or(MISSING_PARAMETER))
        .replace(LANGUAGE_PLACEHOLDER, language.unwrap_or(MISSING_PARAMETER))
}
