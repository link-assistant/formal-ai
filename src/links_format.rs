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

/// Quote one value exactly the way Links Notation quotes it, and change nothing
/// else about it.
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
///
/// Prefer this whenever the grammar is the document's only reader. Use
/// [`format_lino_value`] when `seed::parser` reads the document back.
pub fn format_lino_value_verbatim(value: &str) -> String {
    let record = format_indented_ordered(PROBE, &[(PROBE, value)], "")
        .expect("a one-field record under a non-empty id is always formattable");

    record
        .strip_prefix(PROBE)
        .and_then(|rest| rest.strip_prefix('\n'))
        .and_then(|rest| rest.strip_prefix(PROBE))
        .and_then(|rest| rest.strip_prefix(' '))
        .map(str::to_owned)
        .expect("the codec writes each field as an indented key-then-value pair on its own line")
}

/// Quote one value for a reader that reads a document one line at a time.
///
/// This is [`format_lino_value_verbatim`] with [`sanitize_lino_value`] in front
/// of it, and the sanitizing is the whole difference. `seed::parser` is
/// line-based — `for line in text.lines()` — so it structurally cannot see a
/// value that spans lines, and every document it reads back must keep each
/// value on the line it opened on.
///
/// That escape is not free: it is the grammar, not `seed::parser`, that defines
/// the notation, and the grammar carries a raw newline inside a quoted value
/// losslessly, nesting included (`experiments/issue_715_nested_newline_probe.rs`).
/// A sanitized value therefore reads back through the grammar with a literal
/// backslash and `n` where the newline was. The two helpers collapse into one
/// the day `seed::parser` can read a value that spans lines.
pub fn format_lino_value(value: &str) -> String {
    format_lino_value_verbatim(&sanitize_lino_value(value))
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

/// Escape a value so that [`crate::seed::parser::unescape_value`] decodes back
/// to exactly this input. Use it only where the value is *quoted* and read back.
///
/// The backslash goes first, and that ordering is the whole correctness
/// argument: this function *introduces* backslashes, so escaping it afterwards
/// would escape its own output. Escaping it at all is what makes the pair
/// invertible. A value is otherwise free to contain a backslash — it is content,
/// not syntax — and an unescaped one arrives at the decoder looking exactly like
/// an escape this function wrote. That is not hypothetical for the subject of
/// issue #715: rewriting `println!("\n")` wrote the value back verbatim, and
/// reading it returned a real newline in place of the two characters the Rust
/// source actually holds.
///
/// Prefer [`flatten_lino_value`] when nothing decodes the value.
pub fn sanitize_lino_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// Keep a value on one line in a document that is *rendered* rather than read
/// back, changing nothing else about it.
///
/// This is what [`sanitize_lino_value`] used to be, and the split is not
/// cosmetic: the two have different correctness conditions, because their
/// documents have different readers.
///
/// A quoted value is decoded by `unescape_value`, so its escape must be
/// invertible — the backslash has to be escaped, or content is read as syntax.
/// These callers instead interpolate the value *unquoted* (`step_0 {kind}
/// {payload}`, `text {statement}`), and `seed::parser` only unescapes a value it
/// found a delimiter around. So nothing decodes these, and escaping the
/// backslash would not survive a round trip — it would simply be shown, turning
/// a summary of `println!("\n")` into a summary of `println!("\\n")` and
/// misreporting the source it exists to describe.
///
/// What these callers do need is the line: the document is line-oriented, and a
/// raw newline in a value would silently invent a record. That is the one job
/// left here.
pub fn flatten_lino_value(value: &str) -> String {
    value
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}
