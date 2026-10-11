use super::{ModuleFunctionRequest, bare, evaluated, relation_expression};
use crate::agentic_coding::write_request::{bare_surfaces, clean_path_token, looks_like_file_path};
use crate::seed;

/// The specification's value at `samples`, computed by the calculator: the
/// clause after the seeded return action with the parameters bound to the
/// samples, or, when that names no operands, the arithmetic relation the
/// clause names around its return action applied to the samples in parameter
/// order. Mirrors `specifiedValue`.
pub(super) fn specified_value(
    request: &ModuleFunctionRequest,
    samples: &[String],
) -> Option<String> {
    let lexicon = seed::lexicon();
    let words: Vec<&str> = request.clause.split_whitespace().collect();
    let returns = words
        .iter()
        .position(|word| lexicon.mentions_role("coding_return_action", &bare(word)))?;
    let after_signature = request
        .clause
        .find(')')
        .map_or(request.clause.as_str(), |close| {
            &request.clause[close + 1..]
        });
    stated_value(
        &words[returns + 1..],
        after_signature,
        &request.parameters,
        samples,
    )
}

/// The value a stated return computes at `samples`.
///
/// The longest expression the words open with, its parameters bound to the
/// samples (`a - b to m.mjs` reads `2 - 3`; the words after it belong to the
/// request, PR #1188 T92), or else the arithmetic relation `relation_text`
/// names applied to the samples. Mirrors `statedValue`.
fn complete_symbolic_expression(expression: &str) -> bool {
    let Ok(pattern) = regex::Regex::new(r"[0-9]+(?:\.[0-9]+)?|[()+*/%−^-]") else {
        return false;
    };
    let lexical = pattern
        .find_iter(expression)
        .map(|token| token.as_str())
        .collect::<Vec<_>>();
    if lexical.join("")
        != expression
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
    {
        return false;
    }
    let mut operand = true;
    let mut depth = 0usize;
    for token in lexical {
        if operand {
            if token == "(" {
                depth += 1;
            } else if !matches!(token, "+" | "-" | "−") {
                if !token.starts_with(|character: char| character.is_ascii_digit()) {
                    return false;
                }
                operand = false;
            }
        } else if token == ")" {
            if depth == 0 {
                return false;
            }
            depth -= 1;
        } else {
            if !matches!(token, "+" | "-" | "−" | "*" | "/" | "%" | "^") {
                return false;
            }
            operand = true;
        }
    }
    !operand && depth == 0
}

pub(in crate::agentic_coding) fn stated_value(
    words: &[&str],
    relation_text: &str,
    parameters: &[String],
    samples: &[String],
) -> Option<String> {
    let bound = words
        .iter()
        .map(|word| {
            let lexical =
                regex::Regex::new(r"^(?:[A-Za-z_][A-Za-z0-9_]*|[0-9]+(?:\.[0-9]+)?|[()+*/%−-])+$")
                    .ok()?;
            if lexical.is_match(word) {
                let identifiers = regex::Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").ok()?;
                return Some(
                    identifiers
                        .replace_all(word, |captures: &regex::Captures<'_>| {
                            parameters
                                .iter()
                                .position(|parameter| parameter == &captures[0])
                                .and_then(|index| samples.get(index))
                                .cloned()
                                .unwrap_or_else(|| captures[0].to_owned())
                        })
                        .into_owned(),
                );
            }
            Some(
                parameters
                    .iter()
                    .position(|parameter| *parameter == bare(word))
                    .and_then(|index| samples.get(index))
                    .map_or_else(|| (*word).to_owned(), Clone::clone),
            )
        })
        .collect::<Option<Vec<_>>>()?;
    for end in (1..=bound.len()).rev() {
        let prefix = bound[..end].join(" ");
        let symbolic = prefix.chars().any(|character| "+*/%−-".contains(character));
        let expression = if symbolic {
            complete_symbolic_expression(&prefix).then_some(prefix)
        } else {
            crate::calculation::calculation_expression_candidates(&prefix)
                .into_iter()
                .next()
                .map(|candidate| candidate.expression)
        };
        if let Some(expression) = expression {
            if let Some(value) = evaluated(&expression) {
                let suffix = &words[end..];
                if suffix.is_empty() {
                    return Some(value);
                }
                let path = clean_path_token(suffix[suffix.len() - 1]);
                let cue = suffix[..suffix.len() - 1]
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase();
                let owned_target = crate::agentic_coding::write_request::safe_relative_path(&path)
                    && looks_like_file_path(&path)
                    && ["file_edit_target_cue", "file_write_destination_cue"]
                        .iter()
                        .flat_map(|role| bare_surfaces(role))
                        .any(|surface| {
                            surface
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .to_lowercase()
                                == cue
                        });
                return owned_target.then_some(value);
            }
        }
    }
    evaluated(&relation_expression(relation_text, samples)?)
}
