//! Lightweight, deterministic language detection driven by seed data.
//!
//! The universal solver tags every impulse with a detected language so the
//! evidence trail carries `language:en`, `language:ru`, `language:hi`,
//! `language:zh`, `language:es`, … or `language:unknown`. Detection is based on
//! Unicode block ranges and question markers — no neural inference and no
//! external service. See `VISION.md` and `REQUIREMENTS.md` for the rules.
//!
//! Issue #706: every fact detection needs — which script a language writes in,
//! which language a script defaults to, which tokens vote for a language that
//! shares its script with another, and which language is the fallback — lives
//! in `data/seed/language-detection.lino`. Adding a language is a seed edit, so
//! this module holds no per-language branch. The named [`Language`] constants
//! below are handles for the seed languages that Rust handlers still address by
//! name; a newly registered language needs none of them.

use alloc::string::String;
use alloc::vec::Vec;

/// The detection registry, embedded so the browser worker (`no_std` + `alloc`,
/// no filesystem) reads exactly the same rules as the native build.
const LANGUAGE_DETECTION: &str = include_str!("../data/seed/language-detection.lino");

/// Detected language, identified by the slug the registry gives it.
///
/// A newtype rather than an enum: the set of languages is data, so the type
/// cannot enumerate it. Slugs are `&'static str` slices of the embedded
/// registry (or of the constants below), which keeps the type `Copy` and
/// usable in the `no_std` worker.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Language(&'static str);

#[allow(non_upper_case_globals)]
impl Language {
    /// English — also the registry's declared fallback.
    pub const English: Self = Self("en");
    /// Russian.
    pub const Russian: Self = Self("ru");
    /// Hindi.
    pub const Hindi: Self = Self("hi");
    /// Chinese.
    pub const Chinese: Self = Self("zh");
    /// No registered language matched the prompt's dominant script.
    pub const Unknown: Self = Self("unknown");

    /// The slug used inside `language:<slug>` evidence links.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        self.0
    }

    /// Wrap an already-static slug, e.g. one sliced out of the registry.
    #[must_use]
    pub const fn from_static_slug(slug: &'static str) -> Self {
        Self(slug)
    }
}

impl core::fmt::Debug for Language {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}

/// Issue #556: during a response-language follow-up the solver replays the
/// previous request with a language forced onto every localizable handler.
/// Handlers derive their output language from [`detect`], so the forced
/// language is applied at that single seam — mirroring
/// `FORCED_RESPONSE_LANGUAGE` in the JS worker.
///
/// On native builds the slot is thread-local so concurrent solves never see
/// each other's forced language. The Rust→WASM worker (issue #658) is
/// `no_std` and single-threaded, so `thread_local!`/`std` are unavailable
/// there; the `wasm32` build keeps the value in a plain `static` cell instead.
#[cfg(not(target_arch = "wasm32"))]
mod forced_language {
    use super::Language;
    use std::cell::Cell;

    thread_local! {
        static FORCED_LANGUAGE: Cell<Option<Language>> = const { Cell::new(None) };
    }

    pub(super) fn replace(language: Option<Language>) -> Option<Language> {
        FORCED_LANGUAGE.with(|slot| slot.replace(language))
    }

    pub(super) fn set(language: Option<Language>) {
        FORCED_LANGUAGE.with(|slot| slot.set(language));
    }

    pub(super) fn get() -> Option<Language> {
        FORCED_LANGUAGE.with(Cell::get)
    }
}

#[cfg(target_arch = "wasm32")]
mod forced_language {
    use super::Language;
    use core::cell::Cell;

    struct Slot(Cell<Option<Language>>);

    // SAFETY: `wasm32-unknown-unknown` runs single-threaded, so the cell is
    // never accessed concurrently.
    unsafe impl Sync for Slot {}

    static FORCED_LANGUAGE: Slot = Slot(Cell::new(None));

    pub(super) fn replace(language: Option<Language>) -> Option<Language> {
        FORCED_LANGUAGE.0.replace(language)
    }

    pub(super) fn set(language: Option<Language>) {
        FORCED_LANGUAGE.0.set(language);
    }

    pub(super) fn get() -> Option<Language> {
        FORCED_LANGUAGE.0.get()
    }
}

/// Force [`detect`] to return `language` until the returned guard is dropped.
///
/// The guard restores the previous forced value on drop, so nested replays
/// stay balanced. Passing `None` clears the override (a plain, non-forced
/// solve).
#[must_use]
pub fn set_forced_language(language: Option<Language>) -> ForcedLanguageGuard {
    let previous = forced_language::replace(language);
    ForcedLanguageGuard { previous }
}

/// Resolve a language slug to a [`Language`], accepting every slug the
/// detection registry declares.
///
/// Returns `None` for slugs the registry does not know, so callers keep a
/// single place to notice an unregistered language.
#[must_use]
pub fn from_slug(slug: &str) -> Option<Language> {
    with_rules(|rules| {
        rules
            .iter()
            .find(|rule| rule.language == slug)
            .map(|rule| Language(rule.language))
    })
}

/// Every language slug the detection registry declares, in registry order.
#[must_use]
pub fn registered_languages() -> Vec<Language> {
    with_rules(|rules| rules.iter().map(|rule| Language(rule.language)).collect())
}

/// The fallback language declared by the registry (`fallback yes`).
#[must_use]
pub fn fallback_language() -> Language {
    with_rules(fallback_of)
}

/// The coverage ledger, embedded next to the detection registry. It carries the
/// English `name` of every registered language, which handlers used to spell out
/// in `match` arms.
const LANGUAGE_LEDGER: &str = include_str!("../data/seed/languages.lino");

/// The English name of a registered language slug (`en` → `English`).
///
/// Issue #706: the names live in `data/seed/languages.lino`, so a newly
/// registered language is named by its ledger entry and never by a Rust arm.
#[must_use]
pub fn language_name(slug: &str) -> Option<&'static str> {
    let mut current: Option<&'static str> = None;
    for line in LANGUAGE_LEDGER.lines() {
        let trimmed = line.trim_start();
        let (key, value) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
        match key {
            "language" => current = Some(unquote(value)),
            "name" if current == Some(slug) => return Some(unquote(value)),
            _ => {}
        }
    }
    None
}

/// Resolve a `language_*` concept slug (`language_spanish`) to its registered
/// language (`es`) by matching the ledger's English name.
///
/// The concept graph names languages in words; the translation pipeline and the
/// Wiktionary client address them by slug. This is the bridge, and it is data,
/// not a table of four hard-coded pairs.
#[must_use]
pub fn language_for_concept_slug(concept_slug: &str) -> Option<Language> {
    let name = concept_slug.strip_prefix("language_")?;
    with_rules(|rules| {
        rules
            .iter()
            .find(|rule| {
                language_name(rule.language).is_some_and(|declared| {
                    declared.len() == name.len()
                        && declared
                            .chars()
                            .zip(name.chars())
                            .all(|(left, right)| left.to_ascii_lowercase() == right)
                })
            })
            .map(|rule| Language(rule.language))
    })
}

/// Whether `surface` contains at least one character the registry attributes to
/// `slug`'s script (or, for the fallback language, at least one letter of the
/// fallback script's range).
///
/// Importers use this to reject a surface form filed under the wrong language
/// without enumerating Unicode ranges per language.
#[must_use]
pub fn surface_matches_language(surface: &str, slug: &str) -> bool {
    with_rules(|rules| {
        let Some(rule) = rules.iter().find(|rule| rule.language == slug) else {
            return false;
        };
        // A language sharing the fallback script (Spanish and English are both
        // Latin) matches any character of that script, not only its accented
        // sub-range, so its plain-ASCII surfaces are not rejected.
        let script = rule.script;
        rules
            .iter()
            .filter(|candidate| candidate.script == script)
            .any(|candidate| {
                surface.chars().any(|character| {
                    let codepoint = u32::from(character);
                    (candidate.start..=candidate.end).contains(&codepoint)
                        && (!candidate.alphabetic_only || character.is_alphabetic())
                })
            })
    })
}

/// RAII guard that restores the previous forced language when dropped.
pub struct ForcedLanguageGuard {
    previous: Option<Language>,
}

impl Drop for ForcedLanguageGuard {
    fn drop(&mut self) {
        forced_language::set(self.previous);
    }
}

/// One parsed `rule` record from `data/seed/language-detection.lino`.
///
/// Borrowed from the embedded registry, so a rule costs no allocation beyond
/// its marker vector.
struct Rule {
    language: &'static str,
    script: &'static str,
    start: u32,
    end: u32,
    /// Count only `char::is_alphabetic` code points inside the range. The
    /// Latin fallback range spans six ASCII punctuation code points that are
    /// not letters and must not inflate the Latin count.
    alphabetic_only: bool,
    fallback: bool,
    /// Tokens that vote for this language when its script alone cannot decide
    /// the prompt — either because the script is shared (Spanish and English
    /// are both Latin) or because the prompt mixes scripts.
    markers: Vec<&'static str>,
}

fn unquote(value: &'static str) -> &'static str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value)
}

fn parse_codepoint(value: &str) -> u32 {
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"));
    digits.map_or(0, |digits| u32::from_str_radix(digits, 16).unwrap_or(0))
}

/// Split a `("a" "b" "c")` list into its quoted items.
fn parse_quoted_list(value: &'static str) -> Vec<&'static str> {
    let mut items = Vec::new();
    let mut rest = value;
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { break };
        items.push(&after[..close]);
        rest = &after[close + 1..];
    }
    items
}

fn parse_rules() -> Vec<Rule> {
    let mut rules: Vec<Rule> = Vec::new();
    for line in LANGUAGE_DETECTION.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let (key, value) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
        if key == "rule" {
            rules.push(Rule {
                language: "",
                script: "",
                start: 0,
                end: 0,
                alphabetic_only: false,
                fallback: false,
                markers: Vec::new(),
            });
            continue;
        }
        let Some(rule) = rules.last_mut() else {
            continue;
        };
        match key {
            "language" => rule.language = unquote(value),
            "script" => rule.script = unquote(value),
            "label" if rule.script.is_empty() => rule.script = unquote(value),
            "start" => rule.start = parse_codepoint(value),
            "end" => rule.end = parse_codepoint(value),
            "alphabetic-only" => rule.alphabetic_only = value == "yes",
            "fallback" => rule.fallback = value == "yes",
            "markers" => rule.markers = parse_quoted_list(value),
            _ => {}
        }
    }
    rules.retain(|rule| !rule.language.is_empty());
    rules
}

// The registry is parsed once per process on native targets. The WASM worker
// resets its bump allocator at the start of every exported call, so a cached
// allocation there would dangle; it reparses inside the caller's epoch, which
// costs a single scan of a file under two kilobytes.
#[cfg(not(target_arch = "wasm32"))]
fn with_rules<R>(action: impl FnOnce(&[Rule]) -> R) -> R {
    use std::sync::OnceLock;
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    action(RULES.get_or_init(parse_rules))
}

#[cfg(target_arch = "wasm32")]
fn with_rules<R>(action: impl FnOnce(&[Rule]) -> R) -> R {
    action(&parse_rules())
}

fn fallback_of(rules: &[Rule]) -> Language {
    rules
        .iter()
        .find(|rule| rule.fallback)
        .map_or(Language::English, |rule| Language(rule.language))
}

fn fallback_script_of(rules: &[Rule]) -> &'static str {
    rules
        .iter()
        .find(|rule| rule.fallback)
        .map_or("", |rule| rule.script)
}

/// The language a script defaults to: the first rule declaring that script.
fn default_language_of(rules: &[Rule], script: &str) -> Language {
    rules
        .iter()
        .find(|rule| rule.script == script && rule.fallback)
        .or_else(|| rules.iter().find(|rule| rule.script == script))
        .map_or_else(|| fallback_of(rules), |rule| Language(rule.language))
}

/// Per-script character counts, keyed by script name in registry order.
struct ScriptCounts {
    counts: Vec<(&'static str, usize)>,
    other: usize,
    first: Option<&'static str>,
}

impl ScriptCounts {
    fn of(&self, script: &str) -> usize {
        self.counts
            .iter()
            .find(|(name, _)| *name == script)
            .map_or(0, |(_, count)| *count)
    }

    fn bump(&mut self, script: &'static str) {
        if let Some(entry) = self.counts.iter_mut().find(|(name, _)| *name == script) {
            entry.1 += 1;
        } else {
            self.counts.push((script, 1));
        }
        self.first.get_or_insert(script);
    }

    /// The highest count among scripts other than `script`.
    fn max_excluding(&self, script: &str) -> usize {
        self.counts
            .iter()
            .filter(|(name, _)| *name != script)
            .map(|(_, count)| *count)
            .max()
            .unwrap_or(0)
    }

    fn total(&self) -> usize {
        self.counts.iter().map(|(_, count)| *count).sum::<usize>() + self.other
    }
}

fn count_scripts(prompt: &str, rules: &[Rule]) -> ScriptCounts {
    let mut counts = ScriptCounts {
        counts: Vec::new(),
        other: 0,
        first: None,
    };
    for character in prompt.chars() {
        let codepoint = u32::from(character);
        let matched = rules.iter().find(|rule| {
            (rule.start..=rule.end).contains(&codepoint)
                && (!rule.alphabetic_only || character.is_alphabetic())
        });
        match matched {
            Some(rule) => counts.bump(rule.script),
            None if character.is_alphabetic() => {
                counts.other += 1;
                counts.first.get_or_insert("");
            }
            None => {}
        }
    }
    counts
}

/// The language whose markers appear in the prompt, preferring the one whose
/// script carries the most characters.
///
/// Markers are weaker evidence than a script: they are ordinary tokens that can
/// appear inside a foreign proper name ("Расскажи о julián andrés quiñones"
/// carries Spanish markers but is Russian). So a marker rule only votes when no
/// *rival* script — a registered script other than the rule's own and other than
/// the fallback script every language may borrow identifiers from — appears in
/// the prompt; otherwise the script rules below decide.
fn marker_language(
    prompt: &str,
    rules: &[Rule],
    counts: &ScriptCounts,
    fallback_script: &str,
) -> Option<Language> {
    let normalized: String = prompt.to_lowercase();
    let mut best: Option<(usize, Language)> = None;
    for rule in rules.iter().filter(|rule| !rule.markers.is_empty()) {
        let count = counts.of(rule.script);
        if count == 0 {
            continue;
        }
        let contested = rules.iter().any(|other| {
            other.script != rule.script
                && other.script != fallback_script
                && counts.of(other.script) > 0
        });
        if contested {
            continue;
        }
        if !rule
            .markers
            .iter()
            .any(|marker| normalized.contains(&marker.to_lowercase()))
        {
            continue;
        }
        match best {
            Some((best_count, _)) if count <= best_count => {}
            _ => best = Some((count, Language(rule.language))),
        }
    }
    best.map(|(_, language)| language)
}

/// Detect the dominant language of a prompt.
///
/// Counts characters per registered script. Text written only in the fallback
/// script resolves to the fallback language unless another language's markers
/// claim it; when a registered non-fallback script opens the prompt or its
/// markers appear alongside fallback-script identifiers, that language wins.
/// Text dominated by a script no rule claims returns [`Language::Unknown`] so
/// the loop can record an explicit `language:unknown` event.
#[must_use]
pub fn detect(prompt: &str) -> Language {
    // Issue #556: a forced response language overrides detection for the whole
    // replay, so every localizable handler renders in the requested language.
    if let Some(forced) = forced_language::get() {
        return forced;
    }
    with_rules(|rules| detect_with(prompt, rules))
}

fn detect_with(prompt: &str, rules: &[Rule]) -> Language {
    let fallback = fallback_of(rules);
    let fallback_script = fallback_script_of(rules);
    let counts = count_scripts(prompt, rules);

    if counts.total() == 0 {
        return fallback;
    }

    let fallback_count = counts.of(fallback_script);
    if counts.other > fallback_count && counts.other >= counts.max_excluding(fallback_script) {
        return Language::Unknown;
    }

    if fallback_count > 0 {
        if let Some(language) = marker_language(prompt, rules, &counts, fallback_script) {
            return language;
        }
        if let Some(first) = counts.first
            && !first.is_empty()
            && first != fallback_script
        {
            let rival = counts
                .counts
                .iter()
                .filter(|(name, _)| *name != fallback_script && *name != first)
                .map(|(_, count)| *count)
                .max()
                .unwrap_or(0);
            if counts.of(first) >= rival {
                return default_language_of(rules, first);
            }
        }
    }

    for rule in rules.iter().filter(|rule| !rule.fallback) {
        let count = counts.of(rule.script);
        if count > 0 && count >= counts.max_excluding(rule.script) {
            return default_language_of(rules, rule.script);
        }
    }
    fallback
}
