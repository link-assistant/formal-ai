//! The output half of independent task acceptance (issue #1166): exact output
//! literals and the supported subset of output conditions, read straight from
//! the task text without the reader that bound the program's stdout.
//! JavaScript twin: js/agentic/task_condition.mjs.
//!
//! Supported condition subset (anything else is an unresolved gap):
//!
//! ```text
//! statement := effect MARKER condition [SEP OTHERWISE effect]
//!            | MARKER condition SEP effect [SEP OTHERWISE effect]
//! condition := atom (CONNECTIVE atom)*   one connective kind per condition
//! atom      := operand [NEGATION] relation operand | operand symbol operand
//! operand   := identifier | integer      after function words are dropped
//! ```
//!
//! MARKER, OTHERWISE, relations, connectives and fillers are seed roles; the
//! program's inputs are the condition identifiers in code-point order, given as
//! command-line arguments, and the truth table enumerates small integers.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::{Value, json};

use crate::formalization::proposition_structure::{PhraseRole, operator_symbols, structure_phrases};
use crate::intent_formalization::explicit_count;
use crate::summarization::vocabulary::{function_words, negation_cues};

/// Literal delimiters: opening and closing characters, read verbatim between them.
const LITERAL_PAIRS: [(&str, &str); 5] = [("`", "`"), ("\"", "\""), ("«", "»"), ("“", "”"), ("「", "」")];
const SMALL_VALUES: [i64; 3] = [0, 1, 2];
const MAXIMUM_ROWS: usize = 81;
const MAXIMUM_INPUTS: usize = 3;
const SEPARATORS: [char; 7] = [',', ';', ':', '，', '；', '：', '、'];
/// Relation roles and the comparison each one writes, `left REL right`.
const RELATION_ROLES: [(&str, &str); 6] = [
    ("number_constraint_lower_strict", "gt"),
    ("number_constraint_lower_inclusive", "ge"),
    ("number_constraint_upper_strict", "lt"),
    ("number_constraint_upper_inclusive", "le"),
    ("comparison-equal", "eq"),
    ("comparison-not-equal", "ne"),
];
/// Seed operator relations (`symbolic-operator` rule) as comparisons.
const SYMBOL_RELATIONS: [(&str, [&str; 2]); 4] = [
    ("less", ["lt", "gt"]),
    ("less-or-equal", ["le", "ge"]),
    ("equal", ["eq", "eq"]),
    ("not-equal", ["ne", "ne"]),
];

/// One quoted or code-spanned literal of a task, with its byte span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLiteral {
    /// The literal's exact value.
    pub text: String,
    /// The byte offset of the opening delimiter.
    pub start: usize,
    /// The byte offset just past the closing delimiter.
    pub end: usize,
    /// Whether a double-quoted literal's closing quote was escaped, so its value is not read.
    pub escaped: bool,
}

fn negated(comparison: &str) -> &'static str {
    match comparison {
        "lt" => "ge",
        "le" => "gt",
        "gt" => "le",
        "ge" => "lt",
        "eq" => "ne",
        _ => "eq",
    }
}

fn is_word_character(character: Option<char>) -> bool {
    character.is_some_and(|character| character.is_alphanumeric() || character == '_')
}

fn character_width(text: &str, index: usize) -> usize {
    text[index..].chars().next().map_or(1, char::len_utf8)
}

/// The quoted and code-spanned literals of `text` in byte offsets. A
/// double-quoted literal whose closing quote is escaped is marked `escaped`:
/// its exact value is not read.
#[must_use]
pub fn task_literals(text: &str) -> Vec<TaskLiteral> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        let Some((open, close)) = LITERAL_PAIRS.iter().find(|(open, _)| rest.starts_with(open)) else {
            index += character_width(text, index);
            continue;
        };
        if *open == "`" && rest.starts_with("```") {
            index = text[index + 3..].find("```").map_or(text.len(), |end| index + 3 + end + 3);
            continue;
        }
        let stops = |cursor: usize| text[cursor..].starts_with(close) || text[cursor..].starts_with('\n');
        let mut cursor = index + open.len();
        let mut escaped = false;
        while cursor < text.len() && !stops(cursor) {
            cursor += character_width(text, cursor);
        }
        while *open == "\"" && cursor < text.len() && text[..cursor].ends_with('\\') {
            escaped = true;
            cursor += character_width(text, cursor);
            while cursor < text.len() && !stops(cursor) {
                cursor += character_width(text, cursor);
            }
        }
        if cursor >= text.len() || text[cursor..].starts_with('\n') {
            index += open.len();
            continue;
        }
        let value = &text[index + open.len()..cursor];
        if !value.is_empty() {
            out.push(TaskLiteral { text: value.to_owned(), start: index, end: cursor + close.len(), escaped });
        }
        index = cursor + close.len();
    }
    out
}

fn is_fence(line: &str) -> bool {
    let trimmed = line.trim_start_matches([' ', '\t']);
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// The lines inside the first fenced block of `text`, or `None`.
#[must_use]
pub fn fence_lines(text: &str) -> Option<Vec<String>> {
    let lines: Vec<&str> = text.split('\n').collect();
    let open = lines.iter().position(|line| is_fence(line))?;
    let close = lines.iter().enumerate().position(|(index, line)| index > open && is_fence(line))?;
    Some(lines[open + 1..close].iter().map(|line| (*line).to_owned()).collect())
}

/// The role's surfaces, lowercased, longest first.
fn role_phrases(role: &str) -> Arc<Vec<String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Vec<String>>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    Arc::clone(guard.entry(role.to_owned()).or_insert_with(|| {
        let mut phrases: Vec<String> = Vec::new();
        for word in crate::seed::lexicon().words_for_role(role) {
            let word = word.to_lowercase();
            if !phrases.contains(&word) {
                phrases.push(word);
            }
        }
        phrases.sort_by(|left, right| right.chars().count().cmp(&left.chars().count()).then_with(|| left.cmp(right)));
        Arc::new(phrases)
    }))
}

/// Every `(start, end)` where `phrase` occurs in `lowered` on word boundaries
/// (CJK phrases need none).
fn phrase_matches(lowered: &str, phrase: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    if phrase.is_empty() {
        return out;
    }
    let bounded = !crate::coding::contains_cjk(phrase);
    let mut from = 0;
    while let Some(found) = lowered[from..].find(phrase) {
        let start = from + found;
        let end = start + phrase.len();
        let before = lowered[..start].chars().next_back();
        let after = lowered[end..].chars().next();
        if !bounded || (!is_word_character(before) && !is_word_character(after)) {
            out.push((start, end));
        }
        from = start + character_width(lowered, start);
    }
    out
}

/// The earliest (then longest) match of a role phrase, or `None`.
#[must_use]
pub fn first_role_match(lowered: &str, role: &str) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    for phrase in role_phrases(role).iter() {
        for found in phrase_matches(lowered, phrase) {
            if best.is_none_or(|best| found.0 < best.0 || (found.0 == best.0 && found.1 > best.1)) {
                best = Some(found);
            }
        }
    }
    best
}

/// `text` lowercased with every literal blanked, so a marker inside a literal is not read.
fn masked(text: &str, literals: &[TaskLiteral]) -> String {
    let mut out = String::new();
    let mut last = 0;
    for literal in literals {
        out.push_str(&text[last..literal.start].to_lowercase());
        out.push_str(&"\0".repeat(literal.end - literal.start));
        last = literal.end;
    }
    out.push_str(&text[last..].to_lowercase());
    out
}

/// The literal lines of `literals` with their explicit counts.
fn branch_lines(text: &str, literals: &[&TaskLiteral], end: usize) -> Vec<String> {
    let mut out = Vec::new();
    for (index, literal) in literals.iter().enumerate() {
        let next = literals.get(index + 1).map_or(end, |next| next.start);
        let window = text.get(literal.end..next.max(literal.end)).unwrap_or("");
        for _ in 0..explicit_count(window) {
            out.push(literal.text.clone());
        }
    }
    out
}

/// Where the condition text that starts at `from` stops.
fn condition_end(lowered: &str, from: usize, limit: usize) -> usize {
    let Some(window) = lowered.get(from..limit) else {
        return limit;
    };
    let mut characters = window.char_indices().peekable();
    while let Some((offset, character)) = characters.next() {
        if character == '\0' || SEPARATORS.contains(&character) {
            return from + offset;
        }
        let digit_follows = characters.peek().is_some_and(|(_, next)| next.is_ascii_digit());
        if (character == '.' || character == '。') && !digit_follows {
            return from + offset;
        }
    }
    limit
}

fn is_integer(token: &str) -> bool {
    let digits = token.strip_prefix('-').unwrap_or(token);
    (1..=6).contains(&digits.len()) && digits.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_identifier(token: &str) -> bool {
    let mut characters = token.chars();
    characters.next().is_some_and(|first| first.is_alphabetic() || first == '_')
        && characters.all(|character| character.is_alphanumeric() || character == '_')
}

/// The identifier or integer one side of an atom names, or `None`.
fn operand(side: &str, fillers: &[String]) -> Option<Value> {
    let tokens: Vec<&str> = side
        .split(|character: char| character.is_whitespace() || character == '\0')
        .map(|token| {
            token
                .trim_start_matches(['(', ')', '\'', '"'])
                .trim_end_matches(['(', ')', '\'', '"', '?', '!'])
        })
        .filter(|token| !token.is_empty())
        .collect();
    let mut kept: Vec<&str> = tokens.iter().copied().filter(|token| !fillers.iter().any(|filler| filler == token)).collect();
    // A one-letter operand can spell a function word (`a`, `и`): it is kept
    // when nothing else on the side names an operand.
    if kept.is_empty() {
        kept = tokens.iter().copied().filter(|token| token.chars().count() == 1).collect();
    }
    let [token] = kept.as_slice() else {
        return None;
    };
    if is_integer(token) {
        return token.parse::<i64>().ok().map(|integer| json!({ "integer": integer }));
    }
    is_identifier(token).then(|| json!({ "name": token }))
}

/// One relation phrase or symbol found in an atom.
struct RelationMatch {
    start: usize,
    end: usize,
    comparison: &'static str,
}

/// Every relation phrase or symbol in `atom`.
fn relation_matches(atom: &str) -> Vec<RelationMatch> {
    let mut out = Vec::new();
    for (role, comparison) in RELATION_ROLES {
        for phrase in role_phrases(role).iter() {
            for (start, end) in phrase_matches(atom, phrase) {
                out.push(RelationMatch { start, end, comparison });
            }
        }
    }
    for entry in operator_symbols() {
        let Some((_, pair)) = SYMBOL_RELATIONS.iter().find(|(name, _)| *name == entry.relation.op) else {
            continue;
        };
        if entry.symbol.is_empty() {
            continue;
        }
        let mut from = 0;
        while let Some(found) = atom[from..].find(entry.symbol.as_str()) {
            let start = from + found;
            let comparison = pair[usize::from(entry.relation.converse)];
            out.push(RelationMatch { start, end: start + entry.symbol.len(), comparison });
            from = start + character_width(atom, start);
        }
    }
    out
}

/// One comparison `{comparison, left, right}` or a gap reason.
fn read_atom(atom: &str, fillers: &[String], negations: &[String]) -> Result<Value, &'static str> {
    let matches = relation_matches(atom);
    let width = |found: &RelationMatch| atom[found.start..found.end].chars().count();
    let mut chosen = matches.first().ok_or("condition_relation_missing")?;
    for found in &matches {
        if width(found) > width(chosen) || (width(found) == width(chosen) && found.start < chosen.start) {
            chosen = found;
        }
    }
    if matches.iter().any(|found| found.end <= chosen.start || found.start >= chosen.end) {
        return Err("condition_relation_repeated");
    }
    let mut flipped = false;
    let mut cleaned = Vec::new();
    for side in [&atom[..chosen.start], &atom[chosen.end..]] {
        let kept: Vec<&str> = side
            .split_whitespace()
            .filter(|token| {
                if !negations.iter().any(|negation| negation == token) {
                    return true;
                }
                flipped = !flipped;
                false
            })
            .collect();
        cleaned.push(kept.join(" "));
    }
    let left = operand(&cleaned[0], fillers);
    let right = operand(&cleaned[1], fillers);
    let (Some(left), Some(right)) = (left, right) else {
        return Err("condition_operand_unsupported");
    };
    let comparison = if flipped { negated(chosen.comparison) } else { chosen.comparison };
    Ok(json!({ "comparison": comparison, "left": left, "right": right }))
}

/// `{connective, atoms}` or a gap reason.
pub fn read_condition(text: &str) -> Result<Value, &'static str> {
    struct Cut {
        start: usize,
        end: usize,
        role: &'static str,
    }
    let lowered = text.to_lowercase();
    let phrases = structure_phrases(crate::language::detect(text).slug());
    let mut cuts: Vec<Cut> = Vec::new();
    for entry in phrases.iter() {
        let PhraseRole::Connective(token) = entry.role else {
            continue;
        };
        for (start, end) in phrase_matches(&lowered, &entry.phrase) {
            if !cuts.iter().any(|cut| start < cut.end && cut.start < end) {
                cuts.push(Cut { start, end, role: token.slug() });
            }
        }
    }
    cuts.sort_by_key(|cut| cut.start);
    let mut roles: Vec<&str> = Vec::new();
    for cut in &cuts {
        if !roles.contains(&cut.role) {
            roles.push(cut.role);
        }
    }
    if roles.len() > 1 {
        return Err("condition_connectives_mixed");
    }
    let mut fillers: Vec<String> = function_words().to_vec();
    fillers.extend(role_phrases("comparison-filler").iter().cloned());
    let negations = negation_cues();
    let mut atoms = Vec::new();
    let mut from = 0;
    let bounds = cuts.iter().map(|cut| (cut.start, cut.end)).chain(std::iter::once((lowered.len(), lowered.len())));
    for (start, end) in bounds {
        atoms.push(read_atom(lowered[from..start].trim(), &fillers, negations)?);
        from = end;
    }
    Ok(json!({ "connective": roles.first().copied().unwrap_or("conjunction"), "atoms": atoms }))
}

/// The clause's output statement under the supported subset: `None` when the
/// clause states no condition or otherwise branch, else
/// `{condition, negated, then, otherwise, continues}` or `{gap}`. `continues`
/// marks a clause that only states the otherwise branch of the statement
/// before it.
#[must_use]
pub fn conditional_statement(clause: &str) -> Option<Value> {
    let literals: Vec<TaskLiteral> = task_literals(clause).into_iter().filter(|literal| !literal.escaped).collect();
    let lowered = masked(clause, &literals);
    let otherwise = first_role_match(&lowered, "otherwise-marker");
    let main_end = otherwise.map_or(lowered.len(), |found| found.0);
    let main = &lowered[..main_end];
    let negated_marker = first_role_match(main, "negated-condition-marker");
    let marker = negated_marker.or_else(|| first_role_match(main, "condition-marker"));
    if marker.is_some() && first_role_match(main, "execution-circumstance").is_some() {
        return None;
    }
    let else_literals: Vec<&TaskLiteral> = match otherwise {
        Some(found) => literals.iter().filter(|literal| literal.start >= found.1).collect(),
        None if marker.is_none() => return None,
        None => Vec::new(),
    };
    let otherwise_lines = branch_lines(clause, &else_literals, clause.len());
    let Some(marker) = marker else {
        return Some(json!({ "continues": true, "otherwise": otherwise_lines }));
    };
    let condition_stop = condition_end(&lowered, marker.1, main_end);
    let condition = match read_condition(clause.get(marker.1..condition_stop).unwrap_or("")) {
        Ok(condition) => condition,
        Err(gap) => return Some(json!({ "gap": gap })),
    };
    let then_literals: Vec<&TaskLiteral> = literals
        .iter()
        .filter(|literal| literal.end <= main_end && (literal.end <= marker.0 || literal.start >= condition_stop))
        .collect();
    Some(json!({
        "condition": condition,
        "negated": negated_marker.is_some(),
        "then": branch_lines(clause, &then_literals, main_end),
        "otherwise": otherwise_lines,
        "continues": false,
    }))
}

/// Whether the statement's condition (with `negated`) holds for `values`.
fn holds(statement: &Value, values: &HashMap<String, i64>) -> bool {
    let value = |side: &Value| {
        side.get("name").and_then(Value::as_str).map_or_else(
            || side.get("integer").and_then(Value::as_i64).unwrap_or_default(),
            |name| values.get(name).copied().unwrap_or_default(),
        )
    };
    let compare = |atom: &Value| {
        let (left, right) = (value(&atom["left"]), value(&atom["right"]));
        match atom["comparison"].as_str().unwrap_or("") {
            "lt" => left < right,
            "le" => left <= right,
            "gt" => left > right,
            "ge" => left >= right,
            "eq" => left == right,
            _ => left != right,
        }
    };
    let condition = &statement["condition"];
    let atoms = condition["atoms"].as_array().map_or(&[][..], Vec::as_slice);
    let truth = if condition["connective"] == "disjunction" {
        atoms.iter().any(compare)
    } else {
        atoms.iter().all(compare)
    };
    truth != statement["negated"].as_bool().unwrap_or(false)
}

fn string_list(value: &Value) -> Vec<Value> {
    value.as_array().cloned().unwrap_or_default()
}

/// The expected stdout of the conditional statements for every row of small
/// inputs, `{inputs, rows: [{args, lines}]}`, or a gap reason.
pub fn condition_table(statements: &[Value]) -> Result<Value, &'static str> {
    let mut inputs: Vec<String> = Vec::new();
    let mut constants: Vec<i64> = Vec::new();
    for statement in statements {
        for atom in statement["condition"]["atoms"].as_array().map_or(&[][..], Vec::as_slice) {
            for side in [&atom["left"], &atom["right"]] {
                if let Some(name) = side.get("name").and_then(Value::as_str)
                    && !inputs.iter().any(|input| input == name)
                {
                    inputs.push(name.to_owned());
                }
                if let Some(integer) = side.get("integer").and_then(Value::as_i64) {
                    constants.push(integer);
                }
            }
        }
    }
    inputs.sort();
    if inputs.len() > MAXIMUM_INPUTS {
        return Err("condition_inputs_exceeded");
    }
    let mut domain: Vec<i64> = SMALL_VALUES.to_vec();
    domain.extend(constants.iter().flat_map(|constant| [constant - 1, *constant, constant + 1]));
    domain.sort_unstable();
    domain.dedup();
    let exponent = u32::try_from(inputs.len()).unwrap_or(u32::MAX);
    if domain.len().checked_pow(exponent).is_none_or(|rows| rows > MAXIMUM_ROWS) {
        return Err("condition_inputs_exceeded");
    }
    let mut rows: Vec<Vec<i64>> = vec![Vec::new()];
    for _ in &inputs {
        rows = rows
            .iter()
            .flat_map(|row| domain.iter().map(move |value| [row.as_slice(), &[*value]].concat()))
            .collect();
    }
    let rows: Vec<Value> = rows
        .iter()
        .map(|row| {
            let values: HashMap<String, i64> = inputs.iter().cloned().zip(row.iter().copied()).collect();
            let lines: Vec<Value> = statements
                .iter()
                .flat_map(|statement| {
                    let branch = if holds(statement, &values) { "then" } else { "otherwise" };
                    string_list(&statement[branch])
                })
                .collect();
            json!({ "args": row.iter().map(ToString::to_string).collect::<Vec<_>>(), "lines": lines })
        })
        .collect();
    Ok(json!({ "inputs": inputs, "rows": rows }))
}
