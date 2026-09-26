//! Harness bookkeeping prompts that *quote* the task instead of posing it.
//!
//! `@link-assistant/agent` summarizes a session by sending the model the
//! session's own text inside `The following is the text to summarize: <text>
//! … </text>`. Routed like any other request, the quoted task is executed a
//! second time -- issue #1133's replay fetched the issue, wrote the file and
//! compiled it again inside a summarization call. The envelope is a protocol
//! marker the CLI emits verbatim, so it is matched as one; the answer is a
//! summary of the quoted text and never a tool call.

/// The summary a text-to-summarize envelope asks for, when `received` is one.
pub(super) fn summarize_request(received: &str) -> Option<String> {
    let trimmed = received.trim_start();
    let rest = trimmed.strip_prefix(SUMMARIZE_LEAD)?;
    let quoted = rest.split_once("<text>")?.1;
    let quoted = quoted
        .rsplit_once("</text>")
        .map_or(quoted, |(inside, _)| inside)
        .trim();
    Some(summary_of(quoted))
}

const SUMMARIZE_LEAD: &str = "The following is the text to summarize:";

/// The leading sentence of `text`, which is what a one-line summary of a
/// request is: the first sentence says what is asked, the rest says how.
fn summary_of(text: &str) -> String {
    let first_line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default();
    let sentence_end = first_line
        .char_indices()
        .find(|(index, character)| {
            *character == '.'
                && first_line[index + 1..]
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace)
        })
        .map_or(first_line.len(), |(index, _)| index + 1);
    first_line[..sentence_end].to_owned()
}
