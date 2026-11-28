use super::*;

pub(super) fn collection_summary_owns(goals: &[Goal]) -> bool {
    use unicode_normalization::UnicodeNormalization;
    if goals.len() < 2 || goals[0].kind != GoalKind::Unsupported {
        return false;
    }
    let mut targets = std::collections::BTreeSet::new();
    for goal in &goals[1..] {
        if goal.kind != GoalKind::LiteralFile {
            return false;
        }
        let ObligationExpectation::FileBytes { path, .. } = &goal.node.expectation else {
            return false;
        };
        if !crate::agentic_coding::write_request::safe_relative_path(path) {
            return false;
        }
        let canonical = path
            .split('/')
            .filter(|part| *part != ".")
            .collect::<Vec<_>>()
            .join("/");
        let canonical = canonical.nfc().collect::<String>().to_lowercase();
        if canonical.is_empty() || !targets.insert(canonical) {
            return false;
        }
    }
    let clause = goals[0]
        .node
        .clause
        .trim_end_matches(|character: char| {
            character.is_whitespace() || ".!?。！？।;；".contains(character)
        })
        .trim()
        .nfc()
        .collect::<String>()
        .to_lowercase();
    let lexicon = crate::seed::lexicon();
    for cardinal in lexicon.meanings_with_role("cardinal_number_word") {
        let values = cardinal
            .lexemes
            .iter()
            .flat_map(|lexeme| &lexeme.words)
            .filter(|word| {
                !word.text.is_empty()
                    && word.text.len() <= 9
                    && word.text.bytes().all(|byte| byte.is_ascii_digit())
            })
            .filter_map(|word| word.text.parse::<usize>().ok())
            .collect::<std::collections::BTreeSet<_>>();
        if values.len() != 1 || !values.contains(&targets.len()) {
            continue;
        }
        for lexeme in &cardinal.lexemes {
            for noun_meaning in lexicon.meanings_with_role("file-read-object-noun") {
                for noun_lexeme in &noun_meaning.lexemes {
                    if noun_lexeme.language != lexeme.language {
                        continue;
                    }
                    for word in &lexeme.words {
                        for noun in &noun_lexeme.words {
                            let number = word.text.nfc().collect::<String>().to_lowercase();
                            let object = noun.text.nfc().collect::<String>().to_lowercase();
                            if clause == format!("{number} {object}")
                                || lexeme.language == "zh" && clause == format!("{number}{object}")
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}
