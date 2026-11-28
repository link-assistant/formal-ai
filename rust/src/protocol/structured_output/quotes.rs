//! Bind quote fields to unchanged spans of the request's source text.

use super::{field_description, key_name, resolved_field, schema, verdict};
use regex::Regex;
use serde_json::{Map, Value};
use std::cell::LazyCell;
use std::collections::{BTreeMap, HashSet};
use std::sync::LazyLock;

const MAXIMUM_SOURCE_BYTES: usize = 8192;

fn pattern(rule: &str) -> Regex {
    let source = crate::seed::agentic_message(&format!("structured-quote-{rule}"), &[]);
    Regex::new(&format!("(?i){source}")).expect("structured quote pattern")
}

static SOURCE: LazyLock<Regex> = LazyLock::new(|| pattern("source"));
static INSTRUCTION: LazyLock<Regex> = LazyLock::new(|| pattern("instruction"));
static TARGET: LazyLock<Regex> = LazyLock::new(|| pattern("target"));
static CUE: LazyLock<Regex> = LazyLock::new(|| pattern("cue"));
static CRITERIA: LazyLock<Regex> = LazyLock::new(|| pattern("criteria"));
static ASSIGNMENT: LazyLock<Regex> = LazyLock::new(|| pattern("assignment"));
static BOUNDARY: LazyLock<Regex> = LazyLock::new(|| pattern("boundary"));

fn spans(text: &str) -> impl Iterator<Item = &str> {
    let mut start = 0;
    BOUNDARY
        .captures_iter(text)
        .map(Some)
        .chain(std::iter::once(None))
        .map(move |boundary| {
            let end = boundary.as_ref().map_or(text.len(), |boundary| {
                let whole = boundary.get(0).expect("quote boundary");
                boundary
                    .get(1)
                    .map_or_else(|| whole.start(), |marks| marks.end())
            });
            let quote = text[start..end].trim();
            start = boundary.map_or(text.len(), |boundary| {
                boundary.get(0).expect("quote boundary").end()
            });
            quote
        })
        .filter(|quote| !quote.is_empty())
}

#[derive(Default)]
struct Criteria<'a> {
    definitions: BTreeMap<String, &'a str>,
    single: Option<&'a str>,
}

fn criteria_from(header: &str) -> Criteria<'_> {
    let Some(marker) = CRITERIA.find(header) else {
        return Criteria::default();
    };
    let tail = &header[marker.end()..];
    let end = BOUNDARY
        .captures_iter(tail)
        .find_map(|boundary| boundary.get(1))
        .map_or(tail.len(), |marks| marks.start());
    let contents = tail[..end].trim();
    let mut definitions = BTreeMap::new();
    let mut assignments = ASSIGNMENT.captures_iter(contents).peekable();
    while let Some(assignment) = assignments.next() {
        let key = key_name(&assignment[1]);
        let start = assignment.get(0).expect("criterion assignment").end();
        let end = assignments
            .peek()
            .and_then(|next| next.get(0))
            .map_or(contents.len(), |next| next.start());
        let definition = contents[start..end].trim();
        definitions
            .entry(key)
            .and_modify(|value| *value = "")
            .or_insert(definition);
    }
    let single = definitions.is_empty().then_some(contents);
    Criteria {
        definitions,
        single,
    }
}

struct QuoteSource<'a> {
    criteria: Criteria<'a>,
    reference: Option<&'a str>,
    instruction: &'a str,
    text: &'a str,
    quotes: Vec<&'a str>,
}

fn quote_source(source: &str, require_instruction: bool) -> Option<QuoteSource<'_>> {
    if source.len() > MAXIMUM_SOURCE_BYTES {
        return None;
    }
    let (header, instruction, text) = if let Some(marker) = SOURCE.find(source) {
        let header = &source[..marker.start()];
        (header, INSTRUCTION.find(header), &source[marker.end()..])
    } else if let Some(instruction) = INSTRUCTION.find(source) {
        let start = source[..instruction.end()].char_indices().next_back()?.0;
        let end = start + source[start..].find([':', '：', '\r', '\n'])?;
        let text = source[end..].chars().next()?;
        (
            &source[..end],
            Some(instruction),
            &source[end + text.len_utf8()..],
        )
    } else {
        if CRITERIA.is_match(source) {
            return None;
        }
        ("", None, source)
    };
    if require_instruction && instruction.is_none() {
        return None;
    }
    let criteria = instruction.map_or(header, |instruction| &header[..instruction.start()]);
    Some(QuoteSource {
        criteria: criteria_from(criteria),
        reference: verdict::criterion_reference(criteria),
        instruction: instruction.map_or("", |instruction| &header[instruction.start()..]),
        text,
        quotes: spans(text).collect(),
    })
}

fn target_from(instruction: &str) -> Option<&str> {
    let cue = TARGET.find(instruction)?;
    let target = spans(&instruction[cue.end()..]).next()?;
    Some(
        target
            .split(',')
            .next()?
            .trim()
            .trim_start_matches(['"', '\'', '“', '«'])
            .trim_end_matches(['.', '!', '?', '"', '\'', '”', '»'])
            .trim(),
    )
}

impl<'a> QuoteSource<'a> {
    fn support(&self, criterion: &str) -> Option<&'a str> {
        if criterion.is_empty()
            || verdict::assess_criterion(criterion, self.text, self.reference)?.result != Some(true)
        {
            return None;
        }
        self.quotes.iter().copied().find(|quote| {
            verdict::assess_criterion(criterion, quote, self.reference)
                .is_some_and(|verdict| verdict.result == Some(true))
        })
    }

    fn single_quote(&self, description: &str) -> Option<&'a str> {
        if let Some(target) = target_from(self.instruction).or_else(|| target_from(description)) {
            return self
                .quotes
                .iter()
                .copied()
                .find(|quote| verdict::mentions_subject(quote, target));
        }
        if let Some(criterion) = self.criteria.single.or_else(|| {
            (self.criteria.definitions.len() == 1)
                .then(|| self.criteria.definitions.values().next().copied())
                .flatten()
        }) {
            return self.support(criterion);
        }
        if self.criteria.definitions.is_empty()
            && let [quote] = self.quotes.as_slice()
        {
            return Some(*quote);
        }
        None
    }
}

pub(super) fn requested_quote(source: &str) -> Option<&str> {
    let input = quote_source(source, true)?;
    target_from(input.instruction)?;
    input.single_quote("")
}

fn has_type(field: &Value, kind: &str) -> bool {
    field.get("type").is_some_and(|types| {
        types == kind
            || types
                .as_array()
                .is_some_and(|types| types.iter().any(|value| value == kind))
    })
}

fn string_shape(field: &Value, root: &Value, visited: &mut HashSet<*const Value>) -> bool {
    let rule = resolved_field(root, field);
    if !visited.insert(std::ptr::from_ref(rule)) {
        return false;
    }
    has_type(rule, "string")
        || rule
            .get("enum")
            .and_then(Value::as_array)
            .is_some_and(|labels| labels.iter().any(Value::is_string))
        || rule.get("const").is_some_and(Value::is_string)
        || ["allOf", "anyOf", "oneOf"].iter().any(|key| {
            rule.get(*key)
                .and_then(Value::as_array)
                .is_some_and(|members| {
                    members
                        .iter()
                        .any(|member| string_shape(member, root, visited))
                })
        })
}

fn quote_field(name: &str, field: &Value, root: &Value) -> bool {
    string_shape(field, root, &mut HashSet::new())
        && (CUE.is_match(&key_name(name))
            || field_description(field, resolved_field(root, field))
                .is_some_and(|text| CUE.is_match(text)))
}

struct QuoteItems<'a> {
    name: &'a str,
    labels: &'a [Value],
    quote_name: &'a str,
}

fn quote_items<'a>(field: &'a Value, root: &'a Value) -> Option<QuoteItems<'a>> {
    let rule = resolved_field(root, field);
    if rule.get("type").is_some() && !has_type(rule, "array") {
        return None;
    }
    let properties = resolved_field(root, rule.get("items")?)
        .get("properties")?
        .as_object()?;
    let mut quotes = properties
        .iter()
        .filter(|(name, field)| quote_field(name, field, root));
    let (quote_name, _) = quotes.next()?;
    if quotes.next().is_some() {
        return None;
    }
    let mut selectors = properties.iter().filter_map(|(name, field)| {
        let labels = resolved_field(root, field).get("enum")?.as_array()?;
        (name != quote_name && labels.iter().all(Value::is_string)).then_some((name, labels))
    });
    let (name, labels) = selectors.next()?;
    selectors.next().is_none().then_some(QuoteItems {
        name,
        labels,
        quote_name,
    })
}

fn criterion_quotes(source: Option<&QuoteSource<'_>>, items: &QuoteItems<'_>) -> Value {
    let Some(source) = source else {
        return Value::Array(Vec::new());
    };
    let mut seen = HashSet::new();
    let quotes = items.labels.iter().filter_map(|label| {
        let label_text = label.as_str()?;
        if !seen.insert(label_text) {
            return None;
        }
        let criterion = if !source.criteria.definitions.is_empty() {
            source
                .criteria
                .definitions
                .get(&key_name(label_text))
                .copied()
        } else if source.criteria.single.is_some() {
            (items.labels.len() == 1)
                .then_some(source.criteria.single)
                .flatten()
        } else {
            Some(label_text)
        };
        let quote = source.support(criterion?)?;
        Some(Value::Object(Map::from_iter([
            (items.name.to_owned(), label.clone()),
            (items.quote_name.to_owned(), Value::String(quote.to_owned())),
        ])))
    });
    Value::Array(quotes.collect())
}

#[derive(Default)]
pub(super) struct QuoteFields<'a> {
    pub values: BTreeMap<&'a str, Option<Value>>,
    pub missing: Vec<&'a str>,
}

pub(super) fn quote_fields<'a>(
    root: &'a Value,
    fields: &'a Value,
    source: &str,
) -> QuoteFields<'a> {
    let bounded = source.len() <= MAXIMUM_SOURCE_BYTES;
    let input = LazyCell::new(|| quote_source(source, false));
    let mut quotes = QuoteFields::default();
    for (name, field) in fields
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        if let Some(items) = quote_items(field, root) {
            let value = bounded.then(|| criterion_quotes(input.as_ref(), &items));
            if value.as_ref().is_some_and(|value| {
                value.as_array().is_some_and(Vec::is_empty) && schema::valid(value, field, root)
            }) {
                quotes.missing.push(name.as_str());
            }
            quotes.values.insert(name.as_str(), value);
        } else if quote_field(name, field, root) {
            let description = field_description(field, resolved_field(root, field)).unwrap_or("");
            let value = input
                .as_ref()
                .and_then(|input| input.single_quote(description))
                .map(|quote| Value::String(quote.to_owned()));
            quotes.values.insert(name.as_str(), value);
        }
    }
    quotes
}
