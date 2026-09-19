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
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
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

#[test]
fn no_seed_template_is_a_whole_algorithm() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let seed = fs::read_to_string(root.join(RUNTIME_TEMPLATE_SEED))
        .unwrap_or_else(|error| panic!("{RUNTIME_TEMPLATE_SEED}: {error}"));

    let mut offenders = Vec::new();
    for line in seed.lines() {
        let trimmed = line.trim();
        let Some(body) = trimmed.strip_prefix("text ") else {
            continue;
        };
        let body = body.trim().trim_matches('"');
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

    assert!(
        offenders.is_empty(),
        "a seed template is a bootstrap fragment, never a whole algorithm; \
         these carry a loop and a return:\n{}",
        offenders.join("\n")
    );
}
