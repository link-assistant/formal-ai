//! Deliver only conclusive observed findings to an explicitly bound evidence file.
//!
//! Recursive investigation retains typed provenance; authoring operands are not
//! evidence destinations, and clarifications, gaps and failures remain chat answers.

mod artifact_header;
mod discovery_delivery;
mod record_observation;
mod record_render;

use artifact_header::names_callable_artifact;
use record_observation::plan_record_readback_step;
use record_render::{render_obligation, sequence_observation, written_observation};
use std::cell::RefCell;
use std::collections::HashMap;

use super::capability_router::tool_for;
use super::final_result::{FinalDisposition, FinalPayloadRole, FinalResult, record};
use super::general_planner::{compose_general_change_plan, has_authoritative_literal_write};
use super::planner::{
    AgenticPlan, Capability, plan_chat_step_resolved, plan_one, plan_settled_routes, trace_route,
    write_arguments,
};
use super::progress::Progress;
use super::request_sequence::VERIFIED_SEQUENCE_ORIGIN;
use super::shell_command::carries_authoring_task;
use super::shell_command_policy::sentences;
use super::write_request::{
    bare_surfaces, delivered_write_target, first_action_cue_start, pinned_first_line, tokens,
};
use crate::obligation_ledger::ObligationOutcome;
use crate::protocol::{ChatMessage, MessageContent, ToolCall};

/// The delivery half of a request, separated from the work it asks for.
struct DeliveryBinding {
    /// The path the answer has to be written to.
    target: String,
    /// The exact opening line the caller pinned, when it pinned one.
    first_line: Option<String>,
    /// Exact `key=value` lines declared in the delivery sentence.
    field_lines: Vec<String>,
    /// Everything the request says that is not about delivery.
    residual: String,
}

/// `text` with every quoted segment that holds whitespace replaced, delimiters
/// included, by as many `x` as it has bytes, so spans still index the
/// original.
///
/// A single-token quote -- a backticked path or name -- stays readable. The
/// lines under a colon-ended first line are the request's payload, whatever
/// they say, so they are masked the same way, line breaks kept (PR #1188 G95:
/// a row that read "append ... to gaps.md" was taken for a delivery, and the
/// answer was written over that file).
fn masked_multi_word_quotes(text: &str) -> String {
    let mut masked = text.to_owned();
    for segment in crate::normal_markov::quoted_segment_spans(text) {
        if segment.text.chars().any(char::is_whitespace) {
            masked.replace_range(
                segment.start..segment.end,
                &"x".repeat(segment.end - segment.start),
            );
        }
    }
    let end = super::positional_edit::instruction_end(text);
    let payload: String = masked[end..]
        .chars()
        .map(|character| {
            if character.is_whitespace() {
                character.to_string()
            } else {
                "x".repeat(character.len_utf8())
            }
        })
        .collect();
    masked.replace_range(end.., &payload);
    masked
}

/// Split a request into its delivery obligation and the investigation left over.
///
/// The obligation is read one sentence at a time, and a sentence carries it only
/// when it applies a seed-defined write action *to* a seed-cued target path. Both
/// halves are required and both must be in the same sentence: "Read the file
/// `Cargo.toml`. Record what you find in `notes/report.md`." cues two paths, and
/// only the second one is being written to. Scoping the pair to a sentence is
/// what tells them apart, and it is the same scoping
/// [`super::shell_command`] uses to tell a command that is named from one that
/// is ordered (issue #907).
///
/// A sentence that asks for an artifact to be authored is never the delivery
/// half, however write-shaped it looks. "Today's date is Sunday. Create a file
/// `main.py` that prints Hello, world!" applies a write action to a cued path,
/// and reading it as delivery inverts the request: the caller's work becomes the
/// destination and their passing statement becomes the investigation, so the
/// Python file was written with prose about the date. The authoring test is
/// [`super::shell_command::carries_authoring_task`], the one requirement 3 of
/// issue #907 already states in exactly these terms.
///
/// Exact fields or a pinned first line belong to the existing evidence delivery grammar.
/// Authoritative literal payloads keep their original ownership and never become records.
pub(super) fn has_typed_evidence_delivery(request: &str) -> bool {
    !has_authoritative_literal_write(request)
        && parse_obligation(request)
            .is_some_and(|binding| !binding.field_lines.is_empty() || binding.first_line.is_some())
}

/// Declines when the residual is empty: a request whose every sentence is about
/// delivery states no work to do, so there is nothing to record.
/// A call-shaped authoring request binds its path as a code operand.
fn parse_obligation(request: &str) -> Option<DeliveryBinding> {
    let mut target = None;
    let mut first_line = None;
    let mut field_lines = Vec::new();
    let mut residual = String::new();
    let mut later_obligation = false;
    // Sentences and delivery cues are read with every multi-word quoted
    // payload masked: `Append "/// Append an empty line to notes.txt." to
    // src/lib.rs.` is one sentence about src/lib.rs, not a delivery to the
    // notes.txt its payload mentions (PR #1188 sub-agent gap 2). The pinned
    // line, field lines and the work before the delivery are read from the
    // request itself, where the quoted text is intact.
    let masked = masked_multi_word_quotes(request);
    for sentence in sentences(&masked) {
        let start = masked[sentence.span.clone()]
            .find(sentence.text)
            .map_or(sentence.span.start, |at| sentence.span.start + at);
        let text = request
            .get(start..start + sentence.text.len())
            .unwrap_or(sentence.text);
        if let Some(line) = pinned_first_line(text) {
            if target.is_some() && !later_obligation {
                first_line = first_line.or(Some(line));
                continue;
            }
            residual.push_str(&request[sentence.span]);
            continue;
        }
        // A sentence states the work rather than a place to put it when it
        // asks for an artifact to be authored, or when the path it names is
        // where the work happens rather than where an answer goes. Both make
        // that path an operand, and reading either as a delivery is
        // destructive: the ladder's leaf says "Edit the tracked file
        // `src/engine_responses.rs`: add \"Good morning\" to the
        // GREETING_EXAMPLES list", so delivery wrote its own status line over
        // the source, the write returned success, and the node reported done
        // having deleted the file it was asked to edit.
        let delivered = (!carries_authoring_task(&crate::engine::normalize_prompt(sentence.text)))
            .then(|| delivered_write_target(sentence.text))
            .flatten()
            .filter(|target| {
                !names_callable_artifact(text, target)
                    && super::function_expectation::test_expectation_question(text, &mut None)
                        .is_none()
            });
        if let Some(named) = delivered {
            if target.is_none() {
                field_lines = exact_field_lines(text, &named);
                target = Some(named);
                if let Some(work) = work_before_delivery(text) {
                    residual.push_str(work);
                    residual.push_str(". ");
                }
                continue;
            }
            // A constraint in the next sentence belongs to this later output,
            // not to the first output selected by this recursive pass. Leave
            // both sentences in the residual so the nested pass sees them
            // together and can bind them correctly.
            later_obligation = true;
        }
        residual.push_str(&request[sentence.span]);
    }
    let residual = residual.trim().to_owned();
    (!residual.is_empty())
        .then_some(())
        .and(target)
        .map(|target| DeliveryBinding {
            target,
            first_line,
            field_lines,
            residual,
        })
}

/// Literal `key=value` field constraints carried by one delivery sentence.
fn exact_field_lines(sentence: &str, target: &str) -> Vec<String> {
    let fields = sentence
        .split('`')
        .enumerate()
        .filter(|(index, _)| index % 2 == 1)
        .map(|(_, quoted)| quoted.trim())
        .filter(|quoted| *quoted != target)
        .filter_map(|quoted| {
            let (key, value) = quoted.split_once('=')?;
            if !key
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic())
                || !key
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                return None;
            }
            if !quoted.chars().any(char::is_whitespace) {
                return Some(quoted.to_owned());
            }
            // A prose description inside angle brackets declares an open field.
            let description = value.strip_prefix('<')?.strip_suffix('>')?;
            (!description.contains(['<', '>', '\r', '\n']) && !description.trim().is_empty())
                .then(|| quoted[..key.len() + 1].to_owned())
        })
        .collect::<Vec<_>>();

    // A field may be quoted once as a source reference and again as the output
    // field the caller declares: "`left_result=` followed by the child's
    // `result=` value, and `result=` followed by the composition". The final
    // occurrence is the declaration's position in the output list. Keep that
    // occurrence only, so the source reference cannot create an extra field or
    // move the aggregate ahead of later child fields.
    fields
        .iter()
        .enumerate()
        .filter(|(index, field)| !fields[index + 1..].contains(field))
        .map(|(_, field)| field.clone())
        .collect()
}

/// The part of a delivery sentence that is not about delivery.
///
/// Two sentences are the tidy way to ask for work and its delivery, and the
/// ladder's own nodes are written that way. English does not require it: "Break
/// the customer import rewrite into sub-tasks and record what you work out in
/// `import-split.md`" coordinates both halves into one sentence, and the
/// delivery half starts at the write action cue. Reading the sentence as
/// delivery and stopping there drops the work along with it, the residual comes
/// out empty, and the request is answered in the transcript with nothing
/// written -- which is the same silence issue #1066 is about, arrived at from
/// the other side.
///
/// Returns [`None`] when the sentence opens with its cue, which is the shape
/// where there is genuinely no work in front of the delivery.
fn work_before_delivery(sentence: &str) -> Option<&str> {
    let start = first_action_cue_start(&tokens(sentence))?;
    let work = without_trailing_separator(sentence.get(..start)?.trim());
    work.chars().any(char::is_alphanumeric).then_some(work)
}

/// `work` with the connective that introduced the delivery clause removed.
///
/// The connective belongs to the clause it opens, so it is delivery's, not the
/// work's. Which words those are is not a fact about writing files: it is the
/// vocabulary that ends one clause and starts the next, which the seed already
/// spells out in every registered language under
/// [`crate::seed::ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR`].
///
/// The separator has to be a whole word. "Run the import command record it in
/// `x.md`" ends its work on *command*, and a match on the last three letters
/// would hand back "Run the import comm".
fn without_trailing_separator(work: &str) -> &str {
    let lowered = work.to_lowercase();
    bare_surfaces(crate::seed::ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR)
        .iter()
        .filter(|separator| lowered.ends_with(separator.as_str()))
        .filter_map(|separator| work.get(..work.len() - separator.len()))
        .filter(|kept| kept.ends_with(char::is_whitespace))
        .map(str::trim_end)
        .min_by_key(|kept| kept.len())
        .unwrap_or(work)
}

/// Plan the next step of a "find this out and record it at PATH" request.
///
/// Three states, in the order they occur:
///
/// * the write has already been attempted — report what happened, truthfully;
/// * the router still wants tool calls for the residual — pass them through, so
///   the investigation runs under the ordinary routes;
/// * the router has an answer — write it to the named path, under the pinned
///   first line when the caller pinned one.
pub(super) fn plan_evidence_record_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let obligation = parse_obligation(task)?;
    // A later producer of the same target owns its bytes.
    if let Some(reason) = later_route_delivering(task, &obligation, tool_names) {
        trace_route("evidence_record", reason);
        return None;
    }
    let write_tool = tool_for(tool_names, Capability::Write)?;
    let progress = Progress::scan(messages);
    match super::task_obligations::observed_file_outcome(task, &obligation.target, messages) {
        ObligationOutcome::Satisfied { .. } => {
            let content = progress.successful_write_content_for(&obligation.target);
            if let Some(plan) = plan_record_readback_step(
                task,
                &obligation.target,
                content.as_deref(),
                &progress,
                tool_names,
                result,
            ) {
                return Some(plan);
            }
            if let Some(plan) = discovery_delivery::recorded_discovery_gap(
                &obligation,
                content.as_deref(),
                &progress,
                tool_names,
                task,
                result,
            ) {
                return Some(plan);
            }
            trace_route("evidence_record", "already_written");
            return Some(record(
                AgenticPlan::Final(
                    progress
                        .successful_write_content_for(&obligation.target)
                        .map(|content| written_observation(&obligation, &content))
                        .filter(|answer| !answer.is_empty())
                        .unwrap_or_else(|| {
                            format!("Recorded the findings in `{}`.", obligation.target)
                        }),
                ),
                FinalDisposition::Finding,
                "evidence_record_observed",
                result,
            ));
        }
        ObligationOutcome::Refuted { .. } | ObligationOutcome::Unsatisfiable { .. } => {
            trace_route("evidence_record", "write_not_observed");
            return Some(record(
                AgenticPlan::Final(format!(
                    "The findings could not be recorded in `{}`: the write step failed.",
                    obligation.target
                )),
                FinalDisposition::Failure,
                "evidence_record_write_failed",
                result,
            ));
        }
        ObligationOutcome::Unattempted => {}
    }
    // A successful checkout inspection is already the grounded answer this
    // delivery exists to persist. Optional research prose elsewhere in a
    // harness must not reopen the question on the web before the result is
    // written. Preserve recursive delivery ordering, though: when the residual
    // names another output, let that inner obligation consume the observation
    // first so one result still reaches every requested artifact.
    if let Some(plan) = discovery_delivery::plan_discovery_delivery(
        &obligation,
        &progress,
        tool_names,
        task,
        write_tool,
        result,
    ) {
        return Some(plan);
    }
    let observed = parse_obligation(&obligation.residual)
        .is_none()
        .then(|| {
            super::workspace_inspection::workspace_inspection_search_for_task(&obligation.residual)
        })
        .flatten()
        .and_then(|_| progress.latest_successful_output(Capability::Grep))
        .filter(|answer| !substantive_result_line(answer, &obligation.residual).is_empty())
        .map(str::to_owned);
    let residual_messages = with_residual_request(messages, &obligation.residual)?;
    let is_workspace_observation = observed.is_some();
    let answer = match observed {
        Some(answer) => answer,
        None => {
            if let Some(resolved) = plan_chat_step_resolved(&residual_messages, tool_names) {
                if !resolved.can_deliver() {
                    trace_route("evidence_record", "residual_not_finding");
                    return Some(resolved.into_plan(result));
                }
                let AgenticPlan::Final(answer) = resolved.plan else {
                    unreachable!()
                };
                if progress.latest_failure().is_some() {
                    return Some(record(
                        AgenticPlan::Final(answer),
                        FinalDisposition::Failure,
                        "evidence_record_residual_failed",
                        result,
                    ));
                }
                if resolved
                    .result
                    .is_some_and(|value| value.origin == VERIFIED_SEQUENCE_ORIGIN)
                {
                    sequence_observation(&answer, &obligation.residual)
                } else {
                    answer
                }
            } else {
                trace_route("evidence_record", "symbolic_residual");
                symbolic_answer(&obligation.residual)?.text
            }
        }
    };
    trace_route("evidence_record", &obligation.target);
    let delivered_answer = if is_workspace_observation {
        substantive_result_line(&answer, &obligation.residual)
    } else {
        &answer
    };
    let content = render_obligation(&obligation, delivered_answer);
    Some(plan_one(
        write_tool,
        write_arguments(&obligation.target, &content),
    ))
}

/// How far the probe below follows a route before concluding it delivers
/// nothing.
///
/// A route reaches the file the caller named early or not at all: the recipes
/// write their artifact first, and the literal-file composer records its plan
/// and then writes it. The bound is what keeps a routing question from costing
/// a whole run.
const DELIVERY_PROBE_TURNS: usize = 4;

#[path = "evidence_record/evidence_relevance.rs"]
mod evidence_relevance;
use evidence_relevance::later_route_delivering;

/// Whether planning the whole request through the routes below this one walks
/// into a call that writes `target`.
fn settled_route_delivers(task: &str, target: &str, tool_names: &[&str]) -> bool {
    let key = (
        task.to_owned(),
        target.to_owned(),
        tool_names.join("\u{1f}"),
    );
    if let Some(known) = DELIVERY_PROBE_ANSWERS.with_borrow(|answers| answers.get(&key).copied()) {
        return known;
    }
    let answer = probe_settled_routes(task, target, tool_names);
    DELIVERY_PROBE_ANSWERS.with_borrow_mut(|answers| {
        // The router is consulted once per turn with the same request, and a
        // long-lived server sees many requests. Recomputing per turn made the
        // issue #1028 ladder suite ten times slower; keeping every answer it
        // ever gave would trade that for unbounded memory. Start over instead --
        // the answers are derived, so losing them costs one recomputation.
        if answers.len() >= DELIVERY_PROBE_MEMO_CAPACITY {
            answers.clear();
        }
        answers.insert(key, answer);
    });
    answer
}

thread_local! {
    /// Probe answers already computed on this thread, keyed by the inputs they
    /// were computed from.
    static DELIVERY_PROBE_ANSWERS: RefCell<HashMap<(String, String, String), bool>> =
        RefCell::new(HashMap::new());
}

/// How many probe answers one thread remembers before starting over.
const DELIVERY_PROBE_MEMO_CAPACITY: usize = 512;

fn probe_settled_routes(task: &str, target: &str, tool_names: &[&str]) -> bool {
    let mut probe = vec![ChatMessage::user(task)];
    for turn in 0..DELIVERY_PROBE_TURNS {
        let Some(AgenticPlan::ToolCalls(calls)) =
            plan_settled_routes(task, &probe, tool_names, &mut None)
        else {
            return false;
        };
        for (index, call) in calls.iter().enumerate() {
            if planned_destination(&call.arguments).is_some_and(|path| path == target) {
                return true;
            }
            let id = format!("delivery-probe-{turn}-{index}");
            probe.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                &id,
                &call.tool,
                call.arguments.clone(),
            )]));
            probe.push(ChatMessage::tool_result(id, &call.tool, "ok"));
        }
    }
    false
}

/// The file a planned call names as its destination, whether the call writes
/// those bytes itself or runs a command that writes them.
fn planned_destination(arguments: &str) -> Option<String> {
    planned_write_path(arguments).or_else(|| planned_command_output_path(arguments))
}

/// The destination a planned command names for its own output.
///
/// A command carries its destination in the option that names it, exactly as a
/// write call carries it in `path`, and `formal-ai statement-audit --root .
/// --output statement-audit.lino` produces that file as surely as a write would.
/// A delivery that cannot see it peels "and write statement-audit.lino" off the
/// request and records its own status line over the report the command had just
/// written -- which is what issue #1069's agent-CLI audit run did, on the turn
/// after the audit succeeded.
fn planned_command_output_path(arguments: &str) -> Option<String> {
    let command = super::tool_result::command_argument(arguments)?;
    let parts = command.split_whitespace().collect::<Vec<_>>();
    let index = parts.iter().position(|part| *part == OUTPUT_OPTION)?;
    parts.get(index + 1).map(|path| (*path).to_owned())
}

/// The option a command names its output destination with.
const OUTPUT_OPTION: &str = "--output";

/// The file a planned call writes, under whichever key its client expects.
fn planned_write_path(arguments: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(arguments).ok()?;
    ["path", "filePath", "file_path", "absolute_path"]
        .iter()
        .find_map(|key| {
            value
                .get(*key)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
}

/// The concrete line of an answer that best addresses the requested fact.
///
/// Grouped grep output is transport-ordered, not relevance-ordered. Release
/// notes can therefore precede the source declaration a workspace question
/// asked for. Rank otherwise substantive lines by overlap with the request's
/// seed-derived fact terms. The final fact term is the requested property, and
/// a declaration answers a storage/representation question more directly than
/// a later method that happens to use that property.
fn substantive_result_line<'a>(answer: &'a str, task: &str) -> &'a str {
    let mut current_source_authority = 0;
    let mut candidates = Vec::new();
    for raw_line in answer.lines() {
        let line = raw_line.trim();
        if looks_like_result_path_heading(line) {
            current_source_authority = source_authority(line);
            continue;
        }
        if !line.is_empty()
            && line != "```text"
            && line != "```"
            && !line.ends_with("command completed. Output:")
            && !is_match_count(line)
        {
            candidates.push((line, current_source_authority.max(source_authority(line))));
        }
    }
    let terms = super::workspace_inspection::workspace_inspection_terms_for_task(task);
    let lexicon = crate::seed::lexicon();
    let condition_requested =
        lexicon.mentions_role(crate::seed::ROLE_CODING_CONDITION_SUBJECT_KIND, task);
    let implementation_requested = lexicon.mentions_role(
        crate::seed::ROLE_CODING_SOURCE_IMPLEMENTATION_SUBJECT_KIND,
        task,
    );
    let serialized_relationship = super::workspace_inspection::serialized_relationship_term(task);
    trace_route(
        "evidence_record_relationship",
        serialized_relationship.as_deref().unwrap_or("none"),
    );
    trace_route("evidence_record_terms", &terms.join(","));
    let Some(&(mut selected, selected_authority)) = candidates.first() else {
        return "";
    };
    let mut selected_score = relevance_score(
        selected,
        &terms,
        condition_requested,
        implementation_requested,
        serialized_relationship.as_deref(),
        selected_authority,
    );
    for (candidate, authority) in candidates.into_iter().skip(1) {
        let score = relevance_score(
            candidate,
            &terms,
            condition_requested,
            implementation_requested,
            serialized_relationship.as_deref(),
            authority,
        );
        if score > selected_score {
            selected = candidate;
            selected_score = score;
        }
    }
    trace_route("evidence_record_selected", selected);
    selected
}

use evidence_relevance::relevance_score;

/// Whether a source quotation is shaped as a boolean decision.
///
/// This intentionally recognises syntax classes rather than identifiers: a
/// condition may begin with a control-flow keyword or continue a compound
/// expression on its own line. Calls and declarations without a decision
/// operator remain ordinary source facts.
fn looks_like_condition(line: &str) -> bool {
    let source = source_text(line);
    if is_quoted_or_commented_source(source) {
        return false;
    }
    ["if ", "while ", "match ", "when "]
        .iter()
        .any(|prefix| source.starts_with(prefix))
        || ["&&", "||", "==", "!=", ">=", "<="]
            .iter()
            .any(|operator| source.contains(operator))
        || source.starts_with('!')
}

/// An instance-qualified predicate describes the invariant of the inspected
/// object more directly than a construction-time check of a similarly named
/// local value. Recognize common member-access syntax without depending on a
/// project identifier or a particular natural-language request.
fn looks_like_instance_condition(line: &str) -> bool {
    let source = source_text(line);
    ["self.", "this.", "self->", "this->", "$this->"]
        .iter()
        .any(|receiver| source.contains(receiver))
}

/// Local bindings can repeat the type of the model field a representation
/// question asks about. They are declarations, but a public/type-level
/// declaration is the more authoritative description when both are present.
fn looks_like_local_binding(line: &str) -> bool {
    let source = source_text(line);
    ["let ", "var ", "auto ", "local "]
        .iter()
        .any(|prefix| source.starts_with(prefix))
}

/// An exposed declaration describes the model's supported representation more
/// authoritatively than an unqualified function parameter with the same name
/// and type.
fn looks_like_exposed_declaration(line: &str) -> bool {
    let source = source_text(line);
    source.split_whitespace().next().is_some_and(|keyword| {
        matches!(keyword, "pub" | "public" | "export" | "exported") || keyword.starts_with("pub(")
    })
}

/// Recognize a source declaration without assuming a particular language.
///
/// Grep renders matches as `Line N: source`; discard that transport prefix,
/// then recognize an identifier-bearing left side followed by a single colon.
/// Namespace/generic punctuation such as `sum::<usize>()` is deliberately not
/// a declaration.
fn looks_like_declaration(line: &str) -> bool {
    let source = source_text(line);
    if is_quoted_or_commented_source(source) {
        return false;
    }
    let Some((left, right)) = source.split_once(':') else {
        return false;
    };
    !right.starts_with(':')
        && !left.contains(['(', ')', '=', '+'])
        && left.split_whitespace().next_back().is_some_and(|name| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
        })
}

fn source_text(line: &str) -> &str {
    line.split_once(": ")
        .map_or(line, |(_, source)| source)
        .trim()
}

fn is_quoted_or_commented_source(source: &str) -> bool {
    source.starts_with("//")
        || source.starts_with("/*")
        || source.starts_with('*')
        || source.starts_with('#')
        || source.starts_with("<!--")
        || source
            .chars()
            .next()
            .is_some_and(|character| matches!(character, '"' | '\'' | '`'))
}

fn is_match_count(line: &str) -> bool {
    line.strip_prefix("Found ")
        .and_then(|rest| rest.strip_suffix(" matches"))
        .is_some_and(|count| count.chars().all(|character| character.is_ascii_digit()))
}

fn looks_like_result_path_heading(line: &str) -> bool {
    line.ends_with(':') && (line.starts_with('/') || line.starts_with("./"))
}

/// Prefer a production source fact when an otherwise equivalent test merely
/// asserts that fact. If every match is outside `src`, the ordinary semantic
/// ranking remains decisive.
fn source_authority(path_or_line: &str) -> usize {
    usize::from(
        path_or_line
            .trim_end_matches(':')
            .split(['/', '\\'])
            .any(|component| component == "src"),
    )
}

/// What Formal AI answers about the residual, when it reaches a conclusion.
///
/// The layering is the one [`super::general_execution`] already uses: the
/// agentic planner is a client of the symbolic engine, not a replacement for
/// it. Declining on an inconclusive answer is what keeps the delivery honest --
/// the caller asked for what was found out, and "nothing was" is not something
/// to write to a file and call evidence.
///
/// An answer that defers to the open web
/// ([`crate::engine::SymbolicAnswer::defers_to_the_open_web`]) is declined for
/// the same reason, and it is the sharper of the two cases because the text
/// reads like prose about the subject. "Today's date is Sunday. Create a file
/// `main.py` that prints Hello, world!" put the first sentence to the engine,
/// which described the search it would run for it; delivering that description
/// wrote a paragraph about `DuckDuckGo` into the caller's Python file. Where the
/// engine would search, this route has nothing to record and stands aside, and
/// the request goes on to the routes that recognise it.
///
/// An answer that announces an enumeration and enumerates nothing
/// ([`crate::engine::SymbolicAnswer::announces_a_list_it_does_not_make`]) is
/// declined last, and it is the case this whole route is most exposed to: the
/// text is non-empty, so it survives every check made of the file afterwards,
/// and a harness reading the file finds a heading with no list and calls the
/// node proved.
fn symbolic_answer(residual: &str) -> Option<FinalResult> {
    // Any gap (`capability_gap`, `write_program_skill_gap`) is such a refusal
    // (PR #1188 G102).
    let answer = crate::engine::FormalAiEngine.answer(residual);
    // A gap refusal is a definite statement about the surface, not a finding
    // to record: left through, the delivery wrote the refusal into the file
    // the caller named (issue #1138).
    if answer.intent.ends_with(GAP_INTENT_SUFFIX)
        || answer.is_inconclusive()
        || answer.defers_to_the_open_web()
        || answer.announces_a_list_it_does_not_make()
    {
        return None;
    }
    let text = answer.answer.trim().to_owned();
    (!text.is_empty()).then_some(FinalResult {
        text,
        disposition: FinalDisposition::Finding,
        origin: answer.intent,
        payload_role: FinalPayloadRole::Finding,
        artifact: None,
        research: None,
        discovery: None,
    })
}

/// The suffix of an intent that reports a gap: a refusal, never content.
const GAP_INTENT_SUFFIX: &str = "_gap";

/// The same conversation with the served request reduced to `residual`.
///
/// Only the request text changes: every tool result the investigation has
/// already collected stays in place, so the residual is re-planned with the
/// progress it has actually made rather than from scratch. The turn reduced is
/// the request the run is serving -- the last user message that is not a
/// continuation cue. Reducing the cue itself (issue #1138) put the residual at
/// the end of the transcript, where every evidence window opened after it and
/// the investigation re-read what it had already read, forever.
fn with_residual_request(messages: &[ChatMessage], residual: &str) -> Option<Vec<ChatMessage>> {
    let index = messages.iter().rposition(|message| {
        message.role == "user"
            && !super::planner::is_continuation_cue(&message.content.plain_text())
    })?;
    let mut residual_messages = messages.to_vec();
    residual_messages[index].content = MessageContent::Text(residual.to_owned());
    // Every user message after the served request is a continuation ping
    // (the rposition above makes it so). Dropping them -- while keeping every
    // assistant turn and tool result in between -- leaves the residual as the
    // transcript's last user message. Left in place, the ping resumed the
    // standing task through its summary envelope, which re-entered this peel
    // with an identical residual and overflowed the stack (issue #1069).
    residual_messages.truncate(index + 1);
    residual_messages.extend(
        messages[index + 1..]
            .iter()
            .filter(|message| !message.role.eq_ignore_ascii_case("user"))
            .cloned(),
    );
    Some(residual_messages)
}
