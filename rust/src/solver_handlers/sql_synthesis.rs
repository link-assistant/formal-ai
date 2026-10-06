//! SQL-synthesis handler (issue #1177, E142 code-task family).
//!
//! Recognizes "write a SQL query …" / "select all …" requests (English and
//! Russian; cue phrases live in `data/seed/code-task-cues.lino`, comparative
//! word maps too), extracts table, columns, filters, ordering, limit and
//! aggregates from the token stream, and composes a single SELECT with a
//! clause-by-clause mapping back to the request.
//!
//! The project has no SQL-parser dependency, so the statement is verified
//! **by construction** — every emitted clause maps to one constraint in the
//! request — and the answer (a template from
//! `data/seed/multilingual-responses.lino`) states that honestly; nothing is
//! executed against a database.

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::{LinoNode, parse_lino};

const CUES_PATH: &str = "data/seed/code-task-cues.lino";
const INTENT: &str = "sql_synthesis";

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// Trigger phrases for one intent (optionally one named role) from
/// `data/seed/code-task-cues.lino`.
fn cue_phrases(intent: &str, role: &str) -> Vec<String> {
    let Some(text) = seed_text(CUES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "cues") {
        if record.find_child_value("intent") != intent {
            continue;
        }
        if record
            .children
            .iter()
            .find(|child| child.name == "intent")
            .map_or("", |child| child.find_child_value("role"))
            != role
        {
            continue;
        }
        for phrase in record
            .children
            .iter()
            .flat_map(|child| child.children.iter())
            .filter(|phrase| phrase.name == "phrase")
        {
            if !phrase.id.is_empty() {
                out.push(phrase.id.clone());
            }
        }
    }
    out
}

/// The `entry` records of one word map from `data/seed/code-task-cues.lino`.
fn word_entries(map: &str) -> Vec<LinoNode> {
    let Some(text) = seed_text(CUES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "map") {
        if record.find_child_value("name") != map {
            continue;
        }
        out.extend(
            record
                .children
                .iter()
                .flat_map(|child| child.children.iter())
                .filter(|entry| entry.name == "entry")
                .cloned(),
        );
    }
    out
}

/// Fill a localized response template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// Render the request→emission mapping rows through the shared seed
/// template (the row shape is data, not Rust prose).
fn mapping_rows(rows: &[(String, String)]) -> String {
    rows.iter()
        .map(|(request, emission)| {
            template(
                "mapping_line",
                &[("request", request), ("emission", emission)],
            )
        })
        .collect()
}

/// A contiguous echo of the request's own tokens.
fn echo(tokens: &[&str], from: usize, to: usize) -> String {
    tokens
        .get(from..=to)
        .map(|slice| slice.join(" "))
        .unwrap_or_default()
}

/// The numeric value of a word: a digit string, or a word in the `number`
/// word map.
fn number_value(word: &str, numbers: &[LinoNode]) -> Option<u32> {
    if let Ok(value) = word.parse::<u32>() {
        return Some(value);
    }
    numbers
        .iter()
        .find(|entry| entry.find_child_value("word") == word)
        .and_then(|entry| entry.find_child_value("value").parse::<u32>().ok())
}

fn identifier(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Extract the table name from the token stream.
fn table_name(tokens: &[&str]) -> Option<String> {
    for (index, token) in tokens.iter().enumerate() {
        match *token {
            "from" | "из" => {
                let candidate = tokens
                    .get(index + 1)
                    .filter(|next| **next != "the" && **next != "таблицы")
                    .or_else(|| tokens.get(index + 2))?;
                let name = identifier(candidate);
                if !name.is_empty() {
                    return Some(name);
                }
            }
            "select" | "selects" | "выбери" | "выберет" => {
                if let Some(&"all" | &"все" | &"всех" | &"every") = tokens.get(index + 1) {
                    let candidate = tokens.get(index + 2)?;
                    let name = identifier(candidate);
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
            "table" | "таблицы" | "таблицу" => {
                // "the users table" or "table users"
                if let Some(prev) = index.checked_sub(1).map(|i| tokens[i]) {
                    let name = identifier(prev);
                    if !name.is_empty() && name != "the" {
                        return Some(name);
                    }
                }
                if let Some(next) = tokens.get(index + 1) {
                    let name = identifier(next);
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// One WHERE filter composed from the request.
struct Filter {
    clause: String,
    request: String,
}

/// Extract filters: comparatives from the seed word map ("older than 30" →
/// `age > 30`), generic column comparisons ("age greater than 30"), and
/// equality ("named alice").
fn filters(tokens: &[&str]) -> Vec<Filter> {
    let numbers = word_entries("number");
    let comparatives = word_entries("sql_comparative");
    let mut out = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if let Some(entry) = comparatives
            .iter()
            .find(|entry| entry.find_child_value("word") == *token)
        {
            let value = tokens
                .get(index + 2)
                .and_then(|word| number_value(word, &numbers))
                .or_else(|| {
                    tokens
                        .get(index + 1)
                        .and_then(|word| number_value(word, &numbers))
                });
            if let Some(value) = value {
                let column = entry.find_child_value("column");
                let operator = entry.find_child_value("operator");
                out.push(Filter {
                    clause: format!("{column} {operator} {value}"),
                    request: echo(tokens, index, index + 2),
                });
            }
            continue;
        }
        // generic: "<column> greater/less than N", "<column> at least N"
        if let Some(value) = tokens
            .get(index + 2)
            .and_then(|word| number_value(word, &numbers))
        {
            let (column, operator) = match *token {
                "greater" | "more" | "больше" => {
                    (index.checked_sub(1).map(|i| tokens[i]), ">")
                }
                "less" | "fewer" | "меньше" => (index.checked_sub(1).map(|i| tokens[i]), "<"),
                "least" if index > 1 && tokens[index - 1] == "at" => {
                    (index.checked_sub(2).map(|i| tokens[i]), ">=")
                }
                "most" if index > 1 && tokens[index - 1] == "at" => {
                    (index.checked_sub(2).map(|i| tokens[i]), "<=")
                }
                _ => continue,
            };
            if let Some(column) = column {
                let column = identifier(column);
                if !column.is_empty() {
                    out.push(Filter {
                        clause: format!("{column} {operator} {value}"),
                        request: echo(tokens, index - 1, index + 2),
                    });
                }
            }
        }
        if (*token == "named" || *token == "имени")
            && let Some(value) = tokens.get(index + 1)
        {
            out.push(Filter {
                clause: format!("name = '{value}'"),
                request: echo(tokens, index, index + 1),
            });
        }
    }
    out
}

/// The aggregate in the request, if any: (SQL expression, request echo).
fn aggregate(tokens: &[&str]) -> Option<(String, String)> {
    for (index, token) in tokens.iter().enumerate() {
        let function = match *token {
            "many" | "count" | "сколько" => {
                return Some(("COUNT(*)".to_owned(), echo(tokens, index, index)));
            }
            "average" | "mean" | "среднее" => "AVG",
            "sum" | "total" | "сумма" => "SUM",
            "maximum" | "highest" | "max" => "MAX",
            "minimum" | "lowest" | "min" => "MIN",
            _ => continue,
        };
        let column = tokens
            .get(index + 1)
            .map(|word| identifier(word))
            .unwrap_or_default();
        if column.is_empty() {
            return None;
        }
        return Some((
            format!("{function}({column})"),
            echo(tokens, index, index + 1),
        ));
    }
    None
}

/// ORDER BY from "sorted by X" / "alphabetical order", plus DESC markers.
fn order_clause(tokens: &[&str]) -> Option<(String, String)> {
    for (index, token) in tokens.iter().enumerate() {
        if (*token == "sorted" || *token == "ordered" || *token == "по")
            && tokens.get(index + 1) == Some(&"by")
        {
            let column = tokens
                .get(index + 2)
                .map(|word| identifier(word))
                .unwrap_or_default();
            if !column.is_empty() {
                let direction = if tokens.contains(&"descending") || tokens.contains(&"reverse") {
                    " DESC"
                } else {
                    ""
                };
                return Some((
                    [" ORDER BY ", &column, direction].concat(),
                    echo(tokens, index, index + 2),
                ));
            }
        }
        if *token == "alphabetical" || *token == "алфавитном" {
            return Some((" ORDER BY name".to_owned(), echo(tokens, index, index)));
        }
    }
    None
}

/// LIMIT from "top N" / "first N" / "limit N".
fn limit_clause(tokens: &[&str], numbers: &[LinoNode]) -> Option<(String, String)> {
    for (index, token) in tokens.iter().enumerate() {
        if matches!(*token, "top" | "first" | "limit" | "первые" | "топ")
            && let Some(value) = tokens
                .get(index + 1)
                .and_then(|word| number_value(word, numbers))
        {
            return Some((format!(" LIMIT {value}"), echo(tokens, index, index + 1)));
        }
    }
    None
}

/// The SELECT column list: named columns between "select" and "from", else
/// the aggregate, else `*`. Returns (list, request echo).
fn select_columns(tokens: &[&str]) -> (String, String) {
    if let Some(start) = tokens
        .iter()
        .position(|t| matches!(*t, "select" | "selects" | "выбери" | "выберет"))
        && let Some(end) = tokens
            .iter()
            .skip(start + 1)
            .position(|t| *t == "from" || *t == "из")
            .map(|offset| start + 1 + offset)
    {
        let columns: Vec<String> = tokens[start + 1..end]
            .iter()
            .filter(|t| !matches!(**t, "all" | "the" | "and" | "все" | "всех"))
            .map(|t| identifier(t))
            .filter(|t| !t.is_empty())
            .collect();
        if !columns.is_empty() {
            let list = columns.join(", ");
            return (list, echo(tokens, start, end - 1));
        }
    }
    let all_at = tokens
        .iter()
        .position(|t| matches!(*t, "all" | "все" | "всех"));
    (
        "*".to_owned(),
        all_at.map(|at| echo(tokens, at, at)).unwrap_or_default(),
    )
}

/// Extract the code under discussion (a prompt that carries code is the
/// explanation/review/debugging family's territory, not SQL composition).
fn carries_code(prompt: &str) -> bool {
    let markers = ["def ", "function ", "fn ", "=>", "```"];
    markers.iter().any(|marker| prompt.contains(marker))
}

/// True when the prompt is a SQL request: a cue phrase matched, no
/// file-listing context token present (that is the shell composer's
/// territory), and no code under discussion.
fn is_sql_request(prompt: &str, normalized: &str) -> bool {
    let lower = prompt.to_lowercase();
    let cued = cue_phrases(INTENT, "")
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()));
    if !cued || carries_code(prompt) {
        return false;
    }
    let tokens: Vec<&str> = normalized.split(' ').filter(|t| !t.is_empty()).collect();
    let file_context: Vec<String> = word_entries("file_context")
        .iter()
        .map(|entry| entry.find_child_value("word").to_owned())
        .collect();
    !tokens
        .iter()
        .any(|token| file_context.iter().any(|word| word == token))
}

/// Try to recognize a SQL-synthesis request and compose a SELECT from the
/// stated constraints. Refuses by name when the table is not identifiable.
pub fn handle_sql_synthesis(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !is_sql_request(prompt, normalized) {
        return None;
    }
    let tokens: Vec<&str> = normalized.split(' ').filter(|t| !t.is_empty()).collect();
    log.append(
        "sql_synthesis:request",
        format!("{} token(s)", tokens.len()),
    );

    let Some(table) = table_name(&tokens) else {
        log.append("sql_synthesis:refusal", "no table identified".to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            INTENT,
            "response:sql_synthesis",
            &template("sql_synthesis_refusal", &[]),
            0.4,
        ));
    };
    log.append("sql_synthesis:table", table.clone());

    let numbers = word_entries("number");
    let (aggregate, aggregate_request) = match aggregate(&tokens) {
        Some((expression, request)) => (expression, Some(request)),
        None => (String::new(), None),
    };
    let (columns, columns_request) = if aggregate.is_empty() {
        select_columns(&tokens)
    } else {
        (aggregate, aggregate_request.unwrap_or_default())
    };
    let filter_list = filters(&tokens);
    let where_clause = if filter_list.is_empty() {
        String::new()
    } else {
        format!(
            " WHERE {}",
            filter_list
                .iter()
                .map(|filter| filter.clause.as_str())
                .collect::<Vec<_>>()
                .join(" AND ")
        )
    };
    let (order, order_request) = order_clause(&tokens).unwrap_or_default();
    let (limit, limit_request) = limit_clause(&tokens, &numbers).unwrap_or_default();

    let statement = [
        "SELECT ",
        &columns,
        " FROM ",
        &table,
        &where_clause,
        &order,
        &limit,
        ";",
    ]
    .concat();
    log.append("sql_synthesis:statement", statement.clone());

    let mut rows: Vec<(String, String)> = vec![(columns_request, format!("SELECT {columns}"))];
    rows.push((table.clone(), format!("FROM {table}")));
    for filter in &filter_list {
        rows.push((filter.request.clone(), filter.clause.clone()));
    }
    if !order.is_empty() {
        rows.push((order_request, order.trim().to_owned()));
    }
    if !limit.is_empty() {
        rows.push((limit_request, limit.trim().to_owned()));
    }
    let mapping = mapping_rows(&rows);

    let body = template(
        "sql_synthesis_statement",
        &[("statement", &statement), ("mapping", &mapping)],
    );

    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:sql_synthesis",
        &body,
        0.7,
    ))
}
