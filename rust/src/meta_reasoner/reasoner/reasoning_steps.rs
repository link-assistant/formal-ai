use super::*;

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
    // A bare name after a locative cue that follows a file noun ("any file
    // in data") is a location operand, a path like "js/worker".
    let located = location_operands(&unquoted, language);
    for operand in located.iter().rev() {
        unquoted = [&unquoted[..operand.start], " ", &unquoted[operand.end..]].concat();
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
    // An imperative that opens with an operation its documentation names and
    // gives no data to act on asks for the procedure itself.
    let opening = meta_words(&unquoted).into_iter().next();
    let imperative = !artifact
        && !question
        && literals
            .iter()
            .all(|literal| literal.kind == LiteralKind::Path)
        && opening
            .as_deref()
            .is_some_and(|word| names_operation(word, language));
    let goal = if specified {
        "synthesize_from_examples"
    } else if artifact || imperative {
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
    if let Some(word) = opening.as_deref()
        && imperative
        && !artifact
        && goal == "synthesize_from_meaning"
    {
        trace.emit("evidence", seed.note("imperative", &[("word", word)]));
    }
    for operand in &located {
        trace.emit(
            "evidence",
            seed.note(
                "location",
                &[("word", &operand.value), ("cue", &operand.cue)],
            ),
        );
    }
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
                groundings.push(Grounding::new(
                    &definition.term,
                    "grounded",
                    "request",
                    vec![hypothesis],
                ));
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
        imperative,
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
                located,
            };
            solve_from_meaning(&mut result, &request, &groundings, &evidence);
        }
        _ => {}
    }
    result
}
