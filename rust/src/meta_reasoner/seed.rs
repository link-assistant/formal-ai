//! The meta reasoner's seed (data/seed/meta-reasoning.lino).
//!
//! Cues, grammatical words, affixes, the instruction set, probes, impasse
//! intents and response templates, read through the embedded seed registry.
#![allow(clippy::cast_precision_loss)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use crate::seed::parser::{LinoNode, parse_lino};

use super::text::{is_numeral, locale_compare, meta_words, utf16_len};

/// The registered path of the seed file.
pub const SEED_PATH: &str = "data/seed/meta-reasoning.lino";

/// One operation of the instruction set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Primitive {
    /// Operation id (`split_words`).
    pub id: String,
    /// Input type.
    pub from: String,
    /// Output type.
    pub to: String,
    /// The operation's reference documentation.
    pub doc: String,
    /// The JavaScript the answer shows and the verifier runs.
    pub code: String,
    /// The rule deriving the parameter `{k}` from one example pair.
    pub infer: String,
    /// The runtime the operation needs (`node`), or empty.
    pub environment: String,
    /// True when the operation acts on the world (`effect true`).
    pub effect: bool,
    /// The types of the values the operation reads from the request
    /// (`{k0}`, `{k1}` in its code), in the order the request names them.
    pub takes: Vec<String>,
}

/// A degree of comparison (`superlative`) of one language.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Degree {
    /// Degree id.
    pub id: String,
    /// Its endings `(ending, replacement)`, as the affix endings are read.
    pub endings: Vec<(String, String)>,
    /// The documentation wording naming the selection, empty when the
    /// language shares the English record's.
    pub gloss: String,
}

/// A higher-order operation (`map_each`, `rewrite_file`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Combinator {
    /// Combinator id.
    pub id: String,
    /// Its documentation.
    pub doc: String,
    /// Its JavaScript, with `{f}` standing for the operation it applies.
    pub code: String,
}

/// A comparison a list is filtered by.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// Filter id.
    pub id: String,
    /// Its documentation.
    pub doc: String,
    /// Its JavaScript test `(measure, threshold) => ...`.
    pub test: String,
    /// The measure's result type: `number`, or `text` for a filter that
    /// compares the element's content with a value.
    pub measure: String,
}

/// One documented operation of the doc index with its word forms.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Operation {
    /// Operation id.
    pub id: String,
    /// `primitive`, `combinator` or `filter`.
    pub kind: &'static str,
    /// Lemmas of every word of the id and documentation.
    pub forms: BTreeSet<String>,
}

/// A hypothesis that a word denotes an operation.
#[derive(Debug, Clone, PartialEq)]
pub struct Hypothesis {
    /// The operation id.
    pub operation: String,
    /// Evidence strength.
    pub score: f64,
    /// Where the hypothesis came from (`documentation`, a gloss source URL).
    pub via: String,
}

/// The parsed seed.
#[derive(Debug, Clone, Default)]
pub struct MetaSeed {
    /// Cue markers by role, then by language in seed order.
    pub cues: BTreeMap<String, Vec<(String, Vec<String>)>>,
    /// Closed-class words by language.
    pub grammatical: Vec<(String, BTreeSet<String>)>,
    /// Inflection endings `(ending, replacement)` by language.
    pub affixes: Vec<(String, Vec<(String, String)>)>,
    /// Degrees of comparison by language.
    pub degrees: Vec<(String, Vec<Degree>)>,
    /// The instruction set.
    pub primitives: Vec<Primitive>,
    /// Higher-order operations.
    pub combinators: Vec<Combinator>,
    /// Filters.
    pub filters: Vec<Filter>,
    /// Selectors: a choice of one element by its measure (`test` compares a
    /// measure with the best one so far).
    pub selectors: Vec<Filter>,
    /// Probe samples by type.
    pub probes: BTreeMap<String, Vec<String>>,
    /// Response templates by id, then by language.
    pub responses: BTreeMap<String, Vec<(String, String)>>,
    /// Trace-detail templates by id (`note` records).
    pub notes: BTreeMap<String, String>,
    /// Intents by which a handler admits an impasse.
    pub impasse_intents: Vec<String>,
    /// Intent suffixes by which a handler admits an impasse.
    pub impasse_suffixes: Vec<String>,
    /// Impasse suffixes that already name the missing skill.
    pub named_gap_suffixes: Vec<String>,
    /// The documentation index: every operation with its forms.
    pub operations: Vec<Operation>,
    /// Document frequency of every form over the operations.
    pub frequency: BTreeMap<String, usize>,
}

fn upsert<T>(entries: &mut Vec<(String, T)>, key: &str, value: T) {
    if let Some(entry) = entries.iter_mut().find(|(existing, _)| existing == key) {
        entry.1 = value;
    } else {
        entries.push((key.to_owned(), value));
    }
}

fn fields(record: &LinoNode, name: &str) -> Vec<String> {
    record
        .children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.clone())
        .collect()
}

fn field(record: &LinoNode, name: &str) -> String {
    record.find_child_value(name).to_owned()
}

/// The `ending` fields of a record as `(ending, replacement)` pairs.
///
/// Mirrors the `ending` parsing of the `affix` and `degree` records in
/// `metaSeed` (`js/worker/formal_ai_worker_meta_reasoner.js`).
fn endings(record: &LinoNode) -> Vec<(String, String)> {
    fields(record, "ending")
        .iter()
        .map(|value| {
            let mut parts = value.split_whitespace();
            let ending = parts.next().unwrap_or_default().to_owned();
            let replacement = match parts.next() {
                None | Some("\"\"") => String::new(),
                Some(other) => other.to_owned(),
            };
            (ending, replacement)
        })
        .collect()
}

impl MetaSeed {
    /// Parse the seed text.
    ///
    /// Mirrors `metaSeed` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut seed = Self::default();
        let root = parse_lino(text);
        for top in &root.children {
            for record in &top.children {
                seed.read_record(record);
            }
        }
        seed.build_doc_index();
        seed
    }

    fn read_record(&mut self, record: &LinoNode) {
        let language = field(record, "language");
        match record.name.as_str() {
            "cue" => {
                let role = field(record, "role");
                let by_language = self.cues.entry(role).or_default();
                upsert(by_language, &language, fields(record, "marker"));
            }
            "grammatical" => {
                upsert(
                    &mut self.grammatical,
                    &language,
                    fields(record, "word").into_iter().collect(),
                );
            }
            "affix" => {
                upsert(&mut self.affixes, &language, endings(record));
            }
            "degree" => {
                let degree = Degree {
                    id: record.id.clone(),
                    endings: endings(record),
                    gloss: field(record, "gloss"),
                };
                if let Some(index) = self.degrees.iter().position(|(name, _)| *name == language) {
                    self.degrees[index].1.push(degree);
                } else {
                    self.degrees.push((language, vec![degree]));
                }
            }
            "primitive" => self.primitives.push(Primitive {
                id: record.id.clone(),
                from: field(record, "from"),
                to: field(record, "to"),
                doc: field(record, "doc"),
                code: field(record, "code"),
                infer: field(record, "infer"),
                environment: field(record, "environment"),
                effect: field(record, "effect") == "true",
                takes: field(record, "takes")
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect(),
            }),
            "combinator" => self.combinators.push(Combinator {
                id: record.id.clone(),
                doc: field(record, "doc"),
                code: field(record, "code"),
            }),
            "filter" => {
                let measure = field(record, "measure");
                self.filters.push(Filter {
                    id: record.id.clone(),
                    doc: field(record, "doc"),
                    test: field(record, "test"),
                    measure: if measure.is_empty() {
                        String::from("number")
                    } else {
                        measure
                    },
                });
            }
            "selector" => self.selectors.push(Filter {
                id: record.id.clone(),
                doc: field(record, "doc"),
                test: field(record, "test"),
                measure: String::from("number"),
            }),
            "impasse" => {
                self.impasse_intents.extend(fields(record, "intent"));
                self.impasse_suffixes.extend(fields(record, "suffix"));
                self.named_gap_suffixes
                    .extend(fields(record, "named_suffix"));
            }
            "probe" => {
                self.probes
                    .entry(field(record, "type"))
                    .or_default()
                    .extend(fields(record, "value"));
            }
            "note" => {
                self.notes.insert(record.id.clone(), field(record, "text"));
            }
            "response" => {
                let variants = self.responses.entry(record.id.clone()).or_default();
                upsert(variants, &language, field(record, "text"));
            }
            _ => {}
        }
    }

    /// Document frequency of every doc token over the instruction set, so a
    /// word shared by most operations carries almost no evidence.
    ///
    /// Mirrors `metaDocIndex` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    fn build_doc_index(&mut self) {
        let mut operations: Vec<(String, &'static str, String)> = Vec::new();
        for primitive in &self.primitives {
            operations.push((primitive.id.clone(), "primitive", primitive.doc.clone()));
        }
        for combinator in &self.combinators {
            operations.push((combinator.id.clone(), "combinator", combinator.doc.clone()));
        }
        for filter in &self.filters {
            operations.push((filter.id.clone(), "filter", filter.doc.clone()));
        }
        for selector in &self.selectors {
            operations.push((selector.id.clone(), "selector", selector.doc.clone()));
        }
        let mut indexed = Vec::new();
        let mut frequency: BTreeMap<String, usize> = BTreeMap::new();
        for (id, kind, doc) in operations {
            let text = [id.replace('_', " ").as_str(), " ", &doc].concat();
            let mut forms = BTreeSet::new();
            for word in meta_words(&text) {
                for lemma in self.lemmas(&word, "en") {
                    forms.insert(lemma);
                }
            }
            for form in &forms {
                *frequency.entry(form.clone()).or_default() += 1;
            }
            indexed.push(Operation { id, kind, forms });
        }
        self.operations = indexed;
        self.frequency = frequency;
    }

    /// One response template with its `{slots}` filled.
    ///
    /// Mirrors `metaResponse` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn response(&self, id: &str, language: &str, slots: &[(&str, String)]) -> String {
        let variants = self.responses.get(id);
        let pick = |wanted: &str| {
            variants
                .and_then(|entries| entries.iter().find(|(name, _)| name == wanted))
                .map(|(_, text)| text.as_str())
                .filter(|text| !text.is_empty())
        };
        let mut text = pick(language)
            .or_else(|| pick("en"))
            .unwrap_or_default()
            .to_owned();
        for (name, value) in slots {
            text = text.replace(&["{", name, "}"].concat(), value);
        }
        text
    }

    /// One trace detail from its seeded template (`note` records): the
    /// derivation's wording is data, like every answer's. An unknown id
    /// yields the id itself.
    ///
    /// Mirrors `metaNote` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn note(&self, id: &str, slots: &[(&str, &str)]) -> String {
        let Some(template) = self.notes.get(id) else {
            return id.to_owned();
        };
        let mut text = template.clone();
        for (name, value) in slots {
            text = text.replace(&["{", name, "}"].concat(), value);
        }
        text
    }

    /// A word and the base forms its seeded inflection endings yield.
    ///
    /// Mirrors `metaLemmas` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn lemmas(&self, word: &str, language: &str) -> Vec<String> {
        let mut out = vec![word.to_owned()];
        let affixes = self
            .affixes
            .iter()
            .find(|(name, _)| name == language)
            .or_else(|| self.affixes.iter().find(|(name, _)| name == "en"));
        let Some((_, pairs)) = affixes else {
            return out;
        };
        for (ending, replacement) in pairs {
            if utf16_len(word) > utf16_len(ending) + 2 && word.ends_with(ending.as_str()) {
                let base = [&word[..word.len() - ending.len()], replacement.as_str()].concat();
                if !out.contains(&base) {
                    out.push(base);
                }
            }
        }
        out
    }

    /// True when the word is a closed-class (grammatical) word in any
    /// language, or a numeral.
    ///
    /// Mirrors `metaIsGrammatical` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn is_grammatical(&self, word: &str) -> bool {
        self.grammatical
            .iter()
            .any(|(_, words)| words.contains(word))
            || is_numeral(word)
    }

    /// The cue markers of one role across every language.
    ///
    /// Mirrors `metaCueMarkers` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn cue_markers(&self, role: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for (_, markers) in self.cues.get(role).map(Vec::as_slice).unwrap_or_default() {
            for marker in markers {
                if !out.contains(marker) {
                    out.push(marker.clone());
                }
            }
        }
        out
    }

    /// The document frequency above which a form is structural vocabulary.
    ///
    /// Mirrors `Math.max(2, Math.floor(index.operations.length / 4))` in
    /// `metaDocHypotheses` and `metaGround` (`js/worker/formal_ai_worker_meta_reasoner.js`).
    #[must_use]
    pub fn structural_limit(&self) -> usize {
        (self.operations.len() / 4).max(2)
    }

    /// The document frequency of a form.
    ///
    /// Mirrors `index.frequency.get(lemma) || 0` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn frequency_of(&self, form: &str) -> usize {
        self.frequency.get(form).copied().unwrap_or(0)
    }

    /// Operations whose documentation shares a form with the word, weighted
    /// by inverse document frequency.
    ///
    /// Mirrors `metaDocHypotheses` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn doc_hypotheses(&self, word: &str, language: &str) -> Vec<Hypothesis> {
        let limit = self.structural_limit();
        let mut scores: Vec<(String, f64)> = Vec::new();
        for lemma in self.lemmas(word, language) {
            let frequency = self.frequency_of(&lemma);
            if frequency == 0 || frequency > limit || self.is_grammatical(&lemma) {
                continue;
            }
            for operation in &self.operations {
                if !operation.forms.contains(&lemma) {
                    continue;
                }
                // An operation's own name is stronger evidence than its prose.
                let named = operation
                    .id
                    .split('_')
                    .any(|part| self.lemmas(part, "en").contains(&lemma));
                let weight = if named { 2.0 } else { 1.0 };
                let score = weight / frequency as f64;
                if let Some(entry) = scores.iter_mut().find(|(id, _)| *id == operation.id) {
                    entry.1 = entry.1.max(score);
                } else {
                    scores.push((operation.id.clone(), score));
                }
            }
        }
        let mut out: Vec<Hypothesis> = scores
            .into_iter()
            .map(|(operation, score)| Hypothesis {
                operation,
                score,
                via: String::new(),
            })
            .collect();
        sort_hypotheses(&mut out);
        out
    }

    /// The primitive with this id.
    ///
    /// Mirrors `metaSeed().primitives.find((item) => item.id === id)` in
    /// `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn primitive(&self, id: &str) -> Option<&Primitive> {
        self.primitives.iter().find(|primitive| primitive.id == id)
    }

    /// True when an operation is a measure: a parameter-free primitive to a
    /// number with no effect on the world.
    ///
    /// Mirrors `metaIsMeasure` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn is_measure(&self, id: &str) -> bool {
        self.primitive(id).is_some_and(|primitive| {
            primitive.to == "number"
                && !primitive.effect
                && primitive.infer.is_empty()
                && primitive.takes.is_empty()
        })
    }

    /// The degrees of comparison of a language, the English ones otherwise.
    ///
    /// Mirrors `seed.degrees[context.language] || seed.degrees.en` in
    /// `metaGroundDegree` (`js/worker/formal_ai_worker_meta_reasoner.js`).
    #[must_use]
    pub fn degrees_of(&self, language: &str) -> &[Degree] {
        self.degrees
            .iter()
            .find(|(name, _)| name == language)
            .or_else(|| self.degrees.iter().find(|(name, _)| name == "en"))
            .map(|(_, list)| list.as_slice())
            .unwrap_or_default()
    }

    /// The gloss of a degree, falling back to the English record of the same
    /// degree.
    ///
    /// Mirrors the `gloss` fallback of `metaGroundDegree` in
    /// `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn degree_gloss<'a>(&'a self, degree: &'a Degree) -> &'a str {
        if !degree.gloss.is_empty() {
            return &degree.gloss;
        }
        self.degrees
            .iter()
            .find(|(name, _)| name == "en")
            .and_then(|(_, list)| list.iter().find(|item| item.id == degree.id))
            .map_or("", |item| item.gloss.as_str())
    }

    /// True when an operation only changes representation between a text and
    /// the list of its parts (split / join).
    ///
    /// Mirrors `metaIsView` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn is_view(&self, id: &str) -> bool {
        self.primitive(id).is_some_and(|primitive| {
            let mut pair = [primitive.from.as_str(), primitive.to.as_str()];
            pair.sort_unstable();
            pair == ["list_text", "text"]
        })
    }

    /// True when a handler's answer admits it could not do the task.
    ///
    /// Mirrors `metaIsImpasseIntent` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn is_impasse_intent(&self, intent: &str) -> bool {
        self.impasse_intents.iter().any(|item| item == intent)
            || self
                .impasse_suffixes
                .iter()
                .any(|suffix| intent.ends_with(suffix.as_str()))
    }

    /// True when a handler's impasse already names the missing skill.
    ///
    /// Only a derived program replaces such an answer; an open reply would
    /// lose the named gap. Mirrors the `named` check in `metaResolveImpasse`,
    /// `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn names_gap(&self, intent: &str) -> bool {
        self.named_gap_suffixes
            .iter()
            .any(|suffix| intent.ends_with(suffix.as_str()))
    }
}

/// Sort hypotheses by score, strongest first, then by operation name.
///
/// Mirrors `.sort((a, b) => b.score - a.score || a.operation.localeCompare(b.operation))`
/// in `js/worker/formal_ai_worker_meta_reasoner.js`.
pub fn sort_hypotheses(hypotheses: &mut [Hypothesis]) {
    hypotheses.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| locale_compare(&a.operation, &b.operation))
    });
}

/// The embedded seed text.
fn seed_text() -> &'static str {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == SEED_PATH)
        .map_or("", |(_, text)| text)
}

/// The parsed seed, read once.
///
/// Mirrors the `metaSeedCache` of `metaSeed` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_seed() -> &'static MetaSeed {
    static SEED: OnceLock<MetaSeed> = OnceLock::new();
    SEED.get_or_init(|| MetaSeed::parse(seed_text()))
}
