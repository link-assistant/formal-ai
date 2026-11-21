use super::*;

pub(super) fn owned_command(command: &str) -> Option<Request> {
    if command.len() > 196608 {
        return None;
    }
    let prefix = format!(
        "node -e {} ",
        crate::portable_string_contracts::quote_shell_word(PROGRAM)
    );
    let payload = command
        .strip_prefix(&prefix)?
        .strip_prefix('\'')?
        .strip_suffix('\'')?;
    if !payload
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "+/=".contains(character))
    {
        return None;
    }
    let input = decoded(payload)?;
    let value: Value = serde_json::from_str(&input).ok()?;
    if canonical(&value)? != input {
        return None;
    }
    if search::valid_request(&value) {
        return Some(Request {
            value,
            identity: digest(&input),
            command: command.to_owned(),
        });
    }
    if value["schema"] != "formal-ai-owned-file-request/v1" {
        return None;
    }
    let keys = value.as_object()?.keys().cloned().collect::<BTreeSet<_>>();
    if keys
        != [
            "afterBase64",
            "beforeSha256",
            "operation",
            "path",
            "root",
            "schema",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    {
        return None;
    }
    let operation = value["operation"].as_str()?;
    if !matches!(operation, "root" | "read" | "create" | "replace") {
        return None;
    }
    if operation == "root" {
        if ["root", "path", "beforeSha256", "afterBase64"]
            .iter()
            .any(|key| !value[*key].is_null())
        {
            return None;
        }
    } else if !value["root"].as_str().is_some_and(root_path)
        || !value["path"].as_str().is_some_and(relative)
    {
        return None;
    }
    Some(Request {
        value,
        identity: digest(&input),
        command: command.to_owned(),
    })
}
