use super::{ChatMessage, MessageContent};

/// The most recent user turn, with client-injected blocks removed.
///
/// Two kinds are stripped: `<system-reminder>` markup and an unmarked verbatim
/// echo of the system prompt. Every reader of "what did the user ask" goes
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
    #[must_use]
    pub fn user_request_text(&self) -> String {
        strip_system_reminders(&self.plain_text())
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

fn strip_system_reminders(text: &str) -> String {
    const OPEN: &str = "<system-reminder>";
    const CLOSE: &str = "</system-reminder>";
    let mut remaining = text;
    let mut request = String::new();
    while let Some(start) = remaining.find(OPEN) {
        request.push_str(&remaining[..start]);
        let after_open = &remaining[start + OPEN.len()..];
        let Some(end) = after_open.find(CLOSE) else {
            remaining = "";
            break;
        };
        remaining = &after_open[end + CLOSE.len()..];
    }
    request.push_str(remaining);
    request.trim().to_owned()
}
