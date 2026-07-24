//! One shared issue-report document for every Formal AI surface (#839).
//!
//! Until issue #839 the web reporter (`src/web/app/main.jsx`) and the agentic
//! reporter (`src/agentic_coding/report_issue.rs`) formatted issue bodies
//! independently: the web path emitted six well-formed Markdown sections while
//! the agentic path emitted one intro line plus a byte slice of a proxy trace
//! (see issue #838 for the artifact). This module owns the *format* — section
//! order, the dialog block, record-safe truncation and the title convention —
//! so every surface renders the same document.
//!
//! The module deliberately owns no *facts* and no prose: environment fields,
//! user-context fields and every user-visible label are inputs. Rust callers
//! resolve labels from `data/seed/*.lino` (R379, "data is the interface"); the
//! browser mirror (`src/web/app/issue-report.js`) passes the i18n catalogue.
//! `tests/integration/issue_839_report_parity.rs` renders the same fixture
//! through both implementations and asserts the bytes are identical.

use serde::{Deserialize, Serialize};

/// Section headings, in the order [`ReportBody::render`] emits them.
pub const SECTION_ENVIRONMENT: &str = "## Environment";
/// Heading of the optional user/browser context section.
pub const SECTION_USER_CONTEXT: &str = "## User Context";
/// Heading of the transcript section.
pub const SECTION_REPRODUCTION: &str = "## Reproduction of dialog";
/// Heading of the optional reasoning-trace section.
pub const SECTION_REASONING_TRACE: &str = "## Reasoning Trace";
/// Heading of the free-form description section.
pub const SECTION_DESCRIPTION: &str = "## Description";
/// Heading of the closing memory-attachment section.
pub const SECTION_ATTACH_MEMORY: &str = "## Attach full memory (optional)";

/// Every section a complete report body contains, in emission order.
pub const SECTIONS: [&str; 6] = [
    SECTION_ENVIRONMENT,
    SECTION_USER_CONTEXT,
    SECTION_REPRODUCTION,
    SECTION_REASONING_TRACE,
    SECTION_DESCRIPTION,
    SECTION_ATTACH_MEMORY,
];

/// Placeholder replaced by a count inside an `omitted` label.
pub const COUNT_PLACEHOLDER: &str = "{count}";

/// Longest title the first+last convention may produce (§4 of issue #839).
///
/// GitHub accepts 256 characters; the convention needs room for two quoted
/// turns and still has to stay scannable in a list, so the whole title is
/// capped well below the hard limit and falls back to the first turn alone.
pub const TITLE_MAX_LENGTH: usize = 120;

/// A `- **label**: value` row inside the Environment or User Context section.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportField {
    /// Bold row label.
    pub label: String,
    /// Row value; an empty value drops the row entirely.
    pub value: String,
}

impl ReportField {
    /// Build one row.
    #[must_use]
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// One conversation turn as it appears in the reproduction block.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportTurn {
    /// `user`, `assistant` or `tool`.
    pub role: String,
    /// Turn text; multi-line content is indented under its prefix.
    pub content: String,
    /// Optional intent annotation (`unknown` is always annotated).
    pub intent: String,
    /// Whether this turn is the one the user chose to report.
    pub reported: bool,
    /// Whether this turn asked for the report (used by the title convention).
    pub report_invoking: bool,
}

impl ReportTurn {
    /// Build a plain turn.
    #[must_use]
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
            ..Self::default()
        }
    }

    fn prefix(&self) -> &'static str {
        if self.role.eq_ignore_ascii_case("user") {
            "U"
        } else if self.role.eq_ignore_ascii_case("tool") {
            "T"
        } else {
            "A"
        }
    }
}

/// A fenced block appended after the six standard sections (complete context,
/// exported logs, …).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportAttachment {
    /// Markdown heading of the attachment, including its `###` prefix.
    pub heading: String,
    /// Sentence introducing the block (for example where the full file lives).
    pub note: String,
    /// Fence info string, for example `lino`.
    pub language: String,
    /// Block content, already truncated on record boundaries when needed.
    pub content: String,
}

/// Every user-visible phrase the builder needs, supplied by the caller.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportLabels {
    /// Legend explaining the `U:` / `A:` line prefixes.
    pub legend: String,
    /// Shown instead of the transcript when there are no turns.
    pub no_messages: String,
    /// Marker for turns dropped before the first shown turn (`{count}`).
    pub omitted_earlier: String,
    /// Singular form of `omitted_earlier`, used when exactly one turn was cut.
    pub omitted_earlier_one: String,
    /// Marker for whole records dropped from an attachment (`{count}`).
    pub omitted_records: String,
    /// Line introducing the reasoning trace block.
    pub trace_heading: String,
    /// Comment inviting the reporter to describe the problem.
    pub description_placeholder: String,
    /// Closing paragraph about attaching exported memory.
    pub memory_note: String,
}

impl ReportLabels {
    /// Resolve every phrase from `data/seed/agent-info.lino` (R379).
    #[must_use]
    pub fn from_seed() -> Self {
        let mut info = crate::seed::agent_info();
        let mut take = |key: &str| info.remove(key).unwrap_or_default();
        Self {
            legend: take("issue_report_dialog_legend"),
            no_messages: take("issue_report_no_messages"),
            omitted_earlier: take("issue_report_omitted_messages"),
            omitted_earlier_one: take("issue_report_omitted_message"),
            omitted_records: take("issue_report_omitted_records"),
            trace_heading: take("issue_report_trace_heading"),
            description_placeholder: take("issue_report_description_placeholder"),
            memory_note: take("issue_report_memory_note"),
        }
    }
}

/// A complete report body: facts from the caller, format from this module.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportBody {
    /// User-visible phrases.
    pub labels: ReportLabels,
    /// Rows of the Environment section.
    pub environment: Vec<ReportField>,
    /// Rows of the User Context section; empty drops the section.
    pub user_context: Vec<ReportField>,
    /// The transcript.
    pub turns: Vec<ReportTurn>,
    /// How many turns were dropped before `turns` to fit a size budget.
    pub earlier_omitted: usize,
    /// Pre-rendered reasoning-trace lines; empty drops the section.
    pub reasoning_trace: Vec<String>,
    /// Extra fenced blocks appended after the six sections.
    pub attachments: Vec<ReportAttachment>,
}

impl ReportBody {
    /// Render the Markdown document.
    #[must_use]
    pub fn render(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(SECTION_ENVIRONMENT.to_owned());
        lines.push(String::new());
        push_fields(&mut lines, &self.environment);
        lines.push(String::new());

        if self
            .user_context
            .iter()
            .any(|field| !field.value.is_empty())
        {
            lines.push(SECTION_USER_CONTEXT.to_owned());
            lines.push(String::new());
            push_fields(&mut lines, &self.user_context);
            lines.push(String::new());
        }

        lines.push(SECTION_REPRODUCTION.to_owned());
        lines.push(String::new());
        self.push_dialog(&mut lines);

        // Issue #386: a trace is only meaningful beside the complete dialog, so
        // it is dropped as soon as earlier turns had to be omitted.
        if self.earlier_omitted == 0 && !self.reasoning_trace.is_empty() {
            lines.push(String::new());
            lines.push(SECTION_REASONING_TRACE.to_owned());
            lines.push(String::new());
            lines.push(self.labels.trace_heading.clone());
            lines.push(String::new());
            push_code_block(&mut lines, "", &self.reasoning_trace.join("\n"));
            lines.push(String::new());
        }

        lines.push(String::new());
        lines.push(SECTION_DESCRIPTION.to_owned());
        lines.push(String::new());
        lines.push(self.labels.description_placeholder.clone());
        lines.push(String::new());
        lines.push(SECTION_ATTACH_MEMORY.to_owned());
        lines.push(String::new());
        lines.push(self.labels.memory_note.clone());
        lines.push(String::new());

        for attachment in &self.attachments {
            lines.push(attachment.heading.clone());
            lines.push(String::new());
            if !attachment.note.is_empty() {
                lines.push(attachment.note.clone());
                lines.push(String::new());
            }
            push_code_block(&mut lines, &attachment.language, &attachment.content);
            lines.push(String::new());
        }

        lines.join("\n")
    }

    fn push_dialog(&self, lines: &mut Vec<String>) {
        if self.turns.is_empty() {
            lines.push(self.labels.no_messages.clone());
            return;
        }

        lines.push(self.labels.legend.clone());
        lines.push(String::new());
        let fence = pick_fence(self.turns.iter().map(|turn| turn.content.as_str()));
        lines.push(fence.clone());
        if self.earlier_omitted > 0 {
            let label = if self.earlier_omitted == 1 && !self.labels.omitted_earlier_one.is_empty()
            {
                &self.labels.omitted_earlier_one
            } else {
                &self.labels.omitted_earlier
            };
            lines.push(render_count(label, self.earlier_omitted));
        }
        for turn in &self.turns {
            let mut annotations: Vec<String> = Vec::new();
            if turn.intent == "unknown" {
                annotations.push(format!("intent: {}", turn.intent));
            }
            if turn.reported {
                if !turn.intent.is_empty() && turn.intent != "unknown" {
                    annotations.push(format!("intent: {}", turn.intent));
                }
                annotations.push(String::from("reported"));
            }
            let head = if annotations.is_empty() {
                turn.prefix().to_owned()
            } else {
                format!("{} ({})", turn.prefix(), annotations.join(", "))
            };
            let mut rows = turn.content.split('\n');
            let first = rows.next().unwrap_or_default();
            lines.push(format!("{head}: {first}"));
            for row in rows {
                lines.push(format!("   {row}"));
            }
        }
        lines.push(fence);
    }
}

fn push_fields(lines: &mut Vec<String>, fields: &[ReportField]) {
    for field in fields.iter().filter(|field| !field.value.is_empty()) {
        lines.push(format!("- **{}**: {}", field.label, field.value));
    }
}

fn push_code_block(lines: &mut Vec<String>, language: &str, content: &str) {
    let fence = pick_fence(std::iter::once(content));
    lines.push(format!("{fence}{language}"));
    lines.push(content.to_owned());
    lines.push(fence);
}

/// Choose a fence long enough that no sample can terminate the block early.
fn pick_fence<'a>(samples: impl Iterator<Item = &'a str> + Clone) -> String {
    let mut fence = String::from("```");
    while samples.clone().any(|sample| sample.contains(&fence)) {
        fence.push('`');
    }
    fence
}

/// Substitute `{count}` in an `omitted` label.
#[must_use]
pub fn render_count(label: &str, count: usize) -> String {
    label.replace(COUNT_PLACEHOLDER, &count.to_string())
}

/// Title settings for [`issue_title`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TitleSettings {
    /// Prefix such as `Formal AI: `.
    pub prefix: String,
    /// Title used only when the conversation has no reportable user turn.
    pub default_title: String,
}

impl TitleSettings {
    /// Resolve the prefix and the fallback title from the seed data (R379).
    #[must_use]
    pub fn from_seed() -> Self {
        let mut info = crate::seed::agent_info();
        Self {
            prefix: info.remove("issue_report_title_prefix").unwrap_or_default(),
            default_title: info
                .remove("issue_report_default_title")
                .unwrap_or_default(),
        }
    }
}

/// Build an issue title from the conversation, following §4 of issue #839.
///
/// The turns that invoked the report are dropped first — they are never the
/// subject. When two or more distinct user turns remain and the `` `first` +
/// `last` `` form fits [`TITLE_MAX_LENGTH`], that form is used; otherwise the
/// first turn alone is used, backticked and truncated on a word boundary.
#[must_use]
pub fn issue_title(turns: &[ReportTurn], settings: &TitleSettings) -> String {
    let subjects = title_subjects(turns);
    let Some(first) = subjects.first() else {
        return settings.default_title.clone();
    };

    if let Some(last) = subjects.last().filter(|last| *last != first) {
        let combined = format!("{}`{first}` + `{last}`", settings.prefix);
        if combined.chars().count() <= TITLE_MAX_LENGTH {
            return combined;
        }
    }

    let budget = TITLE_MAX_LENGTH.saturating_sub(settings.prefix.chars().count() + 2);
    format!("{}`{}`", settings.prefix, truncate_words(first, budget))
}

/// The user turns a title may describe: report-invoking turns are dropped from
/// the end, and repeated text is collapsed so `first` and `last` stay distinct.
fn title_subjects(turns: &[ReportTurn]) -> Vec<String> {
    let mut subjects: Vec<(String, bool)> = turns
        .iter()
        .filter(|turn| turn.role.eq_ignore_ascii_case("user"))
        .map(|turn| (normalize_single_line(&turn.content), turn.report_invoking))
        .filter(|(text, _)| !text.is_empty())
        .collect();
    // Rule 1: the turn that asked for the report is never the subject. Only a
    // trailing run is dropped: an earlier report-shaped turn that the agent
    // answered (issue #826's `Зарепорти баг`) is part of the reported story.
    // Rule 4 wins over rule 1 when nothing else remains — a title that quotes
    // the request is still better than the bare default.
    while subjects.len() > 1 && subjects.last().is_some_and(|(_, invoking)| *invoking) {
        subjects.pop();
    }
    let mut subjects: Vec<String> = subjects.into_iter().map(|(text, _)| text).collect();
    subjects.dedup();
    subjects
}

fn normalize_single_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Trim to `max` characters on a word boundary, marking the cut with `…`.
#[must_use]
pub fn truncate_words(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max {
        return text.to_owned();
    }
    // Character indices, not byte offsets: the convention quotes user turns in
    // any script, and issue #826's title is Cyrillic.
    let head: Vec<char> = text.chars().take(max.saturating_sub(1)).collect();
    let boundary = head.iter().rposition(|character| character.is_whitespace());
    let cut: String = match boundary {
        Some(index) if index >= max / 2 => head[..index].iter().collect(),
        _ => head.iter().collect(),
    };
    format!("{}…", cut.trim_end())
}

/// Result of [`truncate_records`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruncatedRecords {
    /// The kept text, always ending at a record boundary.
    pub text: String,
    /// How many whole records were dropped.
    pub omitted: usize,
}

/// Shrink a Links Notation document to `max_bytes` without ever cutting inside
/// a record.
///
/// Issue #838 was filed with `tail -c 12000 | sed '1d'` applied to a `.lino`
/// export: a byte offset lands mid-record and destroys the tree, so the reader
/// receives an unparseable fragment of one base64 HTTP body. Records here are
/// the sibling blocks at the shallowest indented level (each `message` under
/// `messages`, for example). Whole records are kept from the head and the tail;
/// the gap between them carries an explicit `omitted N` marker.
#[must_use]
pub fn truncate_records(text: &str, max_bytes: usize, omitted_label: &str) -> TruncatedRecords {
    if text.len() <= max_bytes {
        return TruncatedRecords {
            text: text.to_owned(),
            omitted: 0,
        };
    }

    let lines: Vec<&str> = text.lines().collect();
    let Some(base_indent) = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| indent_of(line))
        .filter(|indent| *indent > 0)
        .min()
    else {
        // A flat document has no nesting to protect: keep whole lines instead.
        return truncate_lines(&lines, max_bytes, omitted_label);
    };

    let mut header: Vec<&str> = Vec::new();
    let mut records: Vec<Vec<&str>> = Vec::new();
    for line in &lines {
        let starts_record = !line.trim().is_empty() && indent_of(line) == base_indent;
        if starts_record {
            records.push(vec![line]);
        } else if let Some(current) = records.last_mut() {
            current.push(line);
        } else {
            header.push(line);
        }
    }

    let marker = format!("{}{}", " ".repeat(base_indent), omitted_label);
    let budget = max_bytes.saturating_sub(joined_len(&header) + marker.len() + 1);
    let sizes: Vec<usize> = records.iter().map(|record| joined_len(record)).collect();
    let (mut head_count, mut tail_count, mut used) = (0, 0, 0);
    // Half the budget goes to the opening records (what the session was about)
    // and the rest to the closing ones (where it went wrong).
    while head_count < records.len() && used + sizes[head_count] <= budget / 2 {
        used += sizes[head_count];
        head_count += 1;
    }
    while head_count + tail_count < records.len()
        && used + sizes[records.len() - 1 - tail_count] <= budget
    {
        used += sizes[records.len() - 1 - tail_count];
        tail_count += 1;
    }

    let omitted = records.len() - head_count - tail_count;
    if omitted == 0 {
        return TruncatedRecords {
            text: text.to_owned(),
            omitted: 0,
        };
    }

    let rendered_marker = format!(
        "{}{}",
        " ".repeat(base_indent),
        render_count(omitted_label, omitted)
    );
    let mut kept: Vec<&str> = header;
    for record in &records[..head_count] {
        kept.extend(record.iter().copied());
    }
    kept.push(&rendered_marker);
    for record in &records[records.len() - tail_count..] {
        kept.extend(record.iter().copied());
    }
    TruncatedRecords {
        text: format!("{}\n", kept.join("\n")),
        omitted,
    }
}

fn truncate_lines(lines: &[&str], max_bytes: usize, omitted_label: &str) -> TruncatedRecords {
    let mut kept: Vec<&str> = Vec::new();
    let mut used = omitted_label.len() + 1;
    for line in lines {
        if used + line.len() + 1 > max_bytes {
            break;
        }
        used += line.len() + 1;
        kept.push(line);
    }
    let omitted = lines.len() - kept.len();
    let mut text = kept.join("\n");
    if omitted > 0 {
        text.push('\n');
        text.push_str(&render_count(omitted_label, omitted));
    }
    text.push('\n');
    TruncatedRecords { text, omitted }
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn joined_len(lines: &[&str]) -> usize {
    lines.iter().map(|line| line.len() + 1).sum()
}
