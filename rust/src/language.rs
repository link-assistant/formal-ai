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
const LANGUAGE_DETECTION: &str = include_str!("../embedded/data/seed/language-detection.lino");

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
    /// Spanish. Added by issue #1138 plan 06 leaf L2, because the coding
    /// guidance branched on four languages and let Spanish fall through to
    /// English on every program answer.
    pub const Spanish: Self = Self("es");
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

/// The slug of the language forced onto this solve, if any.
///
/// Handlers that cannot derive their output language from [`detect`] (their
/// report language is a fixed default, not a detection) read this instead, so a
/// forced response language still reaches them (issues #556, #531).
#[must_use]
pub fn forced_response_language_slug() -> Option<&'static str> {
    forced_language::get().map(Language::slug)
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
const LANGUAGE_LEDGER: &str = include_str!("../embedded/data/seed/languages.lino");

/// The English name of a registered language slug (`en` → `English`).
///
/// Whether `slug` places its adpositions after the noun (`"on:" के बाद`,
/// `"on:" 行后`), so a positional cue in an edit attaches to the literal
/// *before* it rather than after it (issue #1115). Read from the `adposition`
/// field of `data/seed/languages.lino`; a language that states nothing is
/// prepositional.
#[must_use]
pub fn uses_postpositions(slug: &str) -> bool {
    let mut current = None;
    for line in LANGUAGE_LEDGER.lines() {
        let Some((key, value)) = line.trim_start().split_once(' ') else {
            continue;
        };
        match key {
            "language" => current = Some(unquote(value)),
            "adposition" if current == Some(slug) => return unquote(value) == "postposition",
            _ => {}
        }
    }
    false
}

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

/// Whether `rule` claims `character`: inside its range, and a letter when the
/// rule counts letters only. Mirrored by `ruleClaims` in
/// `js/agentic/crate/language.mjs`.
fn rule_claims(rule: &Rule, character: char) -> bool {
    (rule.start..=rule.end).contains(&u32::from(character))
        && (!rule.alphabetic_only || character.is_alphabetic())
}

fn count_scripts(prompt: &str, rules: &[Rule]) -> ScriptCounts {
    let mut counts = ScriptCounts {
        counts: Vec::new(),
        other: 0,
        first: None,
    };
    for character in prompt.chars() {
        let matched = rules.iter().find(|rule| rule_claims(rule, character));
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

/// Whether a marker occurs in `normalized` at a position that counts.
fn marker_present(normalized: &str, marker: &str, rules: &[Rule], fallback_script: &str) -> bool {
    marker_count(normalized, marker, rules, fallback_script, 1) > 0
}

/// The occurrences of `marker` in `normalized` that count, up to `limit`.
///
/// A marker written in the shared fallback script only counts as a whole word:
/// Spanish "escribe" must not claim English "describe", Spanish "formaliza"
/// must not claim English "formalization" (issue #1557), and a word of a path
/// or an identifier is no language's word (Spanish "formaliza" must not claim
/// `intent_formalization.mjs`, PR #1188 G97). Markers in a script of their own
/// may sit inside native morphology — Chinese "什么" following "是" — so they
/// match anywhere, the way their languages write them. Mirrored by
/// `markerCount` in `js/agentic/crate/language.mjs`.
fn marker_count(
    normalized: &str,
    marker: &str,
    rules: &[Rule],
    fallback_script: &str,
    limit: usize,
) -> usize {
    let Some(first) = marker.chars().next() else {
        return 0;
    };
    let whole_word = rules
        .iter()
        .any(|rule| rule.script == fallback_script && rule_claims(rule, first));
    let mut found = 0;
    let mut from = 0;
    while found < limit {
        let Some(at) = normalized[from..].find(marker) else {
            break;
        };
        let start = from + at;
        from = start + marker.len();
        if whole_word {
            let word_start = normalized[..start]
                .chars()
                .next_back()
                .is_none_or(|previous| !previous.is_alphabetic());
            let word_end = normalized[from..]
                .chars()
                .next()
                .is_none_or(|next| !next.is_alphabetic());
            if !word_start || !word_end || in_identifier(normalized, start) {
                continue;
            }
        }
        found += 1;
    }
    found
}

/// Whether the whitespace-delimited token around byte `at` of `text` is a path
/// or an identifier. Mirrored by `inIdentifier` in
/// `js/agentic/crate/language.mjs`.
fn in_identifier(text: &str, at: usize) -> bool {
    let start = text[..at]
        .char_indices()
        .rev()
        .find(|(_, character)| character.is_whitespace())
        .map_or(0, |(index, character)| index + character.len_utf8());
    let end = text[at..]
        .find(char::is_whitespace)
        .map_or(text.len(), |index| at + index);
    identifier_token(&text[start..end])
}

/// Whether `token` is a path or an identifier: it holds `_`, `/` or `\`, or a
/// `.` before a letter or a digit (a file extension). Mirrored by
/// `identifierToken` in `js/agentic/crate/language.mjs`.
fn identifier_token(token: &str) -> bool {
    token.contains(['_', '/', '\\'])
        || token.char_indices().any(|(index, character)| {
            character == '.'
                && token[index + 1..]
                    .chars()
                    .next()
                    .is_some_and(char::is_alphanumeric)
        })
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
        if !rule.markers.iter().any(|marker| {
            let lowered = marker.to_lowercase();
            marker_present(&normalized, &lowered, rules, fallback_script)
        }) {
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

/// In prose, a registered script other than the fallback's wins once its
/// letters reach a sixth of the fallback script's. Writers of other languages
/// borrow fallback-script names and terms freely, and a fallback-script word
/// spends several letters where an ideographic script spends one, so the
/// fallback script must clearly outnumber a rival script to win (issue #1557).
/// Mirrored by `PROSE_SCRIPT_WEIGHT` in `js/agentic/crate/language.mjs`.
const PROSE_SCRIPT_WEIGHT: usize = 6;

/// A language sharing the fallback script wins prose once its votes — its
/// whole-word markers plus its words holding a letter of its own range — reach
/// a tenth of the fallback-script words, and a lone vote must be half of
/// them, so one marker or one accented name cannot outvote a document or a
/// short sentence (issue #1557). Mirrored by `SHARED_SCRIPT_SHARE`.
const SHARED_SCRIPT_SHARE: usize = 10;

/// An opening quotation mark and the marks that close it.
const QUOTATIONS: &[(char, &str)] = &[
    ('"', "\""),
    ('\u{201C}', "\u{201D}"),
    ('\u{00AB}', "\u{00BB}"),
    ('\u{201E}', "\u{201C}\u{201D}"),
];

/// Punctuation after which a quotation may open, besides whitespace.
const QUOTATION_OPENERS_AFTER: &str = "([{|";

/// A line without its closed `code` spans. Mirrored by `withoutCodeSpans`.
fn without_code_spans(line: &str) -> String {
    let parts: Vec<&str> = line.split('`').collect();
    let last = parts.len() - 1;
    parts
        .iter()
        .enumerate()
        .filter(|(index, _)| index % 2 == 0 || *index == last)
        .map(|(_, part)| *part)
        .collect::<Vec<_>>()
        .join(" ")
}

/// A line without the quotations that open a token and close on the same
/// line. A quote mark inside a token (`alt="Image"`) is markup, not a
/// quotation. Mirrored by `withoutQuotations`.
fn without_quotations(line: &str) -> String {
    let characters: Vec<char> = line.chars().collect();
    let mut kept = String::with_capacity(line.len());
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        let opens = index
            .checked_sub(1)
            .map(|previous| characters[previous])
            .is_none_or(|previous| {
                previous.is_whitespace() || QUOTATION_OPENERS_AFTER.contains(previous)
            });
        if opens
            && let Some((_, closers)) = QUOTATIONS.iter().find(|(open, _)| *open == character)
            && let Some(close) = characters[index + 1..]
                .iter()
                .position(|candidate| closers.contains(*candidate))
        {
            kept.push(' ');
            index += close + 2;
            continue;
        }
        kept.push(character);
        index += 1;
    }
    kept
}

/// The quoted items of the registry's first `key` line — the `prose-syntax`
/// record's fence and link markers (issue #1557). Mirrored by `registryList`.
fn registry_list(key: &str) -> Vec<&'static str> {
    LANGUAGE_DETECTION
        .lines()
        .filter_map(|line| line.trim_start().split_once(' '))
        .find(|(name, _)| *name == key)
        .map_or_else(Vec::new, |(_, value)| parse_quoted_list(value))
}

/// Whether a prose token is a URL, a path, an identifier or a `camelCase` name —
/// words that are no language's. Mirrored by `proseNoise`.
fn prose_noise(token: &str, links: &[&str]) -> bool {
    let lowered = token.to_lowercase();
    links.iter().any(|link| lowered.contains(*link))
        || identifier_token(token)
        || token
            .chars()
            .zip(token.chars().skip(1))
            .any(|(left, right)| left.is_lowercase() && right.is_uppercase())
}

/// The prose of a document: its lines without fenced code blocks, block
/// quotations, code spans, quotations, URLs and identifiers (issue #1557).
/// Mirrored by `proseOf` in `js/agentic/crate/language.mjs`.
#[must_use]
pub fn prose_of(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let fences = registry_list("fence-markers");
    let links = registry_list("link-markers");
    let mut fenced = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if fences.iter().any(|fence| trimmed.starts_with(*fence)) {
            fenced = !fenced;
            continue;
        }
        if fenced || trimmed.starts_with('>') {
            continue;
        }
        let cleaned = without_quotations(&without_code_spans(line));
        let tokens: Vec<&str> = cleaned
            .split_whitespace()
            .filter(|token| !prose_noise(token, &links))
            .collect();
        if !tokens.is_empty() {
            lines.push(tokens.join(" "));
        }
    }
    lines.join("\n")
}

/// The language sharing the fallback script that the prose votes for, if its
/// votes reach [`SHARED_SCRIPT_SHARE`]. Mirrored by `sharedScriptLanguage`.
fn shared_script_language(prose: &str, rules: &[Rule], fallback_script: &str) -> Option<Language> {
    let normalized = prose.to_lowercase();
    let words: Vec<&str> = normalized
        .split(|character: char| !character.is_alphabetic())
        .filter(|word| {
            word.chars().any(|character| {
                rules
                    .iter()
                    .any(|rule| rule.script == fallback_script && rule_claims(rule, character))
            })
        })
        .collect();
    let mut best: Option<(usize, Language)> = None;
    for rule in rules
        .iter()
        .filter(|rule| rule.script == fallback_script && !rule.fallback)
    {
        let markers: usize = rule
            .markers
            .iter()
            .map(|marker| {
                marker_count(
                    &normalized,
                    &marker.to_lowercase(),
                    rules,
                    fallback_script,
                    usize::MAX,
                )
            })
            .sum();
        let letters = words
            .iter()
            .filter(|word| word.chars().any(|character| rule_claims(rule, character)))
            .count();
        let votes = markers + letters;
        // A lone vote (one marker or one accented loanword such as "café")
        // decides only a text it is at least half of.
        let lone = votes == 1 && votes * 2 < words.len();
        if votes == 0 || lone || votes * SHARED_SCRIPT_SHARE < words.len() {
            continue;
        }
        if best.is_some_and(|(best_votes, _)| votes <= best_votes) {
            continue;
        }
        best = Some((votes, Language(rule.language)));
    }
    best.map(|(_, language)| language)
}

/// The language of already-extracted prose, or `None` when it holds no letter.
/// Mirrored by `proseLanguage`.
fn prose_language(prose: &str, rules: &[Rule]) -> Option<Language> {
    let fallback_script = fallback_script_of(rules);
    let counts = count_scripts(prose, rules);
    if counts.total() == 0 {
        return None;
    }
    let fallback_count = counts.of(fallback_script);
    if counts.other > fallback_count && counts.other >= counts.max_excluding(fallback_script) {
        return Some(Language::Unknown);
    }
    let mut rival: Option<(usize, &'static str)> = None;
    for rule in rules.iter().filter(|rule| rule.script != fallback_script) {
        let count = counts.of(rule.script);
        if count > 0 && rival.is_none_or(|(best, _)| count > best) {
            rival = Some((count, rule.script));
        }
    }
    if let Some((count, script)) = rival
        && count * PROSE_SCRIPT_WEIGHT >= fallback_count
    {
        return Some(default_language_of(rules, script));
    }
    Some(
        shared_script_language(prose, rules, fallback_script).unwrap_or_else(|| fallback_of(rules)),
    )
}

/// Detect the language of a document — an issue body, a title, a user turn —
/// from the proportion of its prose, or `None` when it holds no prose letter.
///
/// Unlike [`detect`], which reads a prompt whole, this drops fenced code,
/// block quotations, code spans, quotations, URLs and identifiers first (see
/// [`prose_of`]), then weighs scripts by their share of the remaining letters.
/// A quoted Russian phrase in an English issue or a single Spanish marker
/// therefore cannot outvote the rest of the document (issue #1557). A forced
/// response language (issue #556) still wins. Mirrored by `detectProse` in
/// `js/agentic/crate/language.mjs`.
#[must_use]
pub fn detect_prose(text: &str) -> Option<Language> {
    if let Some(forced) = forced_language::get() {
        return Some(forced);
    }
    with_rules(|rules| prose_language(&prose_of(text), rules))
}
