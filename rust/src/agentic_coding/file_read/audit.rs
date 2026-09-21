//! Answer rendering for completed file reads: per-file listings and the
//! bounded audit summary that keeps an answer inside its line/column limits.

use super::{FileReadMode, seed};

use super::supplied::extract_jsonish_value;

pub(super) fn file_read_final_answer(
    mode: &FileReadMode,
    files: &[(String, String)],
    request: &str,
) -> String {
    match mode {
        FileReadMode::FirstLine => {
            let (path, content) = &files[0];
            let first = content.lines().next().unwrap_or_default();
            format!("First line of `{path}`:\n\n```text\n{first}\n```")
        }
        FileReadMode::ExtractValue(key) => {
            if files.len() == 1 {
                let (path, content) = &files[0];
                let value = extract_jsonish_value(content, key)
                    .unwrap_or_else(|| content.trim().to_owned());
                return format!("Value of `{key}` in `{path}`: {value}");
            }
            let mut lines = Vec::with_capacity(files.len() * 2);
            for (path, content) in files {
                let value = extract_jsonish_value(content, key)
                    .unwrap_or_else(|| content.trim().to_owned());
                lines.push(format!("{path}:"));
                lines.push(format!("{key}={value}"));
            }
            lines.join("\n")
        }
        FileReadMode::Summary => {
            let mut lines = vec![format!("Read {} file(s):", files.len())];
            for (path, content) in files {
                let summary = content.lines().next().unwrap_or_default().trim();
                lines.push(format!("- `{path}`: {summary}"));
            }
            lines.join("\n")
        }
        FileReadMode::Audit => bounded_audit_answer(files, request),
        FileReadMode::Full => {
            if files.len() > 1 {
                return files
                    .iter()
                    .map(|(path, content)| {
                        format!("Contents of `{path}`:\n\n```text\n{}\n```", content.trim_end())
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");
            }
            let (path, content) = &files[0];
            format!(
                "Contents of `{path}`:\n\n```text\n{}\n```",
                content.trim_end()
            )
        }
    }
}

fn bounded_audit_answer(files: &[(String, String)], request: &str) -> String {
    const FINDINGS_PER_FILE: usize = 4;
    const FINDINGS_TOTAL: usize = 24;
    const FINDING_CHARS: usize = 180;
    const PATH_CHARS: usize = 160;

    let markers = seed::lexicon()
        .words_for_role(seed::ROLE_FILE_ANALYSIS_GAP_MARKER)
        .into_iter()
        .map(|surface| surface.to_lowercase())
        .collect::<Vec<_>>();
    let language = crate::language::detect(request).slug();
    let response = |intent: &str| seed::localized_response(intent, language).unwrap_or_default();
    let mut remaining = FINDINGS_TOTAL;
    let mut lines = vec![response("file_analysis_heading")
        .replace("{count}", &files.len().to_string())];
    for (path, content) in files {
        lines.push(format!("- `{}`", capped_text(path, PATH_CHARS)));
        let no_matches = content.trim() == "No files found";
        let grep_result = content.trim_start().starts_with("Found ");
        let findings = content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter(|line| {
                !no_matches
                    && (!grep_result
                        || (!line.starts_with("Found ") && !line.ends_with(':')))
                    && line_contains_gap_marker(line, &markers)
            })
            .take(FINDINGS_PER_FILE.min(remaining))
            .map(|line| capped_text(line, FINDING_CHARS))
            .collect::<Vec<_>>();
        remaining = remaining.saturating_sub(findings.len());
        if findings.is_empty() {
            lines.push(format!("  {}", response("file_analysis_no_marker")));
        } else {
            lines.extend(findings.into_iter().map(|finding| format!("  - {finding}")));
        }
    }
    lines.push(response("file_analysis_absence_boundary"));
    lines.join("\n")
}

fn line_contains_gap_marker(line: &str, markers: &[String]) -> bool {
    let normalized = line.to_lowercase();
    contains_unchecked_box(line)
        || markers.iter().any(|surface| normalized.contains(surface))
}

fn contains_unchecked_box(line: &str) -> bool {
    line.match_indices('[').any(|(start, _)| {
        line[start + 1..]
            .find(']')
            .is_some_and(|end| line[start + 1..start + 1 + end].chars().all(char::is_whitespace))
    })
}

fn capped_text(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let mut capped = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        capped.push('…');
    }
    capped
}
