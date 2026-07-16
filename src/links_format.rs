//! Shared formatting helpers for compact Links Notation records.

use lino_objects_codec::format::format_indented_ordered;

/// A name the codec's key escape leaves alone, so the probe record's shape in
/// [`format_lino_value`] is known exactly.
const PROBE: &str = "v";

pub fn format_lino_record(id: &str, pairs: &[(&str, String)]) -> String {
    let sanitized = pairs
        .iter()
        .map(|(key, value)| (*key, sanitize_lino_value(value)))
        .collect::<Vec<_>>();
    let borrowed = sanitized
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect::<Vec<_>>();

    format_indented_ordered(id, &borrowed, "  ")
        .expect("static Links Notation records should be valid")
}

/// Quote one value the way Links Notation quotes it.
///
/// Links Notation escapes a quote by *doubling* it and picks a delimiter the
/// value does not already carry; it has no backslash escape at all. Renderers
/// that hand-rolled a C-style one therefore emitted documents a reader ends
/// early or rejects outright, which is what [`format_lino_record`] already
/// exists to prevent — but that helper only writes a flat two-level record, so
/// a nested tree had no way to reach it and kept its own escaper.
///
/// The codec's always-quoting encoder is private, and the grammar crate's is
/// too, so the rule is borrowed from the one public function that applies it:
/// a single-field record is formatted and the field taken back off it. That
/// costs an allocation per value and buys the property that matters — this
/// cannot drift from the notation, because it *is* the notation's encoder.
pub fn format_lino_value(value: &str) -> String {
    let sanitized = sanitize_lino_value(value);
    let record = format_indented_ordered(PROBE, &[(PROBE, sanitized.as_str())], "")
        .expect("a one-field record under a non-empty id is always formattable");

    record
        .strip_prefix(PROBE)
        .and_then(|rest| rest.strip_prefix('\n'))
        .and_then(|rest| rest.strip_prefix(PROBE))
        .and_then(|rest| rest.strip_prefix(' '))
        .map(str::to_owned)
        .expect("the codec writes each field as `{indent}{key} {value}` on its own line")
}

/// Write one `name "value"` node at `indent` spaces, the way Links Notation
/// quotes. A node with no value is a bare group header.
pub fn push_lino_node(out: &mut String, indent: usize, name: &str, value: Option<&str>) {
    for _ in 0..indent {
        out.push(' ');
    }
    out.push_str(name);
    if let Some(value) = value {
        out.push(' ');
        out.push_str(&format_lino_value(value));
    }
    out.push('\n');
}

pub fn sanitize_lino_value(value: &str) -> String {
    value
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}
