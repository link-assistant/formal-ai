//! The full loop (lookups between core runs), decomposition of a message
//! into sub-goals, and the reader-facing answer.

use super::grounding::{Knowledge, Sense};
use super::phrases::{
    COMPOSE, DECOMPOSE, LOOKUPS, REASONING_LINE, SUBGOAL, SUBGOAL_ITEM, SUBGOAL_PROGRAM, fill,
};
use super::reasoner::{MetaResult, meta_derivation_lino, meta_reason_core};
use super::seed::meta_seed;
use super::text::{is_js_whitespace, meta_examples, meta_value_literals};
use super::value::json_string;
use super::{BOUNDS, Trace};

/// A dictionary lookup: `(word, language)` to the senses a source gives.
pub type LookupFn<'a> = dyn FnMut(&str, &str) -> Vec<Sense> + 'a;

/// The answer the reasoner gives a reader.
#[derive(Debug, Clone, PartialEq)]
pub struct MetaAnswer {
    /// `meta_reasoned_program`, `meta_reasoned_program_unverified`,
    /// `meta_reasoning_open` or `meta_reasoned_definition`.
    pub intent: String,
    /// The answer text.
    pub content: String,
    /// Confidence in the answer.
    pub confidence: f32,
    /// The derivation's evidence entries.
    pub evidence: Vec<String>,
}

/// The separate artifact requests of a message: sentences that each ask for
/// an artifact or carry their own examples. A message with one request is
/// returned whole.
///
/// Mirrors `metaSubRequests` in `js/worker/formal_ai_worker_meta_composite.js`.
#[must_use]
pub fn meta_sub_requests(prompt: &str) -> Vec<String> {
    let markers = meta_seed().cue_markers("artifact");
    let requests: Vec<String> = split_sentences(prompt)
        .into_iter()
        .filter(|sentence| {
            let lowered = [" ", &sentence.to_lowercase(), " "].concat();
            markers
                .iter()
                .any(|marker| lowered.contains(marker.as_str()))
                || !meta_examples(sentence, &meta_value_literals(sentence)).is_empty()
        })
        .collect();
    if requests.len() > 1 {
        requests
    } else {
        vec![prompt.to_owned()]
    }
}

/// `split(/(?<=[.!?])\s+(?=\p{Lu})|\n+/u)`, trimmed, empty parts dropped.
fn split_sentences(prompt: &str) -> Vec<String> {
    let chars: Vec<(usize, char)> = prompt.char_indices().collect();
    let byte = |index: usize| chars.get(index).map_or(prompt.len(), |(at, _)| *at);
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < chars.len() {
        let character = chars[index].1;
        let after_stop = index > 0 && matches!(chars[index - 1].1, '.' | '!' | '?');
        if after_stop && is_js_whitespace(character) {
            let mut end = index;
            while end < chars.len() && is_js_whitespace(chars[end].1) {
                end += 1;
            }
            if chars.get(end).is_some_and(|(_, next)| {
                matches!(
                    unicode_general_category::get_general_category(*next),
                    unicode_general_category::GeneralCategory::UppercaseLetter
                )
            }) {
                pieces.push(prompt[byte(start)..byte(index)].to_owned());
                start = end;
                index = end;
                continue;
            }
        }
        if character == '\n' {
            let mut end = index;
            while end < chars.len() && chars[end].1 == '\n' {
                end += 1;
            }
            pieces.push(prompt[byte(start)..byte(index)].to_owned());
            start = end;
            index = end;
            continue;
        }
        index += 1;
    }
    pieces.push(prompt[byte(start)..].to_owned());
    pieces
        .into_iter()
        .map(|piece| piece.trim_matches(is_js_whitespace).to_owned())
        .filter(|piece| !piece.is_empty())
        .collect()
}

/// The full loop.
///
/// Run the core, open the lookups its impasses asked for, run again with the
/// new knowledge, until solved or the budget is spent. A message holding
/// several requests is decomposed into sub-goals first.
///
/// Mirrors `metaReason` in `js/worker/formal_ai_worker_meta_reasoner.js`.
pub fn meta_reason(
    prompt: &str,
    language: &str,
    knowledge: &mut Knowledge,
    mut lookup: Option<&mut LookupFn<'_>>,
) -> MetaResult {
    let requests = meta_sub_requests(prompt);
    if requests.len() > 1 {
        return meta_reason_composite(prompt, &requests, language, knowledge, lookup);
    }
    let seed = meta_seed();
    let mut result = meta_reason_core(prompt, language, knowledge);
    let mut rounds = Vec::new();
    for _ in 0..BOUNDS.lookup_rounds {
        if result.status == "solved" || result.needs.is_empty() {
            break;
        }
        if result.goal == "understand" || result.goal == "explain" {
            break;
        }
        let Some(lookup) = lookup.as_deref_mut() else {
            break;
        };
        let mut opened = Vec::new();
        for word in result.needs.iter().take(BOUNDS.lookups_per_round) {
            // A dictionary lists base forms: the surface first, then each lemma.
            let mut senses = Vec::new();
            let mut looked = word.clone();
            for lemma in seed.lemmas(word, language) {
                senses = lookup(&lemma, language);
                looked = lemma;
                if !senses.is_empty() {
                    break;
                }
            }
            let shown_lemma = if looked == *word {
                String::new()
            } else {
                ["(", &looked, ")"].concat()
            };
            opened.push([word.as_str(), &shown_lemma, ":", &senses.len().to_string()].concat());
            knowledge.insert(word.clone(), senses);
        }
        rounds.push(opened.join(" "));
        result = meta_reason_core(prompt, language, knowledge);
    }
    result.lookups = rounds;
    result.derivation_lino = meta_derivation_lino(&result);
    result
}

/// Solve every sub-request as its own goal and compose the results. The
/// composite is solved when every sub-goal is.
///
/// Mirrors `metaReasonComposite` in `js/worker/formal_ai_worker_meta_composite.js`.
fn meta_reason_composite(
    prompt: &str,
    requests: &[String],
    language: &str,
    knowledge: &mut Knowledge,
    mut lookup: Option<&mut LookupFn<'_>>,
) -> MetaResult {
    let mut trace = Trace::default();
    trace.emit("impulse", prompt);
    let listed = requests
        .iter()
        .enumerate()
        .map(|(index, request)| fill(SUBGOAL_ITEM, &[&(index + 1).to_string(), request]))
        .collect::<Vec<_>>()
        .join(" ");
    trace.emit(
        "decompose",
        fill(DECOMPOSE, &[&requests.len().to_string(), &listed]),
    );
    let mut subgoals = Vec::new();
    for request in requests {
        let sub = meta_reason(request, language, knowledge, lookup.as_deref_mut());
        let program = sub.program.as_ref().map_or_else(String::new, |program| {
            fill(SUBGOAL_PROGRAM, &[&program.steps.join(COMPOSE)])
        });
        trace.emit(
            "subgoal",
            fill(SUBGOAL, &[request, &sub.goal, &sub.status, &program]),
        );
        subgoals.push(sub);
    }
    let solved = subgoals
        .iter()
        .all(|sub| sub.status == "solved" && sub.program.is_some());
    let with_program = subgoals.iter().filter(|sub| sub.program.is_some()).count();
    let status = if solved {
        "solved"
    } else if with_program > 0 {
        "partial"
    } else {
        "open"
    };
    trace.emit(
        if solved { "goal_achieved" } else { "partial" },
        meta_seed().note(
            "subgoals_programmed",
            &[
                ("done", &with_program.to_string()),
                ("total", &subgoals.len().to_string()),
            ],
        ),
    );
    let mut result = MetaResult {
        goal: String::from("decompose"),
        language: language.to_owned(),
        examples: Vec::new(),
        definitions: Vec::new(),
        unknowns: subgoals
            .iter()
            .flat_map(|sub| sub.unknowns.clone())
            .collect(),
        needs: Vec::new(),
        program: None,
        verification: None,
        status: status.to_owned(),
        trace,
        explanation: None,
        probe: None,
        lookups: subgoals
            .iter()
            .flat_map(|sub| sub.lookups.clone())
            .collect(),
        derivation_lino: String::new(),
        subgoals: Some(subgoals),
        imperative: false,
    };
    result.derivation_lino = meta_derivation_lino(&result);
    result
}

fn answer_language(result: &MetaResult) -> &'static str {
    if result.language == "ru" { "ru" } else { "en" }
}

fn open_words(result: &MetaResult) -> Vec<String> {
    result
        .unknowns
        .iter()
        .filter(|unknown| unknown.status == "open")
        .map(|unknown| unknown.word.clone())
        .collect()
}

fn derivation_evidence(result: &MetaResult) -> Vec<String> {
    let mut evidence = vec![String::from("meta_reasoner:derivation")];
    for event in &result.trace.events {
        evidence.push(["meta:", &event.kind, ":", &event.detail].concat());
    }
    evidence
}

fn open_unknowns_text(language: &str, terms: &str) -> String {
    let seed = meta_seed();
    seed.response(
        "open_unknowns",
        language,
        &[
            ("terms", terms.to_owned()),
            ("question", seed.response("decide_question", language, &[])),
        ],
    )
}

/// The reader-facing answer for a solved or partial result, or `None`. With
/// `allow_open`, an unsolved synthesis goal answers with what is still
/// unknown.
///
/// Mirrors `metaAnswer` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_answer(result: &MetaResult, allow_open: bool) -> Option<MetaAnswer> {
    if result.subgoals.is_some() {
        return meta_composite_answer(result, allow_open);
    }
    // An imperative names a procedure only when every word of it is
    // understood; otherwise the turn is not evidently a request for one.
    if result.imperative && result.status != "solved" {
        return None;
    }
    let seed = meta_seed();
    let language = answer_language(result);
    let mut reasoning = vec![seed.response("reasoning_heading", language, &[])];
    for event in &result.trace.events {
        if event.kind == "impulse" || event.kind == "unknowns" {
            continue;
        }
        reasoning.push(fill(
            REASONING_LINE,
            &[&event.seq.to_string(), &event.kind, &event.detail],
        ));
    }
    if !result.lookups.is_empty() {
        reasoning.push(fill(LOOKUPS, &[&result.lookups.join("; ")]));
    }
    let evidence = derivation_evidence(result);
    if result.status == "solved"
        && let Some(explanation) = &result.explanation
    {
        return Some(MetaAnswer {
            intent: String::from("meta_reasoned_definition"),
            content: seed.response(
                "defined_in_request",
                language,
                &[
                    ("term", explanation.term.clone()),
                    ("definition", explanation.definition.clone()),
                ],
            ),
            confidence: 0.8,
            evidence,
        });
    }
    let Some(program) = &result.program else {
        if !allow_open
            || !matches!(
                result.goal.as_str(),
                "synthesize_from_examples" | "synthesize_from_meaning"
            )
        {
            return None;
        }
        let unresolved = open_words(result);
        if unresolved.is_empty() {
            return None;
        }
        return Some(MetaAnswer {
            intent: String::from("meta_reasoning_open"),
            content: [
                open_unknowns_text(language, &unresolved.join(", ")),
                String::new(),
                reasoning.join("\n"),
            ]
            .join("\n"),
            confidence: 0.3,
            evidence,
        });
    };
    let open = open_words(result);
    let head = result.verification.as_ref().map_or_else(
        || {
            let probe = result.probe.as_ref().map_or_else(
                || String::from("—"),
                |probe| {
                    fill(
                        super::phrases::ARROW,
                        &[&probe.input.to_json(), &probe.output.to_json()],
                    )
                },
            );
            seed.response("solved_by_meaning", language, &[("probe", probe)])
        },
        |verification| {
            seed.response(
                "solved_by_examples",
                language,
                &[
                    ("passed", verification.passed.to_string()),
                    ("total", verification.total.to_string()),
                ],
            )
        },
    );
    let mut parts = vec![
        head,
        String::new(),
        String::from("```javascript"),
        program.source.clone(),
        String::from("```"),
    ];
    if let Some(argument) = &program.argument {
        let call = ["solution(", &json_string(argument), ")"].concat();
        parts.push(String::new());
        parts.push(seed.response("usage", language, &[("call", call)]));
    }
    if let Some(environment) = &program.environment {
        parts.push(String::new());
        parts.push(seed.response(
            "environment_note",
            language,
            &[("environment", environment.clone())],
        ));
    }
    parts.push(String::new());
    parts.push(reasoning.join("\n"));
    if !open.is_empty() && result.verification.is_none() {
        parts.push(String::new());
        parts.push(open_unknowns_text(language, &open.join(", ")));
    }
    let verified = result.verification.is_some();
    Some(MetaAnswer {
        intent: String::from(if verified {
            "meta_reasoned_program"
        } else {
            "meta_reasoned_program_unverified"
        }),
        content: parts.join("\n"),
        confidence: if verified { 0.9 } else { 0.55 },
        evidence,
    })
}

/// The composed answer: each sub-request with its own answer.
///
/// Mirrors `metaCompositeAnswer` in `js/worker/formal_ai_worker_meta_composite.js`.
fn meta_composite_answer(result: &MetaResult, allow_open: bool) -> Option<MetaAnswer> {
    let seed = meta_seed();
    let language = answer_language(result);
    let subgoals = result.subgoals.as_deref().unwrap_or_default();
    let mut parts = vec![seed.response(
        "decomposed",
        language,
        &[("count", subgoals.len().to_string())],
    )];
    let evidence = derivation_evidence(result);
    let mut answered = 0;
    for (index, sub) in subgoals.iter().enumerate() {
        let answer = meta_answer(sub, allow_open);
        if answer.is_some() {
            answered += 1;
        }
        let request = sub
            .trace
            .events
            .first()
            .map(|event| event.detail.clone())
            .unwrap_or_default();
        let heading = seed.response(
            "subrequest",
            language,
            &[("index", (index + 1).to_string()), ("request", request)],
        );
        let body = answer.map_or_else(
            || {
                let open = open_words(sub);
                let terms = if open.is_empty() {
                    String::from("—")
                } else {
                    open.join(", ")
                };
                open_unknowns_text(language, &terms)
            },
            |answer| answer.content,
        );
        parts.push(String::new());
        parts.push(["### ", &heading].concat());
        parts.push(String::new());
        parts.push(body);
    }
    if answered == 0 {
        return None;
    }
    let verified = subgoals.iter().all(|sub| {
        sub.verification
            .as_ref()
            .is_some_and(|verification| verification.failures.is_empty())
    });
    Some(MetaAnswer {
        intent: String::from(if verified {
            "meta_reasoned_program"
        } else {
            "meta_reasoned_program_unverified"
        }),
        content: parts.join("\n"),
        confidence: if verified { 0.9 } else { 0.55 },
        evidence,
    })
}
