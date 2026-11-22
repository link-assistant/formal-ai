//! Enrich registered noun meanings from the publisher's Wikidata lexeme forms.
//!
//! The registry describes grounding and acquisition inputs. The recipe accepts
//! a complete publisher batch for the requested lexeme identifiers, matches
//! each sense to the declared concept, and derives the grammatical surfaces.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use serde_json::Value;

/// The Wikidata grammatical-feature id for the singular (`Q110786`).
const SINGULAR_FEATURE: &str = "Q110786";
/// The Wikidata grammatical-feature id for the plural (`Q146786`).
const PLURAL_FEATURE: &str = "Q146786";

/// The two grammatical-number features the enrichment distinguishes. Everything
/// else on a form (case, animacy, …) is a *non-number* feature, used only to pair a
/// plural form with its singular counterpart — see [`case_key`].
const NUMBER_FEATURES: [&str; 2] = [SINGULAR_FEATURE, PLURAL_FEATURE];

const SOURCE_ENDPOINT: &str = "https://www.wikidata.org/w/api.php?action=wbgetentities&ids=";
const MEANINGS: &str = include_str!("../../embedded/data/seed/meanings-translation.lino");

/// A lexical entry with a grounded Wikidata concept and declared source lexemes.
#[derive(Debug)]
pub struct Concept {
    /// Stable name of the meaning in the lexicon.
    pub name: String,
    /// Wikidata item that identifies the meaning to match.
    pub grounded_in: String,
    /// Search query for the source lexemes.
    pub search_query: String,
    /// Publisher API request for the complete source list.
    pub source_url: String,
    /// Workspace path of the generated meaning block.
    pub kb_path: String,
    /// Wikidata lexemes declared by the meaning, in render order.
    pub sources: Vec<String>,
    required_senses: Vec<String>,
    extras: Vec<ExtraSurface>,
    keywords: Vec<String>,
}

/// Meaning records with enough metadata to acquire their source forms.
#[must_use]
pub fn concepts() -> &'static [Concept] {
    static CONCEPTS: OnceLock<Vec<Concept>> = OnceLock::new();
    CONCEPTS.get_or_init(|| {
        crate::seed::parse_lino(MEANINGS)
            .children
            .iter()
            .filter(|container| container.name == "meanings")
            .flat_map(|container| container.children.iter())
            .filter_map(parse_concept)
            .collect()
    })
}

fn parse_concept(node: &crate::seed::parser::LinoNode) -> Option<Concept> {
    let name = &node.name;
    let grounded_in = node.find_child_value("grounded-in");
    if !entity_id(grounded_in, 'Q')
        || name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || b"-_".contains(&byte))
    {
        return None;
    }
    let entries: Vec<_> = node
        .children
        .iter()
        .filter(|child| child.name == "source-lexeme" && entity_id(&child.id, 'L'))
        .collect();
    let sources: Vec<_> = entries.iter().map(|entry| entry.id.clone()).collect();
    let required_senses = entries
        .iter()
        .filter(|entry| !entry.find_child_value("sense").is_empty())
        .map(|entry| entry.id.clone())
        .collect();
    if sources.is_empty() || sources.len() > 20 {
        return None;
    }
    let mut extras = Vec::new();
    for lexeme in node
        .children
        .iter()
        .filter(|child| child.name == "lexeme" && is_language_code(&child.id))
    {
        extras.extend(
            lexeme
                .children
                .iter()
                .filter(|surface| surface.name == "surface")
                .filter_map(|surface| safe_surface(surface.find_child_value("text")))
                .map(|text| ExtraSurface {
                    language: lexeme.id.clone(),
                    text: text.to_owned(),
                }),
        );
    }
    let mut keywords = vec![name.clone()];
    keywords.extend(
        node.children
            .iter()
            .filter(|child| child.name == "surface")
            .filter_map(|surface| safe_surface(surface.find_child_value("text")))
            .map(str::to_lowercase),
    );
    keywords.extend(extras.iter().map(|extra| extra.text.to_lowercase()));
    let lexeme_parameters = sources.join("%7C");
    Some(Concept {
        name: name.clone(),
        grounded_in: grounded_in.to_owned(),
        search_query: crate::seed::agentic_message(
            "meaning-detail-search-query",
            &[("name", name), ("lexemes", &sources.join(" "))],
        ),
        source_url: format!("{SOURCE_ENDPOINT}{lexeme_parameters}&format=json&origin=*"),
        kb_path: format!("meanings-{name}-detail.lino"),
        sources,
        required_senses,
        extras,
        keywords,
    })
}

/// Resolve a request that identifies one registered meaning and asks for its forms.
#[must_use]
pub fn concept_for_task(prompt: &str) -> Option<&'static Concept> {
    if !is_meaning_detail_task(prompt) {
        return None;
    }
    let lower = crate::engine::normalize_prompt(prompt);
    let mut candidates = concepts().iter().filter(|concept| {
        concept.keywords.iter().any(|keyword| {
            crate::seed::surface_present(&lower, &crate::engine::normalize_prompt(keyword))
        })
    });
    let matched = candidates.next()?;
    candidates.next().is_none().then_some(matched)
}

/// Whether the request calls for grammatical detail about a meaning.
#[must_use]
pub fn is_meaning_detail_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    crate::seed::agentic_message("meaning_detail_keywords", &[])
        .split('|')
        .filter(|keyword| !keyword.is_empty())
        .any(|keyword| lower.contains(keyword))
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
/// sense linked to the concept, and every form with its grammatical features).
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
/// lexemes, keyed by lexeme id. Returns an empty map for other response formats.
fn parse_entities(text: &str, grounded_in: &str) -> BTreeMap<String, RawLexeme> {
    let mut map = BTreeMap::new();
    let Ok(doc) = serde_json::from_str::<Value>(text) else {
        return map;
    };
    let Some(entities) = doc.get("entities").and_then(Value::as_object) else {
        return map;
    };
    for (id, entity) in entities {
        if let Some(raw) = parse_lexeme_entity(id, entity, grounded_in) {
            map.insert(id.clone(), raw);
        }
    }
    map
}

/// Parse one Wikidata lexeme entity into a [`RawLexeme`], or [`None`] if it lacks
/// the fields the enrichment requires.
fn parse_lexeme_entity(id: &str, entity: &Value, grounded_in: &str) -> Option<RawLexeme> {
    if entity.get("id")?.as_str()? != id || entity.get("type")?.as_str()? != "lexeme" {
        return None;
    }
    let language_item = entity.get("language")?.as_str()?.to_owned();
    let category = entity.get("lexicalCategory")?.as_str()?.to_owned();
    if !entity_id(&language_item, 'Q') || category != "Q1084" {
        return None;
    }
    // The lemma map is keyed by the lexeme's language code (`en`/`ru`); its value is
    // the citation form (nominative singular).
    let lemmas = entity.get("lemmas")?.as_object()?;
    let (language_code, lemma_value) = lemmas.iter().next()?;
    if !is_language_code(language_code) {
        return None;
    }
    let language_code = language_code.clone();
    let lemma = safe_surface(lemma_value.get("value")?.as_str()?)?.to_owned();
    let sense = entity
        .get("senses")
        .and_then(Value::as_array)
        .and_then(|senses| {
            senses.iter().find_map(|sense| {
                let identifier = sense.get("id")?.as_str()?;
                let statements = sense.pointer("/claims/P5137")?.as_array()?;
                (subidentifier(id, identifier, 'S')
                    && statements.iter().any(|claim| {
                        claim.get("rank").and_then(Value::as_str) != Some("deprecated")
                            && claim
                                .pointer("/mainsnak/datavalue/value/id")
                                .and_then(Value::as_str)
                                == Some(grounded_in)
                    }))
                .then(|| identifier.to_owned())
            })
        });
    let mut forms = Vec::new();
    if let Some(array) = entity.get("forms").and_then(Value::as_array) {
        for form in array {
            if let Some(raw) = parse_form(form, id, &language_code) {
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

fn safe_surface(text: &str) -> Option<&str> {
    let text = text.trim();
    (!text.is_empty()
        && text.len() <= 300
        && text
            .chars()
            .all(|character| !character.is_control() && !matches!(character, '#' | '"')))
    .then_some(text)
}

fn entity_id(id: &str, prefix: char) -> bool {
    id.strip_prefix(prefix).is_some_and(|digits| {
        !digits.is_empty() && digits.len() <= 18 && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn is_language_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 20
        && code
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn subidentifier(lexeme: &str, qualified: &str, kind: char) -> bool {
    qualified
        .strip_prefix(lexeme)
        .and_then(|suffix| suffix.strip_prefix('-')?.strip_prefix(kind))
        .is_some_and(|number| {
            !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
        })
}

/// Parse one Wikidata form into a [`RawForm`], or [`None`] if malformed.
fn parse_form(form: &Value, lexeme: &str, language: &str) -> Option<RawForm> {
    let id = form.get("id")?.as_str()?.to_owned();
    if !subidentifier(lexeme, &id, 'F') {
        return None;
    }
    let text = form
        .get("representations")?
        .get(language)?
        .get("value")?
        .as_str()
        .and_then(safe_surface)?
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

/// Derive grammatical forms from a complete response for the requested lexemes.
#[must_use]
pub fn concept_lexemes(concept: &Concept, fetched: &str) -> Option<ConceptLexemes> {
    if fetched.len() > 1_048_576 {
        return None;
    }
    let entities = parse_entities(fetched, &concept.grounded_in);
    let sources = concept
        .sources
        .iter()
        .map(|identifier| {
            let derived = derive_source(identifier, entities.get(identifier)?);
            let sense_verified =
                derived.sense.is_some() || !concept.required_senses.contains(identifier);
            (derived.forms.len() == NUMBER_FEATURES.len() && sense_verified).then_some(derived)
        })
        .collect::<Option<_>>()?;
    Some(ConceptLexemes {
        sources,
        extras: concept.extras.clone(),
    })
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
/// Source identifiers and grammatical features come from the publisher capture.
/// Existing surfaces without a publisher identifier remain intact.
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

/// Build the enriched block for `concept` from explicitly retrieved lexeme data.
#[must_use]
pub fn enrich_block(concept: &Concept, fetched: &str) -> Option<String> {
    Some(render_block(concept, &concept_lexemes(concept, fetched)?))
}

/// The self-contained final answer for `concept`: a natural-language summary plus
/// the enriched block inline.
#[must_use]
pub fn final_answer_for(concept: &Concept, block: &str) -> String {
    crate::seed::agentic_message(
        "meaning_detail_final_answer",
        &[
            ("name", &concept.name),
            ("path", &concept.kb_path),
            ("block", block.trim_end()),
        ],
    )
}
