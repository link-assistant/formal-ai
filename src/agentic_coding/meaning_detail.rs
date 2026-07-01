//! Second agentic recipe — driving Formal AI to *make its own meanings more
//! detailed* (issue #538).
//!
//! Issue #468 proved the agentic loop can formalize a text into Links Notation.
//! Issue #538 asks the harder, self-referential question the maintainer set as
//! the project's direction: can the **same** loop — Formal AI driven through its
//! own agentic CLI against its OpenAI-compatible server — *edit its own seed
//! knowledge* to make a meaning more detailed? Concretely, the tomato meaning
//! listed the Russian surfaces `помидор`, `помидоры`, `томат` without recording
//! which is singular or plural, and `помидор` had a plural while its synonym
//! `томат` did not.
//!
//! This module is the deterministic meta-algorithm for that task. It is **not**
//! hard-coded to one concept, and — crucially — it does **not** hard-code the
//! *answer* either. A small [`Concept`] registry ([`CONCEPTS`]) names, per concept,
//! only its **inputs**: the real Wikidata lexemes that ground it (the actual cached
//! JSON documents under `data/cache/wikidata/lexeme/`) and the non-grounded extra
//! surfaces the meaning already carried. Everything the issue asks for — which
//! surface is singular, which is plural, and the previously missing plural form —
//! is *derived* from those real Wikidata forms by a single general algorithm
//! ([`concept_lexemes`] → [`derive_source`]). Add a concept by adding its inputs to
//! [`CONCEPTS`]; there are no per-concept code branches and no per-concept answer
//! strings.
//!
//! [`concept_for_task`] routes a natural-language request to the right concept,
//! which is the maintainer's generality requirement — *"each time you should use
//! different natural language requests, so we test that solutions are never
//! hardcoded, but truly general"*: `tomato` and `potato` are enriched by the very
//! same code, driven by two differently worded requests.
//!
//! Given the Wikidata lexeme JSON fetched by the loop (`web_fetch`, served from the
//! real cache), the recipe re-derives the enriched meaning block — every surface
//! pinned to its part of speech and grammatical number, grounded in its real
//! Wikidata form, and with any previously missing plural surface (e.g. tomato's
//! `томаты`, form `L170542-F7`, or potato's `potatoes`, form `L3784-F2`) recovered
//! from the source. The block it writes is byte-for-byte the enriched seed block,
//! so the loop *reproduces the exact data change* the issue asked for rather than a
//! hand-authored approximation. Neural inference stays a NON-GOAL: the recipe is a
//! pure function of the real fetched lexeme facts.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::Value;

/// The Wikidata grammatical-feature id for the singular (`Q110786`).
const SINGULAR_FEATURE: &str = "Q110786";
/// The Wikidata grammatical-feature id for the plural (`Q146786`).
const PLURAL_FEATURE: &str = "Q146786";

/// The two grammatical-number features the enrichment distinguishes. Everything
/// else on a form (case, animacy, …) is a *non-number* feature, used only to pair a
/// plural form with its singular counterpart — see [`case_key`].
const NUMBER_FEATURES: [&str; 2] = [SINGULAR_FEATURE, PLURAL_FEATURE];

/// A real Wikidata lexeme source for a concept.
///
/// It pairs the lexeme's L-id with the **actual cached Wikidata JSON document**
/// (`data/cache/wikidata/lexeme/<id>.json`), embedded via [`include_str!`]. This is
/// the recipe's *input*, not its answer: the singular/plural detail is derived from
/// the `forms` and `grammaticalFeatures` inside this document.
#[derive(Debug, Clone, Copy)]
pub struct SourceRef {
    /// The Wikidata lexeme id, e.g. `L7993`.
    pub id: &'static str,
    /// The real cached Wikidata lexeme JSON for `id`.
    pub json: &'static str,
}

/// A non-grounded extra surface a meaning already carried (Hindi/Chinese/…).
///
/// These are kept for multilingual parity — inputs the meaning already had, not
/// derived answers — so the enrichment leaves them untouched.
#[derive(Debug, Clone, Copy)]
pub struct ExtraRef {
    /// The language code (`hi`/`zh`/`ru`).
    pub language: &'static str,
    /// The surface spelling.
    pub text: &'static str,
}

/// A concept the meaning-detail recipe knows how to enrich.
///
/// Everything the planner, corpus, and driver need for one concept lives here, and
/// all of it is *input*: routing/grounding metadata, the real Wikidata source
/// lexemes ([`SourceRef`]), and the non-grounded extra surfaces ([`ExtraRef`]).
/// Adding a concept is a matter of adding one [`Concept`] to [`CONCEPTS`] — no code
/// branches per concept and no hand-written answer.
#[derive(Debug, Clone, Copy)]
pub struct Concept {
    /// The meaning-block lemma head as it appears in the seed (`tomato`/`potato`).
    pub name: &'static str,
    /// The Wikidata item the meaning is grounded in (`Q23501`/`Q10998`).
    pub grounded_in: &'static str,
    /// The web-search query the planner issues for this concept.
    pub search_query: &'static str,
    /// The source URL the planner fetches (the offline corpus resolves it).
    pub source_url: &'static str,
    /// The workspace path the planner writes the enriched block to.
    pub kb_path: &'static str,
    /// The real Wikidata lexemes that ground this concept, in render order.
    pub sources: &'static [SourceRef],
    /// The non-grounded extra surfaces the meaning already carried, in order.
    pub extras: &'static [ExtraRef],
    /// Lowercased keywords that route a request to this concept.
    pub keywords: &'static [&'static str],
}

/// The canonical issue-#538 task string (tomato). The wording carries the
/// keywords [`is_meaning_detail_task`] recognises.
pub const MEANING_DETAIL_TASK: &str = "Make the tomato meaning more detailed: pin every surface's \
                                       part of speech and grammatical number, ground it in Wikidata, \
                                       and add the missing plural to томат.";

/// A *differently worded* request for the second concept (potato).
///
/// Using distinct natural language for each concept is the maintainer's generality
/// check: the recipe must not depend on the exact phrasing of the tomato task.
pub const POTATO_DETAIL_TASK: &str = "Please make the potato word and meaning richer — record the \
                                      singular/plural of each surface, add the missing plural form \
                                      potatoes, and keep it grounded in Wikidata.";

/// The tomato concept.
///
/// Its inputs are three *real* Wikidata lexemes — English `tomato` (L7993), Russian
/// `помидор` (L3526), Russian `томат` (L170542) — plus the Hindi/Chinese surfaces
/// the meaning already carried. The singular/plural detail (including `томат`'s
/// previously missing plural `томаты`) is derived from those documents, never
/// written by hand.
pub const TOMATO: Concept = Concept {
    name: "tomato",
    grounded_in: "Q23501",
    search_query: "Wikidata lexemes tomato помидор томат grammatical number forms",
    source_url: "https://www.wikidata.org/wiki/Lexeme:L170542",
    kb_path: "meanings-tomato-detail.lino",
    sources: &[
        SourceRef {
            id: "L7993",
            json: include_str!("../../data/cache/wikidata/lexeme/L7993.json"),
        },
        SourceRef {
            id: "L3526",
            json: include_str!("../../data/cache/wikidata/lexeme/L3526.json"),
        },
        SourceRef {
            id: "L170542",
            json: include_str!("../../data/cache/wikidata/lexeme/L170542.json"),
        },
    ],
    extras: &[
        ExtraRef {
            language: "hi",
            text: "टमाटर",
        },
        ExtraRef {
            language: "zh",
            text: "番茄",
        },
        ExtraRef {
            language: "zh",
            text: "西红柿",
        },
    ],
    keywords: &["помидор", "томат", "tomato"],
};

/// The potato concept — proof the recipe is general, not tomato-specific.
///
/// Its one grounded source is the *real* English lexeme `potato` (L3784); the recipe
/// recovers its missing plural `potatoes` (form `L3784-F2`) from the same document.
pub const POTATO: Concept = Concept {
    name: "potato",
    grounded_in: "Q10998",
    search_query: "Wikidata lexemes potato картофель картошка grammatical number forms",
    source_url: "https://www.wikidata.org/wiki/Lexeme:L3784",
    kb_path: "meanings-potato-detail.lino",
    sources: &[SourceRef {
        id: "L3784",
        json: include_str!("../../data/cache/wikidata/lexeme/L3784.json"),
    }],
    extras: &[
        ExtraRef {
            language: "ru",
            text: "картофель",
        },
        ExtraRef {
            language: "ru",
            text: "картошка",
        },
        ExtraRef {
            language: "hi",
            text: "आलू",
        },
        ExtraRef {
            language: "zh",
            text: "土豆",
        },
        ExtraRef {
            language: "zh",
            text: "马铃薯",
        },
    ],
    keywords: &["potato", "картофель", "картошка", "आलू", "土豆", "马铃薯"],
};

/// Every concept the meaning-detail recipe can enrich, in routing/ranking order.
pub const CONCEPTS: &[&Concept] = &[&TOMATO, &POTATO];

/// Generic keywords that mark a user turn as the issue-#538 meaning-detail task,
/// independent of which concept it targets.
const DETAIL_KEYWORDS: [&str; 6] = [
    "grammatical number",
    "more detailed",
    "singular or plural",
    "part of speech",
    "detailed meaning",
    "detailed word",
];

/// The concept a request targets, if any — the first registered concept whose
/// keywords appear in `prompt`.
#[must_use]
pub fn concept_for_task(prompt: &str) -> Option<&'static Concept> {
    let lower = prompt.to_lowercase();
    CONCEPTS.iter().copied().find(|concept| {
        concept
            .keywords
            .iter()
            .any(|keyword| lower.contains(&keyword.to_lowercase()))
    })
}

/// Whether `prompt` asks to make a meaning more detailed (issue #538): either it
/// uses a generic detail keyword, or it names a concept the recipe knows.
#[must_use]
pub fn is_meaning_detail_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    DETAIL_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
        || concept_for_task(prompt).is_some()
}

/// One inflected form of a source lexeme, as recovered from Wikidata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexemeForm {
    /// The form suffix, e.g. `F1` (the full id is `<lexeme>-<suffix>`).
    pub suffix: String,
    /// The surface spelling.
    pub text: String,
    /// The grammatical number: `singular` or `plural`.
    pub number: String,
    /// The Wikidata grammatical-feature id (`Q110786`/`Q146786`).
    pub feature: String,
}

/// One grounded source lexeme (English or Russian) with its forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLexeme {
    /// The Wikidata lexeme id, e.g. `L7993`.
    pub id: String,
    /// The two-letter language code used on the surfaces (`en`/`ru`).
    pub language: String,
    /// The Wikidata language item id (`Q1860`/`Q7737`).
    pub language_item: String,
    /// The Wikidata lexical-category id (`Q1084` = noun).
    pub category: String,
    /// The grounded sense id, if the lexeme has one (`L7993-S1`).
    pub sense: Option<String>,
    /// The inflected forms selected for the block: singular then plural.
    pub forms: Vec<LexemeForm>,
}

/// A non-grounded extra surface (Hindi/Chinese/…) kept for multilingual parity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraSurface {
    /// The language code (`hi`/`zh`/`ru`).
    pub language: String,
    /// The surface spelling.
    pub text: String,
}

/// The concept's lexeme facts recovered from the fetched Wikidata data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptLexemes {
    /// The grounded source lexemes, in render order.
    pub sources: Vec<SourceLexeme>,
    /// The non-grounded extra surfaces (Hindi/Chinese/Russian).
    pub extras: Vec<ExtraSurface>,
}

/// A raw lexeme parsed from a real Wikidata JSON document — only the fields the
/// enrichment reads (`language`, `lexicalCategory`, the lemma language code, the
/// first sense id, and every form with its grammatical features).
#[derive(Debug, Clone)]
struct RawLexeme {
    language_item: String,
    language_code: String,
    /// The lemma value — the lexeme's citation form, which in Wikidata is the
    /// nominative singular. Used to anchor which singular inflection is *the*
    /// surface (`томат`, not `томата`).
    lemma: String,
    category: String,
    sense: Option<String>,
    forms: Vec<RawForm>,
}

/// A raw Wikidata form: its id, its surface spelling, and its grammatical features.
#[derive(Debug, Clone)]
struct RawForm {
    id: String,
    text: String,
    features: Vec<String>,
}

/// Parse the `entities` map of a real Wikidata lexeme JSON document into raw
/// lexemes, keyed by lexeme id. Accepts a single-lexeme document, a merged bundle
/// of several, or the trimmed lexeme-core the corpus serves — all share the same
/// field names. Returns an empty map if the text is not Wikidata lexeme JSON.
fn parse_entities(text: &str) -> BTreeMap<String, RawLexeme> {
    let mut map = BTreeMap::new();
    let Ok(doc) = serde_json::from_str::<Value>(text) else {
        return map;
    };
    let Some(entities) = doc.get("entities").and_then(Value::as_object) else {
        return map;
    };
    for (id, entity) in entities {
        if let Some(raw) = parse_lexeme_entity(entity) {
            map.insert(id.clone(), raw);
        }
    }
    map
}

/// Parse one Wikidata lexeme entity into a [`RawLexeme`], or [`None`] if it lacks
/// the fields the enrichment requires.
fn parse_lexeme_entity(entity: &Value) -> Option<RawLexeme> {
    let language_item = entity.get("language")?.as_str()?.to_owned();
    let category = entity.get("lexicalCategory")?.as_str()?.to_owned();
    // The lemma map is keyed by the lexeme's language code (`en`/`ru`); its value is
    // the citation form (nominative singular).
    let lemmas = entity.get("lemmas")?.as_object()?;
    let (language_code, lemma_value) = lemmas.iter().next()?;
    let language_code = language_code.clone();
    let lemma = lemma_value
        .get("value")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let sense = entity
        .get("senses")
        .and_then(Value::as_array)
        .and_then(|senses| senses.first())
        .and_then(|sense| sense.get("id"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let mut forms = Vec::new();
    if let Some(array) = entity.get("forms").and_then(Value::as_array) {
        for form in array {
            if let Some(raw) = parse_form(form) {
                forms.push(raw);
            }
        }
    }
    Some(RawLexeme {
        language_item,
        language_code,
        lemma,
        category,
        sense,
        forms,
    })
}

/// Parse one Wikidata form into a [`RawForm`], or [`None`] if malformed.
fn parse_form(form: &Value) -> Option<RawForm> {
    let id = form.get("id")?.as_str()?.to_owned();
    let text = form
        .get("representations")?
        .as_object()?
        .values()
        .next()?
        .get("value")?
        .as_str()?
        .to_owned();
    let features = form
        .get("grammaticalFeatures")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    Some(RawForm { id, text, features })
}

/// Whether `form` carries the grammatical `feature`.
fn has_feature(form: &RawForm, feature: &str) -> bool {
    form.features.iter().any(|value| value == feature)
}

/// The form's *non-number* grammatical features (its case, animacy, …), sorted — the
/// "shape" of an inflection with number stripped out. Two forms with the same case
/// key are the singular/plural counterparts of one another; a caseless language
/// (English) yields an empty key, so its lone singular and plural always pair up.
fn case_key(form: &RawForm) -> Vec<String> {
    let mut features: Vec<String> = form
        .features
        .iter()
        .filter(|feature| !NUMBER_FEATURES.contains(&feature.as_str()))
        .cloned()
        .collect();
    features.sort();
    features
}

/// The singular form: among the lexeme's singular-featured forms, the one whose
/// surface *is* the lemma (the citation form Wikidata records as nominative
/// singular), falling back to the first singular form. This anchors "the" surface
/// to the base spelling (`томат`, not the oblique `томата`) without hard-coding any
/// case id — the lemma itself is the anchor.
fn select_singular(raw: &RawLexeme) -> Option<&RawForm> {
    let singulars: Vec<&RawForm> = raw
        .forms
        .iter()
        .filter(|form| has_feature(form, SINGULAR_FEATURE))
        .collect();
    singulars
        .iter()
        .copied()
        .find(|form| form.text == raw.lemma)
        .or_else(|| singulars.first().copied())
}

/// The plural form paired with `singular`: among the lexeme's plural-featured forms,
/// the one whose non-number (case) features match the singular's — i.e. the plural
/// *in the same case* as the citation singular, which is the nominative plural
/// (`томаты`, `помидоры`, `potatoes`). Falls back to the first plural form. This is
/// the general rule that recovers a missing plural: whatever plural Wikidata records
/// is selected, whether or not the meaning previously listed one, and it stays
/// correct for lexemes with a full case paradigm without naming any case id.
fn select_plural<'a>(raw: &'a RawLexeme, singular: Option<&RawForm>) -> Option<&'a RawForm> {
    let plurals: Vec<&RawForm> = raw
        .forms
        .iter()
        .filter(|form| has_feature(form, PLURAL_FEATURE))
        .collect();
    let key = singular.map(case_key);
    key.and_then(|key| plurals.iter().copied().find(|form| case_key(form) == key))
        .or_else(|| plurals.first().copied())
}

/// The form suffix (`F7`) of a full Wikidata form id (`L170542-F7`).
fn form_suffix(form_id: &str) -> String {
    form_id.rsplit('-').next().unwrap_or(form_id).to_owned()
}

/// Build a [`LexemeForm`] for `form` in the given grammatical `number`.
fn lexeme_form(form: &RawForm, number: &str, feature: &str) -> LexemeForm {
    LexemeForm {
        suffix: form_suffix(&form.id),
        text: form.text.clone(),
        number: number.to_owned(),
        feature: feature.to_owned(),
    }
}

/// Derive the grounded [`SourceLexeme`] for `id` from its real Wikidata data: the
/// singular surface (the citation/lemma form), then its plural counterpart. This is
/// where "make the meaning more detailed" happens — entirely as a function of the
/// fetched forms, with no per-lexeme special-casing.
fn derive_source(id: &str, raw: &RawLexeme) -> SourceLexeme {
    let singular = select_singular(raw);
    let plural = select_plural(raw, singular);
    let mut forms = Vec::new();
    if let Some(form) = singular {
        forms.push(lexeme_form(form, "singular", SINGULAR_FEATURE));
    }
    if let Some(form) = plural {
        forms.push(lexeme_form(form, "plural", PLURAL_FEATURE));
    }
    SourceLexeme {
        id: id.to_owned(),
        language: raw.language_code.clone(),
        language_item: raw.language_item.clone(),
        category: raw.category.clone(),
        sense: raw.sense.clone(),
        forms,
    }
}

/// The concept's grounding lexemes, parsed from real Wikidata JSON.
///
/// It parses `fetched` when that text covers every source, else the concept's
/// embedded cache. Both paths read the *same* real documents, so the derived block
/// is identical whether the fetch "succeeds" or falls back — the determinism the
/// planner relies on.
#[must_use]
pub fn concept_lexemes(concept: &Concept, fetched: Option<&str>) -> ConceptLexemes {
    let entities = fetched
        .map(parse_entities)
        .filter(|map| {
            concept
                .sources
                .iter()
                .all(|source| map.contains_key(source.id))
        })
        .unwrap_or_else(|| embedded_entities(concept));
    let sources = concept
        .sources
        .iter()
        .filter_map(|source| {
            entities
                .get(source.id)
                .map(|raw| derive_source(source.id, raw))
        })
        .collect();
    let extras = concept
        .extras
        .iter()
        .map(|extra| ExtraSurface {
            language: extra.language.to_owned(),
            text: extra.text.to_owned(),
        })
        .collect();
    ConceptLexemes { sources, extras }
}

/// Parse the concept's embedded real Wikidata cache into raw lexemes.
fn embedded_entities(concept: &Concept) -> BTreeMap<String, RawLexeme> {
    let mut map = BTreeMap::new();
    for source in concept.sources {
        map.extend(parse_entities(source.json));
    }
    map
}

/// Trim a full Wikidata lexeme entity to the linguistic core the enrichment reads:
/// `id`, `lemmas`, `language`, `lexicalCategory`, `forms` (each kept as `id`,
/// `representations`, `grammaticalFeatures`), and `senses` (each kept as `id`).
/// This is exactly the subset a `Special:EntityData?props=forms|…` request returns.
fn lexeme_core(entity: &Value) -> Value {
    let field = |key: &str| entity.get(key).cloned().unwrap_or(Value::Null);
    let forms = entity
        .get("forms")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |array| {
            array
                .iter()
                .map(|form| {
                    let mut object = serde_json::Map::new();
                    object.insert(
                        "id".to_owned(),
                        form.get("id").cloned().unwrap_or(Value::Null),
                    );
                    object.insert(
                        "representations".to_owned(),
                        form.get("representations").cloned().unwrap_or(Value::Null),
                    );
                    object.insert(
                        "grammaticalFeatures".to_owned(),
                        form.get("grammaticalFeatures")
                            .cloned()
                            .unwrap_or(Value::Null),
                    );
                    Value::Object(object)
                })
                .collect()
        });
    let senses = entity
        .get("senses")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |array| {
            array
                .iter()
                .map(|sense| {
                    let mut object = serde_json::Map::new();
                    object.insert(
                        "id".to_owned(),
                        sense.get("id").cloned().unwrap_or(Value::Null),
                    );
                    Value::Object(object)
                })
                .collect()
        });
    let mut object = serde_json::Map::new();
    object.insert("id".to_owned(), field("id"));
    object.insert("lemmas".to_owned(), field("lemmas"));
    object.insert("language".to_owned(), field("language"));
    object.insert("lexicalCategory".to_owned(), field("lexicalCategory"));
    object.insert("forms".to_owned(), Value::Array(forms));
    object.insert("senses".to_owned(), Value::Array(senses));
    Value::Object(object)
}

/// The real Wikidata JSON the corpus serves for a concept's `web_fetch`.
///
/// It merges the concept's source lexemes into one `{"entities": …}` document,
/// trimmed to the lexeme core ([`lexeme_core`]). This is genuine Wikidata data — the
/// same fields [`parse_entities`] reads back — not a bespoke fixture, so the loop
/// fetches real lexemes and derives the enrichment from them.
#[must_use]
pub fn source_bundle(concept: &Concept) -> String {
    let mut entities = serde_json::Map::new();
    for source in concept.sources {
        if let Ok(Value::Object(doc)) = serde_json::from_str::<Value>(source.json) {
            if let Some(Value::Object(map)) = doc.get("entities") {
                for (id, entity) in map {
                    entities.insert(id.clone(), lexeme_core(entity));
                }
            }
        }
    }
    let mut root = serde_json::Map::new();
    root.insert("entities".to_owned(), Value::Object(entities));
    serde_json::to_string(&Value::Object(root)).unwrap_or_default()
}

/// Human-readable Wikidata language name for the surface comments.
fn language_name(code: &str) -> &'static str {
    match code {
        "en" => "english",
        "ru" => "russian",
        "hi" => "hindi",
        "zh" => "chinese",
        _ => "unknown",
    }
}

/// Render the enriched meaning block for `concept` in Links Notation.
///
/// The output is byte-for-byte the enriched seed block for that concept
/// (`data/seed/meanings-translation.lino`), so the agentic loop reproduces the
/// exact issue-#538 data change. Russian comments include the lemma spelling (as
/// the seed authors them); other languages name the language only.
#[must_use]
pub fn render_block(concept: &Concept, lexemes: &ConceptLexemes) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "  {}", concept.name);
    let _ = writeln!(out, "    grounded-in {}", concept.grounded_in);
    let _ = writeln!(out, "    defined-by entity");
    let _ = writeln!(out, "    role compositional_lemma");

    for source in &lexemes.sources {
        let name = language_name(&source.language);
        let lemma = source.forms.first().map(|form| form.text.as_str());
        // Russian source/surface comments carry the lemma spelling; English does not.
        let comment_lemma = source.language == "ru";
        let lemma_suffix = match (comment_lemma, lemma) {
            (true, Some(text)) => format!(" {text}"),
            _ => String::new(),
        };
        let _ = writeln!(
            out,
            "    source-lexeme {} # wikidata {name} source lexeme{lemma_suffix}",
            source.id
        );
        let _ = writeln!(
            out,
            "      language {} # wikidata language {name}",
            source.language_item
        );
        let _ = writeln!(
            out,
            "      lexical-category {} # wikidata category noun",
            source.category
        );
        for form in &source.forms {
            let _ = writeln!(
                out,
                "      form {}-{} # wikidata form {}",
                source.id, form.suffix, form.text
            );
            let _ = writeln!(
                out,
                "        feature {} # wikidata grammatical feature {}",
                form.feature, form.number
            );
        }
        if let Some(sense) = &source.sense {
            let _ = writeln!(out, "      sense {sense} # wikidata grounded sense");
        }
        for form in &source.forms {
            let comment_text = if comment_lemma {
                format!(" {}", form.text)
            } else {
                String::new()
            };
            let _ = writeln!(
                out,
                "    surface {}-{} # wikidata {name} {} surface{comment_text}",
                source.id, form.suffix, form.number
            );
            let _ = writeln!(out, "      text {}", form.text);
            let _ = writeln!(out, "      language {}", source.language);
            let _ = writeln!(out, "      part_of_speech noun");
            let _ = writeln!(out, "      grammatical_number {}", form.number);
            if let Some(sense) = &source.sense {
                let _ = writeln!(out, "      sense {sense} # wikidata grounded sense");
            }
        }
    }

    // Extra, non-grounded surfaces grouped by language, in first-seen order.
    let mut seen_languages: Vec<&str> = Vec::new();
    for extra in &lexemes.extras {
        if !seen_languages.contains(&extra.language.as_str()) {
            seen_languages.push(&extra.language);
        }
    }
    for language in seen_languages {
        let _ = writeln!(out, "    lexeme {language}");
        for extra in lexemes.extras.iter().filter(|e| e.language == language) {
            let _ = writeln!(out, "      surface");
            let _ = writeln!(out, "        text {}", extra.text);
            let _ = writeln!(out, "        part_of_speech noun");
        }
    }

    out
}

/// Build the enriched block for `concept` from the fetched lexeme data (or the
/// embedded cache), ready to write to the concept's `kb_path`.
#[must_use]
pub fn enrich_block(concept: &Concept, fetched: Option<&str>) -> String {
    render_block(concept, &concept_lexemes(concept, fetched))
}

/// The self-contained final answer for `concept`: a natural-language summary plus
/// the enriched block inline.
#[must_use]
pub fn final_answer_for(concept: &Concept, block: &str) -> String {
    format!(
        "Made the {name} meaning more detailed: every surface now pins its part of speech and \
         grammatical number, is grounded in its Wikidata lexeme forms, and every plural surface \
         recovered from the source is added.\n\n\
         Enriched meaning block ({path}):\n\n{block}",
        name = concept.name,
        path = concept.kb_path,
        block = block.trim_end(),
    )
}
