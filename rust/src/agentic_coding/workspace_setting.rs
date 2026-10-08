//! Text a request names by its value (PR #1188 dogfooding).
//!
//! A configuration setting given a new value -- the line assigning a key, the
//! value it holds, and the value a request states it holds now -- and a line
//! replaced whole. The JavaScript twin is `js/agentic/workspace_setting.mjs`.

use super::write_request::{Token, clean_cue_token, tokens};
use crate::normal_markov::quoted_segment_spans;
use crate::seed;
use crate::workspace_change_learning::word_scoped_matches;

/// The seeded role of the words that lead the value a key holds now.
const OLD_LEAD_CUE: &str = "file_edit_old_lead_cue";

/// The key and the value an old clause states it holds now.
///
/// `K from A`, `K с A`, `K को A की जगह`: the seeded `file_edit_old_lead_cue`
/// outside quotes, the value on the side its language's adposition governs;
/// `None` without one.
pub(super) fn stated_old_value(clause: &str) -> Option<(String, String)> {
    let segments = quoted_segment_spans(clause);
    let toks = tokens(clause);
    let quoted = |token: &Token<'_>| {
        segments
            .iter()
            .any(|segment| token.start < segment.end && token.end > segment.start)
    };
    let mut found: Option<(usize, usize, bool)> = None;
    for meaning in seed::lexicon().meanings_with_role(OLD_LEAD_CUE) {
        for lexeme in &meaning.lexemes {
            let postpositional = crate::language::uses_postpositions(&lexeme.language);
            for word in &lexeme.words {
                if let Some((start, end)) = last_run(&toks, &quoted, &word.text.to_lowercase())
                    && found.is_none_or(|(earlier, _, _)| start > earlier)
                {
                    found = Some((start, end, postpositional));
                }
            }
        }
    }
    let (start, end, postpositional) = found?;
    let (key, old) = if postpositional {
        let words: Vec<&str> = clause[..start].split_whitespace().collect();
        let (old, key) = words.split_last()?;
        if key.is_empty() {
            return None;
        }
        (key.join(" "), (*old).to_owned())
    } else {
        (clause[..start].to_owned(), clause[end..].to_owned())
    };
    let old = literal_value(&old);
    let key = key.trim().to_owned();
    (!key.is_empty() && !old.is_empty()).then_some((key, old))
}

/// The byte span of the last run of unquoted tokens spelling `surface`.
fn last_run(
    toks: &[Token<'_>],
    quoted: &impl Fn(&Token<'_>) -> bool,
    surface: &str,
) -> Option<(usize, usize)> {
    let parts: Vec<&str> = surface.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    toks.windows(parts.len())
        .rev()
        .find(|run| {
            run.iter()
                .zip(&parts)
                .all(|(token, part)| !quoted(token) && clean_cue_token(token.text) == *part)
        })
        .map(|run| (run[0].start, run[parts.len() - 1].end))
}

/// A span that is one quoted literal is its text; any other span, trimmed.
fn literal_value(span: &str) -> String {
    let text = span.trim();
    match quoted_segment_spans(text).as_slice() {
        [segment] if segment.start == 0 && segment.end == text.len() => segment.text.clone(),
        _ => text.to_owned(),
    }
}

/// A value without the one pair of quotes around it.
fn unquoted_value(value: &str) -> &str {
    match value.chars().next() {
        Some(quote @ ('"' | '\'')) if value.len() >= 2 && value.ends_with(quote) => {
            &value[1..value.len() - 1]
        }
        _ => value,
    }
}

/// A value every config format writes bare: a boolean, null or a number.
fn is_bare_literal(value: &str) -> bool {
    if matches!(value, "true" | "false" | "null") {
        return true;
    }
    let digits = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = digits
        .split_once('.')
        .map_or((digits, None), |(whole, fraction)| (whole, Some(fraction)));
    let all_digits =
        |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    all_digits(whole) && fraction.is_none_or(all_digits)
}

/// The file with the one line assigning `key` given `value`.
///
/// The line is the one assigning `key` (`"k": v`, `k: v`, `k = v`, `const K:
/// T = v;`) -- holding `held`, when the request stated it -- and `value` is
/// written in the file's own quoting. A key assigned nowhere changes the
/// stated value where it occurs once as a word. `None` when the key is
/// assigned zero or several times, or already holds the value.
pub(super) fn assigned_setting(
    source: &str,
    key: &str,
    value: &str,
    target: &str,
    held: Option<&str>,
) -> Option<String> {
    let lines: Vec<&str> = source.split('\n').collect();
    let assigned: Vec<(usize, &str, &str, &str)> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let (lead, old, tail) = assignment(line, key)?;
            Some((index, lead, old, tail))
        })
        .collect();
    let matches: Vec<&(usize, &str, &str, &str)> = assigned
        .iter()
        .filter(|(_, _, old, _)| held.is_none_or(|held| unquoted_value(old) == held))
        .collect();
    let [found] = matches.as_slice() else {
        let held = held.filter(|_| assigned.is_empty())?;
        let found = word_scoped_matches(source, held);
        let [at] = found.as_slice() else {
            return None;
        };
        return Some([&source[..*at], value, &source[at + held.len()..]].concat());
    };
    let &(index, lead, old, tail) = *found;
    let quote = old
        .chars()
        .next()
        .filter(|first| matches!(first, '"' | '\''));
    let written = match quote {
        Some(quote) if !is_bare_literal(value) => format!("{quote}{value}{quote}"),
        None if std::path::Path::new(target)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            && !is_bare_literal(value)
            && !value.starts_with(['"', '\'', '[', '{']) =>
        {
            serde_json::Value::String(value.to_owned()).to_string()
        }
        _ => value.to_owned(),
    };
    if written == old {
        return None;
    }
    let mut out: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
    out[index] = [lead, written.as_str(), tail].concat();
    Some(out.join("\n"))
}

/// `(head, value, tail)` when `line` assigns `key`.
///
/// Indentation, declaration words (`const`, `let mut`, `pub(crate) static`),
/// the key bare or quoted, a declared type (`: u32`) when words led, `:`, `=`
/// or `:=`, the value, and an optional trailing `,` or `;`.
fn assignment<'a>(line: &'a str, key: &str) -> Option<(&'a str, &'a str, &'a str)> {
    let mut at = line.len() - line.trim_start().len();
    let mut declared = false;
    loop {
        if let Some(found) = keyed_value(line, at, key, declared) {
            return Some(found);
        }
        let rest = &line[at..];
        if !rest.starts_with(|character: char| character.is_ascii_alphabetic() || character == '_')
        {
            return None;
        }
        let word = rest
            .find(|character: char| {
                !(character.is_ascii_alphanumeric() || matches!(character, '_' | '(' | ')'))
            })
            .unwrap_or(rest.len());
        let gap = rest[word..].len() - rest[word..].trim_start_matches([' ', '\t']).len();
        if gap == 0 {
            return None;
        }
        at += word + gap;
        declared = true;
    }
}

/// [`assignment`] with the key at byte `at`.
fn keyed_value<'a>(
    line: &'a str,
    at: usize,
    key: &str,
    declared: bool,
) -> Option<(&'a str, &'a str, &'a str)> {
    let rest = &line[at..];
    let (quote, rest) = match rest.chars().next() {
        Some(quote @ ('"' | '\'')) => (Some(quote), &rest[1..]),
        _ => (None, rest),
    };
    if key.is_empty() {
        return None;
    }
    let rest = rest.strip_prefix(key)?;
    let rest = match quote {
        Some(quote) => rest.strip_prefix(quote)?,
        None => rest,
    };
    let rest = rest.trim_start();
    let after = if let Some(after) = rest.strip_prefix(":=") {
        after
    } else if let (Some(after), false) = (rest.strip_prefix(':'), declared) {
        after
    } else {
        // `= v`, or a declared type before it (`: u32 = v`); never `==` or `=>`.
        let equals = if rest.starts_with('=') {
            0
        } else if rest.starts_with(':') {
            rest.find('=')?
        } else {
            return None;
        };
        let after = &rest[equals + 1..];
        if after.starts_with(['=', '>']) {
            return None;
        }
        after
    };
    let value_start = line.len() - after.trim_start().len();
    let body = line[value_start..].trim_end();
    let value = body.strip_suffix([',', ';']).unwrap_or(body).trim_end();
    let value_end = value_start + value.len();
    Some((
        &line[..value_start],
        &line[value_start..value_end],
        &line[value_end..],
    ))
}

/// `source` with whole lines replaced.
///
/// Every line that is exactly `old` becomes `new`; with none, every line that
/// is `old` but for its indentation becomes `new` at that indentation (unless
/// `new` brings its own); with none, the one occurrence of `old` in the file.
/// After a `context` (found once), only the first such line below it, and
/// never a bare occurrence. `None` otherwise (mirrors `replacedLines`).
pub(super) fn replaced_lines(
    source: &str,
    old: &str,
    new: &str,
    context: Option<&str>,
) -> Option<String> {
    let lines: Vec<&str> = source.split('\n').collect();
    let from = match context {
        Some(context) => {
            let mut found = source.match_indices(context).map(|(at, _)| at);
            let (Some(at), None) = (found.next(), found.next()) else {
                return None;
            };
            source[..at + context.len()].split('\n').count()
        }
        None => 0,
    };
    let bare = |line: &str| line.strip_suffix('\r').unwrap_or(line).len();
    let stripped = |text: &str| text.trim_matches([' ', '\t']).to_owned();
    let exact: Vec<bool> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| index >= from && line[..bare(line)] == *old)
        .collect();
    let loose: Vec<bool> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            index >= from
                && !stripped(old).is_empty()
                && stripped(&line[..bare(line)]) == stripped(old)
        })
        .collect();
    let (mut chosen, indented) = if exact.contains(&true) {
        (exact, false)
    } else if loose.contains(&true) {
        (loose, !new.starts_with([' ', '\t']))
    } else {
        return (context.is_none() && source.matches(old).count() == 1)
            .then(|| source.replacen(old, new, 1));
    };
    if context.is_some() {
        let first = chosen.iter().position(|flag| *flag);
        chosen = (0..chosen.len())
            .map(|index| Some(index) == first)
            .collect();
    }
    let replaced: Vec<String> = lines
        .iter()
        .zip(chosen)
        .map(|(line, replace)| {
            if !replace {
                return (*line).to_owned();
            }
            let indent = if indented {
                &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
            } else {
                ""
            };
            [indent, new, &line[bare(line)..]].concat()
        })
        .collect();
    Some(replaced.join("\n"))
}
