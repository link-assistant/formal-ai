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
    surfaces: BTreeMap<String, Vec<usize>>,
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
        let coordinates: BTreeMap<_, _> = concepts
            .into_iter()
            .enumerate()
            .map(|(index, concept)| (concept, index))
            .collect();
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

    fn vector(&self, text: &str) -> Vec<u8> {
        let normalized = normalize(text);
        let tokens: Vec<_> = words(&normalized).collect();
        let mut vector = vec![0; usize::from(DIMENSIONS)];
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
                for &position in positions {
                    vector[position] = 1;
                }
            }
            start = next;
        }
        vector
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
        .map(|node| (node.id.clone(), node.find_child_value("text").to_owned()))
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

fn encode_base64(vector: Vec<u8>) -> String {
    let bytes: Vec<_> = vector
        .into_iter()
        .flat_map(|weight| f32::from(weight).to_le_bytes())
        .collect();
    STANDARD.encode(bytes)
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
    if request
        .get("dimensions")
        .is_some_and(|value| value.as_f64() != Some(f64::from(DIMENSIONS)))
    {
        return invalid("dimensions");
    }
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
    let data: Vec<_> = inputs
        .iter()
        .enumerate()
        .map(|(index, text)| {
            let vector = catalog.vector(text);
            let embedding = if base64 {
                json!(encode_base64(vector))
            } else {
                json!(vector)
            };
            json!({ "embedding": embedding, "index": index, "object": "embedding" })
        })
        .collect();
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
