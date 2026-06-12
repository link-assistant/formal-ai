//! Benchmark-family text and code edit examples for issue #408.
//!
//! The prompts are self-authored minimal examples derived from public text-edit
//! and code-edit benchmark task families. They pin the deterministic edit
//! operations the local solver can support without inventing neural rewrites.

use formal_ai::{ExecutionSurface, SolverConfig, UniversalSolver};
use lino_objects_codec::format::parse_indented;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const TEXT_EDIT_PROFILE_FIXTURE: &str = "data/benchmarks/text-manipulation-suite.lino";

fn text_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

#[derive(Debug)]
struct Case {
    source: &'static str,
    family: &'static str,
    prompt: &'static str,
    answer: &'static str,
    rule: &'static str,
}

#[derive(Debug)]
struct LinoRecord {
    kind: String,
    id: String,
    fields: Vec<(String, String)>,
}

#[derive(Debug)]
struct TextEditSource {
    id: String,
    title: String,
    group: String,
    domain: String,
    primary_url: String,
    local_profile: String,
}

#[derive(Debug)]
struct TextEditSuite {
    sources: BTreeMap<String, TextEditSource>,
    minimum_pass_count: usize,
    sources_required: usize,
    variations_per_source: usize,
    ratchet_policy: String,
    upstream_payload_policy: String,
}

#[derive(Debug)]
struct ProfileCase {
    source: String,
    prompt: String,
    answer: String,
    rule: &'static str,
}

#[test]
fn benchmark_family_matrix_covers_text_and_code_edit_variations() {
    let cases = [
        Case {
            source: "CoEdIT",
            family: "case_conversion",
            prompt: "Uppercase this text: \"release ready\"",
            answer: "RELEASE READY",
            rule: "rule_uppercase",
        },
        Case {
            source: "CoEdIT",
            family: "case_conversion",
            prompt: "Lowercase this text: \"MIXED Case\"",
            answer: "mixed case",
            rule: "rule_lowercase",
        },
        Case {
            source: "CoEdIT",
            family: "lexical_substitution",
            prompt: "Replace \"colour\" with \"color\": \"colour profile\"",
            answer: "color profile",
            rule: "rule_replace_text",
        },
        Case {
            source: "CoEdIT",
            family: "deletion",
            prompt: "Remove \"very \" from \"very clear\"",
            answer: "clear",
            rule: "rule_remove_text",
        },
        Case {
            source: "CoEdIT",
            family: "whitespace_cleanup",
            prompt: "Normalize whitespace: \"keep   spacing\nreadable\"",
            answer: "keep spacing readable",
            rule: "rule_normalize_whitespace",
        },
        Case {
            source: "CoEdIT",
            family: "whitespace_cleanup",
            prompt: "Trim whitespace: \"  clean  \"",
            answer: "clean",
            rule: "rule_trim_whitespace",
        },
        Case {
            source: "EditEval",
            family: "reordering",
            prompt: "Reverse words: \"first second third\"",
            answer: "third second first",
            rule: "rule_reverse_words",
        },
        Case {
            source: "EditEval",
            family: "deduplication",
            prompt: "Deduplicate lines: \"note\nnote\nship\"",
            answer: "note\nship",
            rule: "rule_deduplicate_lines",
        },
        Case {
            source: "EditEval",
            family: "sorting",
            prompt: "Sort lines: \"zeta\nalpha\"",
            answer: "alpha\nzeta",
            rule: "rule_sort_lines",
        },
        Case {
            source: "EditEval",
            family: "counting",
            prompt: "Count unique words: \"blue blue green\"",
            answer: "2",
            rule: "rule_count_unique_words",
        },
        Case {
            source: "EditEval",
            family: "extraction",
            prompt: "Extract email addresses: \"Mail ada@lab.org now\"",
            answer: "ada@lab.org",
            rule: "rule_extract_email",
        },
        Case {
            source: "EditEval",
            family: "counting",
            prompt: "Count occurrences of \"bug\": \"bug fix bug\"",
            answer: "2",
            rule: "rule_count_occurrences",
        },
        Case {
            source: "InstrEditBench",
            family: "latex_edit",
            prompt: r#"Replace "\alpha" with "\beta": "\alpha + \alpha""#,
            answer: r"\beta + \beta",
            rule: "rule_replace_text",
        },
        Case {
            source: "InstrEditBench",
            family: "comment_deletion",
            prompt: "Remove \"%TODO\n\" from \"%TODO\nx = 1\"",
            answer: "x = 1",
            rule: "rule_remove_text",
        },
        Case {
            source: "InstrEditBench",
            family: "markdown_edit",
            prompt: "Prepend \"# \" to \"Heading\"",
            answer: "# Heading",
            rule: "rule_prepend_text",
        },
        Case {
            source: "InstrEditBench",
            family: "token_insertion",
            prompt: "Append \" END\" to \"BEGIN\"",
            answer: "BEGIN END",
            rule: "rule_append_text",
        },
        Case {
            source: "InstrEditBench",
            family: "sql_cleanup",
            prompt: "Normalize whitespace: \"SELECT   *\nFROM   users\"",
            answer: "SELECT * FROM users",
            rule: "rule_normalize_whitespace",
        },
        Case {
            source: "InstrEditBench",
            family: "import_sort",
            prompt: "Sort lines: \"import z\nimport a\"",
            answer: "import a\nimport z",
            rule: "rule_sort_lines",
        },
        Case {
            source: "CodeEditorBench",
            family: "operator_fix",
            prompt: "Replace \"==\" with \"===\": \"if (a == b) return true;\"",
            answer: "if (a === b) return true;",
            rule: "rule_replace_text",
        },
        Case {
            source: "CodeEditorBench",
            family: "debug_print_removal",
            prompt: "Remove \"console.log(x);\" from \"function f(){console.log(x);return x;}\"",
            answer: "function f(){return x;}",
            rule: "rule_remove_text",
        },
        Case {
            source: "CodeEditorBench",
            family: "requirement_switch",
            prompt: "Append \" return total;\" to \"let total = 0;\"",
            answer: "let total = 0; return total;",
            rule: "rule_append_text",
        },
        Case {
            source: "CodeEditorBench",
            family: "runtime_header",
            prompt: "Prepend \"use strict; \" to \"const x = 1;\"",
            answer: "use strict; const x = 1;",
            rule: "rule_prepend_text",
        },
        Case {
            source: "CodeEditorBench",
            family: "formatting",
            prompt: "Normalize whitespace: \"fn  main()  {   }\"",
            answer: "fn main() { }",
            rule: "rule_normalize_whitespace",
        },
        Case {
            source: "CodeEditorBench",
            family: "code_counting",
            prompt: "Count occurrences of \"return\": \"return a; return b;\"",
            answer: "2",
            rule: "rule_count_occurrences",
        },
        Case {
            source: "CanItEdit",
            family: "identifier_rename",
            prompt: "Replace \"foo\" with \"bar\": \"fn foo() { foo(); }\"",
            answer: "fn bar() { bar(); }",
            rule: "rule_replace_text",
        },
        Case {
            source: "CanItEdit",
            family: "stub_removal",
            prompt: "Remove \"pass\" from \"def f(): pass\"",
            answer: "def f(): ",
            rule: "rule_remove_text",
        },
        Case {
            source: "CanItEdit",
            family: "comment_prefix",
            prompt: "Prepend \"// \" to \"fix later\"",
            answer: "// fix later",
            rule: "rule_prepend_text",
        },
        Case {
            source: "CanItEdit",
            family: "syntax_suffix",
            prompt: "Append \";\" to \"let x = 1\"",
            answer: "let x = 1;",
            rule: "rule_append_text",
        },
        Case {
            source: "CanItEdit",
            family: "symbol_normalization",
            prompt: "Lowercase this text: \"HTTP_STATUS\"",
            answer: "http_status",
            rule: "rule_lowercase",
        },
        Case {
            source: "CanItEdit",
            family: "marker_emphasis",
            prompt: "Uppercase this text: \"todo\"",
            answer: "TODO",
            rule: "rule_uppercase",
        },
        Case {
            source: "EDIT-Bench",
            family: "api_migration",
            prompt: "Replace \"old_api\" with \"new_api\": \"old_api(x); old_api(y);\"",
            answer: "new_api(x); new_api(y);",
            rule: "rule_replace_text",
        },
        Case {
            source: "EDIT-Bench",
            family: "unused_import_removal",
            prompt: "Remove \"unused, \" from \"use crate::{unused, kept};\"",
            answer: "use crate::{kept};",
            rule: "rule_remove_text",
        },
        Case {
            source: "EDIT-Bench",
            family: "statement_sort",
            prompt: "Sort lines: \"z();\na();\"",
            answer: "a();\nz();",
            rule: "rule_sort_lines",
        },
        Case {
            source: "EDIT-Bench",
            family: "duplicate_import_removal",
            prompt: "Deduplicate lines: \"use a;\nuse a;\nuse b;\"",
            answer: "use a;\nuse b;",
            rule: "rule_deduplicate_lines",
        },
        Case {
            source: "EDIT-Bench",
            family: "line_trim",
            prompt: "Trim whitespace: \"  let kept = true;  \"",
            answer: "let kept = true;",
            rule: "rule_trim_whitespace",
        },
        Case {
            source: "EDIT-Bench",
            family: "markdown_formatting",
            prompt: "Normalize whitespace: \"| a |   b |\n| c |   d |\"",
            answer: "| a | b | | c | d |",
            rule: "rule_normalize_whitespace",
        },
        Case {
            source: "HumanEvalFix",
            family: "bug_fix",
            prompt:
                "Replace \"return a - b\" with \"return a + b\": \"def add(a,b): return a - b\"",
            answer: "def add(a,b): return a + b",
            rule: "rule_replace_text",
        },
        Case {
            source: "HumanEvalFix",
            family: "off_by_one_fix",
            prompt: "Replace \"range(n)\" with \"range(n + 1)\": \"for i in range(n): print(i)\"",
            answer: "for i in range(n + 1): print(i)",
            rule: "rule_replace_text",
        },
        Case {
            source: "HumanEvalFix",
            family: "test_count",
            prompt: "Count occurrences of \"assert\": \"assert f(1)\nassert f(2)\"",
            answer: "2",
            rule: "rule_count_occurrences",
        },
        Case {
            source: "HumanEvalFix",
            family: "negation_removal",
            prompt: "Remove \"not \" from \"if not valid: return False\"",
            answer: "if valid: return False",
            rule: "rule_remove_text",
        },
        Case {
            source: "HumanEvalFix",
            family: "test_addition",
            prompt: "Append \" assert add(1, 2) == 3\" to \"tests:\"",
            answer: "tests: assert add(1, 2) == 3",
            rule: "rule_append_text",
        },
        Case {
            source: "HumanEvalFix",
            family: "fix_note_prefix",
            prompt: "Prepend \"# fix: \" to \"off by one\"",
            answer: "# fix: off by one",
            rule: "rule_prepend_text",
        },
        Case {
            source: "SWE-bench",
            family: "configuration_patch",
            prompt: "Replace \"timeout=1\" with \"timeout=5\": \"client(timeout=1)\"",
            answer: "client(timeout=5)",
            rule: "rule_replace_text",
        },
        Case {
            source: "SWE-bench",
            family: "test_marker_removal",
            prompt: "Remove \"xfail \" from \"xfail test_api\"",
            answer: "test_api",
            rule: "rule_remove_text",
        },
        Case {
            source: "SWE-bench",
            family: "issue_text_cleanup",
            prompt: "Normalize whitespace: \"Issue   title\nneeds   patch\"",
            answer: "Issue title needs patch",
            rule: "rule_normalize_whitespace",
        },
        Case {
            source: "SWE-bench",
            family: "trace_deduplication",
            prompt: "Deduplicate lines: \"Traceback\nTraceback\nValueError\"",
            answer: "Traceback\nValueError",
            rule: "rule_deduplicate_lines",
        },
        Case {
            source: "SWE-bench",
            family: "import_ordering",
            prompt: "Sort lines: \"import sys\nimport os\"",
            answer: "import os\nimport sys",
            rule: "rule_sort_lines",
        },
        Case {
            source: "SWE-bench",
            family: "log_count",
            prompt: "Count occurrences of \"ERROR\": \"ERROR open\nINFO ok\nERROR close\"",
            answer: "2",
            rule: "rule_count_occurrences",
        },
        Case {
            source: "issue_408_regression",
            family: "hello_world_replacement",
            prompt: "Replace \"Hello World\" with \"Bye world\": \"Hello, world!\"",
            answer: "Bye world!",
            rule: "rule_replace_text",
        },
        Case {
            source: "issue_408_regression",
            family: "native_replacement",
            prompt: "Замени \"Hello World\" на \"Bye world\": \"Hello, world!\"",
            answer: "Bye world!",
            rule: "rule_replace_text",
        },
    ];

    assert!(
        cases.len() >= 50,
        "issue #408 follow-up requested a 5x/10x wider matrix; got {}",
        cases.len()
    );
    assert!(
        cases
            .iter()
            .filter(|case| case.source != "issue_408_regression")
            .count()
            >= 30,
        "at least 30 examples must come from benchmark-derived task families"
    );

    let expected_sources = BTreeSet::from([
        "CoEdIT",
        "EditEval",
        "InstrEditBench",
        "CodeEditorBench",
        "CanItEdit",
        "EDIT-Bench",
        "HumanEvalFix",
        "SWE-bench",
    ]);
    let covered_sources = cases
        .iter()
        .filter(|case| case.source != "issue_408_regression")
        .map(|case| case.source)
        .collect::<BTreeSet<_>>();
    assert_eq!(covered_sources, expected_sources);

    let solver = text_solver();
    for case in cases {
        let response = solver.solve(case.prompt);
        assert_eq!(
            response.intent, "text_manipulation",
            "{} {} should route to text manipulation, got {} with answer {}",
            case.source, case.family, response.intent, response.answer
        );
        assert_eq!(
            response.answer, case.answer,
            "{} {} should produce the exact documented answer",
            case.source, case.family
        );
        assert!(
            response.links_notation.contains(case.rule),
            "{} {} should record {} in {}",
            case.source,
            case.family,
            case.rule,
            response.links_notation
        );
    }
}

#[test]
fn issue_408_text_code_edit_profile_passes_local_ratchet() {
    let suite = load_text_edit_suite();

    assert_eq!(
        suite.sources.len(),
        suite.sources_required,
        "the issue #408 profile must keep every researched benchmark source executable"
    );
    assert_eq!(
        suite.variations_per_source, 10,
        "review feedback requested 10 variations for each benchmark source"
    );
    assert!(
        suite.ratchet_policy.contains("minimum_pass_count"),
        "ratchet policy should name the pass floor: {}",
        suite.ratchet_policy
    );
    assert!(
        suite.upstream_payload_policy.contains("not vendored"),
        "manifest should keep external payload provenance explicit: {}",
        suite.upstream_payload_policy
    );

    let groups = suite
        .sources
        .values()
        .map(|source| source.group.as_str())
        .collect::<BTreeSet<_>>();
    assert!(
        groups.contains("referenced_edit_benchmark")
            && groups.contains("additional_llm_benchmark"),
        "profile should include both referenced edit benchmarks and additional LLM benchmarks: {groups:?}"
    );

    let cases = suite
        .sources
        .values()
        .flat_map(profile_cases_for_source)
        .collect::<Vec<_>>();
    assert_eq!(
        cases.len(),
        suite.sources.len() * suite.variations_per_source,
        "every benchmark source must expand to exactly the recorded local variation count"
    );
    assert!(
        cases.len() >= suite.minimum_pass_count,
        "case count {} must satisfy minimum_pass_count {}",
        cases.len(),
        suite.minimum_pass_count
    );

    let solver = text_solver();
    let mut passed = 0usize;
    let mut failures = Vec::new();
    for case in &cases {
        let response = solver.solve(&case.prompt);
        let rule_matches = response.links_notation.contains(case.rule);
        if response.intent == "text_manipulation" && response.answer == case.answer && rule_matches
        {
            passed += 1;
        } else {
            failures.push(format!(
                "{} failed prompt {:?}: intent={} answer={:?} expected={:?} rule={} links={}",
                case.source,
                case.prompt,
                response.intent,
                response.answer,
                case.answer,
                case.rule,
                response.links_notation
            ));
        }
    }

    let failed = cases.len() - passed;
    println!(
        "issue #408 text/code edit profile: passed={passed} failed={failed} total={} minimum_pass_count={}",
        cases.len(),
        suite.minimum_pass_count
    );
    assert!(
        failures.is_empty(),
        "all local issue #408 profile cases should pass; failures:\n{}",
        failures.join("\n")
    );
    assert!(
        passed >= suite.minimum_pass_count,
        "profile pass-count floor dropped: passed={passed} minimum_pass_count={}",
        suite.minimum_pass_count
    );
}

fn load_text_edit_suite() -> TextEditSuite {
    let text =
        fs::read_to_string(repo_root().join(TEXT_EDIT_PROFILE_FIXTURE)).expect("benchmark fixture");
    validate_lino_syntax(&text);
    parse_text_edit_suite(&text)
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn validate_lino_syntax(text: &str) {
    for record in split_records(text) {
        parse_indented(&record).expect("benchmark record should be valid Links Notation");
    }
}

fn parse_text_edit_suite(text: &str) -> TextEditSuite {
    let mut sources = BTreeMap::new();
    let mut minimum_pass_count = 0usize;
    let mut sources_required = 0usize;
    let mut variations_per_source = 0usize;
    let mut ratchet_policy = String::new();
    let mut upstream_payload_policy = String::new();

    for record in parse_records(text) {
        match record.kind.as_str() {
            "text_manipulation_suite" => {
                minimum_pass_count =
                    parse_usize_field(&record.fields, "minimum_pass_count").unwrap_or(0);
                sources_required =
                    parse_usize_field(&record.fields, "sources_required").unwrap_or(0);
                variations_per_source =
                    parse_usize_field(&record.fields, "variations_per_source").unwrap_or(0);
                ratchet_policy = field_value(&record.fields, "ratchet_policy");
                upstream_payload_policy = field_value(&record.fields, "upstream_payload_policy");
            }
            "text_manipulation_source" => {
                let source = TextEditSource {
                    id: record.id,
                    title: field_value(&record.fields, "title"),
                    group: field_value(&record.fields, "group"),
                    domain: field_value(&record.fields, "domain"),
                    primary_url: field_value(&record.fields, "primary_url"),
                    local_profile: field_value(&record.fields, "local_profile"),
                };
                sources.insert(source.id.clone(), source);
            }
            _ => {}
        }
    }

    TextEditSuite {
        sources,
        minimum_pass_count,
        sources_required,
        variations_per_source,
        ratchet_policy,
        upstream_payload_policy,
    }
}

fn parse_records(text: &str) -> Vec<LinoRecord> {
    split_records(text)
        .into_iter()
        .map(|record| parse_record(&record))
        .collect()
}

fn split_records(text: &str) -> Vec<String> {
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(current.join("\n"));
            current.clear();
        }
        current.push(line.to_owned());
    }
    if !current.is_empty() {
        records.push(current.join("\n"));
    }
    records
}

fn parse_record(block: &str) -> LinoRecord {
    let mut lines = block.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().expect("record header");
    let fields = lines
        .map(parse_lino_line)
        .filter(|(name, _)| !name.is_empty())
        .collect::<Vec<_>>();
    let kind = field_value(&fields, "record_type");
    let id = field_value(&fields, "id");
    assert!(!kind.is_empty(), "record `{header}` is missing record_type");
    assert!(!id.is_empty(), "record `{header}` is missing id");
    LinoRecord { kind, id, fields }
}

fn parse_lino_line(line: &str) -> (String, String) {
    let content = line.trim();
    if let Some((name, raw_value)) = content.split_once(' ') {
        (name.to_owned(), unescape_quoted(raw_value.trim()))
    } else {
        (content.to_owned(), String::new())
    }
}

fn unescape_quoted(raw: &str) -> String {
    let inner = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw);
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn field_value(fields: &[(String, String)], name: &str) -> String {
    fields
        .iter()
        .find_map(|(field_name, value)| (field_name == name).then(|| value.clone()))
        .unwrap_or_default()
}

fn parse_usize_field(fields: &[(String, String)], name: &str) -> Option<usize> {
    let raw = field_value(fields, name);
    (!raw.is_empty()).then(|| {
        raw.parse::<usize>()
            .unwrap_or_else(|err| panic!("invalid {name} `{raw}`: {err}"))
    })
}

fn profile_cases_for_source(source: &TextEditSource) -> Vec<ProfileCase> {
    assert!(
        !source.title.is_empty()
            && !source.domain.is_empty()
            && source.primary_url.starts_with("https://")
            && !source.local_profile.is_empty(),
        "source metadata should be reviewable: {source:?}"
    );

    let words = source.id.split('_').collect::<Vec<_>>();
    let plain = format!("local {}", words.join(" "));
    let title = format!("Local {}", titleize_words(&words));
    let snake = format!("local_{}", source.id);
    let kebab = format!("local-{}", source.id.replace('_', "-"));
    let camel = format!("local{}", pascalize_words(&words));
    let pascal = format!("Local{}", pascalize_words(&words));
    let line_one = source.id.clone();
    let line_two = String::from("profile");

    vec![
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Title case this text: \"{plain}\""),
            answer: title,
            rule: "rule_title_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Snake case this text: \"{plain}\""),
            answer: snake,
            rule: "rule_snake_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Kebab case this text: \"{plain}\""),
            answer: kebab,
            rule: "rule_kebab_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Camel case this text: \"{plain}\""),
            answer: camel,
            rule: "rule_camel_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Pascal case this text: \"{plain}\""),
            answer: pascal,
            rule: "rule_pascal_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Strip empty lines: \"{line_one}\n\n{line_two}\""),
            answer: format!("{line_one}\n{line_two}"),
            rule: "rule_strip_empty_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Join lines: \"{line_one}\n{line_two}\""),
            answer: format!("{line_one} {line_two}"),
            rule: "rule_join_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Number lines: \"{line_one}\n{line_two}\""),
            answer: format!("1. {line_one}\n2. {line_two}"),
            rule: "rule_number_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Indent lines: \"{line_one}\n{line_two}\""),
            answer: format!("    {line_one}\n    {line_two}"),
            rule: "rule_indent_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Outdent lines: \"    {line_one}\n\t{line_two}\""),
            answer: format!("{line_one}\n{line_two}"),
            rule: "rule_outdent_lines",
        },
    ]
}

fn titleize_words(words: &[&str]) -> String {
    words
        .iter()
        .map(|word| capitalize_ascii_word(word))
        .collect::<Vec<_>>()
        .join(" ")
}

fn pascalize_words(words: &[&str]) -> String {
    words
        .iter()
        .map(|word| capitalize_ascii_word(word))
        .collect::<Vec<_>>()
        .join("")
}

fn capitalize_ascii_word(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = first.to_ascii_uppercase().to_string();
    out.push_str(chars.as_str());
    out
}
