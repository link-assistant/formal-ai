//! Answers that hand back a file the request already supplied in the
//! conversation, instead of re-reading it from disk.

use super::*;

pub(super) fn extract_jsonish_value(content: &str, key: &str) -> Option<String> {
    let line_prefix = format!("{key}=");
    if let Some(value) = content
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(&line_prefix))
    {
        return Some(value.to_owned());
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content)
        && let Some(found) = value.get(key) {
            return Some(match found {
                serde_json::Value::String(text) => text.clone(),
                other => other.to_string(),
            });
        }
    let quoted_key = format!("\"{key}\"");
    let start = content.find(&quoted_key)?;
    let after_key = &content[start + quoted_key.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    if let Some(rest) = after_colon.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_owned());
    }
    Some(
        after_colon
            .split(|character: char| {
                character == ',' || character == '}' || character.is_whitespace()
            })
            .next()
            .unwrap_or_default()
            .to_owned(),
    )
}

/// Answer a file-read request from file contents the client already supplied
/// in the conversation, without any tool at all (issue #671).
///
/// Not every agentic CLI speaks function calling. `aider` never advertises a
/// tool: it asks the user to add files to the chat and then hands their bytes
/// to the model in-band, as a path line followed by a fenced block holding the
/// file's contents.
///
/// Before this route, `read the file alpha.txt and print its contents` from
/// such a client never reached the agentic planner at all — the tool gate in
/// [`crate::protocol`] fell straight through to the general solver, which
/// answered "I could not determine …" about a file whose contents were sitting
/// three messages up. The bytes are already here; refusing to read them is the
/// defect.
pub fn supplied_file_answer(messages: &[ChatMessage]) -> Option<String> {
    let task = crate::protocol::latest_user_request(messages)?;
    let FileReadTask::Direct { path, mode, .. } = file_read_task_for(&task)? else {
        return None;
    };
    let content = supplied_file_content(messages, &path)?;
    Some(supplied_file_final_answer(&mode, &path, &content, &task))
}

/// Render supplied bytes back **without a fenced block**.
///
/// The ordinary reader answers with ```` ```text ```` fences, and that is
/// exactly what a prompt-format client's edit parser is looking for: aider read
/// the fenced answer to `read the file alpha.txt` as a *file listing*, printed
/// "Applied edit to alpha.txt", and swallowed the contents out of its own
/// transcript — so the user saw a heading and nothing under it. Quoting the
/// lines with numbers says the same thing in a shape no edit parser claims.
///
/// The two headings are grounded meanings (R379): they live in
/// `data/seed/multilingual-responses.lino` under the `supplied_file_contents`
/// and `supplied_file_first_line` intents, localized to the request's own
/// language with an English fallback. The `{...}` tokens named here are those
/// templates' placeholders, not Rust format arguments.
#[allow(clippy::literal_string_with_formatting_args)]
fn supplied_file_final_answer(
    mode: &FileReadMode,
    path: &str,
    content: &str,
    request: &str,
) -> String {
    match mode {
        FileReadMode::Full => {
            let body = content
                .lines()
                .enumerate()
                .map(|(index, line)| format!("{:>4} | {line}", index + 1))
                .collect::<Vec<_>>()
                .join("\n");
            supplied_file_template("supplied_file_contents", request, &[("{body}", body)], path)
        }
        FileReadMode::FirstLine => supplied_file_template(
            "supplied_file_first_line",
            request,
            &[(
                "{line}",
                content.lines().next().unwrap_or_default().to_owned(),
            )],
            path,
        ),
        FileReadMode::ExtractValue(_) | FileReadMode::Summary | FileReadMode::Audit => {
            file_read_final_answer(
                mode,
                &[(path.to_owned(), content.to_owned())],
                request,
            )
        }
    }
}

/// Fill a seeded answer template for `intent` with this run's values.
///
/// The `{...}` tokens are seed placeholders, not Rust format arguments, so they
/// are substituted rather than interpolated — which is what the `allow` below
/// says to clippy's nursery lint.
#[allow(clippy::literal_string_with_formatting_args)]
fn supplied_file_template(
    intent: &str,
    request: &str,
    substitutions: &[(&str, String)],
    path: &str,
) -> String {
    let language = crate::language::detect(request);
    let mut rendered = seed::localized_response(intent, language.slug()).unwrap_or_default();
    rendered = rendered.replace("{path}", path);
    for (placeholder, value) in substitutions {
        rendered = rendered.replace(placeholder, value);
    }
    rendered
}

/// The most recently supplied contents of `path`, from a fenced block labelled
/// with that path on the line above it.
///
/// Deliberately conservative: the label line must *be* the path (modulo the
/// same directory-suffix rule tool results use), so ordinary prose that merely
/// mentions a filename before an unrelated code block cannot be mistaken for
/// the file. Later messages win, because a client re-sends the whole file after
/// every edit and the last copy is the current one.
fn supplied_file_content(messages: &[ChatMessage], path: &str) -> Option<String> {
    let mut found = None;
    for message in messages {
        let text = message.content.plain_text();
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let label = line.trim();
            if label.is_empty() || !same_path(label, path) {
                continue;
            }
            let Some(fence) = lines.get(index + 1).map(|line| line.trim_end()) else {
                continue;
            };
            if !fence.trim_start().starts_with("```") {
                continue;
            }
            let body: Vec<&str> = lines[index + 2..]
                .iter()
                .take_while(|line| !line.trim_end().trim_start().starts_with("```"))
                .copied()
                .collect();
            if !body.is_empty() {
                found = Some(body.join("\n"));
            }
        }
    }
    found
}
