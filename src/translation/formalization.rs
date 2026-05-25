//! Wikidata-backed prompt formalization.
//!
//! This module is the offline first slice of the E3 formalization engine:
//! it projects natural-language prompt fragments into scored meaning records
//! anchored on Wikidata P-ids/Q-ids when the local label table or concept seed
//! can identify them. When no Wikidata anchor exists, the result explicitly
//! records a Wikipedia/Wiktionary surface fallback or a raw unresolved term so
//! later translation and selection stages can reason about the gap.

use std::fmt::Write as _;

use crate::concepts;
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
            "  source_text \"{}\"",
            escape_lino_value(&self.source_text)
        );
        let _ = writeln!(out, "  language \"{}\"", self.language);
        let _ = writeln!(out, "  score \"{}\"", self.score);
        for slot in &self.slots {
            let _ = writeln!(
                out,
                "  {}_surface \"{}\"",
                slot.role.slug(),
                escape_lino_value(&slot.surface)
            );
            let _ = writeln!(
                out,
                "  {}_{} \"{}\"",
                slot.role.slug(),
                slot.anchor.kind.lino_suffix(),
                escape_lino_value(&slot.anchor.id)
            );
            let _ = writeln!(
                out,
                "  {}_score \"{}\"",
                slot.role.slug(),
                slot.anchor.score
            );
            let _ = writeln!(
                out,
                "  {}_source \"{}\"",
                slot.role.slug(),
                escape_lino_value(&slot.anchor.source)
            );
        }
        for term in &self.unresolved_terms {
            let _ = writeln!(
                out,
                "  formalization_unresolved \"{}\"",
                escape_lino_value(term)
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

#[derive(Debug, Clone, Copy)]
struct LabelEntry {
    id: &'static str,
    label: &'static str,
    aliases: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
struct PredicatePattern {
    id: &'static str,
    label: &'static str,
    aliases: &'static [&'static str],
    phrases: &'static [&'static str],
}

#[derive(Debug)]
struct ParsedRelation {
    subject: String,
    predicate_surface: String,
    predicate: PredicatePattern,
    object: String,
}

const ITEM_LABELS: &[LabelEntry] = &[
    LabelEntry {
        id: "Q89",
        label: "apple",
        aliases: &["apple", "apples", "яблоко", "яблоки", "सेब", "苹果", "蘋果"],
    },
    LabelEntry {
        id: "Q3314483",
        label: "fruit",
        aliases: &[
            "fruit",
            "fruits",
            "edible fruit",
            "фрукт",
            "фрукты",
            "плод",
            "फल",
            "水果",
            "水果类",
        ],
    },
    LabelEntry {
        id: "Q181593",
        label: "sorting algorithm",
        aliases: &["sorting algorithm", "sorting algorithms", "sort algorithm"],
    },
    LabelEntry {
        id: "Q283",
        label: "water",
        aliases: &["water", "вода", "पानी", "水"],
    },
    LabelEntry {
        id: "Q7802",
        label: "bread",
        aliases: &["bread", "хлеб", "रोटी", "面包", "麵包"],
    },
    LabelEntry {
        id: "Q81",
        label: "carrot",
        aliases: &["carrot", "carrots", "морковь", "गाजर", "胡萝卜", "胡蘿蔔"],
    },
    LabelEntry {
        id: "Q2013",
        label: "Wikidata",
        aliases: &["wikidata", "викидата", "विकिडेटा", "维基数据", "維基數據"],
    },
    LabelEntry {
        id: "Q52",
        label: "Wikipedia",
        aliases: &[
            "wikipedia",
            "википедия",
            "विकिपीडिया",
            "维基百科",
            "維基百科",
        ],
    },
    LabelEntry {
        id: "Q151",
        label: "Wiktionary",
        aliases: &[
            "wiktionary",
            "викисловарь",
            "विक्षनरी",
            "维基词典",
            "維基辭典",
        ],
    },
];

const PROPERTY_PATTERNS: &[PredicatePattern] = &[
    PredicatePattern {
        id: "P279",
        label: "subclass of",
        aliases: &["subclass of", "kind of", "type of", "род", "тип"],
        phrases: &[
            "is a kind of",
            "is a type of",
            "is subclass of",
            "subclass of",
            "kind of",
            "type of",
        ],
    },
    PredicatePattern {
        id: "P31",
        label: "instance of",
        aliases: &["instance of", "is a", "is an", "является", "это", "是", "है"],
        phrases: &[
            "is an",
            "is a",
            "is the",
            "are a",
            "are an",
            "является",
            "это",
            "是",
            "है",
        ],
    },
    PredicatePattern {
        id: "P361",
        label: "part of",
        aliases: &["part of", "belongs to", "часть", "का हिस्सा", "属于"],
        phrases: &[
            "is part of",
            "part of",
            "belongs to",
            "является частью",
            "属于",
        ],
    },
    PredicatePattern {
        id: "P527",
        label: "has part",
        aliases: &["has part", "contains", "includes", "содержит", "包含"],
        phrases: &[
            "has part",
            "has a part",
            "contains",
            "includes",
            "содержит",
            "包含",
        ],
    },
    PredicatePattern {
        id: "P36",
        label: "capital",
        aliases: &["capital", "capital of", "столица", "राजधानी", "首都"],
        phrases: &[
            "is the capital of",
            "is capital of",
            "capital of",
            "столица",
            "首都",
        ],
    },
    PredicatePattern {
        id: "P138",
        label: "named after",
        aliases: &["named after", "назван в честь", "के नाम पर", "命名自"],
        phrases: &["is named after", "named after", "назван в честь", "命名自"],
    },
    PredicatePattern {
        id: "P5972",
        label: "translation",
        aliases: &[
            "translation",
            "translate",
            "translates",
            "переведи",
            "перевести",
            "अनुवाद",
            "翻译",
            "翻譯",
        ],
        phrases: &[
            "translate",
            "translates",
            "переведи",
            "перевести",
            "अनुवाद",
            "翻译",
            "翻譯",
        ],
    },
    PredicatePattern {
        id: "P5137",
        label: "item for this sense",
        aliases: &["item for this sense", "meaning item", "sense item"],
        phrases: &["means", "meaning of", "означает", "अर्थ", "意思"],
    },
];

/// Formalize a natural-language prompt into a single scored meaning candidate.
///
/// The current engine is deliberately deterministic and offline-friendly. It
/// first checks explicit concept-question/action/relation shapes, then tries a
/// standalone surface as a last resort. Callers that need competing readings
/// can rank multiple candidates on top of this structure later.
#[must_use]
pub fn formalize_prompt(prompt: &str, language: &str) -> FormalizationCandidate {
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
        return build_candidate(prompt, &language, slots);
    }

    if let Some(object) = parse_translation_object(prompt) {
        let predicate = translation_predicate();
        slots.push(FormalizationSlot {
            role: FormalizationRole::Predicate,
            surface: predicate.label.to_owned(),
            anchor: property_anchor(predicate, predicate.label, "label:property", 980),
        });
        push_item_slot(&mut slots, FormalizationRole::Object, &object, &language);
        return build_candidate(prompt, &language, slots);
    }

    if let Some(relation) = parse_binary_relation(prompt) {
        push_item_slot(
            &mut slots,
            FormalizationRole::Subject,
            &relation.subject,
            &language,
        );
        slots.push(FormalizationSlot {
            role: FormalizationRole::Predicate,
            surface: relation.predicate_surface,
            anchor: property_anchor(
                relation.predicate,
                relation.predicate.label,
                "label:property",
                970,
            ),
        });
        push_item_slot(
            &mut slots,
            FormalizationRole::Object,
            &relation.object,
            &language,
        );
        return build_candidate(prompt, &language, slots);
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
    build_candidate(prompt, &language, slots)
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
    if let Some(entry) = ITEM_LABELS.iter().find(|entry| {
        entry
            .aliases
            .iter()
            .any(|alias| normalize_lookup(alias) == normalized)
    }) {
        return FormalizationAnchor {
            kind: FormalizationAnchorKind::WikidataItem,
            id: format!("wikidata:{}", entry.id),
            label: entry.label.to_owned(),
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
    predicate: PredicatePattern,
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

fn translation_predicate() -> PredicatePattern {
    *PROPERTY_PATTERNS
        .iter()
        .find(|predicate| predicate.id == "P5972")
        .expect("P5972 translation predicate is present")
}

fn parse_translation_object(prompt: &str) -> Option<String> {
    let trimmed = prompt.trim();
    let lower = trimmed.to_lowercase();

    for prefix in ["translate ", "please translate "] {
        if lower.starts_with(prefix) {
            let rest = trimmed.get(prefix.len()..)?.trim();
            return clean_translation_surface(rest, &[" to ", " into ", " in "]);
        }
    }

    for prefix in ["переведи ", "перевести "] {
        if lower.starts_with(prefix) {
            let rest = trimmed.get(prefix.len()..)?.trim();
            return clean_translation_surface(rest, &[" на ", " в "]);
        }
    }

    if let Some(index) = lower.find(" का ") {
        if lower.contains("अनुवाद") {
            return Some(clean_argument(&trimmed[..index]));
        }
    }

    if let Some(rest_lower) = lower.strip_prefix("把 ") {
        if let Some(index) = rest_lower.find(" 翻译") {
            let start = trimmed.len() - rest_lower.len();
            return Some(clean_argument(&trimmed[start..start + index]));
        }
        if let Some(index) = rest_lower.find(" 翻譯") {
            let start = trimmed.len() - rest_lower.len();
            return Some(clean_argument(&trimmed[start..start + index]));
        }
    }

    for marker in ["翻译", "翻譯"] {
        if let Some(index) = lower.find(marker) {
            let rest = trimmed[index + marker.len()..].trim();
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
    for predicate in PROPERTY_PATTERNS {
        if predicate.id == "P5972" {
            continue;
        }
        for phrase in predicate.phrases.iter().chain(predicate.aliases.iter()) {
            let Some((start, len)) = find_relation_phrase(&lower, phrase) else {
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
                predicate: *predicate,
                object,
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

fn escape_lino_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_prompt_extracts_subject_predicate_object() {
        let candidate = formalize_prompt("apple is a fruit", "en");
        assert_eq!(
            candidate
                .slot(FormalizationRole::Subject)
                .unwrap()
                .anchor
                .id,
            "wikidata:Q89"
        );
        assert_eq!(
            candidate
                .slot(FormalizationRole::Predicate)
                .unwrap()
                .anchor
                .id,
            "wikidata:P31"
        );
        assert_eq!(
            candidate.slot(FormalizationRole::Object).unwrap().anchor.id,
            "wikidata:Q3314483"
        );
    }

    #[test]
    fn russian_translation_prompt_uses_multilingual_label_table() {
        let candidate = formalize_prompt("переведи яблоко на английский", "ru");
        assert_eq!(
            candidate
                .slot(FormalizationRole::Predicate)
                .unwrap()
                .anchor
                .id,
            "wikidata:P5972"
        );
        assert_eq!(
            candidate.slot(FormalizationRole::Object).unwrap().anchor.id,
            "wikidata:Q89"
        );
    }
}
