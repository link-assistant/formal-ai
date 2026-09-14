//! Structured `key=value` trace payloads.
//!
//! A trace line is a *record*, not a sentence. Writing one as a prose-shaped
//! format template (`format!("source={} status={} pages={}", ..)`) buries the
//! field names inside a string literal, where neither the compiler nor the
//! R379 hardcoded-language gate can tell them apart from interface text. Every
//! trace and evidence payload in this crate's retrieval surfaces is therefore
//! built from a field list: the names stay tokens, the order stays explicit,
//! and the rendered text stays byte-identical across runs.
//!
//! See `docs/design/no-hardcoded-natural-language.md` — reader-facing prose
//! belongs in `data/seed/`; machine-readable records belong here.

/// Render `key=value` fields separated by single spaces.
///
/// Values are inserted verbatim: a payload is read by a human auditing a trace
/// and matched by tests, so quoting or escaping would change the very bytes an
/// evidence file records.
#[must_use]
pub fn payload(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Join two already-rendered record fragments, skipping empty ones.
#[must_use]
pub fn join(head: &str, tail: &str) -> String {
    if head.is_empty() {
        return tail.to_owned();
    }
    if tail.is_empty() {
        return head.to_owned();
    }
    let mut joined = String::with_capacity(head.len() + tail.len() + 1);
    joined.push_str(head);
    joined.push(' ');
    joined.push_str(tail);
    joined
}

/// Render fields followed by an already-rendered payload (a nested record, such
/// as a step's provenance).
#[must_use]
pub fn payload_with(fields: &[(&str, String)], nested: &str) -> String {
    join(&payload(fields), nested)
}

/// Render one trace line: an event slug followed by an already-rendered payload.
#[must_use]
pub fn event(event: &str, payload: &str) -> String {
    join(event, payload)
}

/// Render one trace line: an event slug followed by its fields.
#[must_use]
pub fn line(slug: &str, fields: &[(&str, String)]) -> String {
    event(slug, &payload(fields))
}

/// Render a trace line whose fields are followed by an already-rendered payload
/// (a nested record, such as a step's provenance).
#[must_use]
pub fn line_with(slug: &str, fields: &[(&str, String)], nested: &str) -> String {
    event(slug, &payload_with(fields, nested))
}
