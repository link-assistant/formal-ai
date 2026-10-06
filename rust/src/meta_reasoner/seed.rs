//! The meta reasoner's seed (data/seed/meta-reasoning.lino): cues,
//! grammatical words, affixes, the instruction set, probes, impasse intents
//! and response templates, read through the embedded seed registry.
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
    /// The instruction set.
    pub primitives: Vec<Primitive>,
    /// Higher-order operations.
    pub combinators: Vec<Combinator>,
    /// Filters.
    pub filters: Vec<Filter>,
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

impl MetaSeed {
    /// Parse the seed text.
    ///
    /// Mirrors `metaSeed` in js/worker/formal_ai_worker_meta_reasoner.js.
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
                let pairs = fields(record, "ending")
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
                    .collect();
                upsert(&mut self.affixes, &language, pairs);
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
            }),
            "combinator" => self.combinators.push(Combinator {
                id: record.id.clone(),
                doc: field(record, "doc"),
                code: field(record, "code"),
            }),
            "filter" => self.filters.push(Filter {
                id: record.id.clone(),
                doc: field(record, "doc"),
                test: field(record, "test"),
            }),
            "impasse" => {
                self.impasse_intents.extend(fields(record, "intent"));
                self.impasse_suffixes.extend(fields(record, "suffix"));
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
    /// Mirrors `metaDocIndex` in js/worker/formal_ai_worker_meta_reasoner.js.
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
    /// Mirrors `metaResponse` in js/worker/formal_ai_worker_meta_reasoner.js.
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
    /// Mirrors `metaNote` in js/worker/formal_ai_worker_meta_reasoner.js.
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
    /// Mirrors `metaLemmas` in js/worker/formal_ai_worker_meta_reasoner.js.
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
    /// Mirrors `metaIsGrammatical` in js/worker/formal_ai_worker_meta_reasoner.js.
    #[must_use]
    pub fn is_grammatical(&self, word: &str) -> bool {
        self.grammatical
            .iter()
            .any(|(_, words)| words.contains(word))
            || is_numeral(word)
    }

    /// The cue markers of one role across every language.
    ///
    /// Mirrors `metaCueMarkers` in js/worker/formal_ai_worker_meta_reasoner.js.
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
    /// `metaDocHypotheses` and `metaGround` (js/worker/formal_ai_worker_meta_reasoner.js).
    #[must_use]
    pub fn structural_limit(&self) -> usize {
        (self.operations.len() / 4).max(2)
    }

    /// The document frequency of a form.
    ///
    /// Mirrors `index.frequency.get(lemma) || 0` in js/worker/formal_ai_worker_meta_reasoner.js.
    #[must_use]
    pub fn frequency_of(&self, form: &str) -> usize {
        self.frequency.get(form).copied().unwrap_or(0)
    }

    /// Operations whose documentation shares a form with the word, weighted
    /// by inverse document frequency.
    ///
    /// Mirrors `metaDocHypotheses` in js/worker/formal_ai_worker_meta_reasoner.js.
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
    /// js/worker/formal_ai_worker_meta_reasoner.js.
    #[must_use]
    pub fn primitive(&self, id: &str) -> Option<&Primitive> {
        self.primitives.iter().find(|primitive| primitive.id == id)
    }

    /// True when an operation only changes representation between a text and
    /// the list of its parts (split / join).
    ///
    /// Mirrors `metaIsView` in js/worker/formal_ai_worker_meta_reasoner.js.
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
    /// Mirrors `metaIsImpasseIntent` in js/worker/formal_ai_worker_meta_reasoner.js.
    #[must_use]
    pub fn is_impasse_intent(&self, intent: &str) -> bool {
        self.impasse_intents.iter().any(|item| item == intent)
            || self
                .impasse_suffixes
                .iter()
                .any(|suffix| intent.ends_with(suffix.as_str()))
    }
}

/// Sort hypotheses by score, strongest first, then by operation name.
///
/// Mirrors `.sort((a, b) => b.score - a.score || a.operation.localeCompare(b.operation))`
/// in js/worker/formal_ai_worker_meta_reasoner.js.
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
/// Mirrors the `metaSeedCache` of `metaSeed` in js/worker/formal_ai_worker_meta_reasoner.js.
#[must_use]
pub fn meta_seed() -> &'static MetaSeed {
    static SEED: OnceLock<MetaSeed> = OnceLock::new();
    SEED.get_or_init(|| MetaSeed::parse(seed_text()))
}
