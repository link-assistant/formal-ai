use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const HUMANEVAL_SLICE: usize = 164;
const MBPP_SLICE: usize = 500;
const NON_CODING_CANARIES: &[(&str, &str)] = &[
    (
        "gsm8k/question",
        "Janet's ducks lay 16 eggs per day. She eats three for breakfast every morning and bakes muffins for her friends every day with four. She sells the remainder at the farmers' market daily for $2 per fresh duck egg. How much in dollars does she make every day at the farmers' market?",
    ),
    (
        "math/problem",
        "If x = 2 and y = 5, then what is the value of (x^4 + 2y^2) / 6?",
    ),
    (
        "object_counting/input",
        "I have a clarinet, a violin, and a flute. How many musical instruments do I have?",
    ),
    (
        "coedit/src",
        "Paraphrase this sentence: Why are you arresting me?",
    ),
    ("coedit/tgt", "Why am I being arrested?"),
];
// These upstream entry points are also ordinary language, standard-library
// names, or standard mathematics used throughout the runtime; forbidding them
// would flag prose rather than benchmark knowledge. HumanEval and MBPP name
// their tasks after everyday programming vocabulary, so the full slices sweep
// the whole codebase with them. The judgement is made here, in the open, per
// name: `find`, `count`, `match`, `first` are common words the runtime cannot
// avoid; `strlen`, `max_element`, `gcd` predate the benchmarks (libc, the C++
// standard library, school mathematics); `bitwise_xor`, `is_prime`,
// `monotonic` are standard computer-science terms, not task content.
// Compound/specific callable names coined by a benchmark remain forbidden.
const GENERIC_FUNCTION_NAMES: [&str; 50] = [
    "Extract",
    "Sort",
    "add",
    "answer",
    "bf",
    "bitwise_xor",
    "check",
    "common",
    "compare",
    "concatenate",
    "convert",
    "count",
    "digits",
    "divisor",
    "eat",
    "encode",
    "exchange",
    "f",
    "find",
    "first",
    "frequency",
    "func",
    "gcd",
    "histogram",
    "intersection",
    "is_prime",
    "longest",
    "match",
    "max",
    "max_element",
    "max_length",
    "maximum",
    "median",
    "minimum",
    "monotonic",
    "multiply",
    "overlapping",
    "perimeter",
    "power",
    "remove",
    "search",
    "sequence",
    "simplify",
    "solution",
    "solve",
    "strlen",
    "sum",
    "tri",
    "unique",
    "validate",
];

#[test]
fn upstream_slice_names_and_sentences_are_absent_from_runtime_and_seed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let cache = benchmark_cache(root);
    let humaneval = cache.join("humaneval.jsonl");
    let mbpp = cache.join("mbpp.jsonl");

    let mut forbidden_names = BTreeSet::new();
    let mut forbidden_sentences = BTreeSet::new();
    if humaneval.is_file() {
        for value in json_lines(&humaneval, HUMANEVAL_SLICE) {
            if let Some(name) = value.get("entry_point").and_then(serde_json::Value::as_str) {
                forbidden_names.insert(name.to_owned());
            }
            if let Some(prompt) = value.get("prompt").and_then(serde_json::Value::as_str) {
                for sentence in docstring_sentences(prompt) {
                    if sentence.split_whitespace().count() > 4 {
                        forbidden_sentences.insert(sentence);
                    }
                }
            }
        }
    }
    if mbpp.is_file() {
        for value in json_lines(&mbpp, MBPP_SLICE) {
            if let Some(text) = value.get("text").and_then(serde_json::Value::as_str) {
                forbidden_sentences.insert(normalize(text));
            }
            if let Some(tests) = value.get("test_list").and_then(serde_json::Value::as_array) {
                for test in tests {
                    if let Some(name) = test
                        .as_str()
                        .and_then(|line| line.trim().strip_prefix("assert "))
                        .and_then(|assertion| {
                            assertion.split_once('(').map(|(name, _)| name.trim())
                        })
                    {
                        forbidden_names.insert(name.to_owned());
                    }
                }
            }
        }
    }
    add_non_coding_sentences(&cache, &mut forbidden_sentences);
    for (_, sentence) in NON_CODING_CANARIES {
        forbidden_sentences.insert(normalize(sentence));
    }
    forbidden_names.retain(|name| !GENERIC_FUNCTION_NAMES.contains(&name.as_str()));

    let mut violations = Vec::new();
    for path in source_and_seed_files(root) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let normalized = normalize(&text);
        for name in &forbidden_names {
            if contains_identifier(&text, name) {
                violations.push(format!(
                    "{} contains upstream entry point {name}",
                    path.display()
                ));
            }
        }
        for sentence in &forbidden_sentences {
            if !sentence.is_empty() && normalized.contains(sentence) {
                violations.push(format!(
                    "{} contains upstream task sentence {sentence:?}",
                    path.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "benchmark-specific knowledge entered runtime or seed:\n{}",
        violations.join("\n")
    );
}

fn add_non_coding_sentences(cache: &Path, sentences: &mut BTreeSet<String>) {
    for (file, suite) in [
        ("gsm8k-test.jsonl", "gsm8k"),
        ("math-test.jsonl", "math"),
        ("coedit-validation.jsonl", "coedit"),
    ] {
        let path = cache.join(file);
        if path.is_file() {
            for value in json_lines(&path, usize::MAX) {
                add_non_coding_value(suite, &value, sentences);
            }
        } else {
            eprintln!(
                "no-memorization corpus gate: {suite} cache absent at {}; pinned canary remains active",
                path.display()
            );
        }
    }

    let object_counting = cache.join("object-counting-task.json");
    if object_counting.is_file() {
        let text = fs::read_to_string(&object_counting)
            .unwrap_or_else(|error| panic!("{}: {error}", object_counting.display()));
        let document: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{}: {error}", object_counting.display()));
        for example in document
            .get("examples")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            add_non_coding_value("object_counting", example, sentences);
        }
    } else {
        eprintln!(
            "no-memorization corpus gate: object_counting cache absent at {}; pinned canary remains active",
            object_counting.display()
        );
    }
}

fn add_non_coding_value(suite: &str, value: &serde_json::Value, sentences: &mut BTreeSet<String>) {
    let fields: &[&str] = match suite {
        "gsm8k" => &["question"],
        "math" => &["problem"],
        "object_counting" => &["input"],
        "coedit" => &["src", "tgt"],
        other => panic!("unknown non-coding suite {other}"),
    };
    for field in fields {
        if let Some(sentence) = value.get(field).and_then(serde_json::Value::as_str) {
            let normalized = normalize(sentence);
            // Very short answers (for example "3" or "yes") are ordinary
            // language, not identifying benchmark knowledge. Keep the same
            // five-word discriminativeness floor used for HumanEval docstring
            // sentences, while checking both CoEdIT source and target fields.
            if normalized.split_whitespace().count() > 4 {
                sentences.insert(normalized);
            }
        }
    }
}

#[test]
fn non_coding_upstream_fields_are_all_part_of_the_gate() {
    let mut sentences = BTreeSet::new();
    for (suite, record) in [
        (
            "gsm8k",
            r#"{"question":"gsm question sentinel has five words"}"#,
        ),
        (
            "math",
            r#"{"problem":"math problem sentinel has five words"}"#,
        ),
        (
            "object_counting",
            r#"{"input":"big bench input sentinel has words","target":["four","4"]}"#,
        ),
        (
            "coedit",
            r#"{"src":"coedit source sentinel has five words","tgt":"coedit target sentinel has five words"}"#,
        ),
    ] {
        let value: serde_json::Value = serde_json::from_str(record).expect("test record");
        add_non_coding_value(suite, &value, &mut sentences);
    }
    assert_eq!(
        sentences,
        [
            "big bench input sentinel has words",
            "coedit source sentinel has five words",
            "coedit target sentinel has five words",
            "gsm question sentinel has five words",
            "math problem sentinel has five words",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
}

#[test]
fn each_non_coding_suite_has_a_true_upstream_canary() {
    assert_eq!(NON_CODING_CANARIES.len(), 5);
    assert_eq!(
        NON_CODING_CANARIES
            .iter()
            .map(|(field, _)| *field)
            .collect::<BTreeSet<_>>(),
        [
            "coedit/src",
            "coedit/tgt",
            "gsm8k/question",
            "math/problem",
            "object_counting/input",
        ]
        .into_iter()
        .collect()
    );
    assert!(NON_CODING_CANARIES.iter().all(|(_, sentence)| {
        let normalized = normalize(sentence);
        !normalized.is_empty() && normalized.split_whitespace().count() >= 4
    }));
}

fn benchmark_cache(root: &Path) -> PathBuf {
    let repository_cache = root.join("target/formal-ai-benchmarks");
    if repository_cache.is_dir() {
        return repository_cache;
    }
    std::env::var_os("CARGO_TARGET_DIR").map_or(repository_cache, |target| {
        PathBuf::from(target).join("formal-ai-benchmarks")
    })
}

fn json_lines(path: &Path, full_slice: usize) -> Vec<serde_json::Value> {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(full_slice)
        .map(|line| serde_json::from_str(line).expect("upstream JSONL record"))
        .collect()
}

fn docstring_sentences(prompt: &str) -> Vec<String> {
    let Some((_, rest)) = prompt.split_once("\"\"\"") else {
        return Vec::new();
    };
    let Some((docstring, _)) = rest.split_once("\"\"\"") else {
        return Vec::new();
    };
    docstring
        .split(['.', '!', '?', '\n'])
        .map(normalize)
        .filter(|sentence| !sentence.is_empty() && !sentence.starts_with(">>>"))
        .collect()
}

fn source_and_seed_files(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.join("src"), root.join("data/seed")];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            let mut children = fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect::<Vec<_>>();
            children.sort();
            pending.extend(children);
        } else if path.is_file() {
            files.push(path);
        }
    }
    files
}

fn contains_identifier(text: &str, needle: &str) -> bool {
    text.match_indices(needle).any(|(start, found)| {
        let before = text[..start].chars().next_back();
        let after = text[start + found.len()..].chars().next();
        !before.is_some_and(identifier_character) && !after.is_some_and(identifier_character)
    })
}

const fn identifier_character(character: char) -> bool {
    character == '_' || character.is_ascii_alphanumeric()
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '_' {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// Issue #1138, plans 01, 02 L16 and 04 L5: the held-out vocabulary the new
// corpora ask about may never enter the runtime or the seed, and no seed
// template may be a whole algorithm — otherwise the corpora measure recall of
// the seed rather than retrieval from a source.

/// The words the issue #1138 corpora are built from, in five languages.
const ISSUE_1138_HELD_OUT_VOCABULARY: [&str; 9] = [
    "isogram",
    "изограмма",
    "आइसोग्राम",
    "isograma",
    "lipogram",
    "липограмма",
    "lipograma",
    "लिपोग्राम",
    "blorptide",
];

/// The runtime template seed plan 02 L15 empties of whole algorithms.
const RUNTIME_TEMPLATE_SEED: &str = "data/seed/coding-discovery-runtime.lino";

#[test]
fn issue_1138_held_out_vocabulary_is_absent_from_runtime_and_seed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let mut violations = Vec::new();
    for path in source_and_seed_files(root) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let lowered = text.to_lowercase();
        for word in ISSUE_1138_HELD_OUT_VOCABULARY {
            if lowered.contains(&word.to_lowercase()) {
                violations.push(format!("{} contains held-out word {word}", path.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "held-out vocabulary entered runtime or seed:\n{}",
        violations.join("\n")
    );
}

/// The relation seed of issue #1138 plan 04 L5.
const RELATION_SEED: &str = "data/seed/formalization-relations.lino";

/// Issue #1138, plan 04 L5 — the relation seed declares *how a gloss is read*,
/// never *what a word means*.
///
/// The whole plan turns on the difference. A file of eight relations and the
/// cues that evidence them is a reading rule: it says that "a word in which no
/// letter is repeated" names a genus and a property, without knowing anything
/// about words or letters. A file that also declared what `isogram` means would
/// be the closed fairy-tale lexicon again under a new name, and the plan would
/// have moved the ceiling rather than removed it.
///
/// Two things make that testable, and one does not. The schema is closed, so a
/// concept cannot be smuggled in as a record kind; and the relation vocabulary
/// is fixed at the eight the plan declares, so a ninth relation named after a
/// subject matter fails here rather than in review. What no test can decide is
/// whether an individual cue is *really* a connective — that is a judgement,
/// and it is made at the file, in the open, rather than asserted here in a form
/// that would pass whatever was written.
#[test]
fn the_relation_seed_declares_how_a_gloss_is_read_and_never_what_a_word_means() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let text = fs::read_to_string(root.join(RELATION_SEED))
        .unwrap_or_else(|error| panic!("{RELATION_SEED}: {error}"));

    let relations = [
        "is_a",
        "has_property",
        "part_of",
        "requires",
        "produces",
        "precedes",
        "excludes",
        "measured_in",
    ];
    let schema = [
        "relations",
        "determiners",
        "kind",
        "inverse",
        "grounding",
        "lexeme",
        "surface",
        "text",
    ];

    let mut declared: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let head = trimmed.split_whitespace().next().unwrap_or_default();
        if schema.contains(&head) {
            continue;
        }
        assert!(
            relations.contains(&head),
            "{RELATION_SEED} declares `{head}`, which is neither the relation schema \
             nor one of the eight declared relations: a subject matter may not enter \
             this file under any record kind"
        );
        declared.push(head.to_owned());
    }
    declared.sort();
    let mut expected: Vec<String> = relations.iter().map(|name| (*name).to_owned()).collect();
    expected.sort();
    assert_eq!(
        declared, expected,
        "the relation vocabulary is the eight the plan declares, no more and no fewer"
    );

    for line in text.lines() {
        let trimmed = line.trim();
        let Some(surface) = trimmed.strip_prefix("text ") else {
            continue;
        };
        let surface = surface.trim().trim_matches('"');
        assert!(
            surface.split_whitespace().count() <= 4,
            "a cue is a fragment, not a definition: {surface:?} in {RELATION_SEED}"
        );
    }
}

/// The seed surfaces the nine code-task handlers of issue #1177 render their
/// answers from: response templates, explanation meanings, review rules,
/// manual pages and recognition cues.
const CODE_TASK_OUTPUT_SURFACES: [&str; 5] = [
    "data/seed/multilingual-responses-code-tasks.lino",
    "data/seed/meanings-code-structure-explanations.lino",
    "data/seed/code-review-rules.lino",
    "data/seed/manual-pages.lino",
    "data/seed/code-task-cues.lino",
];

/// The nine code-task handlers of issue #1177.
const CODE_TASK_HANDLERS: [&str; 9] = [
    "src/solver_handlers/code_debugging.rs",
    "src/solver_handlers/regex_synthesis.rs",
    "src/solver_handlers/sql_synthesis.rs",
    "src/solver_handlers/shell_command_compose.rs",
    "src/solver_handlers/code_explanation.rs",
    "src/solver_handlers/code_review.rs",
    "src/solver_handlers/test_generation.rs",
    "src/solver_handlers/code_refactoring.rs",
    "src/solver_handlers/format_conversion.rs",
];

/// Every quoted value of `seed` (`key "value"` lines) that is a whole
/// algorithm: more than one statement, at least one loop and a return.
fn whole_algorithm_values(seed: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    for line in seed.lines() {
        let Some((_, rest)) = line.trim().split_once(' ') else {
            continue;
        };
        let rest = rest.trim();
        let Some(body) = rest
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        else {
            continue;
        };
        let statements = body
            .split("\\n")
            .map(str::trim)
            .filter(|statement| !statement.is_empty())
            .collect::<Vec<_>>();
        let loops = statements
            .iter()
            .filter(|statement| statement.starts_with("for ") || statement.starts_with("while "))
            .count();
        let returns = statements
            .iter()
            .filter(|statement| statement.starts_with("return"))
            .count();
        if statements.len() > 1 && loops > 0 && returns > 0 {
            offenders.push(body.to_owned());
        }
    }
    offenders
}

fn repository_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
}

#[test]
fn no_seed_template_is_a_whole_algorithm() {
    let root = repository_root();
    let seed = fs::read_to_string(root.join(RUNTIME_TEMPLATE_SEED))
        .unwrap_or_else(|error| panic!("{RUNTIME_TEMPLATE_SEED}: {error}"));
    let offenders = whole_algorithm_values(&seed);
    assert!(
        offenders.is_empty(),
        "a seed template is a bootstrap fragment, never a whole algorithm; \
         these carry a loop and a return:\n{}",
        offenders.join("\n")
    );
}

/// Issue #1177 R11: the code-task handlers' output surfaces are held to the
/// same rule as the runtime template seed, so no explanation, review, fix or
/// generated test can be a memorized whole program.
#[test]
fn code_task_output_surfaces_carry_no_whole_algorithm() {
    let root = repository_root();
    let mut offenders = Vec::new();
    for surface in CODE_TASK_OUTPUT_SURFACES {
        let seed = fs::read_to_string(root.join(surface))
            .unwrap_or_else(|error| panic!("{surface}: {error}"));
        offenders.extend(
            whole_algorithm_values(&seed)
                .into_iter()
                .map(|body| format!("{surface}: {body}")),
        );
    }
    assert!(
        offenders.is_empty(),
        "a code-task answer surface is composed per request, never a stored \
         whole program; these carry a loop and a return:\n{}",
        offenders.join("\n")
    );
}

/// Issue #1177 R11: the verbatim Rosetta Code rendering belongs to the
/// "example" intent alone; none of the nine code-task handlers reaches it.
#[test]
fn code_task_handlers_never_recite_rosetta_examples() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();
    for handler in CODE_TASK_HANDLERS {
        let source = fs::read_to_string(crate_root.join(handler))
            .unwrap_or_else(|error| panic!("{handler}: {error}"));
        for needle in ["rosetta_request", "render_example"] {
            if source.contains(needle) {
                offenders.push(format!("{handler} reaches {needle}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "the code-task handlers compose from seed meanings and must not recite \
         a fetched example:\n{}",
        offenders.join("\n")
    );
}

/// Issue #1186 R9: the formalization grammar and its runtime twins.
const FORMALIZATION_GRAMMAR_SEED: &str = "data/seed/formal-targets.lino";
const FORMALIZATION_RUNTIME: [&str; 5] = [
    "rust/src/solver_handlers/formalization_task.rs",
    "rust/src/solver_handlers/formalization_task_render.rs",
    "rust/src/solver_handlers/formalization_task_targets.rs",
    "js/worker/formal_ai_worker_formalization_request.js",
    "js/worker/formal_ai_worker_formalization_targets.js",
];
const FORMALIZATION_PROBE_LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];

/// Every `(expected_predicates, expected_fol)` pair of the issue #1186 probe
/// set, across its four languages.
fn formalization_probes(root: &Path) -> Vec<(Vec<String>, String)> {
    let mut probes = Vec::new();
    for language in FORMALIZATION_PROBE_LANGUAGES {
        let path = format!("data/benchmarks/formalization/{language}.lino");
        let text =
            fs::read_to_string(root.join(&path)).unwrap_or_else(|error| panic!("{path}: {error}"));
        let mut predicates = Vec::new();
        for line in text.lines().map(str::trim) {
            let Some((key, value)) = line.split_once(' ') else {
                continue;
            };
            let value = value.trim().trim_matches('"');
            match key {
                "expected_predicates" => {
                    predicates = value.split('|').map(str::to_lowercase).collect();
                }
                "expected_fol" => probes.push((std::mem::take(&mut predicates), value.to_owned())),
                _ => {}
            }
        }
    }
    probes
}

/// Lowercased word tokens of every quoted value in a seed (comment lines
/// skipped), plus each whole value for scripts written without spaces.
fn quoted_seed_words(seed: &str) -> (BTreeSet<String>, Vec<String>) {
    let mut words = BTreeSet::new();
    let mut values = Vec::new();
    for line in seed.lines().map(str::trim) {
        if line.starts_with('#') {
            continue;
        }
        let Some((_, rest)) = line.split_once(' ') else {
            continue;
        };
        let Some(value) = rest
            .trim()
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        else {
            continue;
        };
        let lowered = value.to_lowercase();
        words.extend(
            lowered
                .split(|character: char| {
                    character.is_whitespace()
                        || (character.is_ascii_punctuation() && character != '_')
                })
                .filter(|word| !word.is_empty())
                .map(str::to_owned),
        );
        values.push(lowered);
    }
    (words, values)
}

/// True for a word in a script written without spaces (CJK ideographs).
fn is_unspaced_script(word: &str) -> bool {
    word.chars()
        .all(|character| ('\u{4e00}'..='\u{9fff}').contains(&character))
}

/// Issue #1186 R9: no formalization is memorized per input. The grammar seed
/// names no predicate of any probe sentence (predicate symbols must come
/// from the sentence's own words), and neither the seed nor the Rust and
/// JavaScript handlers carry any probe's whole first-order clause.
#[test]
fn formalization_grammar_and_runtime_memorize_no_probe_clause() {
    let root = repository_root();
    let probes = formalization_probes(root);
    assert!(
        probes.len() >= 40,
        "the four-language probe set should carry ten probes per language, found {}",
        probes.len()
    );
    let seed = fs::read_to_string(root.join(FORMALIZATION_GRAMMAR_SEED))
        .unwrap_or_else(|error| panic!("{FORMALIZATION_GRAMMAR_SEED}: {error}"));
    let (seed_words, seed_values) = quoted_seed_words(&seed);
    let surfaces: Vec<(&str, String)> = std::iter::once(FORMALIZATION_GRAMMAR_SEED)
        .chain(FORMALIZATION_RUNTIME)
        .map(|surface| {
            let text = fs::read_to_string(root.join(surface))
                .unwrap_or_else(|error| panic!("{surface}: {error}"));
            (surface, text)
        })
        .collect();
    let mut offenders = Vec::new();
    for (predicates, fol) in &probes {
        for predicate in predicates {
            let named = seed_words.contains(predicate)
                || (is_unspaced_script(predicate)
                    && seed_values
                        .iter()
                        .any(|value| value.contains(predicate.as_str())));
            if named {
                offenders.push(format!(
                    "{FORMALIZATION_GRAMMAR_SEED} names the probe predicate {predicate}"
                ));
            }
        }
        for (surface, text) in &surfaces {
            if text.contains(fol.as_str()) {
                offenders.push(format!("{surface} carries the whole clause {fol}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "a formalization is derived from the sentence, never stored per probe:\n{}",
        offenders.join("\n")
    );
}

/// Verbatim Hello World program literals still stored under `data/` (issue
/// #1165 R8). Each is the output literal of one per-language Hello World
/// template in `data/seed/hello-world-programs.lino`, the seed bundle the
/// Rust catalog tests hold equal to the catalog tables. The JavaScript
/// catalog (`data/meta/agentic-coding-catalog.lino`) used to carry a second
/// copy of every program; it now names its templates and reads their text
/// from that bundle, so each program is stored once. The ceiling only goes
/// down: a verified procedure that reproduces a language's program replaces
/// its stored template, and the ceiling drops with it until it reaches zero.
const HELLO_WORLD_PROGRAM_LITERALS_MAX: usize = 14;

/// The quote spellings a stored program may wrap its literal in: an escaped
/// double quote, a single quote, the `\x27` escape of a single quote, and a
/// plain double quote inside a larger value.
const PROGRAM_QUOTES: [&str; 4] = ["\\\"", "'", "\\x27", "\""];

/// Hello World output literals written as a *code* string, in any quote
/// spelling a stored program uses. A plain Links Notation or JSON value that
/// is the literal alone (`output "Hello, world!"`, `"expected": "Hello,
/// world!"`) is an expected output, not a program.
fn hello_world_program_literals(text: &str) -> usize {
    const LINE_FEED: &str = "\\\\n";
    let mut count = 0;
    for line in text.to_lowercase().lines() {
        for quote in PROGRAM_QUOTES {
            for (start, _) in line.match_indices(quote) {
                if quote == "\"" && line[..start].ends_with('\\') {
                    continue;
                }
                let rest = &line[start + quote.len()..];
                let Some(rest) = rest.strip_prefix("hello") else {
                    continue;
                };
                let rest = rest.strip_prefix(',').unwrap_or(rest);
                let Some(rest) = rest.strip_prefix(" world") else {
                    continue;
                };
                let rest = rest.strip_prefix('!').unwrap_or(rest);
                let rest = rest.strip_prefix(LINE_FEED).unwrap_or(rest);
                if !rest.starts_with(quote) {
                    continue;
                }
                if quote == "\"" && is_expectation_value(line[..start].trim()) {
                    continue;
                }
                count += 1;
            }
        }
    }
    count
}

/// Whether the text before a double-quoted literal makes it a field's whole
/// value: nothing, an opening bracket, a single field name, or a JSON key or
/// list separator.
fn is_expectation_value(before: &str) -> bool {
    matches!(before, "" | "[" | "(")
        || before.ends_with([':', ','])
        || before
            .chars()
            .all(|character| character.is_alphanumeric() || matches!(character, '_' | '-'))
}

/// The counter sees a stored program in every quote spelling the data files
/// use, and leaves an expected-output value alone.
#[test]
fn the_hello_world_counter_reads_every_program_quote_spelling() {
    let programs = concat!(
        "  code `fn main() {\\n    println!(\\\"Hello, world!\\\");\\n}`\n",
        "  code 'print(\"Hello, world!\")'\n",
        "  code 'puts \"Hello, world!\"'\n",
        "  code \"$this->line(\\x27Hello, world!\\x27);\"\n",
        "  code \"echo 'Hello, world!';\"\n",
    );
    assert_eq!(hello_world_program_literals(programs), 5);
    let expectations = concat!(
        "  output \"Hello, world!\"\n",
        "  {\"expectedOutput\": \"Hello, world!\"}\n",
        "  [\"Hello, world!\", \"Hello\"]\n",
    );
    assert_eq!(hello_world_program_literals(expectations), 0);
}

#[test]
fn data_stores_no_more_verbatim_hello_world_programs_than_the_ratchet() {
    let root = repository_root();
    let mut stack = vec![root.join("data")];
    let mut offenders = Vec::new();
    let mut total = 0;
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory).expect("data/ is readable") {
            let path = entry.expect("a data/ entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let found = hello_world_program_literals(&text);
            if found > 0 {
                total += found;
                offenders.push(format!("{} ({found})", path.display()));
            }
        }
    }
    assert!(
        total <= HELLO_WORLD_PROGRAM_LITERALS_MAX,
        "data/ stores {total} verbatim Hello World program literals, above the ratchet of \
         {HELLO_WORLD_PROGRAM_LITERALS_MAX}: {offenders:?}"
    );
}
