//! Independent task acceptance (issue #1166): every clause of a task gets a
//! verdict -- discharged, contradicted, unresolved or inapplicable -- with a
//! justification and the clause's source span. The specification is read
//! from the task text alone and the check reads only the candidate's files and
//! observed output, never the derivation that wrote the candidate, so a
//! candidate and a test generated from the same misunderstanding are rejected
//! together. A judgment clause ("clear", "best practices") stays unresolved
//! for review; nothing is ever reported as a generic style pass.
//! JavaScript twin: js/agentic/task_acceptance.mjs.
//!
//! Supported subset (anything else is an unresolved `unread` clause):
//!
//! - `output`: a print or output clause; its quoted, code-spanned or fenced
//!   lines with explicit counts, or the conditional statements of
//!   `task_condition` checked row by row;
//! - `program_file`: the requested language's file extension;
//! - `comments`: a comment line in the language's comment syntax;
//! - `run_instructions`: a leading comment that names the program file;
//! - `workflow_*`: path and directory literals (with negated exclusions), push
//!   and `pull_request` triggers, running the program, verifying the expected
//!   output, the runtime, an observed conclusion, a named file and a badge;
//!   branch filters are not read.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use regex::Regex;
use serde_json::{Value, json};

use super::task_condition::{
    TaskLiteral, conditional_statement, condition_table, fence_lines, first_role_match,
    task_literals,
};
use super::work_item_requirements::work_item_requirements;
use super::work_item_steps::{fill, template};
use crate::coding::catalog::program_language_by_slug;
use crate::engine::{ExecutionRecipe, normalize_prompt, stable_id};
use crate::implementation_language::requested_in;
use crate::intent_formalization::explicit_count;
use crate::seed::{self, parser::parse_lino};

const CONTRACTS: &str = include_str!("../../embedded/data/meta/stdout-program-contracts.lino");
const JUDGMENT: &str = "judgment_required";
const UNREAD: [&str; 5] = [
    "unread",
    "output_unread",
    "language_unread",
    "comment_syntax_unread",
    JUDGMENT,
];
const VERDICTS: [&str; 4] = ["inapplicable", "discharged", "unresolved", "contradicted"];

fn pattern(cell: &'static OnceLock<Regex>, source: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(source).expect("StaticTaskAcceptancePattern"))
}

fn heading_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    pattern(&CELL, r"^#{1,6}\s+(.*)$")
}

fn checkbox_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    pattern(&CELL, r"^\[[ xX]\]\s*")
}

fn file_name_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    pattern(&CELL, r"^[\p{L}\p{N}_.-]*[\p{L}\p{N}_]\.[A-Za-z][A-Za-z0-9]{0,4}$")
}

fn trigger_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    pattern(&CELL, r#"^(?:on|"on"|'on'|true)\s*:(.*)$"#)
}

fn jobs_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    pattern(&CELL, r"(?m)^jobs\s*:")
}

fn document_pattern() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    pattern(&CELL, r"(?i)\.(?:md|markdown|rst|txt)$")
}

fn basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn extension(path: &str) -> String {
    let name = basename(path);
    match name.rfind('.') {
        Some(dot) if dot > 0 => name[dot..].to_lowercase(),
        _ => String::new(),
    }
}

fn stem(path: &str) -> &str {
    let name = basename(path);
    match name.rfind('.') {
        Some(dot) if dot > 0 => &name[..dot],
        _ => name,
    }
}

fn lowered(text: &str) -> String {
    normalize_prompt(text).to_lowercase()
}

fn relative(path: &str) -> &str {
    path.strip_prefix("./").unwrap_or(path)
}

fn mentions(role: &str, words: &str) -> bool {
    seed::lexicon().mentions_role(role, words)
}

fn evidenced(slug: &str, words: &str) -> bool {
    seed::lexicon()
        .meaning(slug)
        .is_some_and(|meaning| meaning.evidenced_in(words))
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn rank(verdict: &str) -> usize {
    VERDICTS
        .iter()
        .position(|candidate| *candidate == verdict)
        .unwrap_or_default()
}

/// Stdout as lines, trailing blanks dropped and each line's trailing whitespace trimmed.
#[must_use]
pub fn output_lines(stdout: &str) -> Vec<String> {
    let mut lines: Vec<String> = stdout
        .replace("\r\n", "\n")
        .split('\n')
        .map(|line| line.trim_end().to_owned())
        .collect();
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

/// The line-comment syntax of `language` from the stdout program contracts.
#[must_use]
pub fn comment_prefix(language: &str) -> Option<String> {
    let root = parse_lino(CONTRACTS);
    let contract = root
        .children
        .first()?
        .children
        .iter()
        .find(|node| node.name == "language" && node.id == language)?;
    let prefix = contract.find_child_value("comment");
    (!prefix.is_empty()).then(|| prefix.to_owned())
}

/// File-name tokens of `clause`, quoted or bare.
fn file_names(clause: &str, literals: &[TaskLiteral]) -> Vec<String> {
    let mut out: Vec<String> = literals
        .iter()
        .filter(|literal| file_name_pattern().is_match(&literal.text))
        .map(|literal| literal.text.clone())
        .collect();
    for word in clause.split_whitespace() {
        let token = word
            .trim_start_matches(['(', '"', '\'', '`', '«', '“'])
            .trim_end_matches([')', '"', '\'', '`', '»', '”', ',', ';', ':', '!', '?', '。', '，']);
        let token = token.strip_suffix('.').unwrap_or(token);
        if file_name_pattern().is_match(token)
            && !token.contains('/')
            && !out.iter().any(|name| name == token)
        {
            out.push(token.to_owned());
        }
    }
    out
}

/// One requirement with its section heading and list lead.
struct ClauseContext {
    clause: String,
    start: usize,
    end: usize,
    heading: String,
    lead: Option<String>,
    is_lead: bool,
}

/// Each requirement with its section heading and list lead.
fn clause_contexts(text: &str) -> Vec<ClauseContext> {
    let mut headings: Vec<(usize, String)> = Vec::new();
    let mut offset = 0;
    for line in text.split('\n') {
        if let Some(found) = heading_pattern().captures(line.trim()) {
            headings.push((offset, found[1].to_owned()));
        }
        offset += line.len() + 1;
    }
    let mut lead: Option<String> = None;
    let mut section: Option<usize> = None;
    let mut out = Vec::new();
    for (raw, (start, end)) in work_item_requirements(text) {
        let heading = headings.iter().rposition(|(at, _)| *at <= start);
        if heading != section {
            section = heading;
            lead = None;
        }
        let unboxed = checkbox_pattern().replace(raw.trim(), "");
        let clause = unboxed.replace("**", "").trim().to_owned();
        let is_lead = clause.ends_with(':') || clause.ends_with('：');
        out.push(ClauseContext {
            heading: heading.map(|index| headings[index].1.clone()).unwrap_or_default(),
            lead: if is_lead { None } else { lead.clone() },
            clause: clause.clone(),
            start,
            end,
            is_lead,
        });
        if is_lead {
            lead = Some(clause);
        }
    }
    out
}

fn workflow_context(text: &str) -> bool {
    let words = lowered(text);
    mentions("ci_workflow_request", &words) || mentions("workflow-context-noun", &words)
}

/// The workflow checks a workflow-context clause asks for.
fn workflow_checks(
    clause: &str,
    words: &str,
    literals: &[TaskLiteral],
    language: Option<&str>,
) -> Vec<Value> {
    let mut checks = Vec::new();
    let negation = first_role_match(&clause.to_lowercase(), "statement_negation_cue");
    let paths: Vec<&TaskLiteral> = literals
        .iter()
        .filter(|literal| literal.text.contains('/') && !literal.text.contains(char::is_whitespace))
        .collect();
    if !paths.is_empty() {
        let after = |literal: &TaskLiteral| negation.is_some_and(|found| literal.start >= found.0);
        let included: Vec<&str> = paths.iter().filter(|literal| !after(literal)).map(|literal| relative(&literal.text)).collect();
        let excluded: Vec<&str> = paths.iter().filter(|literal| after(literal)).map(|literal| relative(&literal.text)).collect();
        return vec![json!({ "check": "workflow_path", "included": included, "excluded": excluded })];
    }
    if negation.is_none() {
        let pairs: Vec<(String, String)> = template("workflow-trigger-events")
            .and_then(|source| serde_json::from_str(&source).ok())
            .unwrap_or_default();
        let events: Vec<&str> = pairs
            .iter()
            .filter(|(role, _)| mentions(role, words))
            .map(|(_, event)| event.as_str())
            .collect();
        if !events.is_empty() {
            checks.push(json!({ "check": "workflow_triggers", "events": events }));
        }
    }
    if mentions("workflow-execution-verb", words) {
        checks.push(json!({ "check": "workflow_runs_program" }));
    }
    if evidenced("print_stdout", words) || mentions("output_obligation_verb", words) {
        let lines: Vec<&str> = literals
            .iter()
            .filter(|literal| !literal.escaped)
            .map(|literal| literal.text.as_str())
            .collect();
        let lines = if lines.is_empty() { Value::Null } else { json!(lines) };
        checks.push(json!({ "check": "workflow_verifies_output", "lines": lines }));
    }
    if mentions("workflow-status-obligation", words) {
        checks.push(json!({ "check": "workflow_status" }));
    }
    if mentions("ci_badge_obligation", words) {
        checks.push(json!({ "check": "workflow_badge" }));
    }
    if mentions("file_naming_obligation", words) {
        let names = file_names(clause, literals);
        if !names.is_empty() {
            checks.push(json!({ "check": "workflow_file_name", "names": names }));
        }
    }
    if let Some(language) = language
        && requested_in(clause).as_deref() == Some(language)
    {
        checks.push(json!({ "check": "workflow_runtime", "language": language }));
    }
    if checks.is_empty() {
        checks.push(json!({ "check": "workflow_exists" }));
    }
    checks
}

/// What one clause asks of a candidate.
fn clause_checks(context: &ClauseContext, language: Option<&str>, lead_illustrative: bool) -> Vec<Value> {
    let clause = context.clause.as_str();
    if context.is_lead {
        return vec![json!({ "check": "inapplicable", "reason": "lead_clause" })];
    }
    if lead_illustrative {
        return vec![json!({ "check": "inapplicable", "reason": "illustrative_item" })];
    }
    let words = lowered(clause);
    let literals = task_literals(clause);
    let mut checks = Vec::new();
    if workflow_context(clause)
        || context.lead.as_deref().is_some_and(workflow_context)
        || workflow_context(&context.heading)
    {
        checks.extend(workflow_checks(clause, &words, &literals, language));
    } else {
        let output = evidenced("print_stdout", &words) || mentions("output_obligation_verb", &words);
        if output {
            checks.push(output_check(clause, &literals));
        }
        let runs = mentions("run-instruction-obligation", &words);
        if runs {
            checks.push(json!({ "check": "run_instructions" }));
        }
        if !runs && mentions("code-comment-obligation", &words) {
            checks.push(json!({ "check": "comments" }));
        }
        if mentions("program-file-obligation", &words)
            || (!output && mentions("coding_request_verb", &words) && requested_in(clause).is_some())
        {
            checks.push(json!({ "check": "program_file" }));
        }
    }
    // A named example settles the judgment it illustrates (`a meaningful name like ci.yml`).
    if mentions("judgment-obligation", &words)
        && !checks.iter().any(|check| check["check"] == "workflow_file_name")
    {
        checks.push(json!({ "check": "judgment" }));
    }
    if checks.is_empty() {
        checks.push(json!({ "check": "unread" }));
    }
    checks
}

/// An output clause as fixed lines, a conditional statement or a gap.
fn output_check(clause: &str, literals: &[TaskLiteral]) -> Value {
    if literals.iter().any(|literal| literal.escaped) {
        return json!({ "check": "output", "gap": "escaped_quote" });
    }
    if let Some(statement) = conditional_statement(clause) {
        if let Some(gap) = statement.get("gap") {
            return json!({ "check": "output", "gap": gap });
        }
        return json!({ "check": "output", "statement": statement });
    }
    // A repeated mention restates one line; only an explicit count (`twice`) repeats it.
    let mentions: Vec<Vec<String>> = literals
        .iter()
        .enumerate()
        .map(|(index, literal)| {
            let next = literals.get(index + 1).map_or(clause.len(), |next| next.start);
            let count = explicit_count(clause.get(literal.end..next).unwrap_or(""));
            vec![literal.text.clone(); count]
        })
        .collect();
    let mut lines = merged_lines(&mentions);
    lines.extend(fence_lines(clause).unwrap_or_default());
    json!({ "check": "output", "lines": lines })
}

/// Each distinct line as often as the clause that repeats it most, in first-seen order.
fn merged_lines(groups: &[Vec<String>]) -> Vec<String> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for lines in groups {
        let mut local: Vec<(&String, usize)> = Vec::new();
        for line in lines {
            match local.iter_mut().find(|(seen, _)| *seen == line) {
                Some(entry) => entry.1 += 1,
                None => local.push((line, 1)),
            }
        }
        for (line, count) in local {
            match counts.iter_mut().find(|(seen, _)| seen == line) {
                Some(entry) => entry.1 = entry.1.max(count),
                None => counts.push((line.clone(), count)),
            }
        }
    }
    counts
        .into_iter()
        .flat_map(|(line, count)| std::iter::repeat_n(line, count))
        .collect()
}

/// The specification's output: `{lines}`, `{table}` or `{gap}`.
fn task_output(outputs: &[&Value]) -> Value {
    let mut statements: Vec<Value> = Vec::new();
    for statement in outputs.iter().filter_map(|check| check.get("statement")) {
        let otherwise = statement["otherwise"].as_array().cloned().unwrap_or_default();
        if statement["continues"] == true {
            if let Some(last) = statements.last_mut()
                && let Some(branch) = last["otherwise"].as_array_mut()
            {
                branch.extend(otherwise);
            }
        } else {
            statements.push(statement.clone());
        }
    }
    let fixed: Vec<Vec<String>> = outputs
        .iter()
        .map(|check| strings(&check["lines"]))
        .filter(|lines| !lines.is_empty())
        .collect();
    if let Some(gap) = outputs.iter().find_map(|check| check.get("gap")) {
        return json!({ "gap": gap });
    }
    if !statements.is_empty() && !fixed.is_empty() {
        return json!({ "gap": "mixed_conditional_output" });
    }
    if !statements.is_empty() {
        return match condition_table(&statements) {
            Ok(table) => json!({ "table": table }),
            Err(gap) => json!({ "gap": gap }),
        };
    }
    json!({ "lines": merged_lines(&fixed) })
}

/// The typed obligations of `text` -- `{task_identity, language, clauses,
/// output, meaning}` where `output` is `{lines}`, `{table}` or `{gap}` and
/// `meaning` is the canonical JSON that equivalent paraphrases share.
#[must_use]
pub fn specify_task(text: &str) -> Value {
    let language = requested_in(text);
    let clauses: Vec<Value> = clause_contexts(text)
        .iter()
        .map(|context| {
            let lead_illustrative = context
                .lead
                .as_deref()
                .is_some_and(|lead| mentions("illustrative-marker", &lowered(lead)));
            json!({
                "text": context.clause,
                "span": [context.start, context.end],
                "required": !mentions("optional-obligation-marker", &lowered(&context.clause)),
                "checks": clause_checks(context, language.as_deref(), lead_illustrative),
            })
        })
        .collect();
    let checks: Vec<&Value> = clauses
        .iter()
        .flat_map(|clause| clause["checks"].as_array().map_or(&[][..], Vec::as_slice))
        .collect();
    let outputs: Vec<&Value> = checks.iter().copied().filter(|check| check["check"] == "output").collect();
    let output = task_output(&outputs);
    let pick = |name: &str, field: &str| -> BTreeSet<String> {
        checks
            .iter()
            .filter(|check| check["check"] == name)
            .flat_map(|check| strings(&check[field]))
            .collect()
    };
    let kinds: BTreeSet<&str> = checks
        .iter()
        .filter_map(|check| check["check"].as_str())
        .filter(|kind| *kind != "unread" && *kind != "inapplicable")
        .collect();
    // serde_json sorts object keys, which is the order the JavaScript twin writes.
    let meaning = json!({
        "kinds": kinds,
        "language": language,
        "output": output,
        "workflow": {
            "events": pick("workflow_triggers", "events"),
            "excluded": pick("workflow_path", "excluded"),
            "included": pick("workflow_path", "included"),
            "names": pick("workflow_file_name", "names"),
        },
    });
    json!({
        "task_identity": stable_id("task", text),
        "language": language,
        "clauses": clauses,
        "output": output,
        "meaning": meaning.to_string(),
    })
}

fn clean(word: &str) -> String {
    let word = word.trim();
    let word = word.strip_prefix(['"', '\'']).unwrap_or(word);
    word.strip_suffix(['"', '\'']).unwrap_or(word).to_owned()
}

/// The events of a workflow's `on:` key.
#[must_use]
pub fn workflow_events(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.split('\n').collect();
    let Some(index) = lines.iter().position(|line| trigger_pattern().is_match(line)) else {
        return Vec::new();
    };
    let inline = trigger_pattern()
        .captures(lines[index])
        .map(|found| found[1].to_owned())
        .unwrap_or_default();
    let inline = inline.split('#').next().unwrap_or("").trim();
    if let Some(list) = inline.strip_prefix('[') {
        let list = list.strip_suffix(']').unwrap_or(list);
        return list.split(',').map(clean).filter(|word| !word.is_empty()).collect();
    }
    if !inline.is_empty() {
        return vec![clean(inline)];
    }
    let mut events = Vec::new();
    let mut indent = None;
    for line in &lines[index + 1..] {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let depth = line.len() - line.trim_start().len();
        if depth == 0 {
            break;
        }
        if *indent.get_or_insert(depth) != depth {
            continue;
        }
        let entry = trimmed.strip_prefix('-').map_or(trimmed, str::trim_start);
        let word = clean(entry.split(':').next().unwrap_or(""));
        if !word.is_empty() {
            events.push(word);
        }
    }
    events
}

/// The comment lines of `source` and its leading comment block.
fn comment_lines(source: &str, prefix: &str) -> (Vec<String>, Vec<String>) {
    let lines: Vec<&str> = source.split('\n').map(str::trim).collect();
    let comments = lines
        .iter()
        .filter(|line| line.starts_with(prefix) && !line.starts_with("#!"))
        .map(|line| (*line).to_owned())
        .collect();
    let mut leading = Vec::new();
    for line in &lines {
        if line.is_empty() || line.starts_with("#!") {
            continue;
        }
        if !line.starts_with(prefix) {
            break;
        }
        leading.push((*line).to_owned());
    }
    (comments, leading)
}

/// The observation for `inputs` compared against `expected`.
fn observed_lines(observations: &[Value], inputs: &Value, expected: &[String]) -> Option<bool> {
    let empty = json!([]);
    let observation = observations.iter().find(|candidate| {
        candidate.get("args").filter(|args| !args.is_null()).unwrap_or(&empty) == inputs
    })?;
    let lines = output_lines(observation["stdout"].as_str().unwrap_or(""));
    let compared = if observation["tail"] == true {
        &lines[lines.len().saturating_sub(expected.len())..]
    } else {
        &lines[..]
    };
    Some(compared == expected)
}

type Verdict = (&'static str, String);

fn verdict(verdict: &'static str, justification: &str) -> Verdict {
    (verdict, justification.to_owned())
}

fn check_output(specification: &Value, candidate: &Value) -> Verdict {
    let output = &specification["output"];
    if let Some(gap) = output.get("gap") {
        return verdict("unresolved", gap.as_str().unwrap_or("unread"));
    }
    let observations = candidate["observations"].as_array().map_or(&[][..], Vec::as_slice);
    if let Some(table) = output.get("table") {
        for row in table["rows"].as_array().map_or(&[][..], Vec::as_slice) {
            match observed_lines(observations, &row["args"], &strings(&row["lines"])) {
                None => return verdict("unresolved", "condition_rows_unobserved"),
                Some(false) => return verdict("contradicted", "condition_row_differs"),
                Some(true) => {}
            }
        }
        return verdict("discharged", "condition_rows_match");
    }
    let lines = strings(&output["lines"]);
    if lines.is_empty() {
        return verdict("unresolved", "output_unread");
    }
    match observed_lines(observations, &json!([]), &lines) {
        None => verdict("unresolved", "output_unobserved"),
        Some(true) => verdict("discharged", "output_matches"),
        Some(false) => verdict("contradicted", "output_differs"),
    }
}

/// One candidate file, `(path, source)`.
type CandidateFile<'a> = (&'a str, &'a str);

fn candidate_files(candidate: &Value) -> Vec<CandidateFile<'_>> {
    candidate["files"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .map(|file| (file["path"].as_str().unwrap_or(""), file["source"].as_str().unwrap_or("")))
        .collect()
}

/// One check against the candidate as `(verdict, justification)`.
fn check_one(check: &Value, specification: &Value, candidate: &Value) -> Verdict {
    let files = candidate_files(candidate);
    let program = files
        .iter()
        .copied()
        .find(|(path, _)| Some(*path) == candidate["program_path"].as_str());
    let slug = specification["language"].as_str();
    match check["check"].as_str().unwrap_or("") {
        "inapplicable" => verdict("inapplicable", check["reason"].as_str().unwrap_or("")),
        "unread" => verdict("unresolved", "unread"),
        "judgment" => verdict("unresolved", JUDGMENT),
        "output" => check_output(specification, candidate),
        "program_file" => {
            let Some((path, _)) = program else {
                return verdict("unresolved", "program_unobserved");
            };
            let Some(language) = slug.and_then(program_language_by_slug) else {
                return verdict("unresolved", "language_unread");
            };
            if extension(path) == extension(&language.save_as) {
                verdict("discharged", "extension_matches")
            } else {
                verdict("contradicted", "extension_differs")
            }
        }
        kind @ ("comments" | "run_instructions") => {
            let Some((path, source)) = program else {
                return verdict("unresolved", "program_unobserved");
            };
            let Some(prefix) = slug.and_then(comment_prefix) else {
                return verdict("unresolved", "comment_syntax_unread");
            };
            let (comments, leading) = comment_lines(source, &prefix);
            let name = basename(path);
            if kind == "run_instructions" {
                return if leading.iter().any(|line| line.contains(name)) {
                    verdict("discharged", "run_instructions_present")
                } else {
                    verdict("contradicted", "run_instructions_missing")
                };
            }
            if comments.iter().any(|line| !line.contains(name)) {
                verdict("discharged", "comments_present")
            } else {
                verdict("contradicted", "comments_missing")
            }
        }
        _ => check_workflow(check, specification, candidate, &files, program),
    }
}

/// The workflow files and every candidate file a workflow names, joined.
fn workflow_text(files: &[CandidateFile<'_>], workflows: &[CandidateFile<'_>]) -> String {
    files
        .iter()
        .filter(|(path, _)| {
            workflows.iter().any(|(workflow_path, workflow_source)| {
                workflow_path == path || workflow_source.contains(basename(path))
            })
        })
        .map(|(_, source)| *source)
        .collect::<Vec<_>>()
        .join("\n")
}

fn check_workflow(
    check: &Value,
    specification: &Value,
    candidate: &Value,
    files: &[CandidateFile<'_>],
    program: Option<CandidateFile<'_>>,
) -> Verdict {
    let workflows: Vec<CandidateFile<'_>> = files
        .iter()
        .copied()
        .filter(|(path, source)| {
            std::path::Path::new(path).extension().is_some_and(|extension| extension == "yml" || extension == "yaml")
                && jobs_pattern().is_match(source)
        })
        .collect();
    let kind = check["check"].as_str().unwrap_or("");
    if kind == "workflow_badge" {
        let badge = files.iter().any(|(path, source)| {
            document_pattern().is_match(path)
                && workflows.iter().any(|(workflow, _)| source.contains(basename(workflow)))
        });
        return if badge {
            verdict("discharged", "badge_present")
        } else {
            verdict("unresolved", "badge_unobserved")
        };
    }
    if kind == "workflow_status" {
        return match candidate["workflow_conclusion"].as_str() {
            None => verdict("unresolved", "workflow_status_unobserved"),
            Some("success") => verdict("discharged", "workflow_passed"),
            Some(_) => verdict("contradicted", "workflow_failed"),
        };
    }
    if workflows.is_empty() {
        return verdict("contradicted", "workflow_missing");
    }
    let paths: Vec<&str> = workflows.iter().map(|(path, _)| relative(path)).collect();
    let at = |path: &str, place: &str| {
        if place.ends_with('/') {
            path.starts_with(place)
        } else {
            path == place
        }
    };
    let outcome = |holds: bool, yes: &str, no: &str| {
        if holds {
            verdict("discharged", yes)
        } else {
            verdict("contradicted", no)
        }
    };
    match kind {
        "workflow_path" => {
            let (included, excluded) = (strings(&check["included"]), strings(&check["excluded"]));
            let fits = paths.iter().any(|path| {
                included.iter().all(|place| at(path, place))
                    && !excluded.iter().any(|place| at(path, place))
            });
            outcome(fits, "workflow_path_matches", "workflow_path_differs")
        }
        "workflow_triggers" => {
            let events: BTreeSet<String> = workflows
                .iter()
                .flat_map(|(_, source)| workflow_events(source))
                .collect();
            let all = strings(&check["events"]).iter().all(|event| events.contains(event));
            outcome(all, "workflow_triggers_match", "workflow_trigger_missing")
        }
        "workflow_runs_program" => {
            let Some((path, _)) = program else {
                return verdict("unresolved", "program_unobserved");
            };
            let text = workflow_text(files, &workflows);
            let runs = text.contains(basename(path)) || text.contains(stem(path));
            outcome(runs, "workflow_runs_program", "workflow_does_not_run_program")
        }
        "workflow_verifies_output" => {
            let lines = if check["lines"].is_null() {
                strings(&specification["output"]["lines"])
            } else {
                strings(&check["lines"])
            };
            if lines.is_empty() {
                return verdict("unresolved", "output_unread");
            }
            let text = workflow_text(files, &workflows);
            let verifies = lines.iter().all(|line| text.contains(line.as_str()));
            outcome(verifies, "workflow_verifies_output", "workflow_does_not_verify_output")
        }
        "workflow_file_name" => {
            let names = strings(&check["names"]);
            if paths.iter().any(|path| names.iter().any(|name| name == basename(path))) {
                verdict("discharged", "workflow_file_named")
            } else {
                verdict("unresolved", JUDGMENT)
            }
        }
        "workflow_runtime" => {
            let slug = check["language"].as_str().unwrap_or("");
            let text = workflow_text(files, &workflows).to_lowercase();
            let named = program_language_by_slug(slug)
                .is_some_and(|language| text.contains(&language.name.to_lowercase()));
            outcome(text.contains(slug) || named, "workflow_runtime_present", "workflow_runtime_missing")
        }
        _ => verdict("discharged", "workflow_present"),
    }
}

/// The verdict of every clause of `specification` for `candidate`
/// (`{task_identity, program_path, files: [{path, source}], observations:
/// [{args, stdout, tail?}], workflow_conclusion?}`), with the counts and
/// whether the task is complete. A required clause that is contradicted, or
/// unresolved for any reason but judgment, keeps it incomplete.
#[must_use]
pub fn check_task(specification: &Value, candidate: &Value) -> Value {
    let same_task = candidate["task_identity"] == specification["task_identity"];
    let verdicts: Vec<Value> = specification["clauses"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .map(|clause| {
            let mut current = verdict("inapplicable", "no_obligation");
            for check in clause["checks"].as_array().map_or(&[][..], Vec::as_slice) {
                let next = if same_task || check["check"] == "inapplicable" {
                    check_one(check, specification, candidate)
                } else {
                    verdict("contradicted", "task_identity_differs")
                };
                if rank(next.0) > rank(current.0) {
                    current = next;
                }
            }
            json!({
                "clause": clause["text"],
                "span": clause["span"],
                "required": clause["required"],
                "verdict": current.0,
                "justification": current.1,
            })
        })
        .collect();
    let count = |name: &str| verdicts.iter().filter(|entry| entry["verdict"] == name).count();
    let complete = !verdicts.iter().any(|entry| {
        entry["required"] == true
            && (entry["verdict"] == "contradicted"
                || (entry["verdict"] == "unresolved" && entry["justification"] != JUDGMENT))
    });
    json!({
        "discharged": count("discharged"),
        "contradicted": count("contradicted"),
        "unresolved": count("unresolved"),
        "inapplicable": count("inapplicable"),
        "complete": complete,
        "verdicts": verdicts,
    })
}

/// The verdict lines a finished recipe's answer carries, or an empty string
/// when every clause of `objective` is discharged or inapplicable. The
/// candidate is the recipe's files and the last non-empty verification
/// output, compared at its tail.
#[must_use]
#[allow(clippy::literal_string_with_formatting_args)]
pub fn acceptance_report(objective: &str, recipe: &ExecutionRecipe, outputs: &[String]) -> String {
    let specification = specify_task(objective);
    let stdout = outputs
        .iter()
        .rev()
        .find(|output| !output.trim().is_empty())
        .map_or("", String::as_str);
    let mut files = vec![json!({ "path": recipe.path, "source": recipe.source })];
    files.extend(
        recipe
            .supporting_files
            .iter()
            .map(|file| json!({ "path": file.path, "source": file.source })),
    );
    let report = check_task(
        &specification,
        &json!({
            "task_identity": specification["task_identity"],
            "program_path": recipe.path,
            "files": files,
            "observations": [{ "args": [], "stdout": stdout, "tail": true }],
        }),
    );
    let open: Vec<&Value> = report["verdicts"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter(|entry| entry["verdict"] != "discharged" && entry["verdict"] != "inapplicable")
        .collect();
    // A clause outside the supported subset is a reader gap, not evidence against
    // the candidate: the report is appended only when some verdict is actionable
    // (contradicted, or unresolved for want of an observation).
    let actionable = open.iter().any(|entry| {
        !UNREAD.contains(&entry["justification"].as_str().unwrap_or(""))
    });
    if !actionable {
        return String::new();
    }
    let counts: Vec<(String, String)> = ["discharged", "contradicted", "unresolved", "inapplicable"]
        .iter()
        .map(|name| (format!("{{{name}}}"), report[*name].to_string()))
        .collect();
    let count_pairs: Vec<(&str, &str)> = counts.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect();
    let mut lines = vec![fill("task-acceptance-heading", &count_pairs)];
    for entry in open {
        let span = &entry["span"];
        let (start, end) = (span[0].to_string(), span[1].to_string());
        lines.push(fill(
            "task-acceptance-line",
            &[
                ("{verdict}", entry["verdict"].as_str().unwrap_or("")),
                ("{justification}", entry["justification"].as_str().unwrap_or("")),
                ("{start}", &start),
                ("{end}", &end),
                ("{clause}", &entry["clause"].as_str().unwrap_or("").replace('\n', " ")),
            ],
        ));
    }
    let closing = if report["complete"] == true {
        "task-acceptance-review"
    } else {
        "task-acceptance-incomplete"
    };
    lines.push(fill(closing, &[]));
    lines.join("\n")
}
