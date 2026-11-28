//! OpenAI embeddings whose coordinates are links in the seeded meaning lexicon.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{LazyLock, OnceLock};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use regex::Regex;
use serde_json::{Value, json};
use unicode_normalization::UnicodeNormalization;

use super::{ApiHttpResponse, error_response, json_response};
use crate::engine::estimate_tokens;
use crate::seed::{lexicon, parser::parse_lino, try_resolve_model_id};

pub(super) const MODEL: &str = "formal-ai-embeddings";
pub(super) const MINIMUM_DIMENSIONS: u16 = 1;
pub(super) const DIMENSIONS: u16 = 4096;
const MAXIMUM_INPUTS: usize = 128;
const MAXIMUM_INPUT_TOKENS: u32 = 8192;

static WORD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{L}\p{M}\p{N}]+").expect("valid embedding token pattern"));

fn normalize(text: &str) -> String {
    text.chars().flat_map(char::to_lowercase).nfc().collect()
}

fn words(text: &str) -> impl Iterator<Item = &str> {
    WORD_PATTERN.find_iter(text).map(|word| word.as_str())
}

struct ConceptIndex {
    surfaces: BTreeMap<String, Vec<u16>>,
    maximum_words: usize,
}

impl ConceptIndex {
    fn build() -> Option<Self> {
        let mut surfaces = BTreeMap::<_, BTreeSet<_>>::new();
        let mut concepts = BTreeSet::new();
        let mut maximum_words = 0;
        for meaning in &lexicon().meanings {
            let grounded = meaning
                .wikidata
                .strip_prefix(['Q', 'P'])
                .is_some_and(|digits| {
                    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
                });
            let concept = if grounded {
                format!("wikidata:{}", meaning.wikidata)
            } else {
                format!("meaning:{}", meaning.slug)
            };
            for form in meaning.word_forms().filter(|form| !form.text.contains('…')) {
                let normalized = normalize(&form.text);
                let tokens: Vec<_> = words(&normalized).collect();
                if tokens.is_empty() {
                    continue;
                }
                surfaces
                    .entry(tokens.join(" "))
                    .or_default()
                    .insert(concept.clone());
                concepts.insert(concept.clone());
                maximum_words = maximum_words.max(tokens.len());
            }
        }
        if concepts.len() > usize::from(DIMENSIONS) {
            return None;
        }
        // Byte order is shared with the JavaScript endpoint.
        let coordinates: BTreeMap<_, _> = concepts.into_iter().zip(0..DIMENSIONS).collect();
        let surfaces = surfaces
            .into_iter()
            .map(|(surface, links)| {
                let positions = links.iter().map(|link| coordinates[link]).collect();
                (surface, positions)
            })
            .collect();
        Some(Self {
            surfaces,
            maximum_words,
        })
    }

    fn matches(&self, text: &str) -> BTreeSet<u16> {
        let normalized = normalize(text);
        let tokens: Vec<_> = words(&normalized).collect();
        let mut coordinates = BTreeSet::new();
        let mut start = 0;
        while start < tokens.len() {
            let mut surface = String::new();
            let mut matched = None;
            let mut next = start + 1;
            for (end, word) in tokens
                .iter()
                .enumerate()
                .skip(start)
                .take(self.maximum_words)
            {
                if end > start {
                    surface.push(' ');
                }
                surface.push_str(word);
                if let Some(positions) = self.surfaces.get(&surface) {
                    matched = Some(positions);
                    next = end + 1;
                }
            }
            if let Some(positions) = matched {
                coordinates.extend(positions);
            }
            start = next;
        }
        coordinates
    }
}

static CONCEPT_INDEX: OnceLock<Option<ConceptIndex>> = OnceLock::new();

static MESSAGES: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let root = parse_lino(include_str!(
        "../../embedded/data/meta/server-messages.lino"
    ));
    root.children
        .into_iter()
        .filter(|node| node.name == "server_messages")
        .flat_map(|node| node.children)
        .map(|node| {
            let text = node.find_child_value("text").to_owned();
            (node.id, text)
        })
        .collect()
});

fn message(key: &str) -> &str {
    MESSAGES.get(key).map_or(key, String::as_str)
}

fn invalid(field: &str) -> ApiHttpResponse {
    error_response(
        400,
        &message("invalid-embeddings-request").replace("{field}", field),
    )
}

fn encode_base64(vector: impl IntoIterator<Item = f32>) -> Value {
    let bytes: Vec<_> = vector.into_iter().flat_map(f32::to_le_bytes).collect();
    json!(STANDARD.encode(bytes))
}

fn projection_sign(source: u16, target: u16) -> i32 {
    // Give each source/target pair its own word before mixing with the fixed seed.
    let mut word = u32::from(source) * u32::from(DIMENSIONS) + u32::from(target);
    word = word.wrapping_add(0x9e37_79b9);
    word = (word ^ (word >> 16)).wrapping_mul(0x7feb_352d);
    word = (word ^ (word >> 15)).wrapping_mul(0x846c_a68b);
    word ^= word >> 16;
    if word >> 31 == 0 { 1 } else { -1 }
}

#[allow(clippy::cast_possible_truncation)]
fn project(coordinates: &BTreeSet<u16>, dimensions: u16) -> Vec<f32> {
    let mut weights: Vec<i32> = (0..dimensions)
        .map(|target| {
            coordinates
                .iter()
                .map(|&source| projection_sign(source, target))
                .sum()
        })
        .collect();
    // Very small projections can cancel. Use the first active concept's row in that case.
    if weights.iter().all(|&weight| weight == 0) {
        let source = *coordinates.first().expect("at least one matched concept");
        for (target, weight) in (0..dimensions).zip(&mut weights) {
            *weight = projection_sign(source, target);
        }
    }
    let norm = weights
        .iter()
        .map(|&weight| f64::from(weight * weight))
        .sum::<f64>()
        .sqrt();
    weights
        .into_iter()
        .map(|weight| (f64::from(weight) / norm) as f32)
        .collect()
}

fn embedding(coordinates: &BTreeSet<u16>, dimensions: u16, base64: bool) -> Value {
    if dimensions == DIMENSIONS {
        let mut weights = vec![0u8; usize::from(DIMENSIONS)];
        for &coordinate in coordinates {
            weights[usize::from(coordinate)] = 1;
        }
        if base64 {
            encode_base64(weights.into_iter().map(f32::from))
        } else {
            json!(weights)
        }
    } else {
        let weights = project(coordinates, dimensions);
        if base64 {
            encode_base64(weights)
        } else {
            json!(weights)
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn request_dimensions(request: &serde_json::Map<String, Value>) -> Option<u16> {
    match request.get("dimensions") {
        None => Some(DIMENSIONS),
        Some(value) => value
            .as_f64()
            .filter(|value| {
                value.fract() == 0.0
                    && (f64::from(MINIMUM_DIMENSIONS)..=f64::from(DIMENSIONS)).contains(value)
            })
            .map(|value| value as u16),
    }
}

pub(super) fn handle(body: &str) -> ApiHttpResponse {
    let Ok(Value::Object(request)) = serde_json::from_str(body) else {
        return error_response(400, message("invalid-embeddings-json"));
    };
    let Some(model) = request
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
    else {
        return invalid("model");
    };
    if !model.eq_ignore_ascii_case(MODEL) && try_resolve_model_id(Some(model)).is_none() {
        let error = message("unsupported-embeddings-model")
            .replace("{canonical}", MODEL)
            .replace("{model}", model);
        return error_response(400, &error);
    }
    let Some(dimensions) = request_dimensions(&request) else {
        return invalid("dimensions");
    };
    let base64 = match request.get("encoding_format") {
        None => false,
        Some(value) => match value.as_str() {
            Some("float") => false,
            Some("base64") => true,
            _ => return invalid("encoding_format"),
        },
    };
    let inputs = match request.get("input") {
        Some(Value::String(text)) => vec![text.as_str()],
        Some(Value::Array(items)) if (1..=MAXIMUM_INPUTS).contains(&items.len()) => {
            let Some(texts) = items.iter().map(Value::as_str).collect::<Option<Vec<_>>>() else {
                return invalid("input");
            };
            texts
        }
        _ => return invalid("input"),
    };

    let mut usage = 0;
    for text in &inputs {
        let tokens = estimate_tokens(text);
        if text.trim().is_empty() || tokens > MAXIMUM_INPUT_TOKENS {
            return invalid("input");
        }
        usage += tokens;
    }
    let Some(catalog) = CONCEPT_INDEX.get_or_init(ConceptIndex::build) else {
        return error_response(500, message("embeddings-catalog-capacity"));
    };
    let mut data = Vec::with_capacity(inputs.len());
    for (index, text) in inputs.into_iter().enumerate() {
        let coordinates = catalog.matches(text);
        if coordinates.is_empty() {
            return json_response(
                400,
                &json!({
                    "error": {
                        "code": "no_concepts",
                        "input_index": index,
                        "message": message("embeddings-no-concepts"),
                        "type": "formal_ai_error"
                    }
                }),
            );
        }
        let vector = embedding(&coordinates, dimensions, base64);
        data.push(json!({ "embedding": vector, "index": index, "object": "embedding" }));
    }
    json_response(
        200,
        &json!({
            "data": data,
            "model": MODEL,
            "object": "list",
            "usage": {"prompt_tokens": usage, "total_tokens": usage}
        }),
    )
}
