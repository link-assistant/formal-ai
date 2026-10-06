//! Product-search handler (issues #800 and #872).
//!
//! "Найди мне зарядку для ноутбука Acer Aspire 3 A325-45 на amazon.in"
//! fell through to a research dead end (issue #800) and
//! "игры для малышей … в App Store (iOS)" to unknown (issue #872),
//! because no route understood a shopping request: marketplace, product
//! with model constraints, and qualifiers. This handler composes the
//! site-scoped search those prompts describe.
//!
//! All vocabulary is seed data (`data/seed/product-search-cues.lino`):
//! marketplace hosts and deep-link templates, request cues, qualifier
//! constraints, and per-product verification advice. The handler extracts
//! the product terms (model codes are recognized by shape: an
//! uppercase-dashed alphanumeric token), the matched constraints, and the
//! marketplace, then renders the localized template from
//! `data/seed/multilingual-responses-product-search.lino`.
//!
//! The answer is honest about offline mode: nothing is fetched, the link
//! is the exact query that would run, and the advice says what to verify
//! before buying (a charger must match model, connector, wattage).

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;
use super::finalize_simple;

const CUES_PATH: &str = "data/seed/product-search-cues.lino";
const INTENT: &str = "product_search";

/// One `marketplace` record: host, deep-link template, trigger phrases.
struct Marketplace {
    name: String,
    #[allow(dead_code)] // seed provenance, carried but not yet rendered
    host: String,
    link_template: String,
    phrases: Vec<String>,
}

/// One `constraint` record: a qualifier phrase plus its filter advice.
struct Constraint {
    name: String,
    phrases: Vec<String>,
    advice: String,
}

/// One `product_noun` record: a product word plus its buying advice.
struct ProductNoun {
    noun: String,
    phrases: Vec<String>,
    advice: String,
}

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// Collect the `phrase` ids directly under a record, lowercased.
fn record_phrases(record: &crate::seed::parser::LinoNode) -> Vec<String> {
    record
        .children
        .iter()
        .filter(|child| child.name == "phrase")
        .filter_map(|child| {
            let phrase = child.id.trim().to_lowercase();
            (!phrase.is_empty()).then_some(phrase)
        })
        .collect()
}

/// The whole catalogue from the seed file.
struct Catalogue {
    marketplaces: Vec<Marketplace>,
    constraints: Vec<Constraint>,
    product_nouns: Vec<ProductNoun>,
    cues: Vec<String>,
}

impl Catalogue {
    fn load() -> Self {
        let empty = Self {
            marketplaces: Vec::new(),
            constraints: Vec::new(),
            product_nouns: Vec::new(),
            cues: Vec::new(),
        };
        let Some(text) = seed_text(CUES_PATH) else {
            return empty;
        };
        let tree = parse_lino(text);
        let Some(root) = tree.children.iter().find(|child| child.name == "product_search")
        else {
            return empty;
        };
        let mut out = Self {
            marketplaces: Vec::new(),
            constraints: Vec::new(),
            product_nouns: Vec::new(),
            cues: Vec::new(),
        };
        for record in root.children.iter() {
            match record.name.as_str() {
                "marketplace" => {
                    let name = record.id.clone();
                    if name.is_empty() {
                        continue;
                    }
                    out.marketplaces.push(Marketplace {
                        name,
                        host: record.find_child_value("host").to_string(),
                        link_template: record.find_child_value("link").to_string(),
                        phrases: record_phrases(record),
                    });
                }
                "constraint" => {
                    let name = record.find_child_value("name").to_string();
                    if name.is_empty() {
                        continue;
                    }
                    out.constraints.push(Constraint {
                        name,
                        phrases: record_phrases(record),
                        advice: record.find_child_value("advice").to_string(),
                    });
                }
                "product_noun" => {
                    let noun = record.find_child_value("noun").to_string();
                    if noun.is_empty() {
                        continue;
                    }
                    out.product_nouns.push(ProductNoun {
                        noun,
                        phrases: record_phrases(record),
                        advice: record.find_child_value("advice").to_string(),
                    });
                }
                "intent_cues" => {
                    if record.find_child_value("intent") == INTENT {
                        out.cues = record_phrases(record);
                    }
                }
                _ => {}
            }
        }
        out
    }
}

/// Percent-encode a query for a URL path slot (unreserved characters
/// stay; everything else becomes `%XX` per byte).
fn percent_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push_str("%20"),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// A model code: at least four characters of uppercase letters and
/// digits with a hyphen or digit boundary, like `A325-45` or `A325`.
fn is_model_code(token: &str) -> bool {
    let alphanumeric: String = token
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    alphanumeric.len() >= 4
        && alphanumeric.chars().any(|c| c.is_ascii_digit())
        && alphanumeric.chars().any(|c| c.is_ascii_alphabetic())
        && alphanumeric
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

/// The product terms: the matched product noun plus any model codes and
/// the brand words around them (`Acer Aspire 3 A325-45`).
fn product_terms(prompt: &str, nouns: &[ProductNoun]) -> Vec<String> {
    let lower = prompt.to_lowercase();
    let mut terms: Vec<String> = Vec::new();
    for noun in nouns {
        if noun
            .phrases
            .iter()
            .any(|phrase| lower.contains(phrase.as_str()))
        {
            terms.push(noun.noun.clone());
        }
    }
    for token in prompt.split_whitespace() {
        let trimmed = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-');
        if is_model_code(trimmed) && !terms.iter().any(|term| term == trimmed) {
            terms.push(trimmed.to_owned());
        }
    }
    // Brand words directly preceding a model code (`Acer Aspire 3`).
    let tokens: Vec<&str> = prompt.split_whitespace().collect();
    for (index, token) in tokens.iter().enumerate() {
        let trimmed = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-');
        if is_model_code(trimmed) {
            for lookback in (index.saturating_sub(3)..index).rev() {
                let word = tokens[lookback];
                let candidate = word.trim_matches(|c: char| !c.is_ascii_alphanumeric());
                if candidate.chars().all(|c| c.is_alphabetic())
                    && candidate.len() >= 3
                    && candidate.chars().next().is_some_and(|c| c.is_uppercase())
                    && !terms.iter().any(|term| term.eq_ignore_ascii_case(candidate))
                {
                    terms.push(candidate.to_owned());
                }
            }
        }
    }
    terms
}

/// Fill a localized response template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// The matched product noun's surface phrase (the words the user wrote,
/// e.g. `игры для малышей`), used for the composed query in place of the
/// catalogue slug.
fn matched_noun_surface(lower: &str, nouns: &[ProductNoun]) -> Option<String> {
    nouns
        .iter()
        .find_map(|noun| {
            noun.phrases
                .iter()
                .find(|phrase| lower.contains(phrase.as_str()))
                .cloned()
        })
}

/// Try to recognize a shopping request with a marketplace and compose the
/// site-scoped search. Returns `None` when neither an intent cue nor the
/// structural shopping shape (a marketplace phrase plus a product noun)
/// matches, or no marketplace phrase is present.
pub fn handle_product_search(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let catalogue = Catalogue::load();
    let lower = prompt.to_lowercase();
    let marketplace = catalogue
        .marketplaces
        .iter()
        .find(|marketplace| {
            marketplace
                .phrases
                .iter()
                .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()))
        })?;
    let noun_surface = matched_noun_surface(&lower, &catalogue.product_nouns);
    let cued = catalogue
        .cues
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()));
    // A bare noun phrase with a marketplace ("игры для малышей … в App
    // Store", issue #872) is a shopping request without a request verb;
    // the structural shape carries it.
    if !cued && noun_surface.is_none() {
        return None;
    }
    let terms = product_terms(prompt, &catalogue.product_nouns);
    let matched_noun = catalogue
        .product_nouns
        .iter()
        .find(|noun| terms.iter().any(|term| term == &noun.noun));
    let constraints: Vec<&Constraint> = catalogue
        .constraints
        .iter()
        .filter(|constraint| {
            constraint
                .phrases
                .iter()
                .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()))
        })
        .collect();

    log.append("product_search:marketplace", marketplace.name.clone());
    for term in &terms {
        log.append("product_search:term", term.clone());
    }
    for constraint in &constraints {
        log.append("product_search:constraint", constraint.name.clone());
    }

    // The composed query: the product noun's own surface phrase plus
    // brand/model terms (the noun's catalogue slug is logged, not
    // searched), marketplace-scoped.
    let mut query_terms = Vec::new();
    if let Some(surface) = &noun_surface {
        query_terms.push(surface.clone());
    }
    query_terms.extend(
        terms
            .iter()
            .filter(|term| matched_noun.map(|noun| &noun.noun) != Some(term))
            .cloned(),
    );
    let query = query_terms.join(" ");
    let link = marketplace
        .link_template
        .replace("{query}", &percent_encode(&query));
    let constraints_text = if constraints.is_empty() {
        "(none stated)".to_owned()
    } else {
        constraints
            .iter()
            .map(|constraint| constraint.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let advice = matched_noun
        .map(|noun| noun.advice.clone())
        .unwrap_or_else(|| "confirm the exact model and seller region before ordering".to_owned());
    let mut advice = advice;
    for constraint in &constraints {
        if !constraint.advice.is_empty() {
            advice.push(' ');
            advice.push_str(&constraint.advice);
        }
    }

    let product_display = noun_surface
        .clone()
        .or_else(|| terms.first().cloned())
        .unwrap_or_else(|| prompt.trim().to_owned());
    let body = template(
        "product_search_result",
        &[
            ("product", &product_display),
            ("marketplace", &marketplace.name.replace('_', " ")),
            ("link", &link),
            ("constraints", &constraints_text),
            ("advice", &advice),
        ],
    );
    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:product_search_result",
        &body,
        0.75,
    ))
}
