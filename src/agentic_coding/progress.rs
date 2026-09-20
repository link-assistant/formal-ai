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
    pub(super) succeeded: bool,
    pub(super) detail: String,
    pub(super) arguments: Option<String>,
    /// The advertised tool name the result came back under, when known.
    pub(super) tool: Option<String>,
}

impl ToolAttempt {
    /// Whether this attempt read a work item through the shell (`gh issue
    /// view …`). Its successful payload is page evidence, while its retry
    /// budget remains independent from the fetch capability.
    pub(super) fn is_work_item_read(&self) -> bool {
        self.capability == Capability::Run
            && self
                .arguments
                .as_deref()
                .and_then(command_argument)
                .is_some_and(|command| is_issue_view_command(&command))
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
    /// Work-item URLs already read through a shell `gh issue|pr view` call.
    ///
    /// This is deliberately separate from `attempted_fetches`: an empty or
    /// failed CLI read should fall back to the fetch capability, not make that
    /// independent retrieval route appear exhausted.
    attempted_work_item_reads: Vec<String>,
    pub(super) search_output: Option<String>,
    /// Every shell result of this turn, in arrival order.
    ///
    /// A report runs one command per destination (#839), so keeping only the
    /// last one would drop the export results the moment the issue was filed.
    pub(super) run_outputs: Vec<String>,
    /// Shell observations keyed by the exact command supplied to the client.
    /// General plans use this to keep an auxiliary `gh` read from being
    /// mistaken for their verification command.
    run_observations: Vec<(String, String)>,
    pub(super) fetch_result: Option<String>,
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
        let mut search_output = None;
        let mut run_outputs = Vec::new();
        let mut run_observations = Vec::new();
        let mut fetch_result = None;
        let mut search_result = None;
        // Ignore results from earlier user turns.
        let current_turn = messages
            .iter()
            .rposition(|message| message.role.eq_ignore_ascii_case("user"))
            .map_or(0, |index| index + 1);
        for (index, message) in messages.iter().enumerate().skip(current_turn) {
            if !message.role.eq_ignore_ascii_case("tool") {
                continue;
            }
            let Some(capability) = result_capability(messages, index) else {
                continue;
            };
            let raw = message.content.plain_text();
            let failure = super::tool_result::failure_message(
                &raw,
                message.is_error,
                capability != Capability::Run,
            );
            let arguments =
                result_tool_call(messages, index).map(|call| call.function.arguments.clone());
            let tool = message
                .name
                .clone()
                .or_else(|| result_tool_call(messages, index).map(|call| call.function.name.clone()));
            attempts.push(ToolAttempt {
                capability,
                succeeded: failure.is_none(),
                detail: failure.clone().unwrap_or_else(|| raw.clone()),
                arguments,
                tool,
            });
            if capability == Capability::Fetch {
                let payload = super::tool_result::normalized_payload(&raw);
                fetch_result = Some(payload.clone().unwrap_or_default());
                let fetch_url = result_tool_call(messages, index).and_then(fetch_call_url);
                if let Some(url) = fetch_url.as_ref()
                    && !attempted_fetches.contains(url) {
                        attempted_fetches.push(url.clone());
                    }
                if let Some(text) = payload.filter(|text| !text.trim().is_empty()) {
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
                if let Some(command) = result_tool_call(messages, index)
                    .and_then(|call| command_argument(&call.function.arguments))
                {
                    run_observations.push((command, raw.clone()));
                }
                // A work item read through the client's own `gh` is a fetched
                // page like any other (issue #1133): the command names the
                // URL, and its output is the issue's title and body.
                if let Some(url) = result_tool_call(messages, index).and_then(issue_view_url) {
                    // The attempt is recorded whatever it returned: a read
                    // that came back empty has still been tried, and planning
                    // it again would loop (issue #1133).
                    if !attempted_work_item_reads.contains(&url) {
                        attempted_work_item_reads.push(url.clone());
                    }
                    if failure.is_none()
                        && let Some(text) = super::tool_result::normalized_payload(&raw)
                            .filter(|text| !text.trim().is_empty())
                    {
                        fetched_pages.push((url, text));
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
            search_output,
            run_outputs,
            run_observations,
            fetch_result,
            search_result,
        }
    }

    /// Whether this turn already tried to fetch `url`.
    ///
    /// A fetch the harness echoed back without its `url` -- the argument was
    /// projected away onto a schema that has no such property -- still counts:
    /// the planner asked for exactly one page this turn, so a fetch attempt
    /// that names no URL was the attempt on that page. Reading only the echoed
    /// URL is what let issue #1133's Kotlin run plan the same call 547 times.
    pub(super) fn attempted_fetch_of(&self, url: &str) -> bool {
        self.attempted_fetches.iter().any(|attempted| attempted == url)
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
    ///
    /// Multi-step transformations read both their source and their written
    /// destination. Keying by call arguments keeps a later read-back from
    /// being mistaken for the source observation (or vice versa).
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

    /// Content supplied to the latest successful write of `path`.
    ///
    /// A composed request can deliver one observation to more than one file.
    /// The later delivery must recover the observation from the earlier write,
    /// not use the earlier writer's human-facing completion status as data.
    pub(super) fn successful_write_content_for(&self, path: &str) -> Option<String> {
        self.attempts.iter().rev().find_map(|attempt| {
            (attempt.capability == Capability::Write && attempt.succeeded)
                .then_some(attempt.arguments.as_deref())
                .flatten()
                .filter(|arguments| argument_targets(arguments, path))
                .and_then(argument_content)
        })
    }

    /// The capability of the most recent tool result in this turn.
    ///
    /// `completed` is in arrival order, so this distinguishes *which phase* a
    /// multi-round loop is in — a search that has not been read yet, versus a
    /// completed read — which [`Progress::done`] alone cannot, since it stays
    /// true for every later round.
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

    pub(super) fn fetch_result(&self) -> Option<&str> {
        self.fetch_result.as_deref()
    }

    pub(super) fn search_result(&self) -> Option<&str> {
        self.search_result.as_deref()
    }
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
        || crate::protocol_responses::apply_patch_input(&super::planner::write_arguments(path, content))
            .is_some_and(|patch| arguments.trim() == patch.trim())
}

fn argument_targets(arguments: &str, path: &str) -> bool {
    argument_path(arguments).is_some_and(|observed| {
        observed == path
            || (std::path::Path::new(path).is_relative()
                && std::path::Path::new(&observed).ends_with(path))
    })
}

/// Resolve which capability the tool result at `index` answers. Prefer the
/// result's own `name`; otherwise map its `tool_call_id` back to the tool name in
/// a prior assistant `tool_calls` turn.
pub(super) fn result_capability(messages: &[ChatMessage], index: usize) -> Option<Capability> {
    let message = &messages[index];
    if let Some(name) = &message.name
        && let Some(capability) = classify_tool(name) {
            return Some(capability);
        }
    result_tool_call(messages, index).and_then(|call| classify_tool(&call.function.name))
}

fn result_tool_call(messages: &[ChatMessage], index: usize) -> Option<&crate::protocol::ToolCall> {
    let call_id = messages[index].tool_call_id.as_ref()?;
    messages[..index]
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

fn command_argument(arguments: &str) -> Option<String> {
    let arguments: serde_json::Value = serde_json::from_str(arguments).ok()?;
    arguments
        .get("command")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

fn run_attempt_matches(attempt: &ToolAttempt, command: &str) -> bool {
    attempt.capability == Capability::Run
        && attempt
            .arguments
            .as_deref()
            .and_then(command_argument)
            .is_some_and(|attempted| attempted == command)
}

fn is_issue_view_command(command: &str) -> bool {
    let mut words = command.split_whitespace();
    words.next() == Some("gh")
        && matches!(words.next(), Some("issue" | "pr"))
        && words.next() == Some("view")
}

/// The issue or pull-request URL a `gh issue view <url> …` / `gh pr view <url> …`
/// command reads, when the shell call is one.
fn issue_view_url(call: &crate::protocol::ToolCall) -> Option<String> {
    let command = command_argument(&call.function.arguments)?;
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
