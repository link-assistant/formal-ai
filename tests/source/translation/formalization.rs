//! Wikidata-backed prompt formalization.
//!
//! This module is the offline first slice of the E3 formalization engine:
//! it projects natural-language prompt fragments into scored meaning records
//! anchored on Wikidata P-ids/Q-ids when the local label table or concept seed
//! can identify them. When no Wikidata anchor exists, the result explicitly
//! records a Wikipedia/Wiktionary surface fallback or a raw unresolved term so
//! later translation and selection stages can reason about the gap.

use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::concepts;
use crate::links_format::format_lino_value;
use crate::seed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormalizationAnchorKind {
    WikidataItem,
    WikidataProperty,
    WikipediaArticle,
    WiktionaryEntry,
    RawText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormalizationRole {
    Subject,
    Predicate,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalizationAnchor {
    pub kind: FormalizationAnchorKind,
    pub id: String,
    pub label: String,
    pub source: String,
    pub score: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalizationSlot {
    pub role: FormalizationRole,
    pub surface: String,
    pub anchor: FormalizationAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalizationCandidate {
    pub source_text: String,
    pub language: String,
    pub slots: Vec<FormalizationSlot>,
    pub score: u16,
    pub unresolved_terms: Vec<String>,
}

impl FormalizationCandidate {
    #[must_use]
    pub fn slot(&self, role: FormalizationRole) -> Option<&FormalizationSlot> {
        self.slots.iter().find(|slot| slot.role == role)
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::from("formalization_candidate\n");
        let _ = writeln!(
            out,
            "  source_text {}",
            format_lino_value(&self.source_text)
        );
        let _ = writeln!(out, "  language {}", format_lino_value(&self.language));
        let _ = writeln!(
            out,
            "  score {}",
            format_lino_value(&self.score.to_string())
        );
        for slot in &self.slots {
            let _ = writeln!(
                out,
                "  {}_surface {}",
                slot.role.slug(),
                format_lino_value(&slot.surface)
            );
            let _ = writeln!(
                out,
                "  {}_{} {}",
                slot.role.slug(),
                slot.anchor.kind.lino_suffix(),
                format_lino_value(&slot.anchor.id)
            );
            let _ = writeln!(
                out,
                "  {}_score {}",
                slot.role.slug(),
                format_lino_value(&slot.anchor.score.to_string())
            );
            let _ = writeln!(
                out,
                "  {}_source {}",
                slot.role.slug(),
                format_lino_value(&slot.anchor.source)
            );
        }
        for term in &self.unresolved_terms {
            let _ = writeln!(
                out,
                "  formalization_unresolved {}",
                format_lino_value(term)
            );
        }
        out
    }

    #[must_use]
    pub fn compact_summary(&self) -> String {
        let mut parts: Vec<String> = self
            .slots
            .iter()
            .map(|slot| format!("{}={}", slot.role.slug(), slot.anchor.id))
            .collect();
        if !self.unresolved_terms.is_empty() {
            parts.push(format!("unresolved={}", self.unresolved_terms.join("|")));
        }
        if parts.is_empty() {
            String::from("empty")
        } else {
            parts.join(" ")
        }
    }
}

impl FormalizationRole {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Subject => "subject",
            Self::Predicate => "predicate",
            Self::Object => "object",
        }
    }
}

impl FormalizationAnchorKind {
    #[must_use]
    pub const fn lino_suffix(self) -> &'static str {
        match self {
            Self::WikidataItem => "q",
            Self::WikidataProperty => "p",
            Self::WikipediaArticle => "wikipedia",
            Self::WiktionaryEntry => "wiktionary",
            Self::RawText => "raw",
        }
    }
}

/// One Wikidata item (entity or class), projected from the seed lexicon.
///
/// Built once from the meanings carrying [`seed::ROLE_WIKIDATA_ENTITY_ANCHOR`]
/// (`data/seed/meanings-wikidata.lino`): `id` is the language-independent Q-id
/// recorded in the meaning's `wikidata` field, `label` its canonical English
/// surface (the first English word), and `aliases` every multilingual surface
/// that resolves to the item. The hardcoded `ITEM_LABELS` table this replaced is
/// gone — the formalizer now references the concept by role, not by raw words in
/// one language (issue #386).
#[derive(Debug, Clone)]
struct LabelEntry {
    id: String,
    label: String,
    aliases: Vec<String>,
}

/// One Wikidata binary-relation or translation property, projected from the seed
/// lexicon.
///
/// Built once from the meanings carrying [`seed::ROLE_BINARY_RELATION_PROPERTY`]
/// or [`seed::ROLE_TRANSLATION_PROPERTY`] (`data/seed/meanings-wikidata.lino`):
/// `id` is the language-independent P-id, `label` its canonical English surface,
/// and `surfaces` every multilingual phrase that signals the relation. The
/// hardcoded `PROPERTY_PATTERNS` table this replaced is gone (issue #386).
#[derive(Debug, Clone)]
struct PredicatePattern {
    slug: String,
    id: String,
    label: String,
    surfaces: Vec<PredicateSurface>,
    binary_relation: bool,
}

/// One surface phrase of a [`PredicatePattern`], paired with the meaning slug of
/// the ambiguous property the form also admits, if any.
///
/// `alternative_slug` is the word form's `action`: when a copular form such as
/// `is a` can read as either instance-of (P31) or subclass-of (P279), the seed
/// records the alternative property's slug there. This replaces the hardcoded
/// `P31`→`P279` special case with a data-driven pointer (issue #386).
#[derive(Debug, Clone)]
struct PredicateSurface {
    text: String,
    alternative_slug: String,
}

#[derive(Debug)]
struct ParsedRelation {
    subject: String,
    predicate_surface: String,
    predicate: &'static PredicatePattern,
    object: String,
    alternative_slug: String,
}

/// The Wikidata items, projected once from the seed lexicon and cached.
fn item_entries() -> &'static [LabelEntry] {
    static ITEMS: OnceLock<Vec<LabelEntry>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        seed::lexicon()
            .meanings_with_role(seed::ROLE_WIKIDATA_ENTITY_ANCHOR)
            .map(|meaning| LabelEntry {
                id: meaning.wikidata.clone(),
                label: meaning.word_in("en").unwrap_or_default().to_owned(),
                aliases: meaning.words().map(str::to_owned).collect(),
            })
            .collect()
    })
}

/// The Wikidata properties, projected once from the seed lexicon and cached.
/// Binary-relation properties (in declaration order) come first, then the lone
/// translation property.
fn predicate_patterns() -> &'static [PredicatePattern] {
    static PATTERNS: OnceLock<Vec<PredicatePattern>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let lexicon = seed::lexicon();
        let mut patterns: Vec<PredicatePattern> = Vec::new();
        for (role, binary_relation) in [
            (seed::ROLE_BINARY_RELATION_PROPERTY, true),
            (seed::ROLE_TRANSLATION_PROPERTY, false),
        ] {
            for meaning in lexicon.meanings_with_role(role) {
                patterns.push(PredicatePattern {
                    slug: meaning.slug.clone(),
                    id: meaning.wikidata.clone(),
                    label: meaning.word_in("en").unwrap_or_default().to_owned(),
                    surfaces: meaning
                        .word_forms()
                        .map(|form| PredicateSurface {
                            text: form.text.clone(),
                            alternative_slug: form.action.clone(),
                        })
                        .collect(),
                    binary_relation,
                });
            }
        }
        patterns
    })
}

/// The binary-relation properties in declaration order (the translation property
/// is matched through its own action branch, not as a relation phrase).
fn binary_relation_patterns() -> impl Iterator<Item = &'static PredicatePattern> {
    predicate_patterns()
        .iter()
        .filter(|pattern| pattern.binary_relation)
}

/// The surfaces of a predicate in maximal-munch order: longest phrase first so a
/// specific form (`is subclass of`) is preferred over the shorter one it
/// contains (`subclass of`). Ties keep declaration order (a stable sort), which
/// preserves each language's authored priority.
fn surfaces_by_munch(predicate: &'static PredicatePattern) -> Vec<&'static PredicateSurface> {
    let mut surfaces: Vec<&PredicateSurface> = predicate.surfaces.iter().collect();
    surfaces.sort_by_key(|surface| std::cmp::Reverse(surface.text.chars().count()));
    surfaces
}

fn predicate_by_slug(slug: &str) -> Option<&'static PredicatePattern> {
    predicate_patterns()
        .iter()
        .find(|pattern| pattern.slug == slug)
}

/// Formalize a natural-language prompt into the highest-scored meaning
/// candidate.
///
/// Use [`formalize_prompt_candidates`] when the caller needs to inspect or
/// select among competing interpretations.
#[must_use]
pub fn formalize_prompt(prompt: &str, language: &str) -> FormalizationCandidate {
    formalize_prompt_candidates(prompt, language)
        .into_iter()
        .next()
        .unwrap_or_else(|| {
            let language = normalize_language(language);
            build_candidate(prompt, &language, Vec::new())
        })
}

/// Formalize a natural-language prompt into one or more scored candidates.
///
/// The current engine is deliberately deterministic and offline-friendly. It
/// first checks explicit concept-question/action/relation shapes, then tries a
/// standalone surface as a last resort. Ambiguous relation phrasing emits the
/// plausible alternatives so the temperature selector can choose or ask.
#[must_use]
pub fn formalize_prompt_candidates(prompt: &str, language: &str) -> Vec<FormalizationCandidate> {
    let language = normalize_language(language);
    let mut slots = Vec::new();

    if let Some(query) = concepts::extract_concept_query(prompt) {
        push_item_slot(
            &mut slots,
            FormalizationRole::Subject,
            &query.term,
            &language,
        );
        if let Some(context) = query.context {
            push_item_slot(&mut slots, FormalizationRole::Object, &context, &language);
        }
        return vec![build_candidate(prompt, &language, slots)];
    }

    if let Some(object) = parse_translation_object(prompt) {
        let predicate = translation_predicate();
        slots.push(FormalizationSlot {
            role: FormalizationRole::Predicate,
            surface: predicate.label.clone(),
            anchor: property_anchor(predicate, &predicate.label, "label:property", 980),
        });
        push_item_slot(&mut slots, FormalizationRole::Object, &object, &language);
        return vec![build_candidate(prompt, &language, slots)];
    }

    if let Some(relation) = parse_binary_relation(prompt) {
        let mut candidates = vec![build_relation_candidate(
            prompt,
            &language,
            &relation.subject,
            relation.predicate,
            &relation.predicate_surface,
            &relation.object,
            970,
        )];
        for (predicate, score) in ambiguous_relation_alternatives(&relation) {
            candidates.push(build_relation_candidate(
                prompt,
                &language,
                &relation.subject,
                predicate,
                &predicate.label,
                &relation.object,
                score,
            ));
        }
        sort_candidates(&mut candidates);
        return candidates;
    }

    let standalone = clean_argument(prompt);
    if is_reasonable_standalone_surface(&standalone) {
        push_item_slot(
            &mut slots,
            FormalizationRole::Subject,
            &standalone,
            &language,
        );
    }
    vec![build_candidate(prompt, &language, slots)]
}

fn build_relation_candidate(
    prompt: &str,
    language: &str,
    subject: &str,
    predicate: &PredicatePattern,
    predicate_surface: &str,
    object: &str,
    predicate_score: u16,
) -> FormalizationCandidate {
    let mut slots = Vec::new();
    push_item_slot(&mut slots, FormalizationRole::Subject, subject, language);
    slots.push(FormalizationSlot {
        role: FormalizationRole::Predicate,
        surface: predicate_surface.to_owned(),
        anchor: property_anchor(
            predicate,
            &predicate.label,
            "label:property",
            predicate_score,
        ),
    });
    push_item_slot(&mut slots, FormalizationRole::Object, object, language);
    build_candidate(prompt, language, slots)
}

/// The plausible alternative readings of an ambiguous relation surface.
///
/// Fully data-driven: the matched word form names its alternative property's
/// meaning slug in `action` (for example `is a` → `wikidata_property_subclass_of`).
/// A form with no `action` yields no alternative, so `is the` and `instance of`
/// stay unambiguous P31 even though `is a` does not — exactly the former
/// hardcoded surface gate, now read from the seed (issue #386).
fn ambiguous_relation_alternatives(
    relation: &ParsedRelation,
) -> Vec<(&'static PredicatePattern, u16)> {
    if relation.alternative_slug.is_empty() {
        return Vec::new();
    }
    predicate_by_slug(&relation.alternative_slug)
        .map(|predicate| vec![(predicate, 955)])
        .unwrap_or_default()
}

fn sort_candidates(candidates: &mut Vec<FormalizationCandidate>) {
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.compact_summary().cmp(&right.compact_summary()))
    });
    candidates.dedup_by(|left, right| left.compact_summary() == right.compact_summary());
}

fn build_candidate(
    prompt: &str,
    language: &str,
    slots: Vec<FormalizationSlot>,
) -> FormalizationCandidate {
    let unresolved_terms = slots
        .iter()
        .filter(|slot| slot.anchor.kind == FormalizationAnchorKind::RawText)
        .map(|slot| slot.surface.clone())
        .collect::<Vec<_>>();
    let score = if slots.is_empty() {
        0
    } else {
        let total = slots
            .iter()
            .map(|slot| u32::from(slot.anchor.score))
            .sum::<u32>();
        let count = u32::try_from(slots.len()).unwrap_or(u32::MAX);
        u16::try_from(total / count).unwrap_or(u16::MAX)
    };
    FormalizationCandidate {
        source_text: prompt.to_owned(),
        language: language.to_owned(),
        slots,
        score,
        unresolved_terms,
    }
}

fn push_item_slot(
    slots: &mut Vec<FormalizationSlot>,
    role: FormalizationRole,
    surface: &str,
    language: &str,
) {
    let surface = clean_argument(surface);
    if surface.is_empty() {
        return;
    }
    slots.push(FormalizationSlot {
        role,
        anchor: resolve_item_anchor(&surface, language),
        surface,
    });
}

fn resolve_item_anchor(surface: &str, language: &str) -> FormalizationAnchor {
    let normalized = normalize_lookup(surface);
    if let Some(entry) = item_entries().iter().find(|entry| {
        entry
            .aliases
            .iter()
            .any(|alias| normalize_lookup(alias) == normalized)
    }) {
        return FormalizationAnchor {
            kind: FormalizationAnchorKind::WikidataItem,
            id: format!("wikidata:{}", entry.id),
            label: entry.label.clone(),
            source: String::from("label:wikidata-item"),
            score: 980,
        };
    }

    if let Some(anchor) = resolve_seed_concept_anchor(&normalized) {
        return anchor;
    }

    if looks_unanchored_unknown(&normalized) {
        return FormalizationAnchor {
            kind: FormalizationAnchorKind::RawText,
            id: format!("raw:{normalized}"),
            label: surface.to_owned(),
            source: String::from("fallback:raw"),
            score: 0,
        };
    }

    if surface.split_whitespace().count() > 1 || looks_like_named_article(surface) {
        return FormalizationAnchor {
            kind: FormalizationAnchorKind::WikipediaArticle,
            id: format!("wikipedia:{language}:{}", wikipedia_title(surface)),
            label: surface.to_owned(),
            source: String::from("fallback:wikipedia-surface"),
            score: 520,
        };
    }

    FormalizationAnchor {
        kind: FormalizationAnchorKind::WiktionaryEntry,
        id: format!("wiktionary:{language}:{normalized}"),
        label: surface.to_owned(),
        source: String::from("fallback:wiktionary-surface"),
        score: 560,
    }
}

fn resolve_seed_concept_anchor(normalized: &str) -> Option<FormalizationAnchor> {
    for record in seed::concepts() {
        if record.wikidata.trim().is_empty() {
            continue;
        }
        let mut labels = vec![record.term.as_str()];
        labels.extend(record.aliases.iter().map(String::as_str));
        for localized in &record.localized {
            labels.push(localized.term.as_str());
            labels.extend(localized.aliases.iter().map(String::as_str));
        }
        let slug_label = record
            .slug
            .strip_prefix("concept_")
            .unwrap_or(&record.slug)
            .replace('_', " ");
        if labels
            .iter()
            .any(|label| normalize_lookup(label) == normalized)
            || normalize_lookup(&slug_label) == normalized
        {
            return Some(FormalizationAnchor {
                kind: FormalizationAnchorKind::WikidataItem,
                id: format!("wikidata:{}", record.wikidata.trim()),
                label: record.term,
                source: String::from("seed:concepts"),
                score: 940,
            });
        }
    }
    None
}

fn property_anchor(
    predicate: &PredicatePattern,
    surface: &str,
    source: &str,
    score: u16,
) -> FormalizationAnchor {
    FormalizationAnchor {
        kind: FormalizationAnchorKind::WikidataProperty,
        id: format!("wikidata:{}", predicate.id),
        label: surface.to_owned(),
        source: source.to_owned(),
        score,
    }
}

fn translation_predicate() -> &'static PredicatePattern {
    predicate_patterns()
        .iter()
        .find(|pattern| !pattern.binary_relation)
        .expect("the translation property is present in the seed")
}

fn parse_translation_object(prompt: &str) -> Option<String> {
    use crate::seed::ROLE_TRANSLATION_ACTION;

    if let Some(surface) = crate::translation::extract_unquoted_translation_surface(prompt) {
        return Some(surface);
    }

    let trimmed = prompt.trim();
    let lower = trimmed.to_lowercase();
    // Issue #386: the translation command *verbs* are read from the lexicon
    // (data/seed/meanings-translation.lino, role translation_action) rather than
    // hardcoded per language. Only the closed-class grammatical particles that
    // bound the surface — target prepositions and object/disposal markers —
    // remain as structural delimiters here, the way an extractor's affix anchors
    // do; they carry no translatable concept of their own.
    let lexicon = crate::seed::lexicon();

    // Clause-initial English commands, optionally behind a "please" politeness
    // particle: strip the verb, read the surface up to the first target preposition.
    for stem in lexicon.words_for_role_in_languages(ROLE_TRANSLATION_ACTION, &["en"]) {
        for prefix in [format!("{stem} "), format!("please {stem} ")] {
            if lower.starts_with(&prefix) {
                let rest = trimmed.get(prefix.len()..)?.trim();
                return clean_translation_surface(rest, &[" to ", " into ", " in "]);
            }
        }
    }

    // Clause-initial Russian commands.
    for stem in lexicon.words_for_role_in_languages(ROLE_TRANSLATION_ACTION, &["ru"]) {
        let prefix = format!("{stem} ");
        if lower.starts_with(&prefix) {
            let rest = trimmed.get(prefix.len()..)?.trim();
            return clean_translation_surface(rest, &[" на ", " в "]);
        }
    }

    // Head-final Hindi: the object precedes the का object particle and the verb.
    if let Some(index) = lower.find(" का ") {
        let has_verb = lexicon
            .words_for_role_in_languages(ROLE_TRANSLATION_ACTION, &["hi"])
            .iter()
            .any(|verb| lower.contains(verb.as_str()));
        if has_verb {
            return Some(clean_argument(&trimmed[..index]));
        }
    }

    // Head-final Chinese: an optional 把 disposal particle fronts the object and
    // the verb follows it; otherwise the verb fronts and the object follows.
    let zh_verbs = lexicon.words_for_role_in_languages(ROLE_TRANSLATION_ACTION, &["zh"]);
    if let Some(rest_lower) = lower.strip_prefix("把 ") {
        for verb in &zh_verbs {
            let needle = format!(" {verb}");
            if let Some(index) = rest_lower.find(&needle) {
                let start = trimmed.len() - rest_lower.len();
                return Some(clean_argument(&trimmed[start..start + index]));
            }
        }
    }

    for verb in &zh_verbs {
        if let Some(index) = lower.find(verb.as_str()) {
            let rest = trimmed[index + verb.len()..].trim();
            return clean_translation_surface(rest, &["成", "为", "為", " to "]);
        }
    }

    None
}

fn clean_translation_surface(rest: &str, delimiters: &[&str]) -> Option<String> {
    let lower = rest.to_lowercase();
    let mut end = rest.len();
    for delimiter in delimiters {
        if let Some(index) = lower.find(delimiter) {
            end = end.min(index);
        }
    }
    let surface = clean_argument(&rest[..end]);
    (!surface.is_empty()).then_some(surface)
}

fn parse_binary_relation(prompt: &str) -> Option<ParsedRelation> {
    let trimmed = prompt.trim();
    let lower = trimmed.to_lowercase();
    for predicate in binary_relation_patterns() {
        for surface in surfaces_by_munch(predicate) {
            let Some((start, len)) = find_relation_phrase(&lower, &surface.text) else {
                continue;
            };
            if start >= trimmed.len() || start + len > trimmed.len() {
                continue;
            }
            let subject = clean_argument(&trimmed[..start]);
            let object = clean_argument(&trimmed[start + len..]);
            if subject.is_empty() || object.is_empty() {
                continue;
            }
            return Some(ParsedRelation {
                subject,
                predicate_surface: trimmed[start..start + len].trim().to_owned(),
                predicate,
                object,
                alternative_slug: surface.alternative_slug.clone(),
            });
        }
    }
    None
}

fn find_relation_phrase(lower: &str, phrase: &str) -> Option<(usize, usize)> {
    let padded = format!(" {phrase} ");
    if let Some(index) = lower.find(&padded) {
        return Some((index + 1, phrase.len()));
    }
    let prefix = format!("{phrase} ");
    if lower.starts_with(&prefix) {
        return Some((0, phrase.len()));
    }
    let suffix = format!(" {phrase}");
    if let Some(stem) = lower.strip_suffix(&suffix) {
        return Some((stem.len() + 1, phrase.len()));
    }
    if phrase.is_ascii() {
        None
    } else {
        lower.find(phrase).map(|index| (index, phrase.len()))
    }
}

fn clean_argument(value: &str) -> String {
    let mut cleaned = value
        .trim()
        .trim_matches([
            '"', '\'', '`', '“', '”', '‘', '’', '«', '»', '(', ')', '[', ']',
        ])
        .trim()
        .trim_end_matches(['?', '。', '.', '!', ',', ';', ':'])
        .trim()
        .to_owned();
    let lower = cleaned.to_lowercase();
    for article in ["a ", "an ", "the "] {
        if let Some(rest) = lower.strip_prefix(article) {
            let start = cleaned.len() - rest.len();
            let without_article = cleaned[start..].trim().to_owned();
            cleaned = without_article;
            break;
        }
    }
    cleaned
}

fn normalize_lookup(value: &str) -> String {
    clean_argument(value)
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_language(language: &str) -> String {
    match language.trim().to_lowercase().as_str() {
        "ru" | "russian" => String::from("ru"),
        "hi" | "hindi" => String::from("hi"),
        "zh" | "chinese" | "cn" => String::from("zh"),
        "en" | "english" => String::from("en"),
        other if !other.is_empty() => other.to_owned(),
        _ => String::from("en"),
    }
}

fn is_reasonable_standalone_surface(surface: &str) -> bool {
    let words = surface.split_whitespace().count();
    (1..=4).contains(&words) && surface.chars().any(char::is_alphabetic)
}

fn looks_unanchored_unknown(normalized: &str) -> bool {
    let letters = normalized
        .chars()
        .filter(|character| character.is_alphabetic())
        .collect::<Vec<_>>();
    if letters.is_empty() {
        return true;
    }
    let lower = normalized.to_lowercase();
    if lower.contains("zzq") || lower.contains("qxq") || lower.contains("xq") {
        return true;
    }
    if !letters.iter().all(char::is_ascii) {
        return false;
    }
    let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
    if !letters.iter().any(|character| vowels.contains(character)) {
        return true;
    }
    let mut consonant_run = 0usize;
    for character in letters {
        if vowels.contains(&character) {
            consonant_run = 0;
        } else {
            consonant_run += 1;
            if consonant_run >= 5 {
                return true;
            }
        }
    }
    false
}

fn looks_like_named_article(surface: &str) -> bool {
    surface.chars().next().is_some_and(char::is_uppercase)
        && surface.chars().any(char::is_lowercase)
}

fn wikipedia_title(surface: &str) -> String {
    clean_argument(surface)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
}

#[path = "../source_tests/translation/formalization/tests.rs"]
mod tests;
