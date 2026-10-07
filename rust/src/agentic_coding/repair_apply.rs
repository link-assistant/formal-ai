//! The apply half of the issue #1185 R3 repair loop: the `repair_edit`
//! record rendered back into the artifact's source.
//!
//! [`super::repair_loop::repair_edit_document`] writes the fix as a Links
//! Notation record beside the failing artifact — the language, the
//! diagnostic's file and line, and the fix fragment a fetched source
//! retained. This module reads that record back and lowers it into the
//! target language: the retained fragment takes the place of the line the
//! diagnostic located, re-indented to that line's indentation, and the result
//! is accepted only when the meta-language CST engine (the issue #1167
//! renderer seam, [`crate::coding::cst::parse_program_cst`]) parses it into a
//! valid concrete syntax tree for the record's language. Every refusal is a
//! named [`RepairApplyGap`], never a silent text patch: a record without a
//! retained fix or a location, a line past the end of the source, a language
//! the CST engine has no grammar for (or the engine compiled out), and a
//! rendering that does not parse.
//!
//! Mirrored by `js/agentic/repair_apply.mjs`.

/// A `repair_edit` record read back from its Links Notation document.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RepairEdit {
    pub language: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub error_code: Option<String>,
    pub retained: Option<String>,
}

/// Why a `repair_edit` record could not be rendered into the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairApplyGap {
    /// The document is not a `repair_edit` record.
    NotARecord,
    /// The record retained no fix (`retained none`, R6).
    NoRetainedFix,
    /// The record names no line to render the fix at.
    NoLocation,
    /// The located line is past the end of the source.
    LineOutOfRange { line: usize, lines: usize },
    /// The CST engine has no grammar for the language, or is compiled out,
    /// so the rendering cannot be validated and is not applied.
    Unvalidated { language: String },
    /// The rendered source does not parse into a valid CST.
    SyntaxInvalid { language: String },
}

impl RepairApplyGap {
    /// The stable slug of the gap, for the evidence chain.
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::NotARecord => "not_a_record",
            Self::NoRetainedFix => "no_retained_fix",
            Self::NoLocation => "no_location",
            Self::LineOutOfRange { .. } => "line_out_of_range",
            Self::Unvalidated { .. } => "unvalidated",
            Self::SyntaxInvalid { .. } => "syntax_invalid",
        }
    }
}

/// Read a quoted value that starts just after its opening quote.
///
/// The two escapes `lino_escape` applies are undone. Returns the value and
/// the byte index just past the closing quote.
fn read_quoted(text: &str, start: usize) -> (String, usize) {
    let mut value = String::new();
    let mut escaped = false;
    for (offset, character) in text[start..].char_indices() {
        if escaped {
            value.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return (value, start + offset + 1);
        } else {
            value.push(character);
        }
    }
    (value, text.len())
}

/// The record's `name value` fields in document order.
///
/// A quoted value may span lines (a retained multi-line fragment), which the
/// line-based seed parser cannot read, so the fields are tokenized here.
fn record_fields(document: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let mut cursor = 0usize;
    while cursor < document.len() {
        let rest = &document[cursor..];
        let trimmed = rest.trim_start();
        if trimmed.is_empty() {
            break;
        }
        cursor += rest.len() - trimmed.len();
        let name_end = trimmed
            .find(|character: char| character.is_whitespace())
            .unwrap_or(trimmed.len());
        let name = trimmed[..name_end].to_owned();
        cursor += name_end;
        let after_name = &document[cursor..];
        let line_end = after_name.find('\n').unwrap_or(after_name.len());
        let inline = after_name[..line_end].trim_start();
        if inline.starts_with('"') {
            let quote_at = cursor + (line_end - inline.len());
            let (value, next) = read_quoted(document, quote_at + 1);
            fields.push((name, value));
            cursor = next;
        } else {
            fields.push((name, inline.trim_end().to_owned()));
            cursor += line_end;
        }
    }
    fields
}

/// Read a `repair_edit` document back into its record.
#[must_use]
pub fn parse_repair_edit(document: &str) -> Option<RepairEdit> {
    let fields = record_fields(document);
    if fields.first().map(|(name, _)| name.as_str()) != Some("repair_edit") {
        return None;
    }
    let mut edit = RepairEdit::default();
    for (name, value) in fields.into_iter().skip(1) {
        match name.as_str() {
            "language" => edit.language = value,
            "file" => edit.file = Some(value),
            "line" => edit.line = value.parse().ok(),
            "error_code" => edit.error_code = Some(value),
            "retained" if value != "none" => edit.retained = Some(value),
            _ => {}
        }
    }
    Some(edit)
}

/// The leading whitespace of `line`.
fn indentation(line: &str) -> &str {
    &line[..line.len() - line.trim_start().len()]
}

/// The fragment's lines with their common indentation replaced by `indent`.
fn reindented(fragment: &str, indent: &str) -> Vec<String> {
    let fragment_lines: Vec<&str> = fragment.lines().collect();
    let common = fragment_lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| indentation(line).len())
        .min()
        .unwrap_or(0);
    fragment_lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{indent}{}", &line[common..])
            }
        })
        .collect()
}

/// Splice the retained fragment over the located line, without validation.
///
/// # Errors
/// Returns the named gap when the record retained no fix, names no line, or
/// names a line past the end of `source`.
pub fn splice_repair_edit(edit: &RepairEdit, source: &str) -> Result<String, RepairApplyGap> {
    let fragment = edit
        .retained
        .as_deref()
        .ok_or(RepairApplyGap::NoRetainedFix)?;
    let target = edit
        .line
        .filter(|number| *number > 0)
        .ok_or(RepairApplyGap::NoLocation)?;
    let source_lines: Vec<&str> = source.lines().collect();
    if target > source_lines.len() {
        return Err(RepairApplyGap::LineOutOfRange {
            line: target,
            lines: source_lines.len(),
        });
    }
    let indent = indentation(source_lines[target - 1]);
    let mut out: Vec<String> = source_lines[..target - 1]
        .iter()
        .map(|text| (*text).to_owned())
        .collect();
    out.extend(reindented(fragment, indent));
    out.extend(source_lines[target..].iter().map(|text| (*text).to_owned()));
    let mut rendered = out.join("\n");
    if source.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

/// Render a `repair_edit` document into `source` and validate the result
/// through the meta-language CST engine.
///
/// # Errors
/// Returns the named [`RepairApplyGap`] when the record cannot be rendered
/// or the rendering is not a valid program in the record's language.
pub fn render_repair_edit(document: &str, source: &str) -> Result<String, RepairApplyGap> {
    let edit = parse_repair_edit(document).ok_or(RepairApplyGap::NotARecord)?;
    let rendered = splice_repair_edit(&edit, source)?;
    let Some(cst) = crate::coding::cst::parse_program_cst(&edit.language, &rendered) else {
        return Err(RepairApplyGap::Unvalidated {
            language: edit.language,
        });
    };
    if !cst.is_valid() {
        return Err(RepairApplyGap::SyntaxInvalid {
            language: edit.language,
        });
    }
    Ok(rendered)
}
