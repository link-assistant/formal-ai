//! Bind the caller's operands to local algorithm templates.

use std::sync::OnceLock;

use regex::Regex;

use super::{ProgramSpec, ProgramTemplate, catalog};

const MAX_RANGE_SPAN: i64 = 999;
const MAX_RANGE_MAGNITUDE: i64 = 1_000_000;
const MAX_FACTORIAL: i64 = 18;
const MAX_FIBONACCI: i64 = 77;

pub(crate) struct BoundCatalogProgram {
    pub template: ProgramTemplate,
    pub expected_output: String,
}

pub(crate) fn catalog_parameter_guidance(task: &str) -> Option<&'static str> {
    match task {
        "count_to_three" | "fizzbuzz" | "sum_to_ten" => Some("request-program-range"),
        "factorial" => Some("request-factorial-operand"),
        "reverse_string" => Some("request-reversal-operand"),
        "fibonacci" => Some("request-fibonacci-index"),
        _ => None,
    }
}

/// The parsed values are the only values substituted into the template. Text
/// insertion uses the target language's string literal encoder.
pub(crate) fn bind_catalog_program(
    spec: ProgramSpec<'_>,
    prompt: &str,
) -> Option<BoundCatalogProgram> {
    let mut values = Vec::new();
    let expected_output = match spec.task.slug {
        "count_to_three" | "fizzbuzz" | "sum_to_ten" => {
            let text = number_text(prompt);
            let operands = integer_operands(&text, spec.language.slug)?;
            let (start, end) = integer_range(&text, &operands)?;
            if start < -MAX_RANGE_MAGNITUDE
                || end > MAX_RANGE_MAGNITUDE
                || end < start
                || end - start > MAX_RANGE_SPAN
            {
                return None;
            }
            values.extend([
                ("start", start.to_string()),
                ("end", end.to_string()),
                ("end_exclusive", (end + 1).to_string()),
            ]);
            if spec.task.slug == "sum_to_ten" {
                ((start + end) * (end - start + 1) / 2).to_string()
            } else {
                (start..=end)
                    .map(|n| match (spec.task.slug, n % 3, n % 5) {
                        ("fizzbuzz", 0, 0) => String::from("FizzBuzz"),
                        ("fizzbuzz", 0, _) => String::from("Fizz"),
                        ("fizzbuzz", _, 0) => String::from("Buzz"),
                        _ => n.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        "factorial" => {
            let text = number_text(prompt);
            let operands = integer_operands(&text, spec.language.slug)?;
            let [(n, _)] = operands.as_slice() else {
                return None;
            };
            if !(0..=MAX_FACTORIAL).contains(n) {
                return None;
            }
            values.extend([("n", n.to_string()), ("end_exclusive", (n + 1).to_string())]);
            (1..=*n).product::<i64>().to_string()
        }
        "reverse_string" => {
            let parts = crate::normal_markov::quoted_segment_spans(prompt);
            let text = match parts.as_slice() {
                [quoted] => quoted.text.as_str(),
                [] => trailing_reversal_text(prompt)?,
                _ => return None,
            };
            values.push((
                "text",
                catalog::program_string_literal(spec.language.slug, text)?,
            ));
            text.chars().rev().collect()
        }
        "fibonacci" => {
            let text = number_text(prompt);
            let operands = integer_operands(&text, spec.language.slug)?;
            match operands.as_slice() {
                [] => String::new(),
                [(n, _)] if (0..=MAX_FIBONACCI).contains(n) => {
                    values.extend([
                        ("index", n.to_string()),
                        ("go_fmt", String::from("import \"fmt\"")),
                    ]);
                    let (mut previous, mut current) = (0_i64, 1_i64);
                    for _ in 0..*n {
                        (previous, current) = (current, previous + current);
                    }
                    previous.to_string()
                }
                _ => return None,
            }
        }
        _ => return None,
    };
    let raw = spec.template.code.as_ref();
    let code = if spec.task.slug == "fibonacci" && values.is_empty() {
        raw.lines()
            .filter(|line| {
                !["{{index}}", "{{go_fmt}}"]
                    .iter()
                    .any(|&slot| line.contains(slot))
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_owned()
    } else {
        render_slots(raw, &values)
    };
    Some(BoundCatalogProgram {
        template: ProgramTemplate {
            code: code.into(),
            ..spec.template.clone()
        },
        expected_output,
    })
}

fn render_slots(template: &str, values: &[(&str, String)]) -> String {
    static SLOTS: OnceLock<Regex> = OnceLock::new();
    SLOTS
        .get_or_init(|| Regex::new(r"\{\{([a-z_]+)\}\}").unwrap())
        .replace_all(template, |captures: &regex::Captures<'_>| {
            values
                .iter()
                .find(|(name, _)| *name == &captures[1])
                .map_or_else(|| captures[0].to_owned(), |(_, value)| value.clone())
        })
        .into_owned()
}

/// A single unquoted trailing operand is accepted only after a complete
/// reversal phrase. Quotes delimit operands with whitespace or another clause.
fn trailing_reversal_text(prompt: &str) -> Option<&str> {
    let lower = prompt.to_lowercase();
    if lower.len() != prompt.len() {
        return None;
    }
    crate::seed::lexicon()
        .meaning("program-reverse-operand-introducer")
        .into_iter()
        .flat_map(crate::seed::Meaning::words)
        .filter_map(|alias| {
            let (start, _) = lower.rmatch_indices(alias).next()?;
            let end = start + alias.len();
            if !lower.is_char_boundary(end) {
                return None;
            }
            let word = prompt.get(end..)?.strip_prefix(char::is_whitespace)?.trim();
            (!word.is_empty() && word.chars().all(|ch| ch.is_alphanumeric() || ch == '_'))
                .then_some(word)
                .map(|text| (end, text))
        })
        .max_by_key(|(end, _)| *end)
        .map(|(_, text)| text)
}

fn number_text(prompt: &str) -> String {
    crate::solver_handlers::text_outside_quoted_segments(prompt).to_lowercase()
}

fn integer_range(text: &str, operands: &[(i64, regex::Match<'_>)]) -> Option<(i64, i64)> {
    let connectors = || catalog::alias_surfaces("line_range", "connector");
    match operands {
        [(end, position)] => {
            let prefix = text[..position.start()].trim_end();
            connectors()
                .any(|word| {
                    prefix.strip_suffix(word).is_some_and(|before| {
                        before.is_empty()
                            || before.ends_with(char::is_whitespace)
                            || word.chars().any(|ch| matches!(ch, '到' | '至'))
                    })
                })
                .then_some((1, *end))
        }
        [(start, left), (end, right)] => {
            let separator = text[left.end()..right.start()].trim();
            (separator == ".." || separator == "..=" || connectors().any(|word| word == separator))
                .then_some((*start, *end))
        }
        _ => None,
    }
}

fn integer_operands<'a>(text: &'a str, language: &str) -> Option<Vec<(i64, regex::Match<'a>)>> {
    static TOKENS: OnceLock<Regex> = OnceLock::new();
    let tokens = TOKENS.get_or_init(|| {
        Regex::new(r"[-+]?[0-9]+(?:\.[0-9]+)?(?:st|nd|rd|th)?|[\p{Han}]|[\p{L}\p{M}&&[^\p{Han}]]+")
            .unwrap()
    });
    let mut result = Vec::new();
    let (words, _) = crate::seed::lexicon().arithmetic_normalization_tables();
    let mut previous_end = 0;
    for token in tokens.find_iter(text) {
        let word = token.as_str();
        let after_language = catalog::alias_surfaces("program_language", language).any(|alias| {
            text[..token.start()].trim_end().ends_with(alias)
                && text[..token.start()].trim_end().len() - alias.len() <= previous_end
        });
        previous_end = token.end();
        let number = word.trim_start_matches(['-', '+']);
        if number.as_bytes().first().is_some_and(u8::is_ascii_digit) {
            if after_language
                && token.start() > 0
                && text[..token.start()].ends_with(char::is_whitespace)
            {
                continue;
            }
            let end = ["st", "nd", "rd", "th"]
                .into_iter()
                .find_map(|suffix| word.strip_suffix(suffix))
                .unwrap_or(word);
            result.push((end.parse().ok()?, token));
            continue;
        }
        if word == "一" && text[token.end()..].starts_with(['个', '個']) {
            continue;
        }
        // Short indefinite articles have a separate use as function words.
        let has_one = |surface: &str| surface == word && !matches!(surface, "un" | "una");
        if let Some((_, value)) = words.iter().find(|(surface, _)| has_one(surface)) {
            result.push((value.parse().ok()?, token));
        }
    }
    Some(result)
}
