//! Plan 00 §4.2's `SourceLookup`, implemented exactly once (issue #1138, plan
//! 01 L5–L7).
//!
//! A retrieved sense carries the provenance of the exact bytes it was read
//! from, so a gloss can be quoted and attributed but never inlined into a
//! generated program. Which extractor reads which source is declared by the
//! registry's `extractor` field, never by a host-name `match` in Rust: adding a
//! source is a seed edit plus one reader function, and a source that declares
//! no extractor is reported as contributing nothing rather than guessed at.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde_json::Value;

use crate::how_to_guide::ServicePreferences;
use crate::needs::{Need, NeedKind};
use crate::relative_meta_logic::SourceTier;
use crate::seed::SourceRecord;
use crate::service_accessibility::ServiceAccessibilityCache;
use crate::source_fetch::{CachedSourceClient, SourceCapture, SourceTransport};
use crate::source_walk::{
    CaptureExtractor, Extracted, LookupBounds, SourceLookup, Walk, WalkOutcome, WalkSourceOutcome,
    entry_url_in, walk_sources,
};
use crate::trace_record;

/// One retrieved sense of one surface form, with the provenance of the exact
/// bytes it was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptSense {
    /// The surface the caller asked about, as written.
    pub surface: String,
    /// The headword the source answered under.
    pub lemma: String,
    /// The language the sense is stated in.
    pub language: String,
    /// What the source says the surface means.
    pub gloss: String,
    /// Part of speech, when the source states one.
    pub part_of_speech: String,
    /// Surfaces the source names as sharing this sense.
    pub synonyms: Vec<String>,
    /// Registry id of the source the bytes came from.
    pub source_id: String,
    /// The exact URL whose bytes carry this sense.
    pub source_url: String,
    /// sha256 of those bytes.
    pub sha256: String,
    /// When the bytes were retrieved, as the cache records it.
    pub fetched_at: String,
    /// Whether the bytes came from the capture cache rather than the network.
    pub cached: bool,
    /// The #709 tier the bytes carry.
    pub tier: SourceTier,
    /// License the gloss is quoted under.
    pub license_name: String,
    /// Canonical URL of that license.
    pub license_url: String,
    /// How many link hops past the service's entry request produced the bytes.
    pub depth: usize,
}

impl ConceptSense {
    /// Stable id over the bytes, the headword, the language and the gloss.
    ///
    /// This is the value the forget / rediscover round trip compares: it is
    /// derived from what a source published, so replaying the same captures
    /// reproduces it and nothing else can.
    #[must_use]
    pub fn content_id(&self) -> String {
        crate::engine::stable_id(
            "sense",
            &[
                self.sha256.as_str(),
                self.lemma.as_str(),
                self.language.as_str(),
                self.gloss.as_str(),
            ]
            .join("\u{1f}"),
        )
    }

    /// The sense as a link, gloss and attribution together.
    ///
    /// A gloss never travels without the page and the license it was quoted
    /// from, and it is never projected as a definition of a program symbol: the
    /// notation carries evidence, and `composition` may read it to *select* a
    /// seeded structure, never to emit source.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        crate::links_format::push_lino_node(&mut out, 0, "concept_sense", Some(&self.content_id()));
        let rows: [(&str, &str); 11] = [
            ("surface", &self.surface),
            ("lemma", &self.lemma),
            ("language", &self.language),
            ("gloss", &self.gloss),
            ("part_of_speech", &self.part_of_speech),
            ("source", &self.source_id),
            ("source_url", &self.source_url),
            ("sha256", &self.sha256),
            ("fetched_at", &self.fetched_at),
            ("license_name", &self.license_name),
            ("license_url", &self.license_url),
        ];
        for (name, value) in rows {
            crate::links_format::push_lino_node(&mut out, 2, name, Some(value));
        }
        crate::links_format::push_lino_node(&mut out, 2, "tier", Some(self.tier.slug()));
        crate::links_format::push_lino_node(&mut out, 2, "depth", Some(&self.depth.to_string()));
        for synonym in &self.synonyms {
            crate::links_format::push_lino_node(&mut out, 2, "synonym", Some(synonym));
        }
        out
    }

    /// Exact provenance for one sense, in the order a reviewer checks it.
    #[must_use]
    pub fn provenance(&self) -> String {
        trace_record::payload(&[
            ("source", self.source_id.clone()),
            ("url", self.source_url.clone()),
            ("sha256", self.sha256.clone()),
            ("fetched_at", self.fetched_at.clone()),
            ("cached", self.cached.to_string()),
            ("tier", self.tier.slug().to_owned()),
            ("license", self.license_name.clone()),
            ("depth", self.depth.to_string()),
        ])
    }
}

/// What a lookup produced, or honestly did not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupOutcome {
    /// Evidence, in consultation order. Every item carries the digest of the
    /// bytes it was read from.
    Found(Vec<ConceptSense>),
    /// No source answered. `consulted` names every source and its observed
    /// outcome, so the absence is attributable rather than bare.
    NotFound {
        /// One row per source that could have contributed.
        consulted: Vec<WalkSourceOutcome>,
    },
}

/// Registry-declared extractor bindings, chosen by the registry's `extractor`
/// field rather than by a host-name match in Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenseExtractor {
    surface: String,
    language: String,
}

impl SenseExtractor {
    /// Read `surface` in `language` out of whatever a source published.
    #[must_use]
    pub fn new(surface: &str, language: &str) -> Self {
        Self {
            surface: surface.trim().to_owned(),
            language: language.trim().to_owned(),
        }
    }
}

impl CaptureExtractor for SenseExtractor {
    type Item = ConceptSense;

    fn entry_url(&self, record: &SourceRecord, subject: &str) -> Option<String> {
        entry_url_in(record, subject, &self.language)
    }

    fn read(
        &self,
        record: &SourceRecord,
        capture: &SourceCapture,
        depth: usize,
        produced: usize,
        bounds: &LookupBounds,
    ) -> Extracted<Self::Item> {
        let mut read = Extracted::default();
        let limit = bounds.max_items.saturating_sub(produced);
        if limit == 0 {
            read.detail = Some(trace_record::line(
                "item_bound_reached",
                &[("max_items", bounds.max_items.to_string())],
            ));
            return read;
        }
        if record.extractor.is_empty() {
            // The source answers this need kind but nothing in the tree knows
            // how to read its payload. Saying so is the honest outcome; making
            // one up from the raw bytes is how a scraped fragment becomes an
            // answer (wave F finding 2).
            read.detail = Some(trace_record::line(
                "no_declared_extractor",
                &[("source", record.id.clone())],
            ));
            return read;
        }
        let glosses = read_glosses(&record.extractor, capture.bytes(), &self.surface);
        if glosses.is_empty() {
            read.detail = Some(trace_record::line(
                "no_sense_in_payload",
                &[
                    ("extractor", record.extractor.clone()),
                    ("url", capture.source_url().to_owned()),
                ],
            ));
            return read;
        }
        read.items = glosses
            .into_iter()
            .take(limit)
            .map(|gloss| ConceptSense {
                surface: self.surface.clone(),
                lemma: gloss.lemma,
                language: self.sense_language(record),
                gloss: gloss.gloss,
                part_of_speech: gloss.part_of_speech,
                synonyms: gloss.synonyms,
                source_id: record.id.clone(),
                source_url: capture.source_url().to_owned(),
                sha256: capture.sha256().to_owned(),
                fetched_at: capture.fetched_at().to_owned(),
                cached: capture.cached(),
                tier: record.tier,
                license_name: record.license_name.clone(),
                license_url: record.license_url.clone(),
                depth,
            })
            .collect();
        read
    }
}

impl SenseExtractor {
    /// The language a sense retrieved from `record` is stated in.
    ///
    /// The asked-for language when the caller named one, and otherwise the
    /// language the endpoint leads with. It is never inferred from the bytes:
    /// a source that serves only English says so in the registry.
    fn sense_language(&self, record: &SourceRecord) -> String {
        if !self.language.is_empty() {
            return self.language.clone();
        }
        record
            .api_language
            .first()
            .cloned()
            .unwrap_or_else(|| self.language.clone())
    }
}

/// One gloss as a source published it, before provenance is attached.
struct RawGloss {
    lemma: String,
    gloss: String,
    part_of_speech: String,
    synonyms: Vec<String>,
}

/// Dispatch on the registry's declared extractor, never on a host name.
fn read_glosses(extractor: &str, bytes: &[u8], surface: &str) -> Vec<RawGloss> {
    let text = String::from_utf8_lossy(bytes);
    let json: Option<Value> = serde_json::from_str(text.trim()).ok();
    match (extractor, json) {
        ("wiktionary_entry_v1", Some(value)) => wiktionary_entry(&value, surface),
        ("wordnet_sense_v1", Some(value)) => wordnet_sense(&value, surface),
        ("mediawiki_summary_v1", Some(value)) => mediawiki_summary(&value, surface),
        ("wikidata_entity_v1", Some(value)) => wikidata_entity(&value, surface),
        // The committed `data/cache/**/*.lino` projections carry the same
        // schema as the live payloads, written by issue #398's lossless codec.
        // One reader serves both because both spell a definition `definition`.
        (_, None) => projection_glosses(&text, surface),
        _ => Vec::new(),
    }
}

/// The Free Dictionary API's Wiktionary entry: an array of entries, each with
/// `meanings[].definitions[].definition`.
fn wiktionary_entry(value: &Value, surface: &str) -> Vec<RawGloss> {
    // One source, two published surfaces (plan 01 L10). The Free Dictionary
    // API answers English as an array of entries; every other language is asked
    // through that language edition's own MediaWiki `extracts` endpoint, which
    // answers an object. The registry says they are one service, so one reader
    // recognises both of its own payloads rather than a second registry row
    // taking a second `max_services` slot.
    if !value.is_array() {
        return wiktionary_extract(value, surface);
    }
    let mut out = Vec::new();
    for entry in value.as_array().map(Vec::as_slice).unwrap_or_default() {
        let lemma = string_at(entry, "word").unwrap_or_else(|| surface.to_owned());
        for meaning in array_at(entry, "meanings") {
            let part_of_speech = string_at(meaning, "partOfSpeech").unwrap_or_default();
            let synonyms = strings_at(meaning, "synonyms");
            for definition in array_at(meaning, "definitions") {
                if let Some(gloss) = string_at(definition, "definition") {
                    out.push(RawGloss {
                        lemma: lemma.clone(),
                        gloss,
                        part_of_speech: part_of_speech.clone(),
                        synonyms: synonyms.clone(),
                    });
                }
            }
        }
    }
    out
}

/// Open English `WordNet`'s per-lemma endpoint: an array of synsets, each with a
/// `definition` list and the `members` that share it.
fn wordnet_sense(value: &Value, surface: &str) -> Vec<RawGloss> {
    let mut out = Vec::new();
    for synset in value.as_array().map(Vec::as_slice).unwrap_or_default() {
        let part_of_speech = string_at(synset, "partOfSpeech").unwrap_or_default();
        let synonyms: Vec<String> = array_at(synset, "members")
            .iter()
            .filter_map(|member| string_at(member, "lemma"))
            .filter(|lemma| !lemma.eq_ignore_ascii_case(surface))
            .collect();
        for definition in array_at(synset, "definition") {
            if let Some(gloss) = definition.as_str().map(compact) {
                out.push(RawGloss {
                    lemma: surface.to_owned(),
                    gloss,
                    part_of_speech: part_of_speech.clone(),
                    synonyms: synonyms.clone(),
                });
            }
        }
    }
    out
}

/// `MediaWiki`'s REST summary: one `extract` under the title the wiki resolved
/// the request to, which may be a redirect target and is reported as such.
fn mediawiki_summary(value: &Value, surface: &str) -> Vec<RawGloss> {
    let Some(extract) = string_at(value, "extract").filter(|text| !text.is_empty()) else {
        return Vec::new();
    };
    let lemma = string_at(value, "title").unwrap_or_else(|| surface.to_owned());
    vec![RawGloss {
        lemma,
        gloss: extract,
        part_of_speech: String::new(),
        synonyms: Vec::new(),
    }]
}

/// A per-language Wiktionary entry, read through the `MediaWiki` `extracts` API:
/// `query.pages.<id>.extract`, a plain-text rendering of the wiki page.
///
/// **What counts as a gloss here, and why the rule is this narrow.** The
/// extract is a whole wiki page, not a definition record: it carries
/// hyphenation, morphology, pronunciation, etymology and translation sections,
/// and which heading holds the definition is named differently in every
/// language edition. A reader that took "the first line that is not a heading"
/// would publish a hyphenation string, or `Существительное, неодушевлённое,
/// женский род`, as the *meaning* of the headword — a fabricated gloss dressed
/// in real bytes. A reader
/// that knew the definition heading of each edition would be a list of Russian,
/// Chinese and Hindi words in Rust, which
/// `scripts/check-hardcoded-language.rs` exists to forbid.
///
/// So the rule is the one thing the wikitext conventions guarantee without
/// naming a language: **the lines an entry states directly under its first
/// section heading, before any further heading.** An edition that puts the
/// sense there (zh.wiktionary writes the target-language equivalents straight
/// under `== 英語 ==`) is read; an edition that nests the definition inside a
/// deeper section (ru.wiktionary's `==== Значение ====`, en.wiktionary's
/// `=== Noun ===`) yields **nothing**, and nothing is the honest answer rather
/// than the first line that happened to be there. The headword itself is never
/// a gloss, however the entry respells it.
fn wiktionary_extract(value: &Value, surface: &str) -> Vec<RawGloss> {
    let mut out = Vec::new();
    let Some(pages) = value
        .get("query")
        .and_then(|query| query.get("pages"))
        .and_then(Value::as_object)
    else {
        return out;
    };
    for page in pages.values() {
        if page.get("missing").is_some() {
            continue;
        }
        let lemma = string_at(page, "title").unwrap_or_else(|| surface.to_owned());
        let Some(extract) = page.get("extract").and_then(Value::as_str) else {
            continue;
        };
        let mut seen_heading = false;
        for line in extract.lines().map(str::trim) {
            if line.is_empty() {
                continue;
            }
            if is_wikitext_heading(line) {
                if seen_heading {
                    break;
                }
                seen_heading = true;
                continue;
            }
            if !seen_heading {
                continue;
            }
            let gloss = compact(line);
            if gloss.is_empty()
                || is_respelling_of(&gloss, &lemma)
                || is_respelling_of(&gloss, surface)
            {
                continue;
            }
            out.push(RawGloss {
                lemma: lemma.clone(),
                gloss,
                part_of_speech: String::new(),
                synonyms: Vec::new(),
            });
        }
    }
    out
}

/// A plain-text extract keeps wikitext's `=` heading markers.
fn is_wikitext_heading(line: &str) -> bool {
    line.starts_with('=') && line.ends_with('=') && line.len() > 1
}

/// Whether `candidate` is the headword again, however the entry respells it.
///
/// Entries repeat the headword with hyphenation dots, syllable breaks and
/// stress marks; none of those make it a definition. Comparing the letters
/// alone is language-neutral: everything that is not a letter or a digit is
/// dropped, and so is every combining mark.
fn is_respelling_of(candidate: &str, headword: &str) -> bool {
    let letters = |value: &str| -> String {
        value
            .chars()
            .filter(|character| character.is_alphanumeric())
            .filter(|character| !matches!(u32::from(*character), 0x0300..=0x036F))
            .flat_map(char::to_lowercase)
            .collect()
    };
    let candidate = letters(candidate);
    !candidate.is_empty() && candidate == letters(headword)
}

/// Wikidata's `Special:EntityData` snapshot: the entity's description in the
/// language the caller asked about.
fn wikidata_entity(value: &Value, surface: &str) -> Vec<RawGloss> {
    let mut out = Vec::new();
    let Some(entities) = value.get("entities").and_then(Value::as_object) else {
        return out;
    };
    for (id, entity) in entities {
        let Some(descriptions) = entity.get("descriptions").and_then(Value::as_object) else {
            continue;
        };
        for description in descriptions.values() {
            if let Some(gloss) = string_at(description, "value") {
                out.push(RawGloss {
                    lemma: string_at(entity, "id").unwrap_or_else(|| id.clone()),
                    gloss,
                    part_of_speech: String::new(),
                    synonyms: Vec::new(),
                });
            }
        }
    }
    let _ = surface;
    out
}

/// The committed Links Notation projection of either lexical cache: every
/// `definition` node, with the part of speech its parent entry declares.
fn projection_glosses(text: &str, surface: &str) -> Vec<RawGloss> {
    let tree = crate::seed::parser::parse_lino(text);
    let mut out = Vec::new();
    collect_projection(&tree, "", &mut out, surface);
    out
}

fn collect_projection(
    node: &crate::seed::parser::LinoNode,
    part_of_speech: &str,
    out: &mut Vec<RawGloss>,
    surface: &str,
) {
    let declared = node.find_child_value("partOfSpeech");
    let part_of_speech = if declared.is_empty() {
        part_of_speech
    } else {
        declared
    };
    for child in &node.children {
        if child.name == "definition" {
            let gloss = compact(&child.id);
            if !gloss.is_empty() {
                out.push(RawGloss {
                    lemma: surface.to_owned(),
                    gloss,
                    part_of_speech: part_of_speech.to_owned(),
                    synonyms: Vec::new(),
                });
            }
        } else {
            collect_projection(child, part_of_speech, out, surface);
        }
    }
}

fn string_at(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(compact)
}

fn strings_at(value: &Value, key: &str) -> Vec<String> {
    array_at(value, key)
        .iter()
        .filter_map(|item| item.as_str().map(compact))
        .collect()
}

fn array_at<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// One line of text, whatever the source's line breaks were.
fn compact(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The single implementation of plan 00 §4.2.
pub struct RegistrySourceLookup<'a, T: SourceTransport> {
    client: &'a CachedSourceClient<T>,
    preferences: &'a ServicePreferences,
    availability: &'a mut ServiceAccessibilityCache,
    bounds: LookupBounds,
    language: String,
    now: u64,
    consulted: BTreeSet<String>,
    outcomes: Vec<WalkSourceOutcome>,
}

impl<'a, T: SourceTransport> RegistrySourceLookup<'a, T> {
    /// A lookup over the registry, bounded before it starts.
    #[must_use]
    pub fn new(
        client: &'a CachedSourceClient<T>,
        preferences: &'a ServicePreferences,
        availability: &'a mut ServiceAccessibilityCache,
        bounds: LookupBounds,
        language: &str,
        now: u64,
    ) -> Self {
        Self {
            client,
            preferences,
            availability,
            bounds,
            language: language.to_owned(),
            now,
            consulted: BTreeSet::new(),
            outcomes: Vec::new(),
        }
    }

    /// The outcome row per source, so a caller can report why nothing was found.
    #[must_use]
    pub fn outcomes(&self) -> &[WalkSourceOutcome] {
        &self.outcomes
    }

    /// Every source this lookup has consulted so far, in registry id order.
    #[must_use]
    pub fn consulted(&self) -> Vec<String> {
        self.consulted.iter().cloned().collect()
    }

    /// The bounds this lookup was constructed with.
    #[must_use]
    pub const fn bounds(&self) -> LookupBounds {
        self.bounds
    }

    /// Resolve one surface and keep the outcome rows.
    fn resolve(&mut self, surface: &str, language: &str, bounds: &LookupBounds) -> LookupOutcome {
        let walked = lookup_surface(
            surface,
            language,
            self.client,
            self.preferences,
            bounds,
            self.availability,
            self.now,
        );
        self.consulted
            .extend(walked.outcomes.iter().map(|row| row.source_id.clone()));
        self.outcomes.clone_from(&walked.outcomes);
        if walked.items.is_empty() {
            LookupOutcome::NotFound {
                consulted: walked.outcomes,
            }
        } else {
            LookupOutcome::Found(walked.items)
        }
    }
}

impl<T: SourceTransport> SourceLookup for RegistrySourceLookup<'_, T> {
    fn lookup(&mut self, need: &Need, bounds: &LookupBounds) -> LookupOutcome {
        if need.kind != NeedKind::Concept {
            // One implementation, but not one that pretends every kind is a
            // word. A procedure need belongs to the how-to extractor over the
            // same walk; answering it here would be a second guess dressed as
            // retrieval.
            return LookupOutcome::NotFound {
                consulted: Vec::new(),
            };
        }
        let language = if need.language.is_empty() {
            self.language.clone()
        } else {
            need.language.clone()
        };
        let subject = need.subject.clone();
        self.resolve(&subject, &language, bounds)
    }
}

/// Look one surface up, as the trait does, but returning every sense instead of
/// the first. The universal loop and the coding path both call this.
pub fn lookup_surface<T: SourceTransport>(
    surface: &str,
    language: &str,
    client: &CachedSourceClient<T>,
    preferences: &ServicePreferences,
    bounds: &LookupBounds,
    availability: &mut ServiceAccessibilityCache,
    now: u64,
) -> WalkOutcome<ConceptSense> {
    let extractor = SenseExtractor::new(surface, language);
    let mut walked = walk_sources(
        NeedKind::Concept,
        surface,
        &extractor,
        preferences,
        &Walk {
            client,
            bounds,
            now,
        },
        availability,
    );
    // The per-source reader stops at the item bound within one service; the
    // answer as a whole is bounded here, so `max_items` means the same number
    // whether one source answered or four did.
    walked.items.truncate(bounds.max_items);
    walked
}

/// The surfaces in `normalized` that no seeded meaning accounts for — the words
/// the system must ask about.
///
/// Language-neutral by construction: it asks the lexicon which surfaces are
/// already seeded and returns the rest, so no literal word list decides what is
/// unknown. Quoted spans are excluded: a quoted example is the *subject* of the
/// question, not a concept the system is missing — asking a dictionary what
/// `quick brown fox` means would be asking about the user's own illustration.
#[must_use]
pub fn unknown_surfaces(normalized: &str, language: &str) -> Vec<String> {
    let _ = language;
    let mut out: Vec<String> = Vec::new();
    for token in outside_quotes(normalized)
        .split(crate::source_walk::is_word_boundary)
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        let folded = token.to_lowercase();
        if !is_unknown_surface(token) || out.contains(&folded) {
            continue;
        }
        out.push(folded);
    }
    out
}

/// Whether one token is a surface no seeded meaning accounts for.
///
/// The rule lives here rather than in each caller so the coding path, which
/// asks about a normalized sentence, and the deep formalizer, which asks about
/// an exact span of the user's own bytes, agree on what "unknown" means. A
/// token shorter than three characters or made only of digits is a fragment
/// rather than a concept; everything the seed lexicon already declares is
/// known by definition.
#[must_use]
pub fn is_unknown_surface(token: &str) -> bool {
    let folded = token.to_lowercase();
    token.chars().count() >= 3
        && !token.chars().all(char::is_numeric)
        && !seeded_surfaces().contains(&folded)
}

/// Every unresolved surface in `text`, with the exact byte span it occupies.
///
/// Spans are relative to `text`, so a caller that segmented a document adds the
/// segment's own offset and reaches a span that selects exactly the surface in
/// the original bytes. Quoted spans are skipped for the same reason
/// [`unknown_surfaces`] skips them: a quoted example is the subject of the
/// question, not a concept the system is missing.
#[must_use]
pub fn unknown_surface_spans(text: &str) -> Vec<(String, usize, usize)> {
    const PAIRS: [(char, char); 5] = [
        ('"', '"'),
        ('«', '»'),
        ('\u{201c}', '\u{201d}'),
        ('\u{2018}', '\u{2019}'),
        ('\u{300c}', '\u{300d}'),
    ];
    let mut out: Vec<(String, usize, usize)> = Vec::new();
    let mut closing: Option<char> = None;
    let mut start: Option<usize> = None;
    let push = |start: usize, end: usize, out: &mut Vec<(String, usize, usize)>| {
        let token = &text[start..end];
        if is_unknown_surface(token) {
            out.push((token.to_owned(), start, end));
        }
    };
    for (offset, character) in text.char_indices() {
        match closing {
            Some(close) if character == close => {
                closing = None;
                start = None;
            }
            Some(_) => {}
            None => {
                if let Some((_, close)) = PAIRS.iter().find(|(open, _)| *open == character) {
                    if let Some(begin) = start.take() {
                        push(begin, offset, &mut out);
                    }
                    closing = Some(*close);
                } else if crate::source_walk::is_word_boundary(character) {
                    if let Some(begin) = start.take() {
                        push(begin, offset, &mut out);
                    }
                } else if start.is_none() {
                    start = Some(offset);
                }
            }
        }
    }
    if let Some(begin) = start {
        push(begin, text.len(), &mut out);
    }
    out
}

/// The text of `value` with every quoted span removed, in any of the quotation
/// marks the five seeded languages use.
fn outside_quotes(value: &str) -> String {
    const PAIRS: [(char, char); 5] = [
        ('"', '"'),
        ('«', '»'),
        ('\u{201c}', '\u{201d}'),
        ('\u{2018}', '\u{2019}'),
        ('\u{300c}', '\u{300d}'),
    ];
    let mut out = String::with_capacity(value.len());
    let mut closing: Option<char> = None;
    for character in value.chars() {
        match closing {
            Some(close) if character == close => closing = None,
            Some(_) => {}
            None => {
                if let Some((_, close)) = PAIRS.iter().find(|(open, _)| *open == character) {
                    closing = Some(*close);
                } else {
                    out.push(character);
                }
            }
        }
    }
    out
}

/// Whether `value` carries at least one quoted span. [`unknown_surfaces`]
/// skips quoted spans because a quoted example is the subject of the question,
/// never a concept the system is missing; callers that need the same subject
/// test on the raw prompt use this instead of re-scanning the pairs.
#[must_use]
pub fn has_quoted_span(value: &str) -> bool {
    outside_quotes(value) != value
}

/// Every surface the seed lexicon declares, folded once.
fn seeded_surfaces() -> &'static BTreeSet<String> {
    static CACHE: OnceLock<BTreeSet<String>> = OnceLock::new();
    CACHE.get_or_init(|| {
        crate::seed::lexicon()
            .meanings
            .iter()
            .flat_map(crate::seed::Meaning::words)
            .flat_map(|surface| {
                surface
                    .split_whitespace()
                    .map(str::to_lowercase)
                    .collect::<Vec<_>>()
            })
            .collect()
    })
}
