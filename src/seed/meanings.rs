//! Language-independent *meaning* lexicon (issue #386).
//!
//! The solver must never recognise a user prompt by matching a hardcoded list
//! of words in one language. Instead it references **meanings** — concepts that
//! exist independently of any language and are *self-describing*: every meaning
//! is `defined_by` other meanings (a closed, mutually-referential graph in the
//! spirit of <https://github.com/link-foundation/relative-meta-logic>), carries
//! a human `gloss`, and is anchored to real lexical data via `wiktionary`.
//!
//! A meaning declares the semantic `role`s it can play when read in a prompt
//! (e.g. a sort is both a `program_artifact` a follow-up can refer to and a
//! `program_modification` a follow-up can request). Its `lexeme` blocks list
//! the surface words that *evidence* it, per language. Recognition code asks
//! the lexicon "which words evidence role X?" and stays free of hardcoded
//! natural-language text — the words live once, here in the data.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use super::parser::{parse_lino, LinoNode};
use super::MEANING_FILES;

/// Semantic role: a thing a program produces that a later turn can refer back
/// to (a result, an output, the program/script/code itself, an ordering).
pub const ROLE_PROGRAM_ARTIFACT: &str = "program_artifact";
/// Semantic role: an operation a follow-up turn can request against the active
/// program (sort, reverse, cancel, change, …) — additive or subtractive.
pub const ROLE_PROGRAM_MODIFICATION: &str = "program_modification";
/// Semantic role: a kind of program artifact a user can ask to be authored
/// (a program, a script, code, a function). The noun side of "write a <kind>".
pub const ROLE_PROGRAM_KIND: &str = "program_kind";
/// Semantic role: a verb that requests a program artifact be produced (write,
/// create, show, generate, make, build). The verb side of "write a <kind>".
pub const ROLE_PROGRAM_REQUEST: &str = "program_request";
/// Semantic role: a concrete unit of measurement (metre, byte, kilogram, …).
/// Each such meaning is `defined_by` the [`ROLE_PHYSICAL_DIMENSION`] it measures.
pub const ROLE_MEASUREMENT_UNIT: &str = "measurement_unit";
/// Semantic role: a physical dimension (length, mass, time, …). Units that
/// belong to different dimensions cannot be converted into one another.
pub const ROLE_PHYSICAL_DIMENSION: &str = "physical_dimension";
/// Semantic role: a named day of the week (Monday … Sunday). The meaning slug
/// is the English weekday name so a handler can resolve a matched lexeme back
/// to a position in the seven-day cycle.
pub const ROLE_CALENDAR_WEEKDAY: &str = "calendar_weekday";
/// Semantic role: the "comes after" relation between weekdays — a +1 step in
/// the seven-day cycle (after, next day, после, के बाद, 之后, …).
pub const ROLE_CALENDAR_DIRECTION_NEXT: &str = "calendar_direction_next";
/// Semantic role: the "comes before" relation between weekdays — a -1 step in
/// the seven-day cycle (before, previous day, перед, से पहले, 之前, …).
pub const ROLE_CALENDAR_DIRECTION_PREVIOUS: &str = "calendar_direction_previous";
/// Semantic role: the present day relative to the system clock (today,
/// сегодня, आज, 今天). Distinguishes a "what day is it now?" query.
pub const ROLE_CALENDAR_TODAY: &str = "calendar_today";
/// Semantic role: a reference to a day, date, or week — the noun a calendar
/// question is about (day, weekday, date, week, день, неделя, 星期, …).
pub const ROLE_CALENDAR_DAY_REFERENCE: &str = "calendar_day_reference";
/// Semantic role: an interrogative or imperative asking which day (what,
/// which, какой, कौन, 什么, …). The question side of a calendar query.
pub const ROLE_CALENDAR_QUESTION: &str = "calendar_question";
/// Semantic role: a relation in the knowledge base that maps a subject to a
/// value (capital, population, author, …).
///
/// A fact query detects which relation a prompt asks about by walking every
/// meaning carrying this role, in declaration order; each is `defined_by` the
/// `knowledge_relation` concept.
pub const ROLE_FACT_RELATION: &str = "fact_relation";
/// Semantic role: a follow-up that verifies an already-designed software
/// artifact behaves correctly (test it, run the tests, протестируй, 测试, …).
pub const ROLE_SOFTWARE_FOLLOWUP_VERIFICATION: &str = "software_followup_verification";
/// Semantic role: a follow-up that runs or executes the designed artifact
/// (run it, execute it, запусти, 运行, चलाओ, …).
pub const ROLE_SOFTWARE_FOLLOWUP_EXECUTION: &str = "software_followup_execution";
/// Semantic role: a follow-up that demonstrates the artifact's output
/// (show me, demo it, покажи, 显示, दिखाओ, …).
pub const ROLE_SOFTWARE_FOLLOWUP_DEMONSTRATION: &str = "software_followup_demonstration";
/// Semantic role: a verb that requests a software artifact be authored.
///
/// Surfaces include write, build, create, implement, develop, design, scaffold,
/// … — the verb side of "build me a <artifact>". Distinct from
/// `program_request`, which gates the narrower "write a <program>" synthesis
/// path; the two overlap on the shared verbs, but a software-authoring verb
/// need not trip program synthesis.
pub const ROLE_SOFTWARE_AUTHORING_ACTION: &str = "software_authoring_action";
/// Semantic role: a kind of software artifact an authoring request can ask for.
///
/// Examples are a web app, a CLI tool, a browser extension, a library, …. Each
/// is `defined_by` the `software_artifact` genus; a handler resolves a matched
/// lexeme back to its slug and maps the slug to a canonical English label.
pub const ROLE_SOFTWARE_ARTIFACT_KIND: &str = "software_artifact_kind";
/// Semantic role: a category a software feature requirement falls into.
///
/// Examples are state tracking, data exchange, automation, validation,
/// integration, user interface, and a catch-all project behavior. The union of
/// these meanings' words detects that a clause states a feature requirement;
/// the first category (in declaration order) whose word appears classifies it,
/// so the code knows only the concept "a requirement has a category".
pub const ROLE_SOFTWARE_REQUIREMENT_CATEGORY: &str = "software_requirement_category";
/// Semantic role: the software-feature genus (feature, requirement, …). A
/// prompt that mentions a feature/requirement earns the "requirements"
/// approval gate.
pub const ROLE_SOFTWARE_FEATURE: &str = "software_feature";
/// Semantic role: how the assistant should deliver a software solution.
///
/// The non-default modes — manual instructions, immediate execution, script
/// generation — each carry this role. A handler walks them in declaration
/// order (so the order encodes priority) and selects the first evidenced in
/// the prompt, falling back to code generation when none is.
pub const ROLE_SOFTWARE_DELIVERY_MODE: &str = "software_delivery_mode";
/// Semantic role: the programming language a software implementation targets.
///
/// python, rust, javascript, …. Walked in declaration order; the first
/// evidenced language wins, else the default (typescript) is used.
pub const ROLE_SOFTWARE_IMPLEMENTATION_LANGUAGE: &str = "software_implementation_language";
/// Semantic role: a tabletop/RPG game domain.
///
/// A D&D unit, token, wargame piece, Owlbear scene, …. A request is a
/// game-unit tracker only when it pairs a domain with a mechanic (see
/// [`ROLE_GAME_TRACKER_MECHANIC`]).
pub const ROLE_GAME_TRACKER_DOMAIN: &str = "game_tracker_domain";
/// Semantic role: a combat mechanic a tabletop tracker follows — hit points,
/// damage, protection, resistance, cooldowns. Pairs with the domain above.
pub const ROLE_GAME_TRACKER_MECHANIC: &str = "game_tracker_mechanic";
/// Semantic role: a request to approve the work step by step (each step, step
/// by step, …) — adds the `each_step` approval gate.
pub const ROLE_SOFTWARE_STEP_GRANULARITY: &str = "software_step_granularity";
/// Semantic role: a shell or command-line surface (shell, bash, a command,
/// docker, `WebVM`, …) — adds the `bash_command` approval gate.
pub const ROLE_SOFTWARE_BASH_COMMAND: &str = "software_bash_command";
/// Semantic role: a whole-prompt approval trigger (approve, yes, proceed, …).
///
/// Unlike the other roles this matches the *entire* compacted prompt, not a
/// passing mention: a go-ahead like "approve plan" moves the dialogue from
/// plan to implementation, while "approve the email validation step" does not.
pub const ROLE_SOFTWARE_APPROVAL_TRIGGER: &str = "software_approval_trigger";
/// Semantic role: the subject of a program-synthesis request — the *function*
/// it asks to be written (the noun side of "implement a function …").
pub const ROLE_PROGRAM_SYNTHESIS_SUBJECT: &str = "program_synthesis_subject";
/// Semantic role: a domain signal of a program-synthesis request — the target
/// language (Python) or a data kind it works over (tuple, numbers, vowels).
pub const ROLE_PROGRAM_SYNTHESIS_DOMAIN: &str = "program_synthesis_domain";
/// Semantic role: the request/specification verb of a program-synthesis
/// request (implement, write, return). The verb side of "implement a function".
pub const ROLE_PROGRAM_SYNTHESIS_ACTION: &str = "program_synthesis_action";
/// Semantic role: a surface signal that distinguishes one synthesis task.
///
/// The "distinct numbers"/"differ"/"threshold"/"similar elements"/"count
/// vowels" phrases. A task is `defined_by` the signals that evidence it.
pub const ROLE_PROGRAM_SYNTHESIS_SIGNAL: &str = "program_synthesis_signal";
/// Semantic role: a concrete synthesis task.
///
/// Its slug is the canonical Python function name (`has_close_elements`,
/// `similar_elements`, `count_vowels`). Walked in declaration order; a task is
/// selected when its name is declared or when every `program_synthesis_signal`
/// it is `defined_by` is evidenced in the prompt.
pub const ROLE_PROGRAM_SYNTHESIS_TASK: &str = "program_synthesis_task";
/// Semantic role: the user signalling they did not understand the assistant.
///
/// Asks it to make a prior answer clear ("I don't understand", "не понял",
/// "समझ नहीं आया", "我不明白", …). A meaning carrying this role is `defined_by`
/// the `clarification` and `understanding` concepts.
pub const ROLE_CLARIFICATION_REQUEST: &str = "clarification_request";
/// Semantic role: the user asking what the assistant is able to do.
///
/// A request to enumerate its capabilities ("what can you do", "что ты умеешь",
/// "你能做什么", …). Distinct from [`ROLE_CAPABILITY_QUERY_MORE`], the follow-up.
pub const ROLE_CAPABILITY_QUERY: &str = "capability_query";
/// Semantic role: the user asking what *else* the assistant can do.
///
/// A follow-up that requests capabilities beyond those already named ("what
/// else can you do", "что ещё ты умеешь", …) — a superset signal layered over
/// the base [`ROLE_CAPABILITY_QUERY`].
pub const ROLE_CAPABILITY_QUERY_MORE: &str = "capability_query_more";
/// Semantic role: the user asking the assistant to list facts about itself.
///
/// "facts about yourself", "факты о себе", "自我事实", …. Checked before the
/// broader self-introduction and known-facts queries so it wins the overlap.
pub const ROLE_SELF_FACT_QUERY: &str = "self_fact_query";
/// Semantic role: the user asking the assistant to introduce itself.
///
/// A get-acquainted request ("tell me about yourself", "расскажи о себе",
/// "介绍一下你自己", …). Suppressed when a [`ROLE_SELF_FACT_QUERY`] surface
/// also matches.
pub const ROLE_SELF_INTRODUCTION_REQUEST: &str = "self_introduction_request";
/// Semantic role: the single root of the merged ontology — the `link` meaning.
///
/// Every other meaning descends from it through `defined_by` edges, so the whole
/// lexicon is one connected graph rooted at `link` (the relative-meta-logic
/// "everything is a link" stance). Exactly one meaning carries this role.
pub const ROLE_ONTOLOGY_ROOT: &str = "ontology_root";
/// Semantic role: the root of the type-system sub-ontology — the `type` meaning.
///
/// A distinguished node directly under `link`; the broadest classifications
/// (`entity`, `concept`) are `defined_by` it, giving a merged multi-root
/// ontology whose roots all reduce to `link`.
pub const ROLE_ONTOLOGY_TYPE: &str = "ontology_type";
/// Semantic role: a top-level ontological category each domain genus roots in.
///
/// `entity`, `concept`, `relation`, `action`, `property` — the bridge meanings
/// that connect every domain cluster (programs, calendars, facts, software, …)
/// up to the `link` root.
pub const ROLE_ONTOLOGY_CATEGORY: &str = "ontology_category";

/// A single surface form together with a self-describing note.
///
/// The data must "not just list" the word (issue #386): each form carries a
/// `description` — a concise human note of *that* form's sense and shape (its
/// citation form, grammatical form, script, or romanisation). Recognition code
/// still matches on [`WordForm::text`]; the description makes the form usable
/// for any purpose (diagnostics, learning, lexical grounding), not only the
/// one parsing path baked into a handler.
#[derive(Debug, Clone)]
pub struct WordForm {
    pub text: String,
    pub description: String,
}

/// Surface forms that evidence a meaning in one language.
#[derive(Debug, Clone)]
pub struct Lexeme {
    pub language: String,
    pub words: Vec<WordForm>,
}

/// A language-independent meaning grounded in real lexical data.
#[derive(Debug, Clone)]
pub struct Meaning {
    pub slug: String,
    pub gloss: String,
    pub wiktionary: String,
    pub defined_by: Vec<String>,
    pub roles: Vec<String>,
    pub lexemes: Vec<Lexeme>,
}

impl Meaning {
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Every surface word across every language this meaning lexicalises.
    pub fn words(&self) -> impl Iterator<Item = &str> {
        self.lexemes
            .iter()
            .flat_map(|lexeme| lexeme.words.iter().map(|w| w.text.as_str()))
    }

    /// Every surface form (text plus its self-describing note) across every
    /// language this meaning lexicalises, in declaration order.
    pub fn word_forms(&self) -> impl Iterator<Item = &WordForm> {
        self.lexemes.iter().flat_map(|lexeme| lexeme.words.iter())
    }

    /// Is this meaning evidenced in `normalized` — does any of its surface
    /// words (in any language) appear as a whole token or phrase? Matching is
    /// not language-gated: an English proper noun (e.g. `python`) is evidence
    /// in a prompt written in any language.
    #[must_use]
    pub fn evidenced_in(&self, normalized: &str) -> bool {
        self.words().any(|word| surface_present(normalized, word))
    }

    /// The first surface word this meaning lexicalises in `language`, if any.
    /// Used to render a concept in a chosen language (e.g. a dimension label).
    #[must_use]
    pub fn word_in(&self, language: &str) -> Option<&str> {
        self.lexemes
            .iter()
            .find(|lexeme| lexeme.language == language)
            .and_then(|lexeme| lexeme.words.first().map(|w| w.text.as_str()))
    }

    /// The self-describing note for `word` (matched case-insensitively against
    /// the stored surface text) in any language, if recorded. This is the live
    /// reader that makes [`WordForm::description`] usable — the data describes
    /// each form rather than merely listing it (issue #386).
    #[must_use]
    pub fn describe_word(&self, word: &str) -> Option<&str> {
        self.word_forms()
            .find(|form| form.text.eq_ignore_ascii_case(word))
            .map(|form| form.description.as_str())
    }

    /// Languages this meaning is lexicalised in (used by coverage tests).
    #[must_use]
    pub fn languages(&self) -> BTreeSet<String> {
        self.lexemes.iter().map(|l| l.language.clone()).collect()
    }
}

/// The parsed set of meanings.
#[derive(Debug, Clone, Default)]
pub struct Lexicon {
    pub meanings: Vec<Meaning>,
}

impl Lexicon {
    #[must_use]
    pub fn meaning(&self, slug: &str) -> Option<&Meaning> {
        self.meanings.iter().find(|m| m.slug == slug)
    }

    /// Every meaning carrying `role`, in declaration order. Lets recognition
    /// code walk a semantic category (e.g. every measurement unit) without ever
    /// naming the surface words — those live in the data.
    pub fn meanings_with_role<'a>(&'a self, role: &'a str) -> impl Iterator<Item = &'a Meaning> {
        self.meanings.iter().filter(move |m| m.has_role(role))
    }

    /// Distinct surface words contributed by every meaning carrying `role`,
    /// in declaration order. Useful for diagnostics and tests.
    #[must_use]
    pub fn words_for_role(&self, role: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for meaning in self.meanings.iter().filter(|m| m.has_role(role)) {
            for word in meaning.words() {
                if !out.iter().any(|existing| existing == word) {
                    out.push(word.to_string());
                }
            }
        }
        out
    }

    /// Does `normalized` mention any surface word of any meaning in `role`?
    ///
    /// Mirrors the CJK-substring vs. whitespace-token contract used across the
    /// solver: CJK scripts have no inter-word spaces, so a CJK surface word is
    /// matched as a substring, while space-delimited scripts match a whole
    /// whitespace token or phrase (see [`crate::coding::contains_cjk`]).
    #[must_use]
    pub fn mentions_role(&self, role: &str, normalized: &str) -> bool {
        self.meanings_with_role(role)
            .any(|meaning| meaning.evidenced_in(normalized))
    }

    /// The first meaning carrying `role`, in declaration order, that is
    /// evidenced in `normalized` — or `None`. Declaration order therefore
    /// encodes priority (e.g. the first matching delivery mode wins).
    #[must_use]
    pub fn first_role_match(&self, role: &str, normalized: &str) -> Option<&Meaning> {
        self.meanings
            .iter()
            .filter(|meaning| meaning.has_role(role))
            .find(|meaning| meaning.evidenced_in(normalized))
    }

    /// The single meaning that roots the merged ontology — the one carrying
    /// [`ROLE_ONTOLOGY_ROOT`] (the `link` meaning), or `None` if absent.
    #[must_use]
    pub fn ontology_root(&self) -> Option<&Meaning> {
        self.meanings
            .iter()
            .find(|m| m.has_role(ROLE_ONTOLOGY_ROOT))
    }

    /// Does `slug` reach the ontology root by following `defined_by` edges?
    ///
    /// A breadth-first walk of the `defined_by` graph that visits each meaning
    /// at most once (cycles are expected). Every meaning must reach the root, so
    /// the data forms one connected ontology rather than disjoint islands of
    /// vocabulary — the universal "everything reduces to a link" stance.
    #[must_use]
    pub fn reaches_root(&self, slug: &str) -> bool {
        let Some(root) = self.ontology_root() else {
            return false;
        };
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut stack: Vec<&str> = vec![slug];
        while let Some(current) = stack.pop() {
            if current == root.slug {
                return true;
            }
            if !seen.insert(current) {
                continue;
            }
            if let Some(meaning) = self.meaning(current) {
                for target in &meaning.defined_by {
                    stack.push(target.as_str());
                }
            }
        }
        false
    }

    /// The type-system sub-root of the ontology — the meaning carrying
    /// [`ROLE_ONTOLOGY_TYPE`] (the `type` meaning), or `None` if absent.
    ///
    /// A distinguished node directly under the [`ontology_root`](Self::ontology_root):
    /// the broadest classifications descend from it, so a reasoner can ask "what
    /// kind of thing is this?" by walking up to the type sub-root.
    #[must_use]
    pub fn ontology_type_root(&self) -> Option<&Meaning> {
        self.meanings
            .iter()
            .find(|m| m.has_role(ROLE_ONTOLOGY_TYPE))
    }

    /// The top-level ontological categories — every meaning carrying
    /// [`ROLE_ONTOLOGY_CATEGORY`] (entity, concept, relation, action, property).
    ///
    /// These are the genera each domain cluster roots in, so generic reasoning
    /// can classify any meaning into a small, fixed set of categories rather
    /// than special-casing each domain.
    pub fn ontology_categories(&self) -> impl Iterator<Item = &Meaning> {
        self.meanings_with_role(ROLE_ONTOLOGY_CATEGORY)
    }
}

/// Does the surface word/phrase `expected` appear in `normalized`?
///
/// CJK surfaces have no inter-word spaces, so they match as substrings.
/// Space-delimited scripts match on whole-token boundaries — equal to the
/// whole string, or bounded by spaces — so a multi-word phrase ("each step")
/// matches as a unit and a short word ("api") never matches inside a longer
/// one ("напиши"). An empty surface never matches.
fn surface_present(normalized: &str, expected: &str) -> bool {
    if expected.is_empty() {
        return false;
    }
    if crate::coding::contains_cjk(expected) {
        return normalized.contains(expected);
    }
    normalized == expected
        || normalized.starts_with(&format!("{expected} "))
        || normalized.ends_with(&format!(" {expected}"))
        || normalized.contains(&format!(" {expected} "))
}

fn parse_lexicon(text: &str) -> Lexicon {
    let root = parse_lino(text);
    // The lexicon is split across several files (program, units, …), each
    // wrapping its records under a top-level `meanings` node. When the files
    // are concatenated the document therefore holds one-or-more `meanings`
    // containers; collect the records from every one. If none is present the
    // records sit at the document root (kept for robustness).
    let mut meanings = Vec::new();
    let containers: Vec<&LinoNode> = root
        .children
        .iter()
        .filter(|c| c.name == "meanings")
        .collect();
    let sources: Vec<&LinoNode> = if containers.is_empty() {
        vec![&root]
    } else {
        containers
    };
    for container in sources {
        for node in container.children.iter().filter(|c| c.name == "meaning") {
            meanings.push(parse_meaning(node));
        }
    }
    Lexicon { meanings }
}

fn parse_meaning(node: &LinoNode) -> Meaning {
    let mut defined_by = Vec::new();
    let mut roles = Vec::new();
    let mut lexemes = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "defined_by" => defined_by.push(child.id.clone()),
            "role" => roles.push(child.id.clone()),
            "lexeme" => {
                let words = child
                    .children
                    .iter()
                    .filter(|w| w.name == "word")
                    .map(|w| WordForm {
                        text: w.id.clone(),
                        description: w.find_child_value("description").to_string(),
                    })
                    .collect();
                lexemes.push(Lexeme {
                    language: child.id.clone(),
                    words,
                });
            }
            _ => {}
        }
    }
    Meaning {
        slug: node.id.clone(),
        gloss: node.find_child_value("gloss").to_string(),
        wiktionary: node.find_child_value("wiktionary").to_string(),
        defined_by,
        roles,
        lexemes,
    }
}

/// The parsed meaning lexicon. Cached — the embedded data is immutable at
/// runtime, so parsing happens at most once per process.
#[must_use]
pub fn lexicon() -> &'static Lexicon {
    static CACHE: OnceLock<Lexicon> = OnceLock::new();
    CACHE.get_or_init(|| parse_lexicon(&MEANING_FILES.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The supported languages every meaning must lexicalise so that the
    /// concept truly is "translatable to any language" (issue #386).
    const SUPPORTED_LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];

    #[test]
    fn lexicon_is_non_empty_and_well_formed() {
        let lex = lexicon();
        assert!(lex.meanings.len() >= 10, "expected a real lexicon");
        for meaning in &lex.meanings {
            assert!(!meaning.slug.is_empty(), "meaning needs a slug");
            assert!(
                !meaning.gloss.trim().is_empty(),
                "{} needs a conceptual gloss",
                meaning.slug
            );
            assert!(
                !meaning.wiktionary.trim().is_empty(),
                "{} must be grounded in real lexical data (wiktionary)",
                meaning.slug
            );
            assert!(
                !meaning.roles.is_empty(),
                "{} must declare at least one semantic role",
                meaning.slug
            );
        }
    }

    #[test]
    fn every_meaning_is_self_describing() {
        // relative-meta-logic: each term is defined using other terms. The
        // `defined_by` graph must therefore be closed — every reference
        // resolves to another defined meaning (cycles are allowed and
        // expected; there are no undefined primitives).
        let lex = lexicon();
        let slugs: BTreeSet<&str> = lex.meanings.iter().map(|m| m.slug.as_str()).collect();
        for meaning in &lex.meanings {
            assert!(
                !meaning.defined_by.is_empty(),
                "{} must be defined by other meanings",
                meaning.slug
            );
            for target in &meaning.defined_by {
                assert!(
                    slugs.contains(target.as_str()),
                    "{} is defined_by `{target}`, which is not itself a defined meaning",
                    meaning.slug
                );
            }
        }
    }

    #[test]
    fn every_meaning_covers_all_supported_languages() {
        let lex = lexicon();
        for meaning in &lex.meanings {
            let languages = meaning.languages();
            for language in SUPPORTED_LANGUAGES {
                assert!(
                    languages.contains(language),
                    "{} is missing a `{language}` lexeme — meanings must translate to every supported language",
                    meaning.slug
                );
            }
            for lexeme in &meaning.lexemes {
                assert!(
                    !lexeme.words.is_empty(),
                    "{} / {} lexeme must list at least one surface word",
                    meaning.slug,
                    lexeme.language
                );
            }
        }
    }

    #[test]
    fn word_forms_round_trip_and_describe_word_resolves() {
        // The surface text iterated by `words()` and the text carried by each
        // `WordForm` must be the same set, and `describe_word` must resolve a
        // recorded form (case-insensitively) while rejecting an unknown one.
        let lex = lexicon();
        let meaning = &lex.meanings[0];
        let from_words: Vec<&str> = meaning.words().collect();
        let from_forms: Vec<&str> = meaning.word_forms().map(|f| f.text.as_str()).collect();
        assert_eq!(
            from_words, from_forms,
            "words() and word_forms() must enumerate the same surfaces in order"
        );
        let first = from_words[0];
        assert!(
            meaning.describe_word(first).is_some(),
            "describe_word must resolve a recorded surface form"
        );
        assert!(
            meaning
                .describe_word("\u{0}-definitely-not-a-recorded-surface")
                .is_none(),
            "describe_word must return None for an unknown surface"
        );
    }

    #[test]
    fn descriptions_are_parsed_from_the_seed() {
        // Proves the `description` child is actually read off the wire:
        // at least one surface form must carry a non-empty self-describing note.
        // (The whole-lexicon enforcement invariant lives in
        // `every_word_form_is_described`.)
        let lex = lexicon();
        let described = lex
            .meanings
            .iter()
            .flat_map(Meaning::word_forms)
            .filter(|f| !f.description.trim().is_empty())
            .count();
        assert!(
            described > 0,
            "the parser must read `description` children off the seed"
        );
    }

    #[test]
    fn every_word_form_is_described() {
        // The self-describing-data contract (issue #386): a meaning may not just
        // *list* a surface form, it must *describe* it. Every `word` in every
        // lexeme of every meaning carries a non-empty `description`, so the seed
        // can be consumed for any purpose — not just the parsing order the code
        // happens to use today. This is the permanent ratchet behind the
        // per-language backfill; a new word with no description fails the build.
        let lex = lexicon();
        let mut missing: Vec<String> = Vec::new();
        for meaning in &lex.meanings {
            for lexeme in &meaning.lexemes {
                for form in &lexeme.words {
                    if form.description.trim().is_empty() {
                        missing.push(format!(
                            "{} / {} / {}",
                            meaning.slug, lexeme.language, form.text
                        ));
                    }
                }
            }
        }
        assert!(
            missing.is_empty(),
            "{} word form(s) lack a description, e.g. {}",
            missing.len(),
            missing
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }

    #[test]
    fn program_roles_are_populated() {
        let lex = lexicon();
        assert!(
            !lex.words_for_role(ROLE_PROGRAM_ARTIFACT).is_empty(),
            "program_artifact role must have surface words"
        );
        assert!(
            !lex.words_for_role(ROLE_PROGRAM_MODIFICATION).is_empty(),
            "program_modification role must have surface words"
        );
    }

    #[test]
    fn mentions_role_honours_cjk_and_token_boundaries() {
        let lex = lexicon();
        // Whitespace token (Russian): a substring of a longer token must NOT
        // match, but the standalone token must.
        assert!(lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "отмени сортировку"));
        assert!(!lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "отменительный разговор"));
        // CJK substring: matches inside a space-free run.
        assert!(lex.mentions_role(ROLE_PROGRAM_MODIFICATION, "取消排序"));
        assert!(lex.mentions_role(ROLE_PROGRAM_ARTIFACT, "取消排序"));
    }

    #[test]
    fn the_ontology_has_a_single_link_root() {
        // The merged ontology has exactly one root — the `link` meaning, which
        // is defined_by itself. A type-system sub-root (`type`) sits under it,
        // realising "Link should be the root of ontology, we can also have Type
        // link, for type system ontology" (issue #386).
        let lex = lexicon();
        let roots: Vec<&Meaning> = lex.meanings_with_role(ROLE_ONTOLOGY_ROOT).collect();
        assert_eq!(
            roots.len(),
            1,
            "the merged ontology must have exactly one root, found {}",
            roots.len()
        );
        let root = roots[0];
        assert_eq!(root.slug, "link", "the ontology root must be `link`");
        assert!(
            root.defined_by.iter().any(|t| t == "link"),
            "the root `link` must be defined by itself (self-rooted)"
        );
        let type_root = lex
            .ontology_type_root()
            .expect("a type-system sub-root (role ontology_type) must exist");
        assert!(
            lex.reaches_root(&type_root.slug),
            "the type sub-root must reduce to the link root"
        );
        // The bridge categories (entity, concept, relation, action, property)
        // each sit under the root too, so every domain genus has a category to
        // root in.
        let categories: Vec<&Meaning> = lex.ontology_categories().collect();
        assert!(
            categories.len() >= 2,
            "the ontology must define top-level categories under the root, found {}",
            categories.len()
        );
        for category in categories {
            assert!(
                lex.reaches_root(&category.slug),
                "ontology category {} must reduce to the link root",
                category.slug
            );
        }
    }

    #[test]
    fn every_meaning_reaches_the_link_root() {
        // The whole lexicon is one connected ontology: following `defined_by`
        // from any meaning eventually arrives at the single `link` root. No
        // meaning is an island of vocabulary disconnected from the root concept.
        let lex = lexicon();
        assert!(lex.ontology_root().is_some(), "an ontology root must exist");
        for meaning in &lex.meanings {
            assert!(
                lex.reaches_root(&meaning.slug),
                "{} does not reach the `link` ontology root via defined_by",
                meaning.slug
            );
        }
    }
}
