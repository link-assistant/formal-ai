//! Benchmark-family text and code edit examples for issue #408.
//!
//! The prompts are self-authored minimal examples derived from public text-edit
//! and code-edit benchmark task families. They pin the deterministic edit
//! operations the local solver can support without inventing neural rewrites.

use formal_ai::{ExecutionSurface, SolverConfig, UniversalSolver};
use std::collections::{BTreeMap, BTreeSet};

mod profile;
use profile::{load_text_edit_suite, profile_cases_for_source};

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
            source: "CoEdIT",
            family: "sentence_case",
            prompt: "Sentence case this text: \"hELLO EDIT BENCH\"",
            answer: "Hello edit bench",
            rule: "rule_sentence_case",
        },
        Case {
            source: "EditEval",
            family: "word_count",
            prompt: "Count words in this text: \"red blue red\"",
            answer: "3",
            rule: "rule_count_words",
        },
        Case {
            source: "InstrEditBench",
            family: "url_extraction",
            prompt: "Extract URLs from this text: \"See https://example.test/path. now\"",
            answer: "https://example.test/path",
            rule: "rule_extract_url",
        },
        Case {
            source: "CodeEditorBench",
            family: "comment_lines",
            prompt: "Comment lines: \"let x = 1;\nreturn x;\"",
            answer: "// let x = 1;\n// return x;",
            rule: "rule_comment_lines",
        },
        Case {
            source: "CanItEdit",
            family: "uncomment_lines",
            prompt: "Uncomment lines: \"// let x = 1;\n# return x;\"",
            answer: "let x = 1;\nreturn x;",
            rule: "rule_uncomment_lines",
        },
        Case {
            source: "EDIT-Bench",
            family: "reverse_lines",
            prompt: "Reverse lines: \"first\nsecond\nthird\"",
            answer: "third\nsecond\nfirst",
            rule: "rule_reverse_lines",
        },
        Case {
            source: "HumanEvalFix",
            family: "number_extraction",
            prompt: "Extract numbers from this text: \"expected 3 got -2.5\"",
            answer: "3\n-2.5",
            rule: "rule_extract_number",
        },
        Case {
            source: "SWE-bench",
            family: "punctuation_cleanup",
            prompt: "Remove punctuation: \"bug, fix! now.\"",
            answer: "bug fix now",
            rule: "rule_remove_punctuation",
        },
        Case {
            source: "EditEval",
            family: "sort_words",
            prompt: "Sort words: \"zeta alpha beta\"",
            answer: "alpha beta zeta",
            rule: "rule_sort_words",
        },
        Case {
            source: "SWE-bench",
            family: "line_count",
            prompt: "Count lines: \"one\ntwo\nthree\"",
            answer: "3",
            rule: "rule_count_lines",
        },
        Case {
            source: "CoEdIT",
            family: "character_count",
            prompt: "Count characters: \"abcd\"",
            answer: "4",
            rule: "rule_count_characters",
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
        cases.len() >= 60,
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
        suite.variations_per_source, 30,
        "review feedback requested at least 10 variations for each benchmark source; this profile now records 30"
    );
    assert_eq!(
        suite.local_ten_percent_floor_per_source, 3,
        "30 local tests per source should make the 10% per-source floor explicit"
    );
    assert_eq!(
        suite.minimum_pass_count_per_source, suite.variations_per_source,
        "this ratchet should require every committed local source case to pass"
    );
    assert!(
        suite.ratchet_policy.contains("per-source"),
        "ratchet policy should name the pass floor: {}",
        suite.ratchet_policy
    );
    assert!(
        suite.upstream_payload_policy.contains("30/30 per-source"),
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
    let referenced_count = suite
        .sources
        .values()
        .filter(|source| source.group == "referenced_edit_benchmark")
        .count();
    let additional_count = suite
        .sources
        .values()
        .filter(|source| source.group == "additional_llm_benchmark")
        .count();
    assert_eq!(
        referenced_count, 8,
        "PR #416 should keep the 8 referenced edit benchmark sources"
    );
    assert_eq!(
        additional_count, suite.additional_sources_required,
        "review feedback requested 40 additional popular/current benchmark sources"
    );
    for id in [
        "coedit",
        "editeval",
        "instreditbench",
        "codeeditorbench",
        "canitedit",
        "editbench",
        "humanevalfix",
        "swebench",
        "humaneval",
        "mmlu",
        "ifeval",
        "gpqa",
        "musr",
        "livecodebench",
        "bfcl",
        "simpleqa",
        "mmmu",
        "ruler",
        "longbench",
        "alpacaeval",
        "mt_bench",
        "arena_hard",
        "wildbench",
        "math_five_hundred",
        "aime",
        "mgsm",
        "humaneval_plus",
        "mbpp_plus",
        "multipl_e",
        "apps",
        "ds_one_thousand",
    ] {
        assert!(
            suite.sources.contains_key(id),
            "expanded issue #408 source audit should include `{id}`"
        );
    }

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
    let mut per_source_total = BTreeMap::<String, usize>::new();
    let mut per_source_passed = BTreeMap::<String, usize>::new();
    for case in &cases {
        *per_source_total.entry(case.source.clone()).or_default() += 1;
        let response = solver.solve(&case.prompt);
        let rule_matches = response.links_notation.contains(case.rule);
        if response.intent == "text_manipulation" && response.answer == case.answer && rule_matches
        {
            passed += 1;
            *per_source_passed.entry(case.source.clone()).or_default() += 1;
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
    for source_id in suite.sources.keys() {
        let total = per_source_total.get(source_id).copied().unwrap_or_default();
        let source_passed = per_source_passed
            .get(source_id)
            .copied()
            .unwrap_or_default();
        assert_eq!(
            total, suite.variations_per_source,
            "{source_id} should have exactly {} local benchmark tests",
            suite.variations_per_source
        );
        assert!(
            source_passed >= suite.local_ten_percent_floor_per_source,
            "{source_id} passed {source_passed}/{total}, below the explicit 10% floor {}",
            suite.local_ten_percent_floor_per_source
        );
        assert!(
            source_passed >= suite.minimum_pass_count_per_source,
            "{source_id} passed {source_passed}/{total}, below the per-source ratchet {}",
            suite.minimum_pass_count_per_source
        );
    }
    assert!(
        passed >= suite.minimum_pass_count,
        "profile pass-count floor dropped: passed={passed} minimum_pass_count={}",
        suite.minimum_pass_count
    );
}
