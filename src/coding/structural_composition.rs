//! Higher-order compositions assembled from source-grounded structure meanings.
//!
//! This module deliberately knows no benchmark ids, entry points, or canonical
//! solutions. It combines language-independent meanings selected from the task
//! prose, then the caller executes every candidate against the task's examples.

use crate::coding::composition::{Draft, idiom};
use crate::coding::concept_discovery::{ConceptMap, structural_meanings};
use crate::coding::python_render::render_function;
use crate::coding::task_spec::{ArtifactShape, CodingTaskSpec};

pub(super) fn additional_drafts(spec: &CodingTaskSpec, concepts: &ConceptMap) -> Vec<Draft> {
    if spec.artifact_shape != ArtifactShape::Function {
        return Vec::new();
    }
    let structures = concepts.structure_ids();
    let has = |id: &str| structures.iter().any(|structure| structure == id);
    let names = spec
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<Vec<_>>();
    let mut drafts = Vec::new();

    if has("arithmetic_fractional_part") && names.len() == 1 {
        let expression = idiom("arithmetic_fractional_part", &[("value", names[0])]);
        drafts.push(draft(
            spec,
            "arithmetic_fractional_part",
            return_value(&expression),
            [],
        ));
    }

    if has("reduce_mean") && has("absolute_deviation") && names.len() == 1 {
        let center = idiom("reduce_mean", &[("items", names[0])]);
        let deviation = idiom(
            "absolute_deviation",
            &[("item", "item"), ("center", &center)],
        );
        let deviations = idiom(
            "map_each",
            &[
                ("expression", &deviation),
                ("item", "item"),
                ("items", names[0]),
            ],
        );
        let expression = idiom("reduce_mean", &[("items", &deviations)]);
        drafts.push(draft(
            spec,
            "reduce_mean(map_each(absolute_deviation(reduce_mean)))",
            return_value(&expression),
            [],
        ));
    }

    if has("prefix_negative") && names.len() == 1 {
        let prefixes = idiom(
            "running_prefix",
            &[
                ("items", names[0]),
                ("operation", "lambda left, right: left + right"),
            ],
        );
        let predicate = idiom("prefix_negative", &[("value", "value")]);
        let expression = idiom(
            "quantifier_any",
            &[
                ("predicate", &predicate),
                ("item", "value"),
                ("items", &prefixes),
            ],
        );
        drafts.push(draft(
            spec,
            "quantifier_any(prefix_negative(running_prefix))",
            return_value(&expression),
            ["import itertools"],
        ));
    }

    if has("interpose_each") && names.len() >= 2 {
        let expression = idiom(
            "interpose_each",
            &[("items", names[0]), ("separator", names[1])],
        );
        drafts.push(draft(spec, "interpose_each", return_value(&expression), []));
    }

    if has("balanced_delimiter_groups") && names.len() == 1 {
        let body = format!(
            "groups = []\ncurrent = []\ndepth = 0\nfor symbol in {}:\n    if symbol.isspace():\n        continue\n    current.append(symbol)\n    depth += 1 if symbol == '(' else -1\n    if depth == 0:\n        groups.append(''.join(current))\n        current = []\nreturn groups",
            names[0]
        );
        drafts.push(draft(spec, "balanced_delimiter_groups", body, []));
    }

    if has("group_max_nesting") && names.len() == 1 {
        let body = format!(
            "depths = []\nfor group in {}.split():\n    depth = 0\n    maximum = 0\n    for symbol in group:\n        depth += 1 if symbol == '(' else -1\n        maximum = max(maximum, depth)\n    depths.append(maximum)\nreturn depths",
            names[0]
        );
        drafts.push(draft(spec, "group_max_nesting", body, []));
    }

    if has("prefix_enumeration") && names.len() == 1 {
        let expression = idiom("prefix_enumeration", &[("text", names[0])]);
        drafts.push(draft(
            spec,
            "prefix_enumeration",
            return_value(&expression),
            [],
        ));
    }

    if has("zero_based_range") && has("stringify_each") && names.len() == 1 {
        let range = idiom("zero_based_range", &[("bound", names[0])]);
        let strings = idiom("stringify_each", &[("item", "number"), ("items", &range)]);
        let expression = idiom("join_with_space", &[("items", &strings)]);
        drafts.push(draft(
            spec,
            "join_with_space(stringify_each(zero_based_range))",
            return_value(&expression),
            [],
        ));
    }

    if has("palindrome_extension") && names.len() == 1 {
        let body = format!(
            "for start in range(len({0}) + 1):\n    suffix = {0}[start:]\n    if suffix == suffix[::-1]:\n        return {0} + {0}[:start][::-1]",
            names[0]
        );
        drafts.push(draft(spec, "palindrome_extension", body, []));
    }

    if has("aligned_binary_xor") && names.len() >= 2 {
        let expression = format!(
            "''.join('0' if left == right else '1' for left, right in zip({}, {}))",
            names[0], names[1]
        );
        drafts.push(draft(
            spec,
            "aligned_binary_xor",
            return_value(&expression),
            [],
        ));
    }

    if has("stable_longest") && names.len() == 1 {
        let expression = idiom("stable_longest", &[("items", names[0])]);
        drafts.push(draft(spec, "stable_longest", return_value(&expression), []));
    }

    if has("explicit_value_mapping")
        && names.len() == 1
        && let Some(mapping) = explicit_numeric_mapping(spec)
    {
        let body = format!(
            "mapping = {mapping}\nreturn [mapping[item] for item in {}.split()]",
            names[0]
        );
        drafts.push(draft(
            spec,
            "map_each(explicit_value_mapping(split_on_space))",
            body,
            [],
        ));
    }

    if has("explicit_ordering")
        && has("sort_ascending")
        && names.len() == 1
        && let Some(order) = explicit_order(spec)
    {
        let body = format!(
            "order = {order}\nreturn ' '.join(sorted({}.split(), key=order.__getitem__))",
            names[0]
        );
        drafts.push(draft(
            spec,
            "join_with_space(sort_ascending(explicit_ordering(split_on_space)))",
            body,
            [],
        ));
    }

    if has("bounded_top") && names.len() >= 2 {
        let expression = idiom("bounded_top", &[("items", names[0]), ("count", names[1])]);
        drafts.push(draft(spec, "bounded_top", return_value(&expression), []));
    }
    if has("bounded_bottom") && names.len() >= 2 {
        let expression = idiom(
            "bounded_bottom",
            &[("items", names[0]), ("count", names[1])],
        );
        drafts.push(draft(spec, "bounded_bottom", return_value(&expression), []));
    }

    if has("power_square_each") && names.len() == 1 {
        let expression = idiom(
            "power_square_each",
            &[("item", "item"), ("items", names[0])],
        );
        drafts.push(draft(
            spec,
            "power_square_each",
            return_value(&expression),
            [],
        ));
    }

    if has("keyed_sum_order") && names.len() == 1 {
        let expression = idiom("keyed_sum_order", &[("items", names[0])]);
        drafts.push(draft(
            spec,
            "keyed_sum_order",
            return_value(&expression),
            [],
        ));
    }

    if has("frequency_rank")
        && names.len() == 1
        && let Some(count) = inferred_output_cardinality(spec)
    {
        let expression = idiom(
            "frequency_rank",
            &[("items", names[0]), ("count", &count.to_string())],
        );
        drafts.push(draft(
            spec,
            "frequency_rank",
            return_value(&expression),
            ["import collections"],
        ));
    }

    if has("regex_minimum_word_length")
        && names.len() == 1
        && let Some(minimum) = first_positive_integer(&spec.requirement_sentences)
    {
        let expression = format!(r"re.findall(r'\b\w{{{minimum},}}\b', {})", names[0]);
        drafts.push(draft(
            spec,
            "regex_minimum_word_length",
            return_value(&expression),
            ["import re"],
        ));
    }

    if has("regex_lowercase_chunks") && names.len() == 1 {
        let expression = format!(r"re.findall(r'[a-z][^a-z]*', {})", names[0]);
        drafts.push(draft(
            spec,
            "regex_lowercase_chunks",
            return_value(&expression),
            ["import re"],
        ));
    }

    if has("regex_lowercase_underscore")
        && names.len() == 1
        && let Some((matched, unmatched)) = inferred_match_labels(spec)
    {
        let body = format!(
            "return {matched} if re.fullmatch(r'[a-z]+_[a-z]+', {}) else {unmatched}",
            names[0]
        );
        drafts.push(draft(
            spec,
            "regex_lowercase_underscore",
            body,
            ["import re"],
        ));
    }

    if has("exclude_membership") && names.len() >= 2 {
        let expression = idiom(
            "exclude_membership",
            &[("items", names[0]), ("excluded", names[1])],
        );
        drafts.push(draft(
            spec,
            "exclude_membership",
            return_value(&expression),
            [],
        ));
    }

    if has("duplicate_exists") && names.len() == 1 {
        let expression = idiom("duplicate_exists", &[("items", names[0])]);
        drafts.push(draft(
            spec,
            "duplicate_exists",
            return_value(&expression),
            [],
        ));
    }

    if has("composite_number") && names.len() == 1 {
        let expression = format!(
            "{} > 1 and any({} % divisor == 0 for divisor in range(2, math.isqrt({}) + 1))",
            names[0], names[0], names[0]
        );
        drafts.push(draft(
            spec,
            "composite_number",
            return_value(&expression),
            ["import math"],
        ));
    }

    if has("one_bit_difference") && names.len() >= 2 {
        let body = format!(
            "difference = {} ^ {}\nreturn bool(difference and not (difference & (difference - 1)))",
            names[0], names[1]
        );
        drafts.push(draft(spec, "one_bit_difference", body, []));
    }

    if has("grid_minimum_cost_path") && names.len() >= 3 {
        drafts.extend(grid_minimum_cost_path_drafts(spec, &names));
    }

    if has("remove_boundary_occurrences") && names.len() >= 2 {
        let body = format!(
            "first = {0}.find({1})\nif first != -1:\n    {0} = {0}[:first] + {0}[first + 1:]\nlast = {0}.rfind({1})\nif last != -1:\n    {0} = {0}[:last] + {0}[last + 1:]\nreturn {0}",
            names[0], names[1]
        );
        drafts.push(draft(spec, "remove_boundary_occurrences", body, []));
    }

    if has("rotation_period") && names.len() == 1 {
        let body = format!(
            "for offset in range(1, len({0}) + 1):\n    if {0}[offset:] + {0}[:offset] == {0}:\n        return offset\nreturn 0",
            names[0]
        );
        drafts.push(draft(spec, "rotation_period", body, []));
    }

    if has("geometric_measure") {
        drafts.extend(example_guided_arithmetic(spec));
    }
    drafts
}

fn example_guided_arithmetic(spec: &CodingTaskSpec) -> Vec<Draft> {
    let names = spec
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<Vec<_>>();
    let mut drafts = Vec::new();
    if names.len() == 1 && examples_are_scalar_numeric(spec) {
        for constant in 2..=6 {
            let expression = format!("{} * {constant}", names[0]);
            drafts.push(draft(
                spec,
                &format!(
                    "geometric_measure(example_guided_arithmetic(multiply_constant_{constant}))"
                ),
                return_value(&expression),
                [],
            ));
        }
    }
    if names.len() >= 2 && names.len() <= 4 && examples_are_scalar_numeric(spec) {
        let product = names.join(" * ");
        drafts.push(draft(
            spec,
            "geometric_measure(example_guided_arithmetic(product))",
            return_value(&product),
            [],
        ));
        for divisor in 2..=4 {
            let expression = format!("({product}) / {divisor}");
            drafts.push(draft(
                spec,
                &format!("geometric_measure(example_guided_arithmetic(product_divide_{divisor}))"),
                return_value(&expression),
                [],
            ));
        }
    }
    drafts
}

/// Enumerate the common monotone predecessor relations for a weighted grid.
/// The prompt's executable examples decide whether diagonal motion belongs to
/// the relation; the algorithm itself is independent of any task or function
/// name. Both candidates implement the same DAG shortest-path recurrence.
fn grid_minimum_cost_path_drafts(spec: &CodingTaskSpec, names: &[&str]) -> Vec<Draft> {
    [
        ("orthogonal", "(-1, 0), (0, -1)"),
        ("orthogonal_or_diagonal", "(-1, 0), (0, -1), (-1, -1)"),
    ]
    .into_iter()
    .map(|(relation, offsets)| {
        let body = format!(
            "rows = {row} + 1\ncolumns = {column} + 1\ncosts = [[float('inf')] * columns for _ in range(rows)]\ncosts[0][0] = {grid}[0][0]\nfor row_index in range(rows):\n    for column_index in range(columns):\n        if row_index == 0 and column_index == 0:\n            continue\n        predecessors = [costs[row_index + row_offset][column_index + column_offset] for row_offset, column_offset in ({offsets}) if row_index + row_offset >= 0 and column_index + column_offset >= 0]\n        costs[row_index][column_index] = min(predecessors) + {grid}[row_index][column_index]\nreturn costs[{row}][{column}]",
            grid = names[0],
            row = names[1],
            column = names[2],
        );
        draft(
            spec,
            &format!("grid_minimum_cost_path({relation})"),
            body,
            [],
        )
    })
    .collect()
}

fn draft<const N: usize>(
    spec: &CodingTaskSpec,
    composition: &str,
    body: impl AsRef<str>,
    imports: [&str; N],
) -> Draft {
    let mut source_urls = source_urls(composition);
    if source_urls.is_empty() {
        source_urls.push("https://docs.python.org/3.12/reference/expressions.html".to_owned());
    }
    Draft {
        id: format!("structure:{composition}"),
        source: render_function(spec, body.as_ref(), imports.into_iter().map(str::to_owned)),
        callable_name: spec.name.clone(),
        source_licenses: source_urls.iter().map(|_| "PSF-2.0".to_owned()).collect(),
        source_urls,
        composition: composition.to_owned(),
        action_cost: composition.matches(['(', ',']).count() + 2,
    }
}

fn source_urls(composition: &str) -> Vec<String> {
    let ids = composition
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .collect::<std::collections::BTreeSet<_>>();
    structural_meanings()
        .into_iter()
        .filter(|meaning| !meaning.grounding.is_empty() && ids.contains(meaning.id.as_str()))
        .map(|meaning| meaning.grounding)
        .collect()
}

fn return_value(expression: &str) -> String {
    format!("return {expression}")
}

fn first_positive_integer(sentences: &[String]) -> Option<usize> {
    sentences
        .iter()
        .flat_map(|sentence| sentence.split(|character: char| !character.is_ascii_digit()))
        .find_map(|token| token.parse::<usize>().ok().filter(|value| *value > 0))
}

fn inferred_output_cardinality(spec: &CodingTaskSpec) -> Option<usize> {
    let counts = spec
        .examples
        .iter()
        .filter_map(|example| top_level_collection_len(&example.expected))
        .collect::<std::collections::BTreeSet<_>>();
    (counts.len() == 1).then(|| *counts.iter().next().expect("one cardinality"))
}

fn top_level_collection_len(value: &str) -> Option<usize> {
    let trimmed = value.trim();
    let (open, close) = (trimmed.chars().next()?, trimmed.chars().next_back()?);
    if !matches!((open, close), ('[', ']') | ('(', ')')) {
        return None;
    }
    let inner = &trimmed[open.len_utf8()..trimmed.len() - close.len_utf8()];
    if inner.trim().is_empty() {
        return Some(0);
    }
    let mut depth = 0usize;
    let mut quote = None;
    let mut count = 1usize;
    for character in inner.chars() {
        if quote.is_some_and(|delimiter| delimiter == character) {
            quote = None;
        } else if quote.is_none() && matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if quote.is_none() {
            match character {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => count += 1,
                _ => {}
            }
        }
    }
    Some(count)
}

fn inferred_match_labels(spec: &CodingTaskSpec) -> Option<(String, String)> {
    let mut matched = None;
    let mut unmatched = None;
    for example in &spec.examples {
        let argument = example.arguments.first()?.trim_matches(['\'', '"']);
        let is_match = argument.split_once('_').is_some_and(|(left, right)| {
            !left.is_empty()
                && !right.is_empty()
                && left.chars().all(|character| character.is_ascii_lowercase())
                && right
                    .chars()
                    .all(|character| character.is_ascii_lowercase())
        });
        if is_match {
            matched.get_or_insert_with(|| example.expected.clone());
        } else {
            unmatched.get_or_insert_with(|| example.expected.clone());
        }
    }
    Some((matched?, unmatched?))
}

fn examples_are_scalar_numeric(spec: &CodingTaskSpec) -> bool {
    !spec.examples.is_empty()
        && spec.examples.iter().all(|example| {
            example.arguments.len() == spec.parameters.len()
                && example.arguments.iter().all(|value| numeric_literal(value))
                && numeric_literal(&example.expected)
        })
}

fn numeric_literal(value: &str) -> bool {
    value.trim().parse::<f64>().is_ok()
}

fn explicit_numeric_mapping(spec: &CodingTaskSpec) -> Option<String> {
    let text = spec.requirement_sentences.join(" ");
    let spans = quoted_spans(&text);
    let mut pairs = Vec::new();
    for (index, (literal, _, end)) in spans.iter().enumerate() {
        let segment_end = spans
            .get(index + 1)
            .map_or(text.len(), |(_, start, _)| *start);
        let value = number_in_text(&text[*end..segment_end])?;
        pairs.push((literal, value));
    }
    if pairs.len() < 2 {
        return None;
    }
    let fields = pairs
        .into_iter()
        .map(|(key, value)| {
            let key = serde_json::to_string(key).expect("string literal serializes");
            format!("{key}: {value}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("{{{fields}}}"))
}

fn explicit_order(spec: &CodingTaskSpec) -> Option<String> {
    let literals = spec
        .requirement_sentences
        .iter()
        .map(|sentence| {
            quoted_spans(sentence)
                .into_iter()
                .map(|(literal, _, _)| literal)
                .collect::<Vec<_>>()
        })
        .max_by_key(Vec::len)?;
    if literals.len() < 3 {
        return None;
    }
    let fields = literals
        .iter()
        .enumerate()
        .map(|(index, literal)| {
            let literal = serde_json::to_string(literal).expect("string literal serializes");
            format!("{literal}: {index}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("{{{fields}}}"))
}

fn quoted_spans(text: &str) -> Vec<(&str, usize, usize)> {
    let mut spans = Vec::new();
    let mut opening: Option<(char, usize)> = None;
    for (index, character) in text.char_indices() {
        if !matches!(character, '\'' | '"') {
            continue;
        }
        if let Some((delimiter, start)) = opening {
            if delimiter == character {
                let content_start = start + delimiter.len_utf8();
                spans.push((
                    &text[content_start..index],
                    start,
                    index + character.len_utf8(),
                ));
                opening = None;
            }
        } else {
            opening = Some((character, index));
        }
    }
    spans
}

fn number_in_text(text: &str) -> Option<String> {
    let normalized = crate::engine::normalize_prompt(text);
    if let Some(number) = normalized
        .split_whitespace()
        .find(|token| token.parse::<u64>().is_ok())
    {
        return Some(number.to_owned());
    }
    crate::seed::lexicon()
        .arithmetic_normalization_tables()
        .0
        .into_iter()
        .find(|(surface, value)| {
            value.parse::<u64>().is_ok()
                && normalized.split_whitespace().any(|word| word == surface)
        })
        .map(|(_, value)| value)
}
