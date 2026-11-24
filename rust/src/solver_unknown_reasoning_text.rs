pub(super) fn extract_memory_fact(text: &str) -> Option<(String, String)> {
    for sentence in text.split(['.', '!', '?', '\n']) {
        let statement = sentence
            .trim()
            .strip_prefix("Remember that ")
            .or_else(|| sentence.trim().strip_prefix("remember that "))
            .unwrap_or_else(|| sentence.trim())
            .trim();
        if statement.is_empty() {
            continue;
        }
        if let Some((subject, value)) = split_fact_statement(statement) {
            return Some((subject, value));
        }
    }
    None
}

fn split_fact_statement(statement: &str) -> Option<(String, String)> {
    for separator in [" is ", " = ", ": "] {
        if let Some(index) = statement.find(separator) {
            let subject = statement[..index].trim();
            let value = statement[index + separator.len()..].trim();
            if !subject.is_empty() && !value.is_empty() {
                return Some((subject.to_owned(), value.to_owned()));
            }
        }
    }
    None
}

pub(super) fn clean_focus(value: &str) -> &str {
    value
        .trim()
        .trim_matches(['"', '\'', '`', '“', '”', '‘', '’', '«', '»'])
        .trim_end_matches(['?', '？', '。', '.', '!', '！', ',', ';', ':'])
        .trim()
}

pub(super) fn normalize_fact_subject(value: &str) -> String {
    let cleaned = clean_focus(value).to_lowercase();
    let stripped = cleaned
        .strip_prefix("the ")
        .or_else(|| cleaned.strip_prefix("a "))
        .or_else(|| cleaned.strip_prefix("an "))
        .unwrap_or(cleaned.as_str());
    normalize_search_surface(stripped)
}

pub(super) fn normalize_search_surface(value: &str) -> String {
    clean_focus(value)
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric() || character.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
