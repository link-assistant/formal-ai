//! Reading a turn's tool results back out of the transcript (issue #468).
//!
//! The planner is stateless: it re-derives what to do next from the conversation
//! alone, so "what has already been tried this turn" has to be *read* rather than
//! remembered. That reading is this module, and it is a separate concern from
//! choosing the next step — [`Progress::scan`] answers what happened, and
//! `planner` decides what happens next.

use super::capability_router::classify_tool;
use super::planner::Capability;
use crate::protocol::ChatMessage;

/// One observed client-owned tool execution in transcript order.
#[derive(Debug)]
pub(super) struct ToolAttempt {
    pub(super) capability: Capability,
    source_read: Option<super::tool_result::SourceReadObservation>,
    pub(super) succeeded: bool,
    pub(super) detail: String,
    pub(super) arguments: Option<String>,
    /// The advertised tool name the result came back under, when known.
    pub(super) tool: Option<String>,
}

impl ToolAttempt {
    /// Whether this attempt read a work item through the shell (`gh issue view …`, or one of the REST fallbacks of issue #1155).
    pub(super) fn is_work_item_read(&self) -> bool {
        self.capability == Capability::Run
            && self
                .arguments
                .as_deref()
                .and_then(super::tool_result::command_argument)
                .is_some_and(|command| is_work_item_read_command(&command))
    }
}

/// Tool results produced since the current user turn began.
pub struct Progress {
    /// Observed results, including failures. Existing recipes use this to move
    /// to their fallback or rendering phase after a client-owned attempt.
    completed: Vec<Capability>,
    attempts: Vec<ToolAttempt>,
    pub(super) fetched_text: Option<String>,
    pub(super) fetched_pages: Vec<(String, String)>,
    pub(super) attempted_fetches: Vec<String>,
    /// Work-item URLs already read through a shell read call.
    attempted_work_item_reads: Vec<String>,
    /// Work-item URLs whose shell read failed, with the reason (issue #1155).
    failed_work_item_reads: Vec<(String, String)>,
    pub(super) search_output: Option<String>,
    /// Every shell result of this turn, in arrival order.
    pub(super) run_outputs: Vec<String>,
    /// Shell observations keyed by the exact command supplied to the client.
    run_observations: Vec<(String, String)>,
    pub(super) search_result: Option<String>,
}

impl Progress {
    pub(super) fn scan(messages: &[ChatMessage]) -> Self {
        let mut completed = Vec::new();
        let mut attempts = Vec::new();
        let mut fetched_text = None;
        let mut fetched_pages = Vec::new();
        let mut attempted_fetches = Vec::new();
        let mut attempted_work_item_reads = Vec::new();
        let mut failed_work_item_reads = Vec::new();
        let mut search_output = None;
        let mut run_outputs = Vec::new();
        let mut run_observations = Vec::new();
        let mut search_result = None;
        // Ignore results from earlier user turns -- except a continuation cue,
        // which resumes the standing run instead of opening a new request, so
        // the results gathered for that run stay visible across its pings
        // (issue #1138).
        let current_turn = super::planner::evidence_window_start(messages);
        for (index, message) in messages.iter().enumerate().skip(current_turn) {
            if !message.role.eq_ignore_ascii_case("tool") {
                continue;
            }
            let Some(capability) = result_capability(messages, index) else {
                continue;
            };
            let raw = message.content.plain_text();
            let call = result_tool_call(messages, index);
            let source_read = call
                .filter(|call| {
                    capability == Capability::Read
                        && classify_tool(&call.function.name) == Some(Capability::Read)
                        && source_read_path(&call.function.arguments).is_some()
                        && message
                            .name
                            .as_ref()
                            .is_none_or(|name| name.eq_ignore_ascii_case(&call.function.name))
                })
                .map(|call| {
                    super::tool_result::source_read_observation(
                        &raw,
                        message.is_error,
                        message.source_read.as_ref(),
                        &source_read_path(&call.function.arguments).unwrap_or_default(),
                    )
                });
            let failure = source_read.as_ref().map_or_else(
                || {
                    if matches!(capability, Capability::Read | Capability::Fetch) {
                        message.is_error.then(|| raw.clone())
                    } else {
                        super::tool_result::failure_message(
                            &raw,
                            message.is_error,
                            capability != Capability::Run,
                        )
                    }
                },
                |observation| observation.error.clone(),
            );
            let arguments =
                result_tool_call(messages, index).map(|call| call.function.arguments.clone());
            let tool = message.name.clone().or_else(|| {
                result_tool_call(messages, index).map(|call| call.function.name.clone())
            });
            attempts.push(ToolAttempt {
                capability,
                source_read,
                succeeded: failure.is_none(),
                detail: failure.clone().unwrap_or_else(|| raw.clone()),
                arguments,
                tool,
            });
            if capability == Capability::Fetch {
                let payload = Some(raw.clone());
                let fetch_url = result_tool_call(messages, index).and_then(fetch_call_url);
                if let Some(url) = fetch_url.as_ref()
                    && !attempted_fetches.contains(url)
                {
                    attempted_fetches.push(url.clone());
                }
                if let (Some(reason), Some(url)) = (failure.as_ref(), fetch_url.as_ref()) {
                    failed_work_item_reads.push((url.clone(), reason.clone()));
                }
                if failure.is_none()
                    && let Some(text) = payload.filter(|text| !text.trim().is_empty())
                {
                    if let Some(url) = fetch_url {
                        fetched_pages.push((url, text.clone()));
                    }
                    fetched_text = Some(text);
                }
            }
            if capability == Capability::Search {
                let payload = super::tool_result::normalized_payload(&raw);
                search_result = Some(payload.clone().unwrap_or_default());
                if let Some(text) = payload.filter(|text| !text.trim().is_empty()) {
                    search_output = Some(text);
                }
            }
            if capability == Capability::Run {
                let command = result_tool_call(messages, index).and_then(|call| {
                    super::tool_result::command_argument(&call.function.arguments)
                });
                if let Some(command) = &command {
                    run_observations.push((command.clone(), raw.clone()));
                }
                // A work item read through the client's own `gh` is a fetched
                // page like any other (issue #1133): the command names the
                // URL, and its output is the issue's title and body.
                if let Some(url) = command.as_deref().and_then(work_item_read_url) {
                    // The attempt is recorded whatever it returned: a read
                    // that came back empty has still been tried, and planning
                    // it again would loop (issue #1133).
                    if !attempted_work_item_reads.contains(&url) {
                        attempted_work_item_reads.push(url.clone());
                    }
                    // The read is only page evidence when the command itself
                    // says it succeeded and the output looks like an issue
                    // (issue #1155): clients echo tool output with the status
                    // dropped, which once let an unauthenticated `gh`'s
                    // how-to-authenticate banner pass for an issue body.
                    if let Some(reason) = command.as_deref().and_then(|command| {
                        work_item_read_failure_reason(command, &raw, failure.as_deref())
                    }) {
                        failed_work_item_reads.push((url.clone(), reason));
                    } else if failure.is_none()
                        && let Some(text) = super::tool_result::normalized_payload(&raw)
                            .filter(|text| !text.trim().is_empty())
                    {
                        fetched_pages.push((url, without_exit_sentinel(&text)));
                    }
                }
                run_outputs.push(raw);
            }
            completed.push(capability);
        }
        Self {
            completed,
            attempts,
            fetched_text,
            fetched_pages,
            attempted_fetches,
            attempted_work_item_reads,
            failed_work_item_reads,
            search_output,
            run_outputs,
            run_observations,
            search_result,
        }
    }

    /// Whether this turn already tried to fetch `url`.
    pub(super) fn attempted_fetch_of(&self, url: &str) -> bool {
        self.attempted_fetches
            .iter()
            .any(|attempted| attempted == url)
            || self.attempts.iter().any(|attempt| {
                attempt.capability == Capability::Fetch
                    && attempt
                        .arguments
                        .as_deref()
                        .is_none_or(|arguments| argument_url(arguments).is_none())
            })
    }

    /// Whether this turn already tried to read `url` with `gh issue|pr view`.
    pub(super) fn attempted_work_item_read_of(&self, url: &str) -> bool {
        self.attempted_work_item_reads
            .iter()
            .any(|attempted| attempted == url)
    }

    /// Why the work item at `url` could not be read, when no read of it succeeded (issue #1155).
    pub(super) fn failed_work_item_read_of(&self, url: &str) -> Option<&str> {
        if self.fetched_pages.iter().any(|(fetched, _)| fetched == url) {
            return None;
        }
        self.failed_work_item_reads
            .iter()
            .rev()
            .find(|(failed, _)| failed == url)
            .map(|(_, reason)| reason.as_str())
    }

    /// Every read attempted for `url` this turn, as report lines — the command (or fetch) tried, what it answered, and why it was not page evidence.
    pub(super) fn work_item_read_attempts(&self, url: &str) -> Vec<String> {
        // Failed reads retain source order; consume Fetch reasons as well so
        // each following shell read keeps its own observed failure.
        let mut reasons = self
            .failed_work_item_reads
            .iter()
            .filter(|(failed, _)| failed == url);
        let mut lines = Vec::new();
        for attempt in &self.attempts {
            match attempt.capability {
                Capability::Run => {
                    let Some(command) = attempt
                        .arguments
                        .as_deref()
                        .and_then(super::tool_result::command_argument)
                    else {
                        continue;
                    };
                    let targets_url = issue_view_command_url(&command)
                        .or_else(|| rest_read_url(&command))
                        .is_some_and(|read| read == url);
                    if targets_url {
                        let outcome = attempt.detail.lines().next().unwrap_or("").trim();
                        let reason = reasons.next().map(|(_, reason)| reason.as_str());
                        lines.push(reason.map_or_else(
                            || {
                                super::work_item_steps::fill(
                                    "read_attempt_run_line",
                                    &[
                                        (concat!("{", "command}"), command.as_str()),
                                        (concat!("{", "outcome}"), outcome),
                                    ],
                                )
                            },
                            |reason| {
                                super::work_item_steps::fill(
                                    "read_attempt_run_reason_line",
                                    &[
                                        (concat!("{", "command}"), command.as_str()),
                                        (concat!("{", "reason}"), reason),
                                        (concat!("{", "outcome}"), outcome),
                                    ],
                                )
                            },
                        ));
                    }
                }
                Capability::Fetch => {
                    // A fetch echoed without its `url` counts for the one page
                    // this turn planned, the same reading
                    // [`Progress::attempted_fetch_of`] makes — but only when
                    // it is the only fetch, so an unrelated fetch never gets
                    // reported as a read of this work item.
                    let fetches = self
                        .attempts
                        .iter()
                        .filter(|attempt| attempt.capability == Capability::Fetch)
                        .count();
                    let for_this_url = attempt
                        .arguments
                        .as_deref()
                        .and_then(argument_url)
                        .is_some_and(|fetched| fetched == url)
                        || attempt
                            .arguments
                            .as_deref()
                            .is_none_or(|arguments| argument_url(arguments).is_none())
                            && fetches == 1;
                    if for_this_url {
                        let outcome = attempt.detail.lines().next().unwrap_or("").trim();
                        lines.push(super::work_item_steps::fill(
                            "read_attempt_fetch_line",
                            &[
                                (concat!("{", "url}"), url),
                                (concat!("{", "outcome}"), outcome),
                            ],
                        ));
                        if !attempt.succeeded
                            && attempt
                                .arguments
                                .as_deref()
                                .and_then(argument_url)
                                .is_some_and(|fetched| fetched == url)
                        {
                            let _ = reasons.next();
                        }
                    }
                }
                _ => {}
            }
        }
        lines
    }

    /// Whether this turn already ran `command` exactly as given.
    pub(super) fn has_run(&self, command: &str) -> bool {
        self.run_observations.iter().any(|(ran, _)| ran == command)
    }

    /// The prior attempt one planned call would repeat, if any (issue #1154).
    pub(super) fn repeated_call(
        &self,
        call: &super::planner::PlannedToolCall,
    ) -> Option<&ToolAttempt> {
        let index = self.attempts.iter().rposition(|attempt| {
            if !attempt.succeeded {
                return false;
            }
            let same_tool = attempt
                .tool
                .as_deref()
                .is_some_and(|tool| tool.eq_ignore_ascii_case(&call.tool))
                || attempt.tool.is_none()
                    && super::planner::tool_capability(&call.tool) == Some(attempt.capability);
            same_tool
                && (attempt.arguments.as_deref() == Some(call.arguments.as_str())
                    || Self::same_run_operand(attempt, call))
        })?;
        let progressed = self.attempts[index + 1..]
            .iter()
            .enumerate()
            .any(|(offset, later)| {
                !self.attempts[..=index + offset]
                    .iter()
                    .any(|earlier| Self::same_attempt(earlier, later))
            });
        if progressed {
            None
        } else {
            self.attempts.get(index)
        }
    }

    /// Whether two recorded attempts made the same call: one tool (or one
    /// capability when a tool name is unknown) with the same arguments.
    fn same_attempt(earlier: &ToolAttempt, later: &ToolAttempt) -> bool {
        let same_tool = match (earlier.tool.as_deref(), later.tool.as_deref()) {
            (Some(first), Some(second)) => first.eq_ignore_ascii_case(second),
            _ => earlier.capability == later.capability,
        };
        same_tool && earlier.arguments == later.arguments
    }

    /// Whether an attempt and a planned call address one shell command or one
    /// URL under the client's own argument spellings.
    fn same_run_operand(attempt: &ToolAttempt, call: &super::planner::PlannedToolCall) -> bool {
        let attempted_command = attempt
            .arguments
            .as_deref()
            .and_then(super::tool_result::command_argument);
        let planned_command = super::tool_result::command_argument(&call.arguments);
        if attempted_command.is_some()
            && attempted_command.is_some_and(|command| planned_command == Some(command))
        {
            return true;
        }
        let attempted_url = attempt.arguments.as_deref().and_then(argument_url);
        attempted_url
            .is_some_and(|url| argument_url(&call.arguments).is_some_and(|planned| planned == url))
    }

    /// The latest failed attempt that came back under `tool`.
    pub(super) fn latest_failure_of_tool(&self, tool: &str) -> Option<&ToolAttempt> {
        self.attempts
            .iter()
            .rev()
            .find(|attempt| !attempt.succeeded && attempt.tool.as_deref() == Some(tool))
    }

    /// How many times `tool` has failed this turn with the same report as its
    /// latest failure. Two is the signal that repeating it is not a plan.
    pub(super) fn identical_failures_of(&self, tool: &str) -> usize {
        let Some(latest) = self
            .attempts
            .iter()
            .rev()
            .find(|attempt| !attempt.succeeded && attempt.tool.as_deref() == Some(tool))
        else {
            return 0;
        };
        self.attempts
            .iter()
            .filter(|attempt| {
                !attempt.succeeded
                    && attempt.tool.as_deref() == Some(tool)
                    && attempt.detail == latest.detail
            })
            .count()
    }

    /// Whether a prior tool result already covered `capability`.
    pub(super) fn done(&self, capability: Capability) -> bool {
        self.completed.contains(&capability)
    }

    pub(super) fn count(&self, capability: Capability) -> usize {
        self.completed
            .iter()
            .filter(|done| **done == capability)
            .count()
    }

    /// Most recent successful result payload for one capability.
    pub(super) fn latest_successful_output(&self, capability: Capability) -> Option<&str> {
        self.attempts
            .iter()
            .rev()
            .find(|attempt| attempt.capability == capability && attempt.succeeded)
            .map(|attempt| attempt.detail.as_str())
    }

    /// Arguments of the most recent successful attempt for one capability,
    /// this turn. The transcript's own record of what was asked for.
    pub(super) fn latest_successful_arguments(&self, capability: Capability) -> Option<&str> {
        self.attempts
            .iter()
            .rev()
            .find(|attempt| attempt.capability == capability && attempt.succeeded)
            .and_then(|attempt| attempt.arguments.as_deref())
    }

    /// Whether the recipe the latest successful `capability` attempt opened is still progressing: nothing attempted after it has failed.
    pub(super) fn latest_success_unstalled(&self, capability: Capability) -> bool {
        let Some(index) = self
            .attempts
            .iter()
            .rposition(|attempt| attempt.capability == capability && attempt.succeeded)
        else {
            return false;
        };
        self.attempts[index + 1..]
            .iter()
            .all(|attempt| attempt.succeeded)
    }

    /// Number of run attempts for one exact command, successful or failed.
    pub(super) fn run_count_for(&self, command: &str) -> usize {
        self.attempts
            .iter()
            .filter(|attempt| run_attempt_matches(attempt, command))
            .count()
    }

    /// Number of successful run attempts for one exact command.
    pub(super) fn successful_run_count_for(&self, command: &str) -> usize {
        self.attempts
            .iter()
            .filter(|attempt| attempt.succeeded && run_attempt_matches(attempt, command))
            .count()
    }

    /// Most recent successful output from one exact command.
    pub(super) fn latest_successful_run_output_for(&self, command: &str) -> Option<&str> {
        self.attempts.iter().rev().find_map(|attempt| {
            (attempt.succeeded && run_attempt_matches(attempt, command))
                .then_some(attempt.detail.as_str())
        })
    }

    /// Most recent raw observation from one exact command.
    pub(super) fn latest_run_output_for(&self, command: &str) -> Option<&String> {
        self.run_observations
            .iter()
            .rev()
            .find_map(|(attempted, output)| (attempted == command).then_some(output))
    }

    /// Successful read payload for one concrete workspace path.
    pub(super) fn successful_read_output_for(&self, path: &str) -> Option<&str> {
        self.attempts.iter().rev().find_map(|attempt| {
            (attempt.capability == Capability::Read
                && attempt.succeeded
                && attempt
                    .arguments
                    .as_deref()
                    .is_some_and(|arguments| argument_targets(arguments, path)))
            .then_some(attempt.detail.as_str())
        })
    }

    /// Latest exact current-window source observation, without inventing completion.
    pub(super) fn source_read_for(
        &self,
        path: &str,
    ) -> Option<&super::tool_result::SourceReadObservation> {
        self.attempts
            .iter()
            .rev()
            .find(|attempt| {
                attempt.capability == Capability::Read
                    && attempt
                        .arguments
                        .as_deref()
                        .and_then(source_read_path)
                        .as_deref()
                        == Some(path)
            })
            .and_then(|attempt| attempt.source_read.as_ref())
    }

    pub(super) fn attempted_write_for(&self, path: &str) -> bool {
        self.attempts.iter().any(|attempt| {
            attempt.capability == Capability::Write
                && attempt
                    .arguments
                    .as_deref()
                    .is_some_and(|arguments| argument_targets(arguments, path))
        })
    }

    pub(super) fn successful_write_for(&self, path: &str) -> bool {
        self.attempts.iter().any(|attempt| {
            attempt.capability == Capability::Write
                && attempt.succeeded
                && attempt
                    .arguments
                    .as_deref()
                    .is_some_and(|arguments| argument_targets(arguments, path))
        })
    }

    /// Observation window following the current successful write of this path.
    pub(super) fn attempts_after_latest_write(&self, path: &str) -> Option<&[ToolAttempt]> {
        let index = self.latest_successful_write_index(path)?;
        Some(&self.attempts[index + 1..])
    }

    /// Content supplied to the latest successful write of `path`.
    pub(super) fn successful_write_content_for(&self, path: &str) -> Option<String> {
        self.attempts.iter().rev().find_map(|attempt| {
            (attempt.capability == Capability::Write && attempt.succeeded)
                .then_some(attempt.arguments.as_deref())
                .flatten()
                .filter(|arguments| argument_targets(arguments, path))
                .and_then(argument_content)
        })
    }

    /// The arrival index of the latest successful write of `path`, so a caller can tell which of two files was written last (issue #1185 R3: a fix already rendered into the artifact is not rendered twice).
    pub(super) fn latest_successful_write_index(&self, path: &str) -> Option<usize> {
        self.attempts.iter().rposition(|attempt| {
            attempt.capability == Capability::Write
                && attempt.succeeded
                && attempt
                    .arguments
                    .as_deref()
                    .is_some_and(|arguments| argument_targets(arguments, path))
        })
    }

    /// The capability of the most recent tool result in this turn.
    pub(super) fn last(&self) -> Option<Capability> {
        self.completed.last().copied()
    }

    /// The latest observed result when it failed. Successful observations do
    /// not leave an older failure active.
    pub(super) fn latest_failure(&self) -> Option<&ToolAttempt> {
        self.attempts.last().filter(|attempt| !attempt.succeeded)
    }

    pub(super) fn previous_attempt(&self) -> Option<&ToolAttempt> {
        self.attempts.iter().rev().nth(1)
    }

    /// Bound retries per concrete write target, rather than globally across a
    /// multi-file recipe.
    pub(super) fn failed_write_count_for(&self, path: &str) -> usize {
        self.attempts
            .iter()
            .filter(|attempt| {
                attempt.capability == Capability::Write
                    && !attempt.succeeded
                    && attempt
                        .arguments
                        .as_deref()
                        .is_some_and(|arguments| argument_targets(arguments, path))
            })
            .count()
    }

    pub(super) fn search_result(&self) -> Option<&str> {
        self.search_result.as_deref()
    }
}

/// Conflicting naming fields cannot bind an observation.
fn source_read_path(arguments: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(arguments).ok()?;
    let object = value.as_object()?;
    let fields = ["path", "filePath", "file_path"]
        .iter()
        .filter_map(|key| object.get(*key))
        .collect::<Vec<_>>();
    let first = fields.first()?.as_str()?;
    (!first.trim().is_empty() && fields.iter().all(|field| field.as_str() == Some(first)))
        .then(|| first.to_owned())
}

fn argument_path(arguments: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(arguments).ok()?;
    ["path", "filePath", "file_path"]
        .iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_str))
        .map(str::to_owned)
}

fn argument_content(arguments: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(arguments).ok()?;
    ["content", "contents", "text", "new_string"]
        .iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_str))
        .map(str::to_owned)
}

/// Verify the bytes and destination of a write, including a freeform patch
/// lowered by the protocol adapter. Tool success alone cannot bind operands.
pub(super) fn write_matches(arguments: &str, path: &str, content: &str) -> bool {
    (argument_targets(arguments, path) && argument_content(arguments).as_deref() == Some(content))
        || crate::protocol_responses::apply_patch_input(&super::planner::write_arguments(
            path, content,
        ))
        .is_some_and(|patch| arguments.trim() == patch.trim())
}

fn argument_targets(arguments: &str, path: &str) -> bool {
    argument_path(arguments).is_some_and(|observed| {
        observed == path
            || (std::path::Path::new(path).is_relative()
                && std::path::Path::new(&observed).ends_with(path))
    })
}

/// Resolve which capability the tool result at `index` answers.
pub(super) fn result_capability(messages: &[ChatMessage], index: usize) -> Option<Capability> {
    let message = &messages[index];
    if let Some(name) = &message.name
        && let Some(capability) = classify_tool(name)
    {
        return Some(capability);
    }
    result_tool_call(messages, index).and_then(|call| classify_tool(&call.function.name))
}

fn result_tool_call(messages: &[ChatMessage], index: usize) -> Option<&crate::protocol::ToolCall> {
    let call_id = messages[index].tool_call_id.as_ref()?;
    let start = super::planner::evidence_window_start(messages);
    messages
        .get(start..index)?
        .iter()
        .rev()
        .flat_map(|prior| prior.tool_calls.iter())
        .find(|call| &call.id == call_id)
}

fn fetch_call_url(call: &crate::protocol::ToolCall) -> Option<String> {
    argument_url(&call.function.arguments)
}

fn argument_url(arguments: &str) -> Option<String> {
    let arguments: serde_json::Value = serde_json::from_str(arguments).ok()?;
    arguments
        .get("url")
        .and_then(serde_json::Value::as_str)
        .filter(|url| !url.trim().is_empty())
        .map(str::to_owned)
}

fn run_attempt_matches(attempt: &ToolAttempt, command: &str) -> bool {
    attempt.capability == Capability::Run
        && attempt
            .arguments
            .as_deref()
            .and_then(super::tool_result::command_argument)
            .is_some_and(|attempted| attempted == command)
}

fn is_issue_view_command(command: &str) -> bool {
    let mut words = command.split_whitespace();
    words.next() == Some("gh")
        && matches!(words.next(), Some("issue" | "pr"))
        && words.next() == Some("view")
}

/// Whether a shell command is one of the reads this planner plans for a work
/// item: the `gh` view read, or a REST fallback of issue #1155.
fn is_work_item_read_command(command: &str) -> bool {
    is_issue_view_command(command)
        || (command.starts_with("gh api ") || command.starts_with("curl "))
            && rest_read_url(command).is_some()
}

/// The GitHub work-item URL a shell read command targets, normalized to its `github.com` form.
fn work_item_read_url(command: &str) -> Option<String> {
    issue_view_command_url(command).or_else(|| rest_read_url(command))
}

/// The work-item URL of a `gh issue|pr view <url> …` command, when it is one.
fn issue_view_command_url(command: &str) -> Option<String> {
    let mut words = command.split_whitespace();
    if words.next()? != "gh" {
        return None;
    }
    if !matches!(words.next()?, "issue" | "pr") || words.next()? != "view" {
        return None;
    }
    words
        .next()
        .and_then(super::general_planner::repository_work_reference)
}

/// The work-item URL a `repos/{owner}/{repo}/{kind}/{number}` API path reads.
fn rest_read_url(command: &str) -> Option<String> {
    let start = command.find("repos/")? + "repos/".len();
    let mut segments = command[start..].split('/');
    let owner = segments.next().filter(|owner| !owner.is_empty())?;
    let repo = segments.next().filter(|repo| !repo.is_empty())?;
    let kind = match segments.next()? {
        "issues" => "issues",
        "pulls" => "pulls",
        _ => return None,
    };
    // The number segment runs into the `; printf` sentinel the read commands
    // carry, so only its leading digits are the number.
    let number: String = segments
        .next()?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    if number.is_empty() {
        return None;
    }
    // GitHub prints an issue as `/issues/{n}` and a pull request as
    // `/pull/{n}`; folding `issues` onto a singular `issue` named a page no
    // plan targets, so the REST reads were never matched to their work item.
    let kind = if kind == "pulls" { "pull" } else { "issues" };
    Some(format!("https://github.com/{owner}/{repo}/{kind}/{number}"))
}

/// The exit status a read command printed about itself, when it printed one.
fn exit_sentinel(raw: &str) -> Option<i32> {
    raw.lines()
        .filter_map(|line| line.trim().strip_prefix("__formal_ai_exit="))
        .filter_map(|digits| digits.parse().ok())
        .next_back()
}

/// The output of a read without the sentinel lines it printed about itself —
/// the issue text is the payload, not the plumbing.
fn without_exit_sentinel(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim().starts_with("__formal_ai_exit="))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Why a work-item read's output is not page evidence, when it is not.
fn work_item_read_failure_reason(
    command: &str,
    raw: &str,
    echoed_failure: Option<&str>,
) -> Option<String> {
    if let Some(failure) = echoed_failure {
        return Some(failure.to_owned());
    }
    if let Some(status) = exit_sentinel(raw)
        && status != 0
    {
        return Some(super::work_item_steps::fill(
            "read_failure_exit_status",
            &[(concat!("{", "status}"), status.to_string().as_str())],
        ));
    }
    let text = without_exit_sentinel(raw);
    let shaped = if command.starts_with("curl ") {
        !text.trim().is_empty() && !text.trim_start().starts_with('{')
    } else {
        text.contains("\n\n")
    };
    (!shaped).then(|| {
        if command.starts_with("curl ") {
            "the REST answer was not issue text".to_owned()
        } else {
            "the output did not have the title-and-body shape of a work item".to_owned()
        }
    })
}
