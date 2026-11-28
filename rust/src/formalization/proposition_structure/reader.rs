use super::*;

/// The proposition structure of a clause: `words` are its words
/// (`text_statements::read_words`), `terms` its statement terms, whose ids
/// name operands.
#[must_use]
pub fn read_proposition_structure(
    text: &str,
    language: &str,
    words: &[Word],
    terms: &[&Word],
) -> PropositionStructure {
    let phrases = structure_phrases(language);
    let surfaces: Vec<String> = words
        .iter()
        .map(|word| word.surface.to_lowercase())
        .collect();
    let mut roles = Vec::new();
    let mut repetition = Vec::new();
    let mut index = 0;
    while index < words.len() {
        if let Some((entry, width)) = phrase_at(&surfaces, index, &phrases) {
            match entry.role {
                PhraseRole::Repetition => repetition.push(entry.kind.clone()),
                PhraseRole::Connective(token) => roles.push(Role {
                    token,
                    index,
                    comparison: None,
                }),
            }
            index += width;
            continue;
        }
        let word = &words[index];
        let quantifier = if word.kind == WordKind::Meaning {
            quantifier_of(&word.id)
        } else {
            None
        };
        if word.kind == WordKind::Negation {
            roles.push(Role {
                token: ShapeToken::Negation,
                index,
                comparison: None,
            });
        } else if let Some(token) = quantifier {
            roles.push(Role {
                token,
                index,
                comparison: None,
            });
        } else if word.kind.is_term() {
            roles.push(Role {
                token: ShapeToken::Content,
                index,
                comparison: relation_of(COMPARISON_RULE, &word.id),
            });
        }
        index += 1;
    }
    let (mut relations, mut unresolved) = symbolic_relations(text, terms);
    let operand = |position: usize, forward: bool| -> Option<String> {
        let mut at = position;
        loop {
            at = if forward { at + 1 } else { at.checked_sub(1)? };
            let role = roles.get(at)?;
            if role.token == ShapeToken::Content && role.comparison.is_none() {
                let word = &words[role.index];
                return Some(operand_id(
                    &word.surface,
                    terms.iter().copied().chain(std::iter::once(word)),
                ));
            }
        }
    };
    for (position, role) in roles.iter().enumerate() {
        let Some(comparison) = role.comparison else {
            continue;
        };
        match (operand(position, false), operand(position, true)) {
            (Some(left), Some(right)) => {
                relations.push(relation(comparison.op, comparison.converse, left, right));
            }
            _ => unresolved.push(format!(
                "{}:{}",
                Unresolved::RelationOperand.slug(),
                words[role.index].id
            )),
        }
    }
    let mut tokens: Vec<ShapeToken> = Vec::new();
    for role in &roles {
        if !(role.token == ShapeToken::Content && tokens.last() == Some(&ShapeToken::Content)) {
            tokens.push(role.token);
        }
    }
    let leading = tokens
        .iter()
        .take_while(|token| **token == ShapeToken::Conjunction)
        .count();
    tokens = tokens.split_off(leading);
    while tokens.last().is_some_and(|token| token.connective()) {
        tokens.pop();
    }
    let has = |token: ShapeToken| tokens.contains(&token);
    let significant = has(ShapeToken::Disjunction)
        || has(ShapeToken::Universal)
        || has(ShapeToken::Existential)
        || (has(ShapeToken::Negation) && has(ShapeToken::Conjunction));
    let operands = relations
        .iter()
        .flat_map(|entry| entry.operands.iter().cloned())
        .collect();
    PropositionStructure {
        shape: tokens
            .iter()
            .map(|token| token.slug())
            .collect::<Vec<_>>()
            .join(" "),
        significant,
        relations: sorted(relations.into_iter().map(|entry| entry.text).collect()),
        operands: sorted(operands),
        repetition: sorted(repetition),
        unresolved: sorted(unresolved),
    }
}
