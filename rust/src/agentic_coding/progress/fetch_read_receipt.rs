//! Fetch receipt classification before source payload interpretation.

// Only a leading seeded provider notice supplies legacy status-less Fetch failure evidence.
// JSON and source text mentioning failure later in its body retain their source bytes.
pub(super) fn legacy_fetch_notice(raw: &str) -> bool {
    let normalized = crate::engine::normalize_prompt(raw);
    crate::seed::lexicon()
        .bare_literals_for_role("tool_result_failure_signal")
        .iter()
        .any(|surface| {
            normalized
                .strip_prefix(crate::engine::normalize_prompt(surface).as_str())
                .is_some_and(|tail| {
                    tail.chars()
                        .next()
                        .is_none_or(|character| !character.is_alphanumeric())
                })
        })
}

// Whole single-line protocol failures are transport status, not article quotations.
pub(super) fn standalone_http_failure(raw: &str) -> bool {
    let line = raw.trim_matches([' ', '\t', '\r', '\n']);
    if line.chars().any(|character| {
        matches!(character, '\u{85}' | '\u{2028}' | '\u{2029}' | '\u{feff}')
            || (character.is_ascii_control() && character != '\t')
    }) {
        return false;
    }
    let Some((protocol, tail)) = line.split_once([' ', '\t']) else {
        return false;
    };
    let Some(version) = protocol.strip_prefix("HTTP/") else {
        return false;
    };
    let mut parts = version.split('.');
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    if !parts.next().is_some_and(digits)
        || parts.next().is_some_and(|part| !digits(part))
        || parts.next().is_some()
    {
        return false;
    }
    let Some(code) = tail.split([' ', '\t']).find(|part| !part.is_empty()) else {
        return false;
    };
    let code = code.strip_suffix(':').unwrap_or(code);
    code.len() == 3
        && code.bytes().all(|byte| byte.is_ascii_digit())
        && matches!(code.as_bytes()[0], b'4' | b'5')
}
