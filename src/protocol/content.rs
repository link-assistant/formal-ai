use super::MessageContent;

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
