#!/usr/bin/env rust-script
//! One generated LLM-task-parity surface (issue #1171, E136).
//!
//! `docs/llm-task-parity.md` compares every class of task people bring to
//! LLMs with Formal AI's formal version of the answer: what LLMs do well,
//! the public benchmark that measures it, the white-box formal route, the
//! latest measured verdict, the evidence pointer and the tracking issue.
//! The document is generated, never hand-edited, exactly like
//! `docs/status.md` (modelled on `scripts/render-status.rs`).
//!
//! Input, in order of preference:
//!   1. `data/meta/llm-task-classes.lino` — the class registry the issue's
//!      R1 defines, once it lands;
//!   2. the embedded seed registry below, which carries the same schema so
//!      the generator runs before the registry file exists. The registry's
//!      rows were hand-derived from the issue body (the 0.347.0 probe
//!      batches and the class → tracking-issue table) plus the handlers and
//!      parity cases landed on this branch; because this drafting round ran
//!      under a strict no-build constraint, the committed document was
//!      hand-rendered to this script's exact output shape rather than by
//!      executing it — the first `--write` run must reproduce those bytes.
//!
//! Usage:
//!   rust-script scripts/generate-llm-task-parity.rs --write
//!   rust-script scripts/generate-llm-task-parity.rs --check
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::fs;
use std::path::{Path, PathBuf};

const DOCUMENT: &str = "docs/llm-task-parity.md";
const REGISTRY: &str = "data/meta/llm-task-classes.lino";
const ISSUE_URL_PREFIX: &str = "https://github.com/link-assistant/formal-ai/issues/";

/// One LLM task class, parsed from either input.
struct Class {
    id: String,
    name: String,
    llm_strength: String,
    benchmarks: Vec<(String, Option<String>)>,
    formal_version: String,
    status: String,
    tracking_issue: String,
    evidence: String,
}

// registry-fallback:begin
const FALLBACK_REGISTRY: &str = r#"llm_task_classes
  class factual_qa
    name "Factual question answering"
    llm_strength "Recall of widely stated facts from pretrained knowledge; weak on long-tail and recent facts"
    benchmark "Natural Questions" benchmark_url "https://ai.google.com/research/NaturalQuestions"
    benchmark "TriviaQA" benchmark_url "https://nlp.cs.washington.edu/triviaqa/"
    formal_version "Formalize the question to a subject and a property, read the live statement from the grounded store, and answer with its cited references"
    status "0.347.0: wrong (answered the US capital for an Australia question); branch: word-boundary subject match landed, unmeasured until the release probe runs"
    tracking_issue 1172
    evidence "rust/tests/unit/issue_1172_factual_qa_subject_match.rs"
  class explanation
    name "Explanation (ELI5)"
    llm_strength "Fluent multi-sentence explanations assembled from pretrained knowledge, unsourced"
    benchmark "ELI5" benchmark_url "https://facebookresearch.github.io/ELI5/"
    formal_version "Decompose the question into obligations, ground each part in fetched sources, and answer part by part with the fetch trace"
    status "0.347.0: canned search paragraph, no answer; branch: the fallback now executes the search it describes, unmeasured until the release probe runs"
    tracking_issue 1173
    evidence "rust/tests/unit/issue_1173_fallback_executes_search.rs"
  class definition
    name "Definition lookup"
    llm_strength "Definitions of common words from pretrained glosses"
    benchmark "WordNet" benchmark_url "https://wordnet.princeton.edu/"
    benchmark "Wiktionary" benchmark_url "https://www.wiktionary.org/"
    formal_version "Read the grounded Wiktionary or WordNet entry for the lemma and answer with its recorded senses and license"
    status "0.347.0: canned search paragraph, no answer; branch: the fallback now executes the search it describes, unmeasured until the release probe runs"
    tracking_issue 1172
    evidence "rust/tests/unit/issue_1173_fallback_executes_search.rs"
  class comparison
    name "Comparison (A versus B)"
    llm_strength "Side-by-side tradeoff prose from pretrained knowledge"
    formal_version "Formalize both subjects, ground each side's properties, and answer over the aligned property pairs"
    status "0.347.0: only one side's encyclopedic definition answered, the second side and the comparison missing"
    tracking_issue 1172
    evidence "https://github.com/link-assistant/formal-ai/issues/1172"
  class document_qa
    name "Question answering over a given document"
    llm_strength "Strong extractive spans from provided context"
    benchmark "SQuAD 2.0" benchmark_url "https://rajpurkar.github.io/SQuAD-explorer/"
    formal_version "Index the given text into statements and answer from the matching statement with its span"
    status "0.347.0: canned search paragraph instead of reading the given text"
    tracking_issue 1172
    evidence "rust/tests/unit/issue_1101_documentation_question_parity.rs"
  class summarization
    name "Summarization"
    llm_strength "Abstractive compression in the style of news leads"
    benchmark "CNN/DailyMail" benchmark_url "https://github.com/abisee/cnn-dailymail"
    benchmark "XSum" benchmark_url "https://github.com/EdinburghNLP/XSum"
    formal_version "Bound the source into statements, select by topic under the length constraint, and emit the selection with its trace"
    status "0.347.0: echoed the input with a task-recorded note; branch: summarization handler landed with parity case e1174, unmeasured until the release probe runs"
    tracking_issue 1174
    evidence "data/parity/cross-runtime-synthesis.json"
  class translation
    name "Translation"
    llm_strength "High-quality translation between high-resource language pairs"
    benchmark "WMT" benchmark_url "https://www2.statmt.org/"
    benchmark "FLORES-200" benchmark_url "https://github.com/facebookresearch/flores"
    formal_version "Parse the source sentence, project the parse through the language pair's rules, and render with a back-check"
    status "0.347.0: could not identify a source phrase; branch: free-sentence translation pipeline landed, unmeasured until the release probe runs"
    tracking_issue 1174
    evidence "rust/tests/unit/issue_1174_text_transform.rs"
  class rewriting
    name "Rewriting and style transfer"
    llm_strength "Register and tone rewriting of fluent prose"
    benchmark "GYAFC" benchmark_url "https://github.com/raosudha89/GYAFC"
    formal_version "Decompose the request into text operations, apply each as a traced substitution, and verify the stated constraints"
    status "0.347.0: canned search paragraph, no answer; branch: text-rewrite handler landed, unmeasured until the release probe runs"
    tracking_issue 1174
    evidence "rust/tests/unit/issue_1174_text_transform.rs"
  class grammar_correction
    name "Grammar correction"
    llm_strength "Fluent minimal edits of near-native text"
    benchmark "BEA-2019" benchmark_url "https://www.cl.cam.ac.uk/research/nl/bea2019/"
    formal_version "Parse the sentence, detect agreement violations from the rule seed, and propose the minimal repairs"
    status "0.347.0: canned search paragraph, no answer; branch: grammar-correction rules landed with the text-transform family, unmeasured until the release probe runs"
    tracking_issue 1174
    evidence "rust/tests/unit/issue_1174_text_transform.rs"
  class text_writing
    name "Text writing (email, commit message)"
    llm_strength "Conventional short-form drafting in the requested genre"
    benchmark "CommitBench"
    formal_version "Compose from the genre's styleguide seed under the stated constraints, showing which convention each line follows"
    status "0.347.0: canned search paragraph, no answer; branch: genre writing landed with the text-transform family, unmeasured until the release probe runs"
    tracking_issue 1174
    evidence "rust/tests/unit/issue_1174_text_transform.rs"
  class classification
    name "Classification and sentiment"
    llm_strength "Accurate label prediction for common facets"
    benchmark "SST-2" benchmark_url "https://nlp.stanford.edu/sentiment/treebank.html"
    formal_version "Formalize the statement, score it against the labeled seed facets, and answer with the matched label and margin"
    status "0.347.0: canned search paragraph, no answer"
    tracking_issue 1173
    evidence "rust/tests/unit/issue_1173_fallback_executes_search.rs"
  class extraction
    name "Extraction to JSON"
    llm_strength "Span marking into requested schemas"
    benchmark "CoNLL-2003" benchmark_url "https://www.clips.uantwerpen.be/conll2003/ner/"
    formal_version "Parse the text, mark spans against the schema's entity meanings, and emit the JSON with the span trace"
    status "0.347.0: canned search paragraph, no answer"
    tracking_issue 1173
    evidence "rust/tests/unit/issue_1173_fallback_executes_search.rs"
  class math_word
    name "Math word problems"
    llm_strength "Chain-of-thought arithmetic over stated quantities"
    benchmark "GSM8K" benchmark_url "https://github.com/openai/grade-school-math"
    benchmark "MATH" benchmark_url "https://github.com/hendrycks/math"
    formal_version "Formalize the quantities and relations, compute by reduction, and show each arithmetic step"
    status "0.347.0: canned search paragraph, no answer; branch: word-problem route landed with parity case e34_numeric_word_problem_renumbered, unmeasured until the release probe runs"
    tracking_issue 1176
    evidence "data/parity/cross-runtime-synthesis.json"
  class data_analysis
    name "Data analysis over given numbers"
    llm_strength "Summary statistics and trend prose over pasted tables"
    formal_version "Parse the values, compute the requested reductions as defined-by edges, and show each step"
    status "0.347.0: canned search paragraph, no answer; branch: statistics handler landed with parity case e1176, unmeasured until the release probe runs"
    tracking_issue 1176
    evidence "data/parity/cross-runtime-synthesis.json"
  class units_and_dates
    name "Unit conversion and date arithmetic"
    llm_strength "Everyday conversions and calendar arithmetic"
    formal_version "Convert through the unit meaning's definition edges; walk the calendar by the parsed offset"
    status "0.347.0: wrong (100 days after Monday answered Tuesday, correct Wednesday) and canned for miles to kilometers; branch: unit-conversion and calendar-offset handlers landed, unmeasured until the release probe runs"
    tracking_issue 1176
    evidence "rust/tests/unit/issue_1176_quantities_dates.rs"
  class code_generation
    name "Code generation"
    llm_strength "Passing solutions for self-contained function problems"
    benchmark "HumanEval" benchmark_url "https://github.com/openai/human-eval"
    benchmark "MBPP" benchmark_url "https://huggingface.co/datasets/mbpp"
    formal_version "Decompose the request to a plan, compose from the catalog, verify by execution in the bounded workspace, and emit only passing programs"
    status "0.347.0: honest refusal, no synthesis route reached the probe; open (route breadth tracked by issues 1165 and 1167)"
    tracking_issue 1177
    evidence "https://github.com/link-assistant/formal-ai/issues/1177"
  class code_explanation
    name "Code explanation"
    llm_strength "Fluent line-by-line and intent-level summaries"
    benchmark "CodeXGLUE code-to-text" benchmark_url "https://microsoft.github.io/CodeXGLUE/"
    formal_version "Parse the code to its structure, name each construct from the code-structure meanings, and build the explanation from the parse"
    status "0.347.0: canned search paragraph, no answer; branch: explanation handler landed, unmeasured until the release probe runs"
    tracking_issue 1177
    evidence "rust/tests/unit/issue_1177_code_task_handlers.rs"
  class debugging
    name "Debugging"
    llm_strength "Plausible defect hypotheses from pattern memory"
    benchmark "Defects4J" benchmark_url "https://github.com/rjust/defects4j"
    benchmark "BugsInPy" benchmark_url "https://soarsmu.github.io/BugsInPy/"
    formal_version "Parse the snippet, locate the construct whose behavior contradicts the stated intent, and name the defect structurally without executing"
    status "0.347.0: misrouted to a terminal-command prompt; branch: debugging handler landed behind the routing guards, unmeasured until the release probe runs"
    tracking_issue 1177
    evidence "rust/tests/unit/issue_1177_code_task_handlers.rs"
  class code_review
    name "Code review"
    llm_strength "Broad commentary in the style of human reviews"
    formal_version "Check the parse against the review-rules seed and report each violated rule with its location"
    status "0.347.0: canned search paragraph, no answer; branch: review handler landed, unmeasured until the release probe runs"
    tracking_issue 1177
    evidence "rust/tests/unit/issue_1177_code_task_handlers.rs"
  class refactoring
    name "Refactoring"
    llm_strength "Idiomatic rewrites of small snippets"
    formal_version "Propose the structural rewrite as an edit program over the parse, never a blind textual replace"
    status "0.347.0: canned search paragraph, no answer; branch: refactoring handler landed, unmeasured until the release probe runs"
    tracking_issue 1177
    evidence "rust/tests/unit/issue_1177_code_task_handlers.rs"
  class test_generation
    name "Test generation"
    llm_strength "Thorough-looking test suites that may not assert the contract"
    formal_version "Derive the test set from the parsed contract shapes and emit it with the coverage note"
    status "0.347.0: refused claiming the language was missing although given; branch: test-generation handler landed, unmeasured until the release probe runs"
    tracking_issue 1177
    evidence "rust/tests/unit/issue_1177_code_task_handlers.rs"
  class regex
    name "Regular expression synthesis"
    llm_strength "Compact patterns for common match shapes"
    benchmark "NL-RX"
    formal_version "Compose the expression from the match-shape meanings and state each part's role"
    status "0.347.0: misrouted to an extension project plan by the surface word extension; branch: regex handler landed behind the routing guards, unmeasured until the release probe runs"
    tracking_issue 1177
    evidence "rust/tests/unit/issue_1177_code_task_handlers.rs"
  class sql
    name "SQL synthesis"
    llm_strength "Correct queries for common relational shapes"
    benchmark "Spider" benchmark_url "https://yale-lily.github.io/spider"
    benchmark "BIRD" benchmark_url "https://bird-bench.github.io/"
    formal_version "Map the question to the schema's relations and compose the query as a relational plan"
    status "0.347.0: canned search paragraph, no answer; branch: SQL handler landed, unmeasured until the release probe runs"
    tracking_issue 1177
    evidence "rust/tests/unit/issue_1177_code_task_handlers.rs"
  class shell_command
    name "Shell command composition"
    llm_strength "Recall of common command incantations"
    benchmark "NL2Bash" benchmark_url "https://github.com/TellinaDev/nl2bash"
    formal_version "Map the intent to the command grammar's flags and compose with the safety check"
    status "0.347.0: only an offer to run a command in Agent mode; branch: shell-compose handler landed, unmeasured until the release probe runs"
    tracking_issue 1177
    evidence "rust/tests/unit/issue_1177_code_task_handlers.rs"
  class format_conversion
    name "Format conversion (JSON, YAML, CSV)"
    llm_strength "Lossless re-serialization of small documents"
    formal_version "Parse the source format to the meta structure and re-render in the target format"
    status "0.347.0: canned search paragraph, no answer; branch: format-conversion handler landed, unmeasured until the release probe runs"
    tracking_issue 1177
    evidence "rust/tests/unit/issue_1177_code_task_handlers.rs"
  class creative_writing
    name "Creative writing"
    llm_strength "Fluent constrained prose and verse"
    benchmark "WritingPrompts"
    formal_version "Compose under explicit constraints (line count, rhyme) from the genre styleguides, verifying each constraint"
    status "0.347.0: canned search paragraph, no answer; open"
    tracking_issue 1178
    evidence "https://github.com/link-assistant/formal-ai/issues/1178"
  class brainstorming
    name "Brainstorming"
    llm_strength "Wide association lists in the requested frame"
    formal_version "Enumerate combinations over the seed's idea space under the stated constraints"
    status "0.347.0: canned search paragraph, no answer; open"
    tracking_issue 1178
    evidence "https://github.com/link-assistant/formal-ai/issues/1178"
  class planning
    name "Planning"
    llm_strength "Coherent multi-step plans with implicit feasibility"
    benchmark "TravelPlanner" benchmark_url "https://github.com/OSU-NLP-Group/TravelPlanner"
    formal_version "Search the plan space against the stated constraints and emit the feasible plan with the constraint checks"
    status "0.347.0: misrouted to a terminal-command prompt; open"
    tracking_issue 1178
    evidence "https://github.com/link-assistant/formal-ai/issues/1178"
  class advice
    name "Advice with evidence"
    llm_strength "Confident recommendations with unsourced confidence"
    benchmark "TruthfulQA" benchmark_url "https://github.com/sylinrl/TruthfulQA"
    benchmark "HealthBench"
    formal_version "Ground each recommendation in cited sources with the uncertainty stated"
    status "0.347.0: canned search paragraph, no answer; open"
    tracking_issue 1178
    evidence "https://github.com/link-assistant/formal-ai/issues/1178"
  class fact_checking
    name "Fact checking"
    llm_strength "Claim verification against pretrained knowledge, unsourced"
    benchmark "FEVER" benchmark_url "https://fever.ai/"
    formal_version "Formalize the claim, retrieve the grounded statements, and answer supported, refuted or unproven with the contradiction trace"
    status "0.347.0: canned search paragraph, no answer; open (statement-audit false positives tracked by the issue)"
    tracking_issue 1179
    evidence "rust/tests/unit/issue_845_fact_checking.rs"
  class formalization
    name "Formalization"
    llm_strength "Translation of informal statements into proof-assistant syntax"
    benchmark "miniF2F" benchmark_url "https://github.com/openai/miniF2F"
    benchmark "ProofNet"
    formal_version "Render the informal statement in the target formal system through the relative meta logic and round-trip check"
    status "0.347.0: canned search paragraph, no answer; open"
    tracking_issue 1186
    evidence "https://github.com/link-assistant/formal-ai/issues/1186"
  class repository_qa
    name "Repository question answering"
    llm_strength "Recall of popular-library APIs, weak on private code"
    benchmark "RepoQA"
    formal_version "Index the repository's own sources and answer from the matched definition with its file location"
    status "0.347.0: wrong subject (a generic self-description instead of the asked function); open"
    tracking_issue 1180
    evidence "https://github.com/link-assistant/formal-ai/issues/1180"
  class multi_turn_conversation
    name "Multi-turn conversation"
    llm_strength "Context retention across turns with persona consistency"
    benchmark "MT-Bench"
    formal_version "Carry the dialog state as links and resolve each turn against it"
    status "not probed on the one-shot CLI; the probe set is to be added by this issue"
    tracking_issue 1171
    evidence "https://github.com/link-assistant/formal-ai/issues/1171"
  class agentic_coding
    name "Multi-step agentic coding"
    llm_strength "Autonomous repository editing with test-driven verification"
    benchmark "SWE-bench" benchmark_url "https://www.swebench.com/"
    formal_version "Plan over the repository world model, execute each step in the bounded workspace, and repair on diagnostics"
    status "not probed on the one-shot CLI; the Hive Mind ladder is tracked by issues 1162 and 1170"
    tracking_issue 1162
    evidence "https://github.com/link-assistant/formal-ai/issues/1170"
  class ocr_image_description
    name "OCR and image description"
    llm_strength "Dense captioning from pretrained vision encoders"
    formal_version "Not on the formal surface yet; this registry row fixes the target before implementation"
    status "not yet probed; no formal route"
    tracking_issue 1171
    evidence "https://github.com/link-assistant/formal-ai/issues/1171"
  class long_document_qa
    name "Long-document question answering"
    llm_strength "Needle-in-haystack retrieval over long context windows"
    benchmark "NarrativeQA" benchmark_url "https://github.com/deepmind/narrativeqa"
    formal_version "Segment the document into statements and answer from the matching segment with the position trace"
    status "not yet probed; the document-QA route is the nearest ancestor"
    tracking_issue 1171
    evidence "https://github.com/link-assistant/formal-ai/issues/1171"
"#;
// registry-fallback:end

/// Splits a field line into its `key value` pairs. A quoted value may contain
/// spaces and may be followed by further pairs on the same line
/// (`benchmark "Natural Questions" benchmark_url "https://…"`), so each value
/// is taken as either the span between one pair of quotes or the next bare
/// word.
fn field_pairs(line: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut rest = line.trim();
    while let Some((key, after)) = rest.split_once(' ') {
        let after = after.trim_start();
        if let Some(quoted) = after.strip_prefix('"')
            && let Some(end) = quoted.find('"')
        {
            pairs.push((key.to_owned(), quoted[..end].to_owned()));
            rest = quoted[end + 1..].trim_start();
        } else {
            let (value, tail) = after
                .split_once(' ')
                .unwrap_or((after, ""));
            pairs.push((key.to_owned(), value.to_owned()));
            rest = tail.trim_start();
        }
    }
    pairs
}

/// Parses the registry schema: a `llm_task_classes` root, `class <id>`
/// children at two spaces, fields at four. `benchmark` and `benchmark_url`
/// share one line, so each field line is split into key/value pairs while
/// text remains.
fn parse_classes(source: &str) -> Result<Vec<Class>, String> {
    let mut classes = Vec::new();
    let mut current: Option<Class> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(id) = line.strip_prefix("  class ") {
            if let Some(done) = current.take() {
                classes.push(done);
            }
            current = Some(Class {
                id: id.trim().to_owned(),
                name: String::new(),
                llm_strength: String::new(),
                benchmarks: Vec::new(),
                formal_version: String::new(),
                status: String::new(),
                tracking_issue: String::new(),
                evidence: String::new(),
            });
        } else if current.is_some() && line.starts_with("    ") && !line.starts_with("      ") {
            let class = current.as_mut().expect("a class is open");
            for (key, value) in field_pairs(trimmed) {
                match key.as_str() {
                    "name" => class.name = value,
                    "llm_strength" => class.llm_strength = value,
                    "benchmark" => class.benchmarks.push((value, None)),
                    "benchmark_url" => {
                        if let Some(last) = class.benchmarks.last_mut() {
                            last.1 = Some(value);
                        }
                    }
                    "formal_version" => class.formal_version = value,
                    "status" => class.status = value,
                    "tracking_issue" => class.tracking_issue = value,
                    "evidence" => class.evidence = value,
                    _ => {}
                }
            }
        }
    }
    if let Some(done) = current.take() {
        classes.push(done);
    }
    for class in &classes {
        for field in [
            ("name", &class.name),
            ("llm_strength", &class.llm_strength),
            ("formal_version", &class.formal_version),
            ("status", &class.status),
            ("tracking_issue", &class.tracking_issue),
            ("evidence", &class.evidence),
        ] {
            if field.1.is_empty() {
                return Err(format!("class {}: empty {}", class.id, field.0));
            }
        }
    }
    if classes.is_empty() {
        Err("registry has no class rows".to_owned())
    } else {
        Ok(classes)
    }
}

fn benchmark_cell(class: &Class) -> String {
    if class.benchmarks.is_empty() {
        return "no dedicated public benchmark; probed in-repo".to_owned();
    }
    class
        .benchmarks
        .iter()
        .map(|(name, url)| match url {
            Some(url) => format!("[{name}]({url})"),
            None => name.clone(),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn tracking_cell(class: &Class) -> String {
    format!("[#{0}]({ISSUE_URL_PREFIX}{0})", class.tracking_issue)
}

fn evidence_cell(class: &Class) -> String {
    if let Some(number) = class.evidence.strip_prefix(ISSUE_URL_PREFIX) {
        format!("[issue {number}]({})", class.evidence)
    } else if class.evidence.starts_with("http") {
        let label = class
            .evidence
            .rsplit('/')
            .next()
            .unwrap_or("evidence")
            .to_owned();
        format!("[{label}]({})", class.evidence)
    } else {
        format!("`{}`", class.evidence)
    }
}

fn class_cell(class: &Class) -> String {
    format!("`{}` — {}", class.id, class.name)
}

fn document(classes: &[Class], input_source: &str) -> String {
    let mut output = format!(
        "<!-- Generated by `rust-script scripts/generate-llm-task-parity.rs --write`. Never edited \
         by hand. -->\n\
         Input: {input_source}\n\n\
         # LLM task parity\n\n\
         One row per class of task people bring to LLMs, with what current LLMs do well, the \
         public benchmark that measures it, and the formal version of the answer Formal AI \
         targets. The status column carries the latest measured verdict; the first measurement \
         (formal-ai 0.347.0, 34 probes) found 0 of 34 useful answers and 3 wrong ones, recorded \
         in the history section below.\n\n\
         | Task class | What LLMs do well | Public benchmark | Formal version of the answer | \
         Status | Evidence | Tracking |\n\
         | --- | --- | --- | --- | --- | --- | --- |\n"
    );
    for class in classes {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            class_cell(class),
            class.llm_strength,
            benchmark_cell(class),
            class.formal_version,
            class.status,
            evidence_cell(class),
            tracking_cell(class)
        ));
    }
    output.push_str(
        "\n## History\n\n\
         ### formal-ai 0.347.0 — first measurement, 34 probes\n\n\
         0 of 34 useful; 3 wrong: `factual_qa` (the US capital for an Australia question), \
         `units_and_dates` (the day arithmetic ignored the stated offset) and `repository_qa` \
         (a self-description instead of the asked function). 22 further probes returned the \
         canned web-search paragraph, and the rest were honest refusals or surface-word \
         misroutes. Full transcripts: issue #1171.\n",
    );
    output
}

fn input_source(root: &Path) -> Result<(String, Vec<Class>), String> {
    let registry_path = root.join(REGISTRY);
    match fs::read_to_string(&registry_path) {
        Ok(source) => Ok((
            format!("`{REGISTRY}`"),
            parse_classes(&source).map_err(|error| format!("{REGISTRY}: {error}"))?,
        )),
        Err(_) => Ok((
            format!(
                "the embedded seed registry in `scripts/generate-llm-task-parity.rs` \
                 (`{REGISTRY}` from issue #1171 R1 is not landed yet)"
            ),
            parse_classes(FALLBACK_REGISTRY)
                .map_err(|error| format!("embedded seed registry: {error}"))?,
        )),
    }
}

fn main() {
    let root = std::env::current_dir().expect("current directory");
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "--check".to_owned());
    if !matches!(mode.as_str(), "--write" | "--check") {
        eprintln!("generate-llm-task-parity: unknown mode {mode}; expected --write or --check");
        std::process::exit(2);
    }
    let (input_source_value, classes) =
        input_source(&root).unwrap_or_else(|error| {
            eprintln!("generate-llm-task-parity: {error}");
            std::process::exit(1);
        });
    let path = PathBuf::from(DOCUMENT);
    let content = document(&classes, &input_source_value);
    if mode == "--write" {
        fs::write(&path, &content)
            .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
        println!("rendered {DOCUMENT} from {input_source_value}");
        return;
    }
    if fs::read_to_string(&path).ok().as_ref() == Some(&content) {
        println!("{DOCUMENT} is current");
    } else {
        eprintln!("stale generated document: {DOCUMENT}");
        eprintln!("run rust-script scripts/generate-llm-task-parity.rs --write");
        std::process::exit(1);
    }
}
