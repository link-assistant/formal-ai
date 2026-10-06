//! The synchronous core: parse, open unknowns, ground them, plan and verify;
//! explanation-based learning; the derivation in links notation.
#![allow(clippy::float_cmp)]

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::seed::parser::parse_lino;

use super::catalog::{Step, catalog};
use super::grounding::{Chunk, Context, Grounding, Knowledge, ground, ground_text, meta_clauses};
use super::interpreter::{Param, Runtime, run_program};
use super::phrases::{ARROW, COMPOSE, LEARNED as LEARNED_PHRASE, NO_PROBE, NONE, fill};
use super::seed::{Hypothesis, meta_seed};
use super::synthesis::{
    MeaningGoal, Verification, synthesize_from_examples, synthesize_from_meaning, verify,
};
use super::text::{
    Definition, Example, LiteralKind, js_trim, locale_compare, meta_examples,
    meta_request_definitions, meta_value_literals, meta_words,
};
use super::value::{Value, js_number, json_string};
use super::{BOUNDS, Trace};

/// A derived program as the answer shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgramInfo {
    /// Step labels (`split_words`, `each(reverse_text)`).
    pub steps: Vec<String>,
    /// The steps themselves, runnable by the interpreter.
    pub step_refs: Vec<Step>,
    /// The program's parameter.
    pub parameter: Param,
    /// The rendered JavaScript.
    pub source: String,
    /// Equally evidenced alternatives.
    pub alternatives: usize,
    /// Candidates evaluated.
    pub evaluated: usize,
    /// The environment the program needs and this runtime lacks.
    pub environment: Option<String>,
    /// A path the request states, the program's argument.
    pub argument: Option<String>,
}

/// One input the probe ran and what the program made of it.
#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    /// The sample.
    pub input: Value,
    /// The program's output.
    pub output: Value,
}

/// How one word of the request ended up.
#[derive(Debug, Clone, PartialEq)]
pub struct Unknown {
    /// The word.
    pub word: String,
    /// Its grounding status.
    pub status: &'static str,
    /// Where it was grounded.
    pub origin: &'static str,
    /// Its strongest hypothesis.
    pub best: Option<Hypothesis>,
}

/// What the reasoner derived for one request.
#[derive(Debug, Clone, PartialEq)]
pub struct MetaResult {
    /// `synthesize_from_examples`, `synthesize_from_meaning`, `explain`,
    /// `understand` or `decompose`.
    pub goal: String,
    /// The request's language.
    pub language: String,
    /// The request's examples.
    pub examples: Vec<Example>,
    /// The definitions the request gives.
    pub definitions: Vec<Definition>,
    /// Every content word and how it was grounded.
    pub unknowns: Vec<Unknown>,
    /// Words a lookup would have to open.
    pub needs: Vec<String>,
    /// The derived program.
    pub program: Option<ProgramInfo>,
    /// The verification against the examples.
    pub verification: Option<Verification>,
    /// `solved`, `partial` or `open`.
    pub status: String,
    /// The numbered decisions.
    pub trace: Trace,
    /// A definition the request answers itself.
    pub explanation: Option<Definition>,
    /// The probe run, for a program without examples.
    pub probe: Option<Probe>,
    /// The lookup rounds of the loop.
    pub lookups: Vec<String>,
    /// The derivation in links notation.
    pub derivation_lino: String,
    /// Sub-goals of a decomposed message.
    pub subgoals: Option<Vec<Self>>,
}

static LEARNED_CHUNKS: Mutex<BTreeMap<String, Chunk>> = Mutex::new(BTreeMap::new());
static NEW_CHUNKS: Mutex<BTreeMap<String, Chunk>> = Mutex::new(BTreeMap::new());

fn lock(store: &Mutex<BTreeMap<String, Chunk>>) -> MutexGuard<'_, BTreeMap<String, Chunk>> {
    store.lock().unwrap_or_else(PoisonError::into_inner)
}

fn learned_snapshot() -> BTreeMap<String, Chunk> {
    lock(&LEARNED_CHUNKS).clone()
}

/// Forget every learned chunk (a fresh session).
///
/// Mirrors clearing `metaLearnedChunks` / `metaNewChunks` in
/// `js/worker/formal_ai_worker_meta_reasoner.js`.
pub fn forget_learned() {
    lock(&LEARNED_CHUNKS).clear();
    lock(&NEW_CHUNKS).clear();
}

/// Explanation-based learning: a word grounded only through a capture
/// becomes a chunk keyed by the word once a program used its operation.
///
/// Mirrors `metaLearn` in `js/worker/formal_ai_worker_meta_reasoner.js`.
fn meta_learn(groundings: &[Grounding], steps: &[Step], trace: &mut Trace) {
    let catalog = catalog();
    let mut used = Vec::new();
    for step in steps {
        catalog.step_ops(*step, &mut used);
    }
    let used: Vec<&str> = used
        .iter()
        .filter_map(|op| catalog.ops.get(*op as usize).map(String::as_str))
        .collect();
    for grounding in groundings {
        if grounding.origin != "capture" {
            continue;
        }
        let Some(hypothesis) = grounding
            .hypotheses
            .iter()
            .find(|item| used.contains(&item.operation.as_str()))
        else {
            continue;
        };
        let chunk = Chunk {
            operation: hypothesis.operation.clone(),
            score: hypothesis.score,
            via: hypothesis.via.clone(),
        };
        lock(&LEARNED_CHUNKS).insert(grounding.word.clone(), chunk.clone());
        lock(&NEW_CHUNKS).insert(grounding.word.clone(), chunk);
        trace.emit(
            "chunk",
            fill(LEARNED_PHRASE, &[&grounding.word, &hypothesis.operation]),
        );
    }
}

fn sorted_chunks(store: &BTreeMap<String, Chunk>) -> Vec<(&String, &Chunk)> {
    let mut entries: Vec<(&String, &Chunk)> = store.iter().collect();
    entries.sort_by(|a, b| locale_compare(a.0, b.0));
    entries
}

/// Learned chunks as links notation, so a session can persist them.
///
/// Mirrors `metaLearnedLino` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_learned_lino() -> String {
    let store = learned_snapshot();
    let mut lines = vec![String::from("meta_learned_chunks")];
    for (word, chunk) in sorted_chunks(&store) {
        lines.push(["  chunk ", &json_string(word)].concat());
        lines.push(["    operation ", &chunk.operation].concat());
        lines.push(["    via ", &json_string(&chunk.via)].concat());
    }
    lines.join("\n")
}

/// Load learned chunks from the app's memory statements
/// (`meta_learned_chunk` records); returns how many were loaded.
///
/// Mirrors `metaImportLearned` in `js/worker/formal_ai_worker_meta_composite.js`.
#[must_use]
pub fn import_learned(memory: &[String]) -> usize {
    let mut loaded = 0;
    for statement in memory {
        for node in parse_lino(statement).children {
            if node.name != "meta_learned_chunk" {
                continue;
            }
            let word = node.find_child_value("word").to_owned();
            let operation = node.find_child_value("operation").to_owned();
            if word.is_empty() || operation.is_empty() || lock(&LEARNED_CHUNKS).contains_key(&word)
            {
                continue;
            }
            let score = node
                .find_child_value("score")
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|value| *value != 0.0 && !value.is_nan())
                .unwrap_or(0.5);
            let via = node.find_child_value("via");
            let via = if via.is_empty() { "memory" } else { via };
            lock(&LEARNED_CHUNKS).insert(
                word,
                Chunk {
                    operation,
                    score,
                    via: via.to_owned(),
                },
            );
            loaded += 1;
        }
    }
    loaded
}

/// The chunks learned since the last call, as one `meta_learned_chunk`
/// statement for the memory store to append, or `None`.
///
/// Mirrors `metaTakeLearned` in `js/worker/formal_ai_worker_meta_composite.js`.
#[must_use]
pub fn take_learned() -> Option<String> {
    let taken = core::mem::take(&mut *lock(&NEW_CHUNKS));
    if taken.is_empty() {
        return None;
    }
    let mut lines = Vec::new();
    for (word, chunk) in sorted_chunks(&taken) {
        lines.push(String::from("meta_learned_chunk"));
        lines.push(["  word ", &json_string(word)].concat());
        lines.push(["  operation ", &chunk.operation].concat());
        lines.push(["  score ", &js_number(chunk.score)].concat());
        lines.push(["  via ", &json_string(&chunk.via)].concat());
    }
    Some(lines.join("\n"))
}

/// The derivation as links notation: goal, unknowns, program, verification
/// and every trace event in order, then each sub-goal's derivation.
///
/// Mirrors `metaDerivationLino` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_derivation_lino(result: &MetaResult) -> String {
    let mut lines = vec![
        String::from("derivation"),
        ["  goal ", &result.goal].concat(),
        ["  status ", &result.status].concat(),
    ];
    for unknown in &result.unknowns {
        lines.push(["  unknown ", &json_string(&unknown.word)].concat());
        lines.push(["    status ", unknown.status].concat());
        lines.push(["    origin ", unknown.origin].concat());
        if let Some(best) = &unknown.best {
            lines.push(["    operation ", &best.operation].concat());
        }
    }
    if let Some(program) = &result.program {
        lines.push(["  program ", &json_string(&program.steps.join(COMPOSE))].concat());
    }
    if let Some(verification) = &result.verification {
        lines.push(
            [
                "  verified ",
                &verification.passed.to_string(),
                "/",
                &verification.total.to_string(),
            ]
            .concat(),
        );
    }
    for event in &result.trace.events {
        lines.push(["  event ", &event.seq.to_string()].concat());
        lines.push(["    kind ", &event.kind].concat());
        lines.push(["    detail ", &json_string(&event.detail)].concat());
    }
    for (index, sub) in result.subgoals.iter().flatten().enumerate() {
        lines.push(["  subgoal ", &(index + 1).to_string()].concat());
        for line in meta_derivation_lino(sub).split('\n').skip(1) {
            lines.push(["  ", line].concat());
        }
    }
    lines.join("\n")
}

/// True when this runtime offers an environment a primitive needs. The
/// rendered program is JavaScript for Node; the Rust engine does not run
/// Node, so like the web worker it offers no environment.
///
/// Mirrors `metaEnvironmentAvailable` in `js/worker/formal_ai_worker_meta_reasoner.js`.
const fn environment_available(environment: &str) -> bool {
    environment.is_empty()
}

/// The top hypotheses of a grounding (tied with its strongest).
fn top_of(grounding: &Grounding) -> Vec<Hypothesis> {
    grounding.top().into_iter().cloned().collect()
}

/// The parsed request a goal branch works from.
struct Request<'a> {
    prompt: &'a str,
    unquoted: String,
    lowered: String,
    language: &'a str,
    literals: Vec<super::text::Literal>,
}

/// The synchronous core: parse, open unknowns, ground them, plan and verify.
/// `knowledge` maps a word to dictionary senses already fetched.
///
/// Mirrors `metaReasonCore` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_reason_core(prompt: &str, language: &str, knowledge: &Knowledge) -> MetaResult {
    let seed = meta_seed();
    let mut trace = Trace::default();
    trace.emit("impulse", prompt);
    let literals = meta_value_literals(prompt);
    let examples = meta_examples(prompt, &literals);
    let definitions = meta_request_definitions(prompt);
    let mut unquoted = prompt.to_owned();
    for literal in literals.iter().rev() {
        unquoted = [&unquoted[..literal.start], " ", &unquoted[literal.end..]].concat();
    }
    let lowered = [" ", &unquoted.to_lowercase(), " "].concat();
    let artifact = seed
        .cue_markers("artifact")
        .iter()
        .any(|marker| lowered.contains(marker.as_str()));
    let question = seed
        .cue_markers("question")
        .iter()
        .any(|marker| lowered.contains(marker.as_str()));
    // Examples embedded in prose that asks for no artifact ("on the 18th at
    // 17:00") are not a specification: an arrow, an artifact request or bare
    // examples make the examples the goal.
    let prose = meta_words(&unquoted)
        .iter()
        .any(|word| !seed.is_grammatical(word));
    let specified = !examples.is_empty()
        && (artifact || !prose || examples.iter().any(|example| example.symbolic));
    let goal = if specified {
        "synthesize_from_examples"
    } else if artifact {
        "synthesize_from_meaning"
    } else if question {
        "explain"
    } else {
        "understand"
    };
    trace.emit(
        "formalize",
        meta_seed().note(
            "formalized",
            &[
                ("goal", goal),
                ("examples", &examples.len().to_string()),
                ("definitions", &definitions.len().to_string()),
            ],
        ),
    );
    let chunks = learned_snapshot();
    let artifact_words = seed.cue_markers("artifact");
    let mut content_words: Vec<String> = Vec::new();
    for word in meta_words(&unquoted) {
        if seed.is_grammatical(&word) || content_words.contains(&word) {
            continue;
        }
        if artifact_words
            .iter()
            .any(|marker| word.starts_with(js_trim(marker)))
        {
            continue;
        }
        if definitions.iter().any(|item| item.term == word) && goal != "explain" {
            continue;
        }
        content_words.push(word);
    }
    let shown = if content_words.is_empty() {
        String::from(NONE)
    } else {
        content_words.join(", ")
    };
    trace.emit("unknowns", shown);
    let mut context = Context {
        trace: &mut trace,
        definitions: &definitions,
        knowledge,
        language,
        needs: Vec::new(),
        chunks: &chunks,
    };
    let mut groundings: Vec<Grounding> = if goal == "understand" {
        Vec::new()
    } else {
        content_words
            .iter()
            .map(|word| ground(word, &mut context, 0, &[]))
            .collect()
    };
    for grounding in &groundings {
        for need in &grounding.needs {
            if !context.needs.contains(need) {
                context.needs.push(need.clone());
            }
        }
    }
    if goal != "explain" {
        for definition in &definitions {
            let stack = [definition.term.clone()];
            for hypothesis in ground_text(&definition.definition, &mut context, 1, &stack) {
                groundings.push(Grounding {
                    word: definition.term.clone(),
                    status: "grounded",
                    origin: "request",
                    hypotheses: vec![hypothesis],
                    needs: Vec::new(),
                });
            }
        }
    }
    let needs = core::mem::take(&mut context.needs);
    let catalog = catalog();
    let mut evidence: Vec<f64> = vec![0.0; catalog.ops.len()];
    for grounding in &groundings {
        for hypothesis in &grounding.hypotheses {
            if let Some(op) = catalog.op(&hypothesis.operation)
                && let Some(slot) = evidence.get_mut(op as usize)
            {
                *slot = slot.max(hypothesis.score);
            }
        }
    }
    let mut result = MetaResult {
        goal: goal.to_owned(),
        language: language.to_owned(),
        examples,
        definitions,
        unknowns: groundings
            .iter()
            .map(|grounding| Unknown {
                word: grounding.word.clone(),
                status: grounding.status,
                origin: grounding.origin,
                best: grounding.hypotheses.first().cloned(),
            })
            .collect(),
        needs,
        program: None,
        verification: None,
        status: String::from("open"),
        trace,
        explanation: None,
        probe: None,
        lookups: Vec::new(),
        derivation_lino: String::new(),
        subgoals: None,
    };
    match goal {
        "explain" => {
            let defined = result
                .definitions
                .iter()
                .find(|item| content_words.contains(&item.term))
                .cloned();
            if let Some(defined) = defined {
                result.status = String::from("solved");
                result.trace.emit(
                    "goal_achieved",
                    meta_seed().note("self_defined", &[("term", &defined.term)]),
                );
                result.explanation = Some(defined);
            }
        }
        "synthesize_from_examples" => solve_from_examples(&mut result, &groundings, &evidence),
        "synthesize_from_meaning" => {
            let request = Request {
                prompt,
                unquoted,
                lowered,
                language,
                literals,
            };
            solve_from_meaning(&mut result, &request, &groundings, &evidence);
        }
        _ => {}
    }
    result
}

/// The `synthesize_from_examples` branch of `metaReasonCore`
/// (`js/worker/formal_ai_worker_meta_reasoner.js`).
fn solve_from_examples(result: &mut MetaResult, groundings: &[Grounding], evidence: &[f64]) {
    let catalog = catalog();
    let Some(found) =
        synthesize_from_examples(catalog, &result.examples, evidence, &mut result.trace)
    else {
        return;
    };
    let source = catalog.render(&found.steps, &found.parameter.to_js());
    let verification = verify(catalog, &found.steps, found.parameter, &result.examples);
    let failed = !verification.failures.is_empty();
    result.trace.emit(
        if failed { "impasse" } else { "verified" },
        meta_seed().note(
            "verified",
            &[
                ("passed", &verification.passed.to_string()),
                ("total", &verification.total.to_string()),
            ],
        ),
    );
    result.program = Some(ProgramInfo {
        steps: found
            .steps
            .iter()
            .map(|step| catalog.step_label(*step))
            .collect(),
        step_refs: found.steps.clone(),
        parameter: found.parameter,
        source,
        alternatives: found.alternatives,
        evaluated: found.evaluated,
        environment: None,
        argument: None,
    });
    result.verification = Some(verification);
    result.status = String::from(if failed { "open" } else { "solved" });
    if !failed {
        meta_learn(groundings, &found.steps, &mut result.trace);
    }
}

/// The `synthesize_from_meaning` branch of `metaReasonCore`
/// (`js/worker/formal_ai_worker_meta_reasoner.js`).
fn solve_from_meaning(
    result: &mut MetaResult,
    request: &Request<'_>,
    groundings: &[Grounding],
    evidence: &[f64],
) {
    let seed = meta_seed();
    let catalog = catalog();
    let trace = &mut result.trace;
    let mut groups: Vec<Vec<String>> = Vec::new();
    for grounding in groundings {
        if grounding.hypotheses.is_empty() {
            continue;
        }
        let group: Vec<String> = grounding
            .top()
            .into_iter()
            .map(|hypothesis| hypothesis.operation.clone())
            .collect();
        // A word that ties between many operations describes data, not an
        // action; it stays evidence instead of a required operation.
        if group.len() > BOUNDS.required_group_size {
            trace.emit(
                "evidence",
                meta_seed().note(
                    "evidence_only",
                    &[
                        ("word", &grounding.word),
                        ("count", &group.len().to_string()),
                    ],
                ),
            );
            continue;
        }
        if !groups.contains(&group) {
            groups.push(group);
        }
    }
    if groups.is_empty() {
        trace.emit("impasse", meta_seed().note("nothing_grounded", &[]));
        return;
    }
    let open = groundings
        .iter()
        .filter(|grounding| grounding.status == "open")
        .count();
    if open > groups.len() {
        trace.emit(
            "impasse",
            meta_seed().note(
                "open_outweigh",
                &[
                    ("open", &open.to_string()),
                    ("grounded", &groups.len().to_string()),
                ],
            ),
        );
        return;
    }
    let stated = request.literals.iter().find(|literal| {
        literal.kind != LiteralKind::Path && matches!(literal.value, Value::Number(_))
    });
    let stated_value = stated.and_then(|literal| match literal.value {
        Value::Number(number) => Some(number),
        _ => None,
    });
    // Words that tie across many operations describe data; they neither
    // require nor credit an operation.
    let words: Vec<(usize, Vec<Hypothesis>)> = groundings
        .iter()
        .enumerate()
        .map(|(index, grounding)| (index, top_of(grounding)))
        .filter(|(_, hypotheses)| {
            !hypotheses.is_empty() && hypotheses.len() <= BOUNDS.required_group_size
        })
        .collect();
    // A word tied across operations that all take one type names the
    // argument: "the lines of a text" is a function of a text.
    let mut input_types: Vec<String> = Vec::new();
    for grounding in groundings {
        let top = top_of(grounding);
        if top.len() <= BOUNDS.required_group_size {
            continue;
        }
        let mut froms: Vec<Option<String>> = Vec::new();
        for hypothesis in &top {
            let from = seed
                .primitive(&hypothesis.operation)
                .map(|primitive| primitive.from.clone());
            if !froms.contains(&from) {
                froms.push(from);
            }
        }
        if let [Some(kind)] = froms.as_slice()
            && !kind.is_empty()
            && !input_types.contains(kind)
        {
            input_types.push(kind.clone());
            trace.emit(
                "evidence",
                meta_seed().note(
                    "argument_type",
                    &[("word", &grounding.word), ("type", kind)],
                ),
            );
        }
    }
    let stated_path = request
        .literals
        .iter()
        .find(|literal| literal.kind == LiteralKind::Path)
        .and_then(|literal| literal.value.as_str().map(str::to_owned));
    if let Some(path) = &stated_path
        && !input_types.iter().any(|kind| kind == "path")
    {
        input_types.insert(0, String::from("path"));
        trace.emit(
            "evidence",
            meta_seed().note("stated_path", &[("path", path)]),
        );
    }
    let clauses = meta_clauses(&request.unquoted, groundings, trace);
    // Words tied across many operations still break ties among programs
    // equal in everything else, and may name a filter's measure.
    let tied: Vec<(&Grounding, Vec<Hypothesis>)> = groundings
        .iter()
        .map(|grounding| (grounding, top_of(grounding)))
        .filter(|(_, hypotheses)| hypotheses.len() > BOUNDS.required_group_size)
        .collect();
    let weak_words: Vec<Vec<Hypothesis>> = tied
        .iter()
        .map(|(_, hypotheses)| hypotheses.clone())
        .collect();
    // A tied plural noun names a representation ("the lines").
    let weak_views: Vec<Vec<Hypothesis>> = tied
        .iter()
        .filter(|(grounding, _)| {
            seed.lemmas(&grounding.word, request.language)
                .iter()
                .any(|lemma| *lemma != grounding.word)
        })
        .map(|(_, hypotheses)| {
            hypotheses
                .iter()
                .filter(|hypothesis| seed.is_view(&hypothesis.operation))
                .cloned()
                .collect::<Vec<_>>()
        })
        .filter(|hypotheses| !hypotheses.is_empty())
        .collect();
    let quantifier = seed
        .cue_markers("universal")
        .into_iter()
        .find(|marker| request.lowered.contains(marker.as_str()));
    if let Some(marker) = &quantifier {
        trace.emit(
            "evidence",
            meta_seed().note("universal", &[("word", js_trim(marker))]),
        );
    }
    // The grounded word right after a stated bound is its unit ("longer than
    // 80 characters"): it names what a filter measures.
    let mut by_word: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, grounding) in groundings.iter().enumerate() {
        by_word.insert(&grounding.word, index);
    }
    let unit = stated.and_then(|literal| {
        meta_words(&request.prompt[literal.end..])
            .iter()
            .filter_map(|word| by_word.get(word.as_str()).copied())
            .find(|index| !groundings[*index].hypotheses.is_empty())
    });
    if let (Some(index), Some(value)) = (unit, stated_value) {
        trace.emit(
            "evidence",
            meta_seed().note(
                "measure_unit",
                &[
                    ("word", &groundings[index].word),
                    ("bound", &js_number(value)),
                ],
            ),
        );
    }
    // The unit credits the measure, not the main program.
    let main_words: Vec<Vec<Hypothesis>> = words
        .iter()
        .filter(|(index, _)| Some(*index) != unit)
        .map(|(_, hypotheses)| hypotheses.clone())
        .collect();
    let measure_words: Option<Vec<Vec<Hypothesis>>> =
        unit.map(|index| vec![groundings[index].hypotheses.clone()]);
    let goal = MeaningGoal {
        groups: &groups,
        evidence,
        parameter: stated_value,
        words: &main_words,
        input_types: &input_types,
        clauses: &clauses,
        weak_words: &weak_words,
        weak_views: &weak_views,
        universal: quantifier.is_some(),
        measure_words: measure_words.as_deref(),
    };
    let Some(found) = synthesize_from_meaning(catalog, &goal, trace) else {
        return;
    };
    if found.parameter != Param::Null {
        trace.emit(
            "bind",
            meta_seed().note("parameter", &[("value", &found.parameter.to_js())]),
        );
    }
    let source = catalog.render(&found.steps, &found.parameter.to_js());
    let open_unknowns = result
        .unknowns
        .iter()
        .any(|unknown| unknown.status == "open");
    let status = String::from(if open_unknowns { "partial" } else { "solved" });
    let labels: Vec<String> = found
        .steps
        .iter()
        .map(|step| catalog.step_label(*step))
        .collect();
    // An operation that needs an environment this runtime lacks is not
    // probed here; the answer says so.
    let needs = found
        .steps
        .iter()
        .map(|step| catalog.step_environment(*step))
        .find(|environment| !environment.is_empty())
        .map(str::to_owned);
    if let Some(needs) = needs
        && !environment_available(&needs)
    {
        result.trace.emit(
            "probe",
            meta_seed().note("probe_skipped", &[("environment", &needs)]),
        );
        result.program = Some(ProgramInfo {
            steps: labels,
            step_refs: found.steps.clone(),
            parameter: found.parameter,
            source,
            alternatives: 0,
            evaluated: 0,
            environment: Some(needs),
            argument: stated_path,
        });
        result.probe = None;
        result.status = status;
        meta_learn(groundings, &found.steps, &mut result.trace);
        return;
    }
    // Probe with the seeded samples of the argument type; the first one the
    // program changes is shown. A program no sample changes is a no-op.
    let mut probe: Option<Probe> = None;
    for sample in seed
        .probes
        .get(&found.from_type)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let input = if found.from_type == "text" {
            Some(Value::Text(sample.clone()))
        } else {
            Value::parse_json(sample)
        };
        let Some(input) = input else {
            continue;
        };
        let Some(output) = run_program(
            catalog,
            &found.steps,
            &input,
            found.parameter,
            Runtime::Worker,
        ) else {
            continue;
        };
        let changed = input.to_json() != output.to_json();
        if probe.is_none() || changed {
            probe = Some(Probe { input, output });
        }
        if changed {
            break;
        }
    }
    let shown = probe.as_ref().map_or_else(
        || String::from(NO_PROBE),
        |probe| fill(ARROW, &[&probe.input.to_json(), &probe.output.to_json()]),
    );
    result.trace.emit("probe", shown);
    let unchanged = probe
        .as_ref()
        .is_none_or(|probe| probe.input.to_json() == probe.output.to_json());
    if unchanged {
        result.trace.emit("impasse", meta_seed().note("no_op", &[]));
        return;
    }
    result.program = Some(ProgramInfo {
        steps: labels,
        step_refs: found.steps.clone(),
        parameter: found.parameter,
        source,
        alternatives: 0,
        evaluated: 0,
        environment: None,
        argument: stated_path,
    });
    result.probe = probe;
    result.status = status;
    meta_learn(groundings, &found.steps, &mut result.trace);
}
