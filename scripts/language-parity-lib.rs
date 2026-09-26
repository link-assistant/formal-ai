//! Structural five-language parity census shared by the parity gate and the
//! repository debt ratchet.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEBT_FILE: &str = "data/meta/language-parity-debt.lino";
const SEED_ROOT: &str = "data/seed";
const TARGET_LANGUAGES: [&str; 5] = ["en", "ru", "hi", "zh", "es"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct LanguageGap {
    source: String,
    owner_path: String,
    meaning: String,
    present: Vec<String>,
    missing: Vec<String>,
}

impl LanguageGap {
    fn key(&self) -> (String, String) {
        (self.source.clone(), self.owner_path.clone())
    }

    fn description(&self) -> String {
        format!(
            "meaning `{}` lacks lexeme languages: {}",
            self.meaning,
            self.missing.join(",")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnerLexemes {
    source: String,
    owner_path: String,
    meaning: String,
    languages: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DebtRow {
    source: String,
    owner_path: String,
    meaning: String,
    present: Vec<String>,
    missing: Vec<String>,
    observed_on: String,
    gap: String,
}

impl DebtRow {
    fn key(&self) -> (String, String) {
        (self.source.clone(), self.owner_path.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DebtFile {
    measured_on: String,
    rows: Vec<DebtRow>,
}

fn indentation(line: &str, source: &str, line_number: usize) -> Result<usize, String> {
    let prefix = line.len() - line.trim_start_matches([' ', '\t']).len();
    if line[..prefix].contains('\t') {
        return Err(format!(
            "{source}:{line_number}: tab indentation is not a structural Links Notation indent"
        ));
    }
    Ok(prefix)
}

/// Length-prefix every structural label. Unlike a slash-joined display path,
/// this is collision-free even when a label itself contains a separator.
fn encode_path(labels: &[String]) -> String {
    let mut encoded = String::new();
    for label in labels {
        let _ = write!(encoded, "{}:{label}", label.len());
    }
    encoded
}

fn gaps_from_documents<'a>(
    documents: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<Vec<LanguageGap>, String> {
    let targets: BTreeSet<&str> = TARGET_LANGUAGES.into_iter().collect();
    let mut owners: BTreeMap<(String, String), OwnerLexemes> = BTreeMap::new();

    for (source, text) in documents {
        let mut stack: Vec<(usize, String)> = Vec::new();
        for (index, line) in text.lines().enumerate() {
            let line_number = index + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = indentation(line, source, line_number)?;
            while stack.last().is_some_and(|(level, _)| *level >= indent) {
                stack.pop();
            }

            let mut words = trimmed.split_whitespace();
            if words.next() == Some("lexeme")
                && let Some(language) = words.next()
                && targets.contains(language)
            {
                let Some((_, owner)) = stack.last() else {
                    return Err(format!(
                        "{source}:{line_number}: target lexeme `{language}` has no structural owner"
                    ));
                };
                let owner_labels: Vec<String> =
                    stack.iter().map(|(_, label)| label.clone()).collect();
                let owner_path = encode_path(&owner_labels);
                let key = (source.to_owned(), owner_path.clone());
                let entry = owners.entry(key).or_insert_with(|| OwnerLexemes {
                    source: source.to_owned(),
                    owner_path,
                    meaning: owner.clone(),
                    languages: BTreeSet::new(),
                });
                if !entry.languages.insert(language.to_owned()) {
                    return Err(format!(
                        "{source}:{line_number}: meaning `{owner}` declares duplicate `lexeme {language}`"
                    ));
                }
            }
            stack.push((indent, trimmed.to_owned()));
        }
    }

    let mut gaps = Vec::new();
    for owner in owners.into_values() {
        let present: Vec<String> = TARGET_LANGUAGES
            .iter()
            .filter(|language| owner.languages.contains(**language))
            .map(|language| (*language).to_owned())
            .collect();
        let missing: Vec<String> = TARGET_LANGUAGES
            .iter()
            .filter(|language| !owner.languages.contains(**language))
            .map(|language| (*language).to_owned())
            .collect();
        if !missing.is_empty() {
            gaps.push(LanguageGap {
                source: owner.source,
                owner_path: owner.owner_path,
                meaning: owner.meaning,
                present,
                missing,
            });
        }
    }
    gaps.sort_by_key(LanguageGap::key);
    Ok(gaps)
}

fn lino_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
        let path = entry.path();
        if path.is_dir() {
            lino_files(&path, output)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("lino") {
            output.push(path);
        }
    }
    Ok(())
}

fn repository_gaps(root: &Path) -> Result<Vec<LanguageGap>, String> {
    let directory = root.join(SEED_ROOT);
    let mut files = Vec::new();
    lino_files(&directory, &mut files)?;
    files.sort();
    let mut owned = Vec::new();
    for path in files {
        let source = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        owned.push((source, text));
    }
    gaps_from_documents(
        owned
            .iter()
            .map(|(source, text)| (source.as_str(), text.as_str())),
    )
}

/// The live, data-derived debt measurement consumed by the strict ratchet.
#[allow(dead_code)] // Used by check-debt-ratchet.rs when this module is embedded there.
pub fn current_gap_count(root: &Path) -> Result<u64, String> {
    Ok(repository_gaps(root)?.len() as u64)
}

fn quote(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other => output.push(other),
        }
    }
    output.push('"');
    output
}

fn unquote(value: &str) -> Result<String, String> {
    let value = value.trim();
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        return Err(format!("expected a quoted value, found `{value}`"));
    }
    let mut output = String::new();
    let mut escaped = false;
    for character in value[1..value.len() - 1].chars() {
        if escaped {
            output.push(match character {
                '\\' => '\\',
                '"' => '"',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => return Err(format!("unsupported escape `\\{other}` in {value}")),
            });
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    if escaped {
        return Err(format!("unfinished escape in {value}"));
    }
    Ok(output)
}

fn parse_csv(value: &str) -> Result<Vec<String>, String> {
    let value = unquote(value)?;
    if value.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(value.split(',').map(str::to_owned).collect())
    }
}

fn split_quoted_words(value: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut escaped = false;
    let mut active = false;
    for character in value.chars() {
        if escaped {
            current.push(match character {
                '\\' => '\\',
                '"' => '"',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => return Err(format!("unsupported escape `\\{other}` in `{value}`")),
            });
            escaped = false;
        } else if quoted && character == '\\' {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
            active = true;
        } else if character.is_whitespace() && !quoted {
            if active {
                words.push(std::mem::take(&mut current));
                active = false;
            }
        } else {
            current.push(character);
            active = true;
        }
    }
    if quoted || escaped {
        return Err(format!("unfinished quoted value in `{value}`"));
    }
    if active {
        words.push(current);
    }
    Ok(words)
}

const fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn valid_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 3
        || parts[0].len() != 4
        || parts[1].len() != 2
        || parts[2].len() != 2
        || !value
            .chars()
            .all(|character| character.is_ascii_digit() || character == '-')
    {
        return false;
    }
    let Ok(year) = parts[0].parse::<u32>() else {
        return false;
    };
    let Ok(month) = parts[1].parse::<u32>() else {
        return false;
    };
    let Ok(day) = parts[2].parse::<u32>() else {
        return false;
    };
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };
    year > 0 && day > 0 && day <= days
}

fn finish_row(
    fields: &mut BTreeMap<String, String>,
    rows: &mut Vec<DebtRow>,
) -> Result<(), String> {
    if fields.is_empty() {
        return Ok(());
    }
    let take = |fields: &mut BTreeMap<String, String>, name: &str| {
        fields
            .remove(name)
            .ok_or_else(|| format!("uncovered_behavior row lacks `{name}`"))
    };
    let source = unquote(&take(fields, "source")?)?;
    let owner_path = unquote(&take(fields, "owner_path")?)?;
    let meaning = unquote(&take(fields, "meaning")?)?;
    let present = parse_csv(&take(fields, "present_languages")?)?;
    let missing = parse_csv(&take(fields, "missing_languages")?)?;
    let observed_on = unquote(&take(fields, "observed_on")?)?;
    let gap = unquote(&take(fields, "gap")?)?;
    if !fields.is_empty() {
        return Err(format!(
            "uncovered_behavior row has unknown fields: {:?}",
            fields.keys().collect::<Vec<_>>()
        ));
    }
    if !valid_date(&observed_on) {
        return Err(format!("invalid observed_on date `{observed_on}`"));
    }
    rows.push(DebtRow {
        source,
        owner_path,
        meaning,
        present,
        missing,
        observed_on,
        gap,
    });
    Ok(())
}

fn parse_debt(text: &str) -> Result<DebtFile, String> {
    let mut header = BTreeMap::new();
    let mut rows = Vec::new();
    let mut saw_root = false;

    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = indentation(line, DEBT_FILE, line_number)?;
        if !saw_root {
            if indent != 0 || trimmed != "language_parity_debt" {
                return Err(format!(
                    "{DEBT_FILE}:{line_number}: expected `language_parity_debt` root"
                ));
            }
            saw_root = true;
            continue;
        }
        if indent == 2 && trimmed.starts_with("uncovered_behavior ") {
            let words = split_quoted_words(trimmed)?;
            if words.first().map(String::as_str) != Some("uncovered_behavior") || words.len() != 15
            {
                return Err(format!(
                    "{DEBT_FILE}:{line_number}: uncovered_behavior must name exactly seven fields"
                ));
            }
            let mut fields = BTreeMap::new();
            let mut pairs = words[1..].iter();
            while let (Some(key), Some(value)) = (pairs.next(), pairs.next()) {
                if fields.insert(key.clone(), quote(value)).is_some() {
                    return Err(format!(
                        "{DEBT_FILE}:{line_number}: duplicate debt row field `{key}`"
                    ));
                }
            }
            finish_row(&mut fields, &mut rows)?;
            continue;
        }
        let Some((key, value)) = trimmed.split_once(' ') else {
            return Err(format!(
                "{DEBT_FILE}:{line_number}: expected a field and value"
            ));
        };
        if indent == 2 && rows.is_empty() {
            if header.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(format!("duplicate debt header field `{key}`"));
            }
        } else {
            return Err(format!(
                "{DEBT_FILE}:{line_number}: unexpected indentation or field `{trimmed}`"
            ));
        }
    }
    if !saw_root {
        return Err(format!("{DEBT_FILE}: missing root"));
    }
    let expected_headers = [
        "measured_on",
        "record_type",
        "source_root",
        "target_languages",
    ];
    if header.keys().map(String::as_str).collect::<BTreeSet<_>>()
        != expected_headers.into_iter().collect()
    {
        return Err(format!(
            "debt header fields must be exactly {expected_headers:?}; found {:?}",
            header.keys().collect::<Vec<_>>()
        ));
    }
    let record_type = unquote(&header["record_type"])?;
    let measured_on = unquote(&header["measured_on"])?;
    let source_root = unquote(&header["source_root"])?;
    let target_languages = unquote(&header["target_languages"])?;
    if record_type != "language_parity_debt" {
        return Err(format!("unexpected record_type `{record_type}`"));
    }
    if source_root != SEED_ROOT {
        return Err(format!("unexpected source_root `{source_root}`"));
    }
    if target_languages != TARGET_LANGUAGES.join(",") {
        return Err(format!("unexpected target_languages `{target_languages}`"));
    }
    if !valid_date(&measured_on) {
        return Err(format!("invalid measured_on date `{measured_on}`"));
    }
    Ok(DebtFile { measured_on, rows })
}

fn expected_row(gap: &LanguageGap, date: &str) -> DebtRow {
    DebtRow {
        source: gap.source.clone(),
        owner_path: gap.owner_path.clone(),
        meaning: gap.meaning.clone(),
        present: gap.present.clone(),
        missing: gap.missing.clone(),
        observed_on: date.to_owned(),
        gap: gap.description(),
    }
}

fn render_debt(gaps: &[LanguageGap], date: &str) -> String {
    let mut output = format!(
        "language_parity_debt\n  record_type \"language_parity_debt\"\n  measured_on {}\n  \
         target_languages \"en,ru,hi,zh,es\"\n  source_root \"data/seed\"\n",
        quote(date)
    );
    for gap in gaps {
        let row = expected_row(gap, date);
        let _ = writeln!(
            output,
            "  uncovered_behavior source {} owner_path {} meaning {} present_languages {} \
             missing_languages {} observed_on {} gap {}",
            quote(&row.source),
            quote(&row.owner_path),
            quote(&row.meaning),
            quote(&row.present.join(",")),
            quote(&row.missing.join(",")),
            quote(&row.observed_on),
            quote(&row.gap)
        );
    }
    output
}

fn check_debt(gaps: &[LanguageGap], text: &str) -> Vec<String> {
    let debt = match parse_debt(text) {
        Ok(debt) => debt,
        Err(error) => return vec![error],
    };
    let mut failures = Vec::new();
    let mut actual = BTreeMap::new();
    for gap in gaps {
        actual.insert(gap.key(), gap);
    }
    let mut declared: BTreeMap<(String, String), &DebtRow> = BTreeMap::new();
    for row in &debt.rows {
        let key = row.key();
        if declared.insert(key.clone(), row).is_some() {
            failures.push(format!(
                "duplicate debt row for {} at structural owner {}",
                key.0, key.1
            ));
        }
    }
    for (key, gap) in &actual {
        match declared.get(key) {
            None => failures.push(format!(
                "missing debt row for {} meaning `{}` (missing {})",
                gap.source,
                gap.meaning,
                gap.missing.join(",")
            )),
            Some(row) if **row != expected_row(gap, &debt.measured_on) => failures.push(format!(
                "stale debt row for {} meaning `{}`; present/missing languages, date, or gap text no longer matches",
                gap.source, gap.meaning
            )),
            Some(_) => {}
        }
    }
    for (key, row) in declared {
        if !actual.contains_key(&key) {
            failures.push(format!(
                "stale debt row for {} meaning `{}`; the structural owner is complete or absent",
                row.source, row.meaning
            ));
        }
    }
    if failures.is_empty() && text != render_debt(gaps, &debt.measured_on) {
        failures.push(format!(
            "{DEBT_FILE} is not in canonical source/owner order; regenerate it with --write --date {}",
            debt.measured_on
        ));
    }
    failures
}

pub fn run(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    let mut root = PathBuf::from(".");
    let mut write_date: Option<String> = None;
    let mut count_only = false;
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--repo" => {
                root = PathBuf::from(args.next().ok_or("--repo requires a path")?);
            }
            "--write" => {
                if write_date.is_none() {
                    write_date = Some(String::new());
                }
            }
            "--date" => {
                let date = args.next().ok_or("--date requires YYYY-MM-DD")?;
                write_date = Some(date);
            }
            "--count" => count_only = true,
            "--check" => {}
            "--help" | "-h" => {
                return Ok(
                    "usage: check-language-parity.rs [--repo <path>] [--check|--count|--write --date YYYY-MM-DD]"
                        .to_owned(),
                );
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    let gaps = repository_gaps(&root)?;
    if count_only {
        if write_date.is_some() {
            return Err("--count and --write are mutually exclusive".to_owned());
        }
        return Ok(gaps.len().to_string());
    }
    if let Some(date) = write_date {
        if !valid_date(&date) {
            return Err("--write requires --date YYYY-MM-DD".to_owned());
        }
        fs::write(root.join(DEBT_FILE), render_debt(&gaps, &date))
            .map_err(|error| format!("cannot write {DEBT_FILE}: {error}"))?;
        return Ok(format!(
            "recorded {} language-parity gaps in {DEBT_FILE}",
            gaps.len()
        ));
    }
    let debt = fs::read_to_string(root.join(DEBT_FILE))
        .map_err(|error| format!("cannot read {DEBT_FILE}: {error}"))?;
    let failures = check_debt(&gaps, &debt);
    if failures.is_empty() {
        Ok(format!(
            "language parity: {} explicit dated gaps; debt rows are exact",
            gaps.len()
        ))
    } else {
        Err(failures.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_gap() -> LanguageGap {
        gaps_from_documents([(
            "data/seed/catalog.lino",
            "catalog\n  asset ETH\n    lexeme en\n      surface\n        text ether\n    lexeme es\n      surface\n        text éter\n",
        )])
        .expect("fixture parses")
        .remove(0)
    }

    #[test]
    fn derives_lexeme_owners_from_structure_not_filename_or_root_name() {
        let gap = sample_gap();
        assert_eq!(gap.source, "data/seed/catalog.lino");
        assert_eq!(gap.meaning, "asset ETH");
        assert_eq!(gap.present, ["en", "es"]);
        assert_eq!(gap.missing, ["ru", "hi", "zh"]);
        assert_eq!(gap.owner_path, "7:catalog9:asset ETH");
    }

    #[test]
    fn complete_owner_is_not_debt_and_non_lexeme_language_rows_do_not_count() {
        let text = "registry\n  language es\n    status partial\n  idea\n    lexeme en\n    lexeme ru\n    lexeme hi\n    lexeme zh\n    lexeme es\n";
        assert!(
            gaps_from_documents([("data/seed/not-a-meanings-file.lino", text)])
                .expect("fixture parses")
                .is_empty()
        );
    }

    #[test]
    fn duplicate_language_under_one_owner_is_rejected() {
        let error = gaps_from_documents([(
            "data/seed/duplicate.lino",
            "root\n  idea\n    lexeme en\n    lexeme en\n",
        )])
        .expect_err("a duplicate lexeme is ambiguous source structure");
        assert!(error.contains("duplicate `lexeme en`"), "{error}");
    }

    #[test]
    fn exact_generated_debt_is_accepted() {
        let gaps = vec![sample_gap()];
        let debt = render_debt(&gaps, "2026-09-17");
        assert!(check_debt(&gaps, &debt).is_empty());
    }

    #[test]
    fn missing_debt_row_is_rejected() {
        let gaps = vec![sample_gap()];
        let debt = render_debt(&[], "2026-09-17");
        let failures = check_debt(&gaps, &debt);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("missing debt row"))
        );
    }

    #[test]
    fn duplicate_debt_row_is_rejected() {
        let gaps = vec![sample_gap()];
        let one = render_debt(&gaps, "2026-09-17");
        let row = one
            .lines()
            .find(|line| line.starts_with("  uncovered_behavior "))
            .expect("rendered row");
        let duplicate = format!("{one}{row}\n");
        let failures = check_debt(&gaps, &duplicate);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("duplicate debt row"))
        );
    }

    #[test]
    fn stale_debt_row_is_rejected_after_the_owner_becomes_complete() {
        let debt = render_debt(&[sample_gap()], "2026-09-17");
        let failures = check_debt(&[], &debt);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("stale debt row"))
        );
    }

    #[test]
    fn invalid_calendar_date_is_rejected() {
        let debt = render_debt(&[sample_gap()], "2026-02-30");
        let failures = check_debt(&[sample_gap()], &debt);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("invalid") && failure.contains("date"))
        );
    }
}
