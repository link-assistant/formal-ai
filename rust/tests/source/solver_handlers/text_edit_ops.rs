use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WordSpan {
    pub word: String,
    pub start: usize,
    pub end: usize,
}

pub(super) fn normalized_word_spans(text: &str) -> Vec<WordSpan> {
    let mut spans = Vec::new();
    let mut current_start = None;
    let mut current = String::new();
    let mut current_end = 0usize;

    for (index, character) in text.char_indices() {
        if character.is_alphanumeric() {
            if current_start.is_none() {
                current_start = Some(index);
            }
            current.extend(character.to_lowercase());
            current_end = index + character.len_utf8();
        } else if let Some(start) = current_start.take() {
            spans.push(WordSpan {
                word: std::mem::take(&mut current),
                start,
                end: current_end,
            });
        }
    }

    if let Some(start) = current_start {
        spans.push(WordSpan {
            word: current,
            start,
            end: current_end,
        });
    }
    spans
}

pub(super) fn extract_email_addresses(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(clean_email_candidate)
        .filter(|candidate| looks_like_email(candidate))
        .map(ToOwned::to_owned)
        .collect()
}

fn clean_email_candidate(candidate: &str) -> &str {
    candidate
        .trim_matches(|character: char| {
            !(character.is_ascii_alphanumeric() || matches!(character, '@' | '.' | '_' | '-' | '+'))
        })
        .trim_matches('.')
}

fn looks_like_email(candidate: &str) -> bool {
    if candidate
        .chars()
        .filter(|character| *character == '@')
        .count()
        != 1
    {
        return false;
    }
    let Some((local, domain)) = candidate.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && domain
            .split('.')
            .all(|segment| !segment.is_empty() && segment.chars().all(is_email_domain_char))
}

const fn is_email_domain_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '-'
}

pub(super) fn extract_urls(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(clean_url_candidate)
        .filter(|candidate| {
            candidate.starts_with("http://")
                || candidate.starts_with("https://")
                || candidate.starts_with("www.")
        })
        .map(ToOwned::to_owned)
        .collect()
}

fn clean_url_candidate(candidate: &str) -> &str {
    candidate
        .trim_matches(|character: char| {
            matches!(
                character,
                '"' | '\'' | '`' | '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
            )
        })
        .trim_end_matches(['.', '!', '?', ':'])
}

pub(super) fn extract_numbers(input: &str) -> Vec<String> {
    let mut numbers = Vec::new();
    let chars = input.char_indices().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        let (start, character) = chars[index];
        let mut number_start = start;
        if matches!(character, '-' | '+')
            && chars
                .get(index + 1)
                .is_some_and(|(_, ch)| ch.is_ascii_digit())
            && !previous_char_is_alphanumeric(input, start)
        {
            index += 1;
        } else if character.is_ascii_digit() && !previous_char_is_alphanumeric(input, start) {
            number_start = start;
        } else {
            index += 1;
            continue;
        }

        while chars.get(index).is_some_and(|(_, ch)| ch.is_ascii_digit()) {
            index += 1;
        }
        if chars.get(index).is_some_and(|(_, ch)| *ch == '.')
            && chars
                .get(index + 1)
                .is_some_and(|(_, ch)| ch.is_ascii_digit())
        {
            index += 1;
            while chars.get(index).is_some_and(|(_, ch)| ch.is_ascii_digit()) {
                index += 1;
            }
        }
        let end = chars
            .get(index)
            .map_or(input.len(), |(char_start, _)| *char_start);
        if chars.get(index).is_some_and(|(_, ch)| ch.is_alphanumeric()) {
            continue;
        }
        numbers.push(input[number_start..end].to_owned());
    }
    numbers
}

fn previous_char_is_alphanumeric(input: &str, byte_index: usize) -> bool {
    input[..byte_index]
        .chars()
        .next_back()
        .is_some_and(char::is_alphanumeric)
}

pub(super) fn count_unique_words(input: &str) -> usize {
    input
        .split_whitespace()
        .map(clean_word)
        .filter(|word| !word.is_empty())
        .collect::<BTreeSet<_>>()
        .len()
}

pub(super) fn count_words(input: &str) -> usize {
    input
        .split_whitespace()
        .map(clean_word)
        .filter(|word| !word.is_empty())
        .count()
}

fn clean_word(word: &str) -> String {
    word.trim_matches(|character: char| !character.is_alphanumeric())
        .to_owned()
}

pub(super) fn title_case(input: &str) -> String {
    case_words(input)
        .iter()
        .map(|word| capitalize_word(word))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn sentence_case(input: &str) -> String {
    let mut out = String::new();
    let mut capitalized = false;
    for character in input.chars().flat_map(char::to_lowercase) {
        if !capitalized && character.is_alphanumeric() {
            out.extend(character.to_uppercase());
            capitalized = true;
        } else {
            out.push(character);
        }
    }
    out
}

pub(super) fn delimiter_case(input: &str, delimiter: &str) -> String {
    case_words(input).join(delimiter)
}

pub(super) fn camel_case(input: &str) -> String {
    let mut words = case_words(input).into_iter();
    let Some(first) = words.next() else {
        return String::new();
    };

    let mut out = first;
    for word in words {
        out.push_str(&capitalize_word(&word));
    }
    out
}

pub(super) fn pascal_case(input: &str) -> String {
    case_words(input)
        .iter()
        .map(|word| capitalize_word(word))
        .collect::<String>()
}

fn case_words(input: &str) -> Vec<String> {
    normalized_word_spans(input)
        .into_iter()
        .map(|span| span.word)
        .collect()
}

fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    let mut out = first.to_uppercase().collect::<String>();
    out.extend(chars.flat_map(char::to_lowercase));
    out
}

pub(super) fn remove_punctuation(input: &str) -> String {
    input
        .chars()
        .filter(|character| character.is_alphanumeric() || character.is_whitespace())
        .collect()
}

pub(super) fn sort_words(input: &str) -> String {
    let mut words = input.split_whitespace().collect::<Vec<_>>();
    words.sort_unstable();
    words.join(" ")
}

pub(super) fn strip_empty_lines(input: &str) -> Vec<String> {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

pub(super) fn join_lines(input: &str) -> String {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn number_lines(input: &str) -> Vec<String> {
    input
        .lines()
        .enumerate()
        .map(|(index, line)| format!("{}. {line}", index + 1))
        .collect()
}

pub(super) fn reverse_lines(input: &str) -> String {
    input.lines().rev().collect::<Vec<_>>().join("\n")
}

pub(super) fn comment_lines(input: &str) -> String {
    input
        .lines()
        .map(|line| format!("// {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn uncomment_lines(input: &str) -> String {
    input
        .lines()
        .map(uncomment_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn uncomment_line(line: &str) -> String {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    for prefix in ["// ", "//", "# ", "#"] {
        if let Some(stripped) = rest.strip_prefix(prefix) {
            return format!("{indent}{stripped}");
        }
    }
    line.to_owned()
}

pub(super) fn outdent_line(line: &str) -> &str {
    line.strip_prefix("    ")
        .or_else(|| line.strip_prefix('\t'))
        .unwrap_or(line)
}

pub(super) fn deduplicate_lines(input: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut lines = Vec::new();
    for line in input.lines() {
        if seen.insert(line.to_owned()) {
            lines.push(line.to_owned());
        }
    }
    lines
}
