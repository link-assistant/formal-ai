use std::fmt::Write as _;

use super::{GRAMMATICAL_NUMBER, PART_OF_SPEECH};

const RULES: &str = include_str!("../../embedded/data/meta/notation-rules.lino");

/// Project only source-registered shard output after original shard boundaries.
pub(super) fn project(file_name: &str, content: String) -> String {
    if !selected(RULES, file_name) {
        return content;
    }
    factor(&content)
}

fn selected(rules: &str, file_name: &str) -> bool {
    let mut shared_family = false;
    let mut selected = false;
    for line in rules.lines() {
        if line.starts_with("  family ") {
            shared_family = false;
        } else if let Some(rule) = line.strip_prefix("    rule ") {
            shared_family = rule == "shared-lexeme-fields";
        } else if shared_family && let Some(source) = line.strip_prefix("    files ") {
            let relative = source
                .strip_prefix("data/seed/")
                .expect("shared lexeme output must name a seed source");
            assert!(
                !relative.contains(['*', '?', '[', ']']),
                "explicit shared source required"
            );
            selected |= relative == file_name;
        }
    }
    selected
}

fn factor(content: &str) -> String {
    assert!(content.ends_with('\n'), "generated shard must end with LF");
    let lines: Vec<_> = content.lines().collect();
    let mut output = String::new();
    let mut start = 0;
    while start < lines.len() {
        if !lines[start].starts_with("  ") || lines[start].starts_with("    ") {
            let _ = writeln!(output, "{}", lines[start]);
            start += 1;
            continue;
        }
        let mut end = start + 1;
        while end < lines.len() && lines[end].starts_with("    ") {
            end += 1;
        }
        let record = &lines[start..end];
        let part = format!("        {} {}", "part_of_speech", PART_OF_SPEECH);
        let number = format!("        {} {}", "grammatical_number", GRAMMATICAL_NUMBER);
        let surfaces = record
            .iter()
            .filter(|line| **line == "      surface")
            .count();
        let facets = record.iter().filter(|line| **line == part).count();
        assert_eq!(surfaces, facets, "generated surfaces must own both facets");
        if surfaces < 2 {
            for line in record {
                let _ = writeln!(output, "{line}");
            }
        } else {
            let _ = writeln!(output, "{}", record[0]);
            let _ = writeln!(output, "    shared-lexeme-fields lexical");
            let _ = writeln!(output, "      part_of_speech {PART_OF_SPEECH}");
            let _ = writeln!(output, "      grammatical_number {GRAMMATICAL_NUMBER}");
            let mut index = 1;
            while index < record.len() {
                if record[index] == part {
                    assert_eq!(
                        record.get(index + 1),
                        Some(&number.as_str()),
                        "facet pair required"
                    );
                    let _ = writeln!(output, "        use-lexeme-fields lexical");
                    index += 2;
                } else {
                    let _ = writeln!(output, "{}", record[index]);
                    index += 1;
                }
            }
        }
        start = end;
    }
    output
}

#[cfg(test)]
#[path = "../../tests/fixtures/lexeme-import-notation.rs"]
mod tests;
