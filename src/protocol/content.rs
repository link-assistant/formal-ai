use super::{ChatMessage, MessageContent};

/// The most recent user turn, with client-injected blocks removed.
///
/// Two kinds are stripped: every seed-declared caller-context block markup
/// (`<system-reminder>`, `<session_context>`, `<environment_context>`, `<env>`)
/// and an unmarked verbatim echo of the system prompt. Every reader of "what did
/// the user ask" goes
/// through here so a new client's decoration cannot be handled in one code path
/// and missed in another.
#[must_use]
pub fn latest_user_request(messages: &[ChatMessage]) -> Option<String> {
    let system = system_prompt_text(messages);
    messages
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| {
            message
                .content
                .user_request_text_without_system_echo(&system)
        })
}

/// The working directory the client says it is running in, when it says so.
///
/// Absolutising a planned path (issue #671) is only correct against the
/// *client's* directory, and the client is the one that knows it: the server may
/// well be running somewhere else — the issue-#715 Agent CLI E2E starts
/// `formal-ai serve` in the repository and the CLI in a fresh temporary
/// workspace, and a report absolutised against the server's own directory landed
/// in the repository root while the harness looked for it in the workspace.
///
/// Every pattern below is copied from a recorded request body:
/// `agent` and `opencode` send `<env>\n  Working directory: …`, `codex` sends
/// `<environment_context>\n<cwd>…</cwd>`, and `gemini` lists
/// `- **Workspace Directories:**` followed by one indented path per line. A
/// client that declares nothing is asked indirectly instead, through
/// [`observed_directory`]; only when that is silent too does the caller fall
/// back to the server's own directory, which is the shared-directory case the
/// matrix runs.
#[must_use]
pub fn client_working_directory(messages: &[ChatMessage]) -> Option<String> {
    let text = messages
        .iter()
        .map(|message| message.content.plain_text())
        .collect::<Vec<_>>()
        .join("\n");
    declared_directory(&text).or_else(|| observed_directory(messages))
}

/// The directory the client *demonstrated*, when it declared none (issue #1075).
///
/// A client that says nothing about where it runs still answers the question
/// every time it reports back: the planner names a file the way the request
/// spelled it, relative, and the client's result echoes the path it actually
/// resolved. `read {"filePath": "alpha.txt"}` answering
/// `/tmp/session-31/alpha.txt` states that the workspace is `/tmp/session-31` —
/// observed through the client rather than assumed of the server, and at no
/// extra call.
///
/// Only a whole-segment suffix match counts. The planned name has to be
/// preceded by a separator in the echoed path, or the match is a splice through
/// the middle of a segment rather than a directory: `/usr/bin/tools` does not
/// say that a planned `ls` resolved under `/usr/bin/too`.
#[must_use]
pub fn observed_directory(messages: &[ChatMessage]) -> Option<String> {
    let planned: Vec<String> = messages
        .iter()
        .flat_map(|message| &message.tool_calls)
        .filter_map(|call| serde_json::from_str::<serde_json::Value>(&call.function.arguments).ok())
        .flat_map(|arguments| {
            arguments
                .as_object()
                .map(|object| object.values().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .filter(|value| {
            let path = std::path::Path::new(value);
            path.is_relative() && path.file_name().is_some() && !value.contains(['\n', ' '])
        })
        .collect();
    if planned.is_empty() {
        return None;
    }
    messages
        .iter()
        .filter(|message| message.role.eq_ignore_ascii_case("tool") && !message.is_error)
        .flat_map(|message| {
            message
                .content
                .plain_text()
                .split(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '`' | ','))
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .find_map(|token| {
            planned.iter().find_map(|relative| {
                let root = token
                    .strip_suffix(relative.as_str())
                    .and_then(|root| root.strip_suffix('/'))?
                    .trim_end_matches('/');
                (!root.is_empty() && std::path::Path::new(root).is_absolute())
                    .then(|| root.to_owned())
            })
        })
}

/// The directory the client said it is running in.
///
/// A declaration that exists on this machine is preferred, because a client and
/// server sharing a host may both be able to see it. A declaration that does
/// *not* exist here is still followed (issue #1075): the server and the client
/// are frequently different containers, and the agentic session that failed on
/// Scala ran the client in `/tmp/gh-issue-solver-1788563504540` while the server
/// saw only its own sidecar. Discarding that declaration did not produce "no
/// workspace" -- it produced the server's own directory, and the file was
/// written into the sidecar under `/home/box`, where the task could never see
/// it. A directory the server cannot stat is exactly the case the declaration
/// exists to cover.
fn declared_directory(text: &str) -> Option<String> {
    const WORKSPACE_LIST: &str = "**Workspace Directories:**";
    let tagged = text
        .split("<cwd>")
        .skip(1)
        .filter_map(|rest| rest.split("</cwd>").next());
    let labelled = text
        .lines()
        .filter_map(|line| line.split_once("Working directory:").map(|(_, path)| path));
    // Gemini's list is read from the marker onwards, and only while the lines
    // are still bullets: an unanchored bullet scan would happily follow any
    // other existing directory the prompt happens to mention.
    let listed = text
        .split_once(WORKSPACE_LIST)
        .into_iter()
        .flat_map(|(_, rest)| {
            rest.lines()
                .skip(1)
                .take_while(|line| line.trim_start().starts_with("- "))
                .filter_map(|line| line.trim_start().strip_prefix("- "))
        });
    let declared: Vec<&str> = tagged
        .chain(labelled)
        .chain(listed)
        .map(str::trim)
        .filter(|path| std::path::Path::new(path).is_absolute())
        .collect();
    declared
        .iter()
        .find(|path| std::path::Path::new(path).is_dir())
        .or_else(|| declared.first())
        .map(|path| (*path).to_string())
}

/// Everything the client said as `system`, joined in order.
#[must_use]
pub fn system_prompt_text(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .filter(|message| message.role.eq_ignore_ascii_case("system"))
        .map(|message| message.content.plain_text())
        .collect::<Vec<_>>()
        .join("\n")
}

impl MessageContent {
    #[must_use]
    pub fn plain_text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Parts(parts) => parts
                .iter()
                .filter_map(|part| part.text.as_deref())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    /// User-authored request text with client-injected startup metadata removed.
    ///
    /// Qwen Code places `<system-reminder>` blocks in a `user` content part.
    /// Those blocks describe the client and its deferred tools; treating them as
    /// the task lets their keywords override the actual request that follows.
    /// Every other CLI does the same with a marker of its own — the gemini CLI's
    /// `<session_context>` announces the date, codex's `<environment_context>`
    /// the sandbox, agent's and opencode's `<env>` the working directory — so
    /// the tags are read from [`crate::seed::caller_context_vocabulary`] rather than
    /// hardcoded one client at a time (issue #907).
    #[must_use]
    pub fn user_request_text(&self) -> String {
        strip_caller_context_blocks(&self.plain_text())
    }

    /// The same text, with a reminder the client re-appended from its own system
    /// prompt removed.
    ///
    /// Aider does exactly this: the user turn the `aider` leg of the issue-#671
    /// matrix sent was `read the file alpha.txt and print its contents` followed
    /// by 830 characters repeated verbatim from its system prompt — the
    /// *file listing* format, complete with the example
    /// `// entire file content ...`. The server answered about the example. Qwen
    /// Code marks the same kind of block with `<system-reminder>`; aider marks it
    /// with nothing at all, so the tell used here is the duplication itself:
    /// text the client already said as the system prompt is the client talking,
    /// not the user.
    #[must_use]
    pub fn user_request_text_without_system_echo(&self, system: &str) -> String {
        strip_system_echo(&self.user_request_text(), system)
    }
}

/// Drop the longest line-aligned suffix of `request` that appears verbatim in
/// `system`.
///
/// Line-aligned, because a client appends whole blocks; verbatim, because a
/// paraphrase is the user's own words. A short tail is left alone — a user who
/// answers "yes" to a system prompt containing "yes" still said it — and a
/// request that is *nothing but* an echo is returned unchanged, since dropping
/// everything would turn a real turn into an empty one.
fn strip_system_echo(request: &str, system: &str) -> String {
    const MIN_ECHO: usize = 40;
    if system.trim().is_empty() {
        return request.to_owned();
    }
    let lines: Vec<&str> = request.lines().collect();
    for start in 1..lines.len() {
        let tail = lines[start..].join("\n");
        let tail = tail.trim();
        if tail.len() < MIN_ECHO || !system.contains(tail) {
            continue;
        }
        let head = lines[..start].join("\n").trim().to_owned();
        if !head.is_empty() {
            return head;
        }
    }
    request.to_owned()
}

/// Remove every seed-declared client-injected block from `text`.
///
/// The client's own framing is not the user's request: whatever it says about
/// the date, the sandbox, or the operating system must not be able to answer for
/// the user. An unterminated block swallows the rest of the turn, because a
/// client that opened a context block and never closed it is still talking.
fn strip_caller_context_blocks(text: &str) -> String {
    crate::seed::caller_context_vocabulary()
        .injected_blocks
        .iter()
        .fold(text.to_owned(), |text, block| {
            strip_block(&text, &block.open(), &block.close())
        })
        .trim()
        .to_owned()
}

fn strip_block(text: &str, open: &str, close: &str) -> String {
    let mut remaining = text;
    let mut request = String::new();
    while let Some(start) = remaining.find(open) {
        request.push_str(&remaining[..start]);
        let after_open = &remaining[start + open.len()..];
        let Some(end) = after_open.find(close) else {
            remaining = "";
            break;
        };
        remaining = &after_open[end + close.len()..];
    }
    request.push_str(remaining);
    request
}
