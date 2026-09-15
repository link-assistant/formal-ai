//! Bounded discovery of integer-sequence programs from the official OEIS API.
//!
//! Search results are only hypotheses. A result becomes executable when its
//! source definition can be reduced to a deliberately small arithmetic or
//! linear-recurrence grammar and the resulting program passes the task's
//! examples in the ordinary composition sandbox.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::coding::python_render::{render_function, runtime_template};
use crate::coding::task_spec::CodingTaskSpec;
use crate::source_fetch::{CachedSourceClient, FetchError, SourceCapture, SourceTransport};

const API: &str = "https://oeis.org";
const LICENSE: &str = "CC-BY-SA-4.0";
const MAX_REFERENCES: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceProgram {
    pub id: String,
    pub label: String,
    pub source: String,
    pub callable_name: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
    pub license: String,
    pub composition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceDiscovery {
    pub programs: Vec<SequenceProgram>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct Record {
    number: u64,
    #[serde(default)]
    data: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    comment: Vec<String>,
    #[serde(default)]
    link: Vec<String>,
    #[serde(default)]
    xref: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LinearRecurrence {
    coefficients: Vec<i128>,
    initial: Vec<i128>,
}

/// Discover source-backed sequence programs only for task shapes that expose
/// an integer sequence or a named-number membership predicate.
pub fn discover_programs<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    spec: &CodingTaskSpec,
) -> SequenceDiscovery {
    let queries = research_queries(spec);
    let mut diagnostics = Vec::new();
    let mut programs = Vec::new();
    for query in queries {
        match discover_query(client, spec, &query) {
            Ok((mut found, mut misses)) => {
                programs.append(&mut found);
                diagnostics.append(&mut misses);
            }
            Err(error) => diagnostics.push(format!("oeis:{query}:{error}")),
        }
    }
    programs.sort_by(|left, right| left.id.cmp(&right.id));
    programs.dedup_by(|left, right| left.id == right.id);
    SequenceDiscovery {
        programs,
        diagnostics,
    }
}

fn discover_query<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    spec: &CodingTaskSpec,
    query: &str,
) -> Result<(Vec<SequenceProgram>, Vec<String>), FetchError> {
    let search = client.fetch(&format!(
        "{API}/search?q={}&fmt=json",
        encode_component(query)
    ))?;
    let initial = parse_records(&search)?;
    let mut ids = initial
        .first()
        .map(|record| vec![record.number])
        .unwrap_or_default();
    ids.extend(referenced_ids(&initial));
    let mut seen = BTreeSet::new();
    let mut programs = Vec::new();
    let mut diagnostics = Vec::new();
    for id in ids
        .into_iter()
        .filter(|id| seen.insert(*id))
        .take(1 + MAX_REFERENCES)
    {
        let url = format!("{API}/A{id:06}?fmt=json");
        let capture = match client.fetch(&url) {
            Ok(capture) => capture,
            Err(error) => {
                diagnostics.push(format!("oeis:{url}:{error}"));
                continue;
            }
        };
        let record = match parse_records(&capture) {
            Ok(records) => records.into_iter().next(),
            Err(error) => {
                diagnostics.push(format!("oeis:{url}:{error}"));
                continue;
            }
        };
        let Some(record) = record else { continue };
        programs.extend(programs_for_record(spec, &record, &capture));
        if programs.len() >= 4 {
            break;
        }
    }
    Ok((programs, diagnostics))
}

fn parse_records(capture: &SourceCapture) -> Result<Vec<Record>, FetchError> {
    if let Ok(records) = serde_json::from_slice::<Vec<Record>>(capture.bytes()) {
        return Ok(records);
    }
    serde_json::from_slice::<Record>(capture.bytes())
        .map(|record| vec![record])
        .map_err(|error| {
            FetchError::Transport(format!(
                "oeis_invalid_json:{}:{error}",
                capture.source_url()
            ))
        })
}

fn referenced_ids(records: &[Record]) -> Vec<u64> {
    let mut ids = Vec::new();
    for record in records {
        for text in std::iter::once(&record.name)
            .chain(record.comment.iter())
            .chain(record.xref.iter())
            .chain(record.link.iter())
        {
            ids.extend(oeis_ids_in(text));
        }
    }
    ids
}

fn oeis_ids_in(text: &str) -> Vec<u64> {
    let bytes = text.as_bytes();
    let mut ids = Vec::new();
    for index in 0..bytes.len().saturating_sub(6) {
        let end = index + 7;
        if bytes[index] != b'A'
            || end > bytes.len()
            || !bytes[index + 1..end].iter().all(u8::is_ascii_digit)
            || bytes.get(end).is_some_and(u8::is_ascii_digit)
        {
            continue;
        }
        if let Ok(id) = text[index + 1..end].parse::<u64>() {
            ids.push(id);
        }
    }
    ids
}

fn programs_for_record(
    spec: &CodingTaskSpec,
    record: &Record,
    capture: &SourceCapture,
) -> Vec<SequenceProgram> {
    let mut programs = Vec::new();
    if boolean_examples(spec)
        && let Some(formula) = arithmetic_formula(&record.name)
    {
        for start in [0, 1] {
            let body = formula_membership_body(&spec.parameters[0].name, &formula, start);
            programs.push(sequence_program(
                spec,
                record,
                capture,
                &format!("formula-membership-start-{start}"),
                &body,
                format!("oeis_formula_membership(index_start={start})"),
            ));
        }
    }
    if integer_examples(spec)
        && let Some(recurrence) = linear_recurrence(&record.name)
    {
        for (divisor, offset) in matching_index_maps(spec, record) {
            let body = recurrence_body(&spec.parameters[0].name, &recurrence, divisor, offset);
            programs.push(sequence_program(
                spec,
                record,
                capture,
                &format!("linear-recurrence-divisor-{divisor}-offset-{offset}"),
                &body,
                format!("oeis_linear_recurrence(index=floor(n/{divisor})+{offset})"),
            ));
        }
    }
    programs
}

fn sequence_program(
    spec: &CodingTaskSpec,
    record: &Record,
    capture: &SourceCapture,
    variant: &str,
    body: &str,
    composition: String,
) -> SequenceProgram {
    SequenceProgram {
        id: format!("A{:06}:{variant}", record.number),
        label: record_label(record),
        source: render_function(spec, body, []),
        callable_name: spec.name.clone(),
        source_url: capture.source_url().to_owned(),
        sha256: capture.sha256().to_owned(),
        fetched_at: capture.fetched_at().to_owned(),
        license: LICENSE.to_owned(),
        composition,
    }
}

fn research_queries(spec: &CodingTaskSpec) -> Vec<String> {
    if boolean_examples(spec) && spec.parameters.len() == 1 {
        let terms = spec
            .name
            .split('_')
            .filter(|term| !matches!(*term, "is" | "check" | "test" | "number"))
            .collect::<Vec<_>>();
        if !terms.is_empty() {
            return vec![terms.join(" ")];
        }
    }
    if integer_examples(spec)
        && let Some(query) = tiling_query(&spec.requirement_sentences.join(" "))
    {
        return vec![query];
    }
    Vec::new()
}

fn tiling_query(text: &str) -> Option<String> {
    let normalized = crate::engine::normalize_prompt(text);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    let dimension_windows = tokens
        .windows(3)
        .enumerate()
        .filter(|(_, window)| {
            window[1] == "x" && dimension_token(window[0]) && dimension_token(window[2])
        })
        .collect::<Vec<_>>();
    let (first_index, _) = dimension_windows.first()?;
    let (last_index, dimensions) = dimension_windows.last()?;
    let object = tokens[first_index + 3..*last_index]
        .iter()
        .copied()
        .find(|token| token.chars().count() > 2)
        .map(singularize_english_noun)?;
    Some(template(
        "oeis_dimension_query",
        &[
            ("left", dimensions[0]),
            ("right", dimensions[2]),
            ("object", &object),
        ],
    ))
}

/// Reduce the tile noun to the form sequence indexes conventionally use.
///
/// The dimension grammar already establishes that this token names the repeated
/// object; the morphology is deliberately domain-neutral (`berries`, `boxes`,
/// `dominoes`, `tiles`) rather than a list of benchmark objects.
fn singularize_english_noun(word: &str) -> String {
    if word.len() > 4
        && let Some(stem) = word.strip_suffix("ies")
    {
        return format!("{stem}y");
    }
    if word.len() > 4
        && ["ches", "shes", "xes", "zes", "ses", "oes"]
            .iter()
            .any(|suffix| word.ends_with(suffix))
    {
        return word[..word.len() - 2].to_owned();
    }
    if word.len() > 3 && word.ends_with('s') && !word.ends_with("ss") {
        return word[..word.len() - 1].to_owned();
    }
    word.to_owned()
}

fn dimension_token(token: &str) -> bool {
    token.chars().all(|character| character.is_ascii_digit())
        || (token.len() == 1 && token.chars().all(char::is_alphabetic))
}

fn boolean_examples(spec: &CodingTaskSpec) -> bool {
    !spec.examples.is_empty()
        && spec
            .examples
            .iter()
            .all(|example| matches!(example.expected.trim(), "True" | "False"))
}

fn integer_examples(spec: &CodingTaskSpec) -> bool {
    spec.parameters.len() == 1
        && !spec.examples.is_empty()
        && spec.examples.iter().all(|example| {
            example.arguments.len() == 1
                && example.arguments[0].trim().parse::<i128>().is_ok()
                && example.expected.trim().parse::<i128>().is_ok()
        })
}

fn arithmetic_formula(name: &str) -> Option<String> {
    let expression = name
        .split_once(':')
        .map(|(_, expression)| expression)
        .or_else(|| name.split_once("a(n) =").map(|(_, expression)| expression))?
        .split('.')
        .next()?
        .trim()
        .replace('^', "**");
    let compact = expression.replace(' ', "");
    if !compact.contains('n')
        || compact
            .split_once('(')
            .is_some_and(|(callee, _)| callee == "a")
        || !compact.chars().all(|character| {
            character.is_ascii_digit()
                || matches!(character, 'n' | '+' | '-' | '*' | '/' | '(' | ')')
        })
    {
        return None;
    }
    Some(compact)
}

fn formula_membership_body(parameter: &str, formula: &str, start: usize) -> String {
    let expression = replace_variable(formula, "source_index");
    let start = start.to_string();
    template(
        "oeis_formula_membership",
        &[
            ("start", &start),
            ("expression", &expression),
            ("parameter", parameter),
        ],
    )
}

fn replace_variable(expression: &str, replacement: &str) -> String {
    expression
        .chars()
        .map(|character| {
            if character == 'n' {
                replacement.to_owned()
            } else {
                character.to_string()
            }
        })
        .collect()
}

fn linear_recurrence(name: &str) -> Option<LinearRecurrence> {
    let normalized = name.replace(' ', "");
    let normalized = normalized.trim_end_matches('.');
    let (transition, bases) = normalized.strip_prefix("a(n)=")?.split_once(",with")?;
    let first_end = transition.find("a(n-1)")?;
    let first = parse_coefficient(&transition[..first_end])?;
    let second_start = first_end + "a(n-1)".len();
    let second_end = transition.find("a(n-2)")?;
    let second = parse_coefficient(&transition[second_start..second_end])?;
    let (base_zero, base_one) = bases.split_once(',')?;
    Some(LinearRecurrence {
        coefficients: vec![first, second],
        initial: vec![
            base_zero.strip_prefix("a(0)=")?.parse().ok()?,
            base_one.strip_prefix("a(1)=")?.parse().ok()?,
        ],
    })
}

fn parse_coefficient(value: &str) -> Option<i128> {
    let value = value.trim_end_matches('*');
    match value {
        "" | "+" => Some(1),
        "-" => Some(-1),
        _ => value.parse().ok(),
    }
}

fn matching_index_maps(spec: &CodingTaskSpec, record: &Record) -> Vec<(i128, i128)> {
    let terms = record
        .data
        .split(',')
        .filter_map(|term| term.trim().parse::<i128>().ok())
        .collect::<Vec<_>>();
    let examples = spec
        .examples
        .iter()
        .filter_map(|example| {
            Some((
                example.arguments.first()?.trim().parse::<i128>().ok()?,
                example.expected.trim().parse::<i128>().ok()?,
            ))
        })
        .collect::<Vec<_>>();
    let mut maps = Vec::new();
    for divisor in 1..=6 {
        for offset in -3..=3 {
            let matches = examples.iter().all(|(argument, expected)| {
                if argument % divisor != 0 {
                    return *expected == 0;
                }
                let index = argument / divisor + offset;
                usize::try_from(index)
                    .ok()
                    .and_then(|index| terms.get(index))
                    == Some(expected)
            });
            if matches {
                maps.push((divisor, offset));
            }
        }
    }
    maps
}

fn recurrence_body(
    parameter: &str,
    recurrence: &LinearRecurrence,
    divisor: i128,
    offset: i128,
) -> String {
    let transition = recurrence
        .coefficients
        .iter()
        .enumerate()
        .map(|(index, coefficient)| {
            let coefficient = coefficient.to_string();
            let index = (index + 1).to_string();
            template(
                "oeis_recurrence_term",
                &[("coefficient", &coefficient), ("index", &index)],
            )
        })
        .collect::<Vec<_>>()
        .join(" + ");
    let divisor = divisor.to_string();
    let offset = offset.to_string();
    let initial = format!("{:?}", recurrence.initial);
    template(
        "oeis_linear_recurrence",
        &[
            ("parameter", parameter),
            ("divisor", &divisor),
            ("offset", &offset),
            ("initial", &initial),
            ("transition", &transition),
        ],
    )
}

fn template(id: &str, values: &[(&str, &str)]) -> String {
    runtime_template(id, values).unwrap_or_else(|| panic!("missing runtime template {id}"))
}

fn record_label(record: &Record) -> String {
    std::iter::once(record.name.as_str())
        .chain(record.comment.iter().map(String::as_str))
        .take(4)
        .collect::<Vec<_>>()
        .join(" ")
}

fn encode_component(value: &str) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    encoded
}
