//! Recording what an investigation found into a file the caller named (#1066).
//!
//! A caller can ask for two things at once: *find something out*, and *leave the
//! answer at a named place*. The Agent-CLI ladder does exactly that — "Inspect
//! the decomposition data model … Leave observable evidence in
//! `.agent-ladder/node-1.2-proof.md`. The first line must be exactly
//! `node_path=1.2`" — and so does any harness that reads a run's result from a
//! file rather than from the transcript.
//!
//! The literal-write route ([`super::general_planner`]) declines this shape: it
//! composes a write only when the request spells the bytes out, and here the
//! bytes are the *outcome* of work that has not happened yet.
//!
//! Left to the remaining routes, the named path is the only file-shaped token in
//! the request, so it was opened for reading and the run ended with the
//! evidence file never written.
//!
//! This module closes that gap without duplicating either half. It splits the
//! request into the delivery obligation (where the answer goes, and what its
//! first line must be) and the residual investigation, re-plans the residual
//! through the ordinary router, and turns the answer that router eventually
//! produces into the write the caller asked for.
//!
//! The agentic router is not the only thing that can answer the residual, and
//! for this shape of request it is usually not the thing that does. "Complete
//! recursive decomposition node 1.1.1, covering the four atomic tasks it names"
//! needs no tool: it is a question about task structure, which is what Formal
//! AI's symbolic engine answers. Delivering only what the *agentic* router produces
//! drops the obligation on every residual of that kind, and the request ends
//! with nothing written. So when the router has no plan, the residual is put to
//! [`crate::engine::FormalAiEngine`] and its answer delivered instead.
//!
//! An evidence file is still never invented. The engine's own verdict on
//! whether it reached a conclusion ([`crate::engine::SymbolicAnswer::is_inconclusive`],
//! [`crate::engine::SymbolicAnswer::defers_to_the_open_web`]) decides: an
//! unknown prompt, an ill-formed one, every clarification request and every
//! answer that only describes the web search it would run leave this route
//! declining exactly as before.

use super::capability_router::tool_for;
use super::planner::{
    AgenticPlan, Capability, plan_chat_step, plan_one, trace_route, write_arguments,
};
use super::progress::Progress;
use super::shell_command::carries_authoring_task;
use super::shell_command_policy::sentences;
use super::write_request::{
    bare_surfaces, first_action_cue_start, pinned_first_line, stated_write_target,
    states_write_action, tokens,
};
use crate::protocol::{ChatMessage, MessageContent};

/// The delivery half of a request, separated from the work it asks for.
struct Obligation {
    /// The path the answer has to be written to.
    target: String,
    /// The exact opening line the caller pinned, when it pinned one.
    first_line: Option<String>,
    /// Exact `key=value` lines declared in the delivery sentence.
    field_lines: Vec<String>,
    /// Everything the request says that is not about delivery.
    residual: String,
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
/// Declines when the residual is empty: a request whose every sentence is about
/// delivery states no work to do, so there is nothing to record.
fn parse_obligation(request: &str) -> Option<Obligation> {
    let mut target = None;
    let mut first_line = None;
    let mut field_lines = Vec::new();
    let mut residual = String::new();
    let mut later_obligation = false;
    for sentence in sentences(request) {
        if let Some(line) = pinned_first_line(sentence.text) {
            if target.is_some() && !later_obligation {
                first_line = first_line.or(Some(line));
                continue;
            }
            residual.push_str(&request[sentence.span]);
            continue;
        }
        let is_delivery = !carries_authoring_task(&crate::engine::normalize_prompt(sentence.text))
            && states_write_action(sentence.text);
        if is_delivery && let Some(named) = stated_write_target(sentence.text) {
            if target.is_none() {
                field_lines = exact_field_lines(sentence.text, &named);
                target = Some(named);
                if let Some(work) = work_before_delivery(sentence.text) {
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
        .map(|target| Obligation {
            target,
            first_line,
            field_lines,
            residual,
        })
}

/// Literal `key=value` field constraints carried by one delivery sentence.
fn exact_field_lines(sentence: &str, target: &str) -> Vec<String> {
    sentence
        .split('`')
        .enumerate()
        .filter(|(index, _)| index % 2 == 1)
        .map(|(_, quoted)| quoted.trim())
        .filter(|quoted| *quoted != target)
        .filter(|quoted| !quoted.chars().any(char::is_whitespace))
        .filter(|quoted| {
            quoted.split_once('=').is_some_and(|(key, _)| {
                key.chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_alphabetic())
                    && key
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_')
            })
        })
        .map(str::to_owned)
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
) -> Option<AgenticPlan> {
    let obligation = parse_obligation(task)?;
    let write_tool = tool_for(tool_names, Capability::Write)?;
    let progress = Progress::scan(messages);
    if progress.attempted_write_for(&obligation.target) {
        trace_route("evidence_record", "already_written");
        return Some(AgenticPlan::Final(
            if progress.successful_write_for(&obligation.target) {
                progress
                    .successful_write_content_for(&obligation.target)
                    .map(|content| written_observation(&obligation, &content))
                    .filter(|answer| !answer.is_empty())
                    .unwrap_or_else(|| format!("Recorded the findings in `{}`.", obligation.target))
            } else {
                format!(
                    "The findings could not be recorded in `{}`: the write step failed.",
                    obligation.target
                )
            },
        ));
    }
    // A successful checkout inspection is already the grounded answer this
    // delivery exists to persist. Optional research prose elsewhere in a
    // harness must not reopen the question on the web before the result is
    // written. Preserve recursive delivery ordering, though: when the residual
    // names another output, let that inner obligation consume the observation
    // first so one result still reaches every requested artifact.
    let observed = parse_obligation(&obligation.residual)
        .is_none()
        .then(|| {
            super::shell_command::workspace_inspection_search_for_task(&obligation.residual)
        })
        .flatten()
        .and_then(|_| progress.latest_successful_output(Capability::Grep))
        .filter(|answer| !substantive_result_line(answer, &obligation.residual).is_empty())
        .map(str::to_owned);
    let residual_messages = with_residual_request(messages, &obligation.residual)?;
    let is_workspace_observation = observed.is_some();
    let answer = match observed {
        Some(answer) => answer,
        None => match plan_chat_step(&residual_messages, tool_names) {
            Some(plan @ AgenticPlan::ToolCalls(_)) => {
                trace_route("evidence_record", "investigating");
                return Some(plan);
            }
            Some(AgenticPlan::Final(answer)) => answer,
            None => {
                trace_route("evidence_record", "symbolic_residual");
                symbolic_answer(&obligation.residual)?
            }
        },
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

/// Render an answer under the constraints attached to its destination.
fn render_obligation(obligation: &Obligation, answer: &str) -> String {
    if !obligation.field_lines.is_empty() {
        let result = substantive_result_line(answer, &obligation.residual);
        let mut rendered = String::new();
        for field in &obligation.field_lines {
            rendered.push_str(field);
            if field.ends_with('=') {
                rendered.push_str(result);
            }
            rendered.push('\n');
        }
        return rendered;
    }
    obligation.first_line.as_ref().map_or_else(
        || format!("{}\n", answer.trim_end()),
        |line| format!("{line}\n\n{}\n", answer.trim_end()),
    )
}

/// The observation carried by a successfully written evidence artifact.
///
/// Remove only the destination's machine header. The remaining bytes are the
/// grounded answer the nested delivery wrote and can feed another artifact.
fn written_observation(obligation: &Obligation, content: &str) -> String {
    let mut lines = content.lines();
    if obligation
        .first_line
        .as_deref()
        .is_some_and(|first_line| lines.clone().next().is_some_and(|line| line.trim() == first_line))
    {
        lines.next();
    }
    lines.collect::<Vec<_>>().join("\n").trim().to_owned()
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
    let terms = super::shell_command::workspace_inspection_terms_for_task(task);
    let lexicon = crate::seed::lexicon();
    let condition_requested =
        lexicon.mentions_role(crate::seed::ROLE_CODING_CONDITION_SUBJECT_KIND, task);
    let implementation_requested = lexicon.mentions_role(
        crate::seed::ROLE_CODING_SOURCE_IMPLEMENTATION_SUBJECT_KIND,
        task,
    );
    let Some(&(mut selected, selected_authority)) = candidates.first() else {
        return "";
    };
    let mut selected_score = relevance_score(
        selected,
        &terms,
        condition_requested,
        implementation_requested,
        selected_authority,
    );
    for (candidate, authority) in candidates.into_iter().skip(1) {
        let score = relevance_score(
            candidate,
            &terms,
            condition_requested,
            implementation_requested,
            authority,
        );
        if score > selected_score {
            selected = candidate;
            selected_score = score;
        }
    }
    selected
}

fn relevance_score(
    line: &str,
    terms: &[String],
    condition_requested: bool,
    implementation_requested: bool,
    source_authority: usize,
) -> (usize, usize, usize, usize, usize, usize, usize, usize, usize) {
    let words = line
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    let matches = |term: &str| {
        words.iter().any(|word| word == term)
            || term
                .split('_')
                .all(|part| words.iter().any(|word| word == part))
    };
    let overlap = terms
        .iter()
        .filter(|term| matches(term))
        .count();
    let requested_property = usize::from(terms.last().is_some_and(|term| matches(term)));
    let declaration_shape = if looks_like_declaration(line) {
        1 + usize::from(!looks_like_local_binding(line))
            + usize::from(looks_like_exposed_declaration(line))
    } else {
        0
    };
    let bound_requested = terms.iter().any(|term| {
        crate::seed::lexicon().mentions_role(crate::seed::ROLE_CODING_BOUND_CUE, term)
    });
    let semantic_overlap = usize::from(
        bound_requested
            && words.iter().any(|word| {
                crate::seed::lexicon()
                    .mentions_role(crate::seed::ROLE_CODING_BOUND_CUE, word)
            }),
    );
    let code_shape = usize::from(line.contains(['{', '}', '(', ')', ';', '=', '<', '>']));
    (
        usize::from(
            implementation_requested && !is_quoted_or_commented_source(source_text(line)),
        ),
        requested_property,
        usize::from(condition_requested && looks_like_condition(line)),
        source_authority,
        usize::from(condition_requested && looks_like_instance_condition(line)),
        declaration_shape,
        semantic_overlap,
        overlap,
        code_shape,
    )
}

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
        matches!(keyword, "pub" | "public" | "export" | "exported")
            || keyword.starts_with("pub(")
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
        && left
            .split_whitespace()
            .next_back()
            .is_some_and(|name| {
                !name.is_empty()
                    && name
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_')
            })
}

fn source_text(line: &str) -> &str {
    line
        .split_once(": ")
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
fn symbolic_answer(residual: &str) -> Option<String> {
    let answer = crate::engine::FormalAiEngine.answer(residual);
    if answer.is_inconclusive()
        || answer.defers_to_the_open_web()
        || answer.announces_a_list_it_does_not_make()
    {
        return None;
    }
    let text = answer.answer.trim().to_owned();
    (!text.is_empty()).then_some(text)
}

/// The same conversation with the latest user turn reduced to `residual`.
///
/// Only the request text changes: every tool result the investigation has
/// already collected stays in place, so the residual is re-planned with the
/// progress it has actually made rather than from scratch.
fn with_residual_request(messages: &[ChatMessage], residual: &str) -> Option<Vec<ChatMessage>> {
    let index = messages.iter().rposition(|message| message.role == "user")?;
    let mut residual_messages = messages.to_vec();
    residual_messages[index].content = MessageContent::Text(residual.to_owned());
    Some(residual_messages)
}
