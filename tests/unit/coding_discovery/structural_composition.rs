use formal_ai::coding_task_spec::{ArtifactShape, CodingTaskSpec, Example, Parameter};
use formal_ai::composition::{VerifiedDraft, compose};
use formal_ai::concept_discovery::{
    ConceptMap, ConceptRequirement, StructuralMeaning, structural_meanings,
};
use formal_ai::needs::NeedState;

fn task(
    name: &str,
    parameters: &[&str],
    requirement: &str,
    examples: &[(&[&str], &str)],
) -> CodingTaskSpec {
    CodingTaskSpec {
        language: "python".to_owned(),
        artifact_shape: ArtifactShape::Function,
        name: name.to_owned(),
        parameters: parameters
            .iter()
            .map(|name| Parameter {
                name: (*name).to_owned(),
                annotation: None,
            })
            .collect(),
        return_annotation: None,
        imports: Vec::new(),
        requirement_sentences: vec![requirement.to_owned()],
        examples: examples
            .iter()
            .map(|(arguments, expected)| Example {
                arguments: arguments.iter().map(|value| (*value).to_owned()).collect(),
                expected: (*expected).to_owned(),
            })
            .collect(),
        expected_stdout: None,
        prose_language: "en".to_owned(),
    }
}

fn concepts(ids: &[&str]) -> ConceptMap {
    let registry = structural_meanings();
    let structures = ids
        .iter()
        .map(|id| {
            registry
                .iter()
                .find(|meaning| meaning.id == *id)
                .cloned()
                .unwrap_or_else(|| StructuralMeaning {
                    id: (*id).to_owned(),
                    idiom: String::new(),
                    grounding: String::new(),
                })
        })
        .collect();
    ConceptMap {
        needs: vec![ConceptRequirement::new(
            "held-out structural requirement",
            "en",
            NeedState::Satisfied,
            structures,
            Vec::new(),
        )],
        evidence: Vec::new(),
    }
}

fn selected(
    ids: &[&str],
    name: &str,
    parameters: &[&str],
    requirement: &str,
    examples: &[(&[&str], &str)],
) -> VerifiedDraft {
    let outcome = compose(
        &task(name, parameters, requirement, examples),
        &concepts(ids),
    );
    let selected = outcome
        .selected
        .unwrap_or_else(|| panic!("no draft passed for {name}: {:#?}", outcome.attempts));
    assert!(
        selected
            .source_urls
            .iter()
            .all(|url| url.starts_with("https://")),
        "{selected:#?}"
    );
    selected
}

#[test]
fn held_out_arithmetic_scan_and_geometry_schemas_execute() {
    selected(
        &["arithmetic_fractional_part"],
        "remainder_below_one",
        &["quantity"],
        "Return the fractional remainder.",
        &[(&["8.625"], "0.625")],
    );
    selected(
        &["reduce_mean", "absolute_deviation"],
        "average_distance_from_center",
        &["samples"],
        "Average the absolute distance from the mean.",
        &[(&["[1, 2, 6]"], "2.0")],
    );
    selected(
        &["running_prefix", "prefix_negative", "quantifier_any"],
        "ever_crosses_floor",
        &["changes"],
        "Whether any running total falls below zero.",
        &[(&["[3, -1, -5]"], "True"), (&["[2, 1, 4]"], "False")],
    );
    let geometry = selected(
        &["geometric_measure"],
        "half_product_measure",
        &["base", "height"],
        "Calculate a geometric area from its dimensions.",
        &[(&["6", "5"], "15.0"), (&["4", "3"], "6.0")],
    );
    assert!(geometry.composition.contains("geometric_measure"));
}

#[test]
fn held_out_sequence_and_symmetry_schemas_execute() {
    selected(
        &["interpose_each"],
        "weave_marker",
        &["values", "marker"],
        "Place the marker between consecutive values.",
        &[(&["[3, 5, 8]", "0"], "[3, 0, 5, 0, 8]")],
    );
    selected(
        &["balanced_delimiter_groups"],
        "partition_balanced_runs",
        &["symbols"],
        "Separate adjacent balanced groups.",
        &[(&["'(())()'"], "['(())', '()']")],
    );
    selected(
        &["group_max_nesting"],
        "depth_per_run",
        &["runs"],
        "Return maximum nesting for each group.",
        &[(&["'(()) ()'"], "[2, 1]")],
    );
    selected(
        &["prefix_enumeration"],
        "growing_leading_slices",
        &["word"],
        "List all prefixes from shortest to longest.",
        &[(&["'wxyz'"], "['w', 'wx', 'wxy', 'wxyz']")],
    );
    selected(
        &["zero_based_range", "stringify_each", "join_with_space"],
        "render_counter_line",
        &["limit"],
        "Create a space-delimited string starting at zero.",
        &[(&["3"], "'0 1 2 3'")],
    );
    selected(
        &["palindrome_extension"],
        "finish_mirror_word",
        &["text"],
        "Append the minimum suffix needed for the shortest palindrome.",
        &[(&["'race'"], "'racecar'")],
    );
    selected(
        &["aligned_binary_xor"],
        "combine_bit_text",
        &["left_bits", "right_bits"],
        "Perform xor across aligned binary strings.",
        &[(&["'1010'", "'0110'"], "'1100'")],
    );
    selected(
        &["stable_longest"],
        "first_widest_label",
        &["labels"],
        "Return the longest one, preserving the first tie.",
        &[(&["['aa', 'bbbb', 'cccc']"], "'bbbb'")],
    );
}

#[test]
fn held_out_relation_ordering_and_pattern_schemas_execute() {
    selected(
        &["explicit_value_mapping"],
        "decode_badges",
        &["badges"],
        "'amber' corresponds to 9 and 'blue' corresponds to 4.",
        &[(&["'blue amber blue'"], "[4, 9, 4]")],
    );
    selected(
        &["explicit_ordering", "sort_ascending"],
        "canonicalize_levels",
        &["levels"],
        "The valid choices are 'low', 'medium', 'high' in the given order.",
        &[(&["'high low medium'"], "'low medium high'")],
    );
    selected(
        &["bounded_top"],
        "greatest_sample",
        &["samples", "amount"],
        "Select the requested number of largest items.",
        &[(&["[7, 1, 9, 3]", "2"], "[9, 7]")],
    );
    selected(
        &["bounded_bottom"],
        "least_sample",
        &["samples", "amount"],
        "Select the requested number of smallest items.",
        &[(&["[7, 1, 9, 3]", "3"], "[1, 3, 7]")],
    );
    selected(
        &["power_square_each"],
        "second_powers",
        &["values"],
        "Square each member.",
        &[(&["[-3, 0, 4]"], "[9, 0, 16]")],
    );
    selected(
        &["keyed_sum_order"],
        "rank_rows_by_total",
        &["rows"],
        "Sort rows in ascending order by row sum.",
        &[(&["[[4, 1], [-2, 1], [3, 0]]"], "[[-2, 1], [3, 0], [4, 1]]")],
    );
    selected(
        &["frequency_rank"],
        "two_frequent_tokens",
        &["tokens"],
        "Return the most common tokens.",
        &[(&["['z', 'a', 'z', 'b', 'a', 'z']"], "[('z', 3), ('a', 2)]")],
    );
    selected(
        &["regex_minimum_word_length"],
        "extract_substantial_words",
        &["sentence"],
        "Find words at least 5 characters long.",
        &[(&["'tiny broad longer'"], "['broad', 'longer']")],
    );
    selected(
        &["regex_lowercase_chunks"],
        "cut_before_lowercase",
        &["text"],
        "Split at lowercase letters.",
        &[(&["'XyZa'"], "['yZ', 'a']")],
    );
    selected(
        &["regex_lowercase_underscore"],
        "classify_snake_pair",
        &["text"],
        "Recognize two lowercase words joined by underscore.",
        &[
            (&["'teal_blue'"], "'accepted'"),
            (&["'Teal_blue'"], "'rejected'"),
        ],
    );
}

#[test]
fn held_out_filter_predicate_and_string_window_schemas_execute() {
    selected(
        &["exclude_membership"],
        "drop_forbidden_symbols",
        &["text", "forbidden"],
        "Exclude characters appearing in the second collection.",
        &[(&["'stargazer'", "'az'"], "'strger'")],
    );
    selected(
        &["duplicate_exists"],
        "has_repeated_member",
        &["values"],
        "Whether the collection contains duplicates.",
        &[(&["[4, 1, 4]"], "True"), (&["[4, 1, 9]"], "False")],
    );
    selected(
        &["composite_number"],
        "has_nontrivial_factor",
        &["candidate"],
        "Whether the candidate is a composite number.",
        &[(&["49"], "True"), (&["47"], "False")],
    );
    selected(
        &["one_bit_difference"],
        "single_toggle_apart",
        &["left", "right"],
        "Whether two integers have one bit difference.",
        &[(&["8", "12"], "True"), (&["8", "14"], "False")],
    );
    selected(
        &["remove_boundary_occurrences"],
        "trim_outer_matches",
        &["text", "symbol"],
        "Remove the first and last occurrence of the symbol.",
        &[(&["'abracadabra'", "'a'"], "'bracadabr'")],
    );
    selected(
        &["rotation_period"],
        "cyclic_repeat_span",
        &["text"],
        "Return the smallest rotation period.",
        &[(&["'xyxyxy'"], "2")],
    );
}
