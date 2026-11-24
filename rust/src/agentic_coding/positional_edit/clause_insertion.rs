use super::*;

/// One clause's insert, with its target apart.
///
/// The target is `None` when the clause names no file. The anchor is the
/// literal the position cue governs; a seeded anchor context (`the line 'y'
/// that follows 'z'`) names a literal the anchor comes after; every other
/// literal is a line inserted, in order, and those lines may be separated only
/// by seeded joiners and commas.
pub(super) fn clause_insert(
    sentence: &str,
    block_text: Option<&str>,
) -> Option<(Option<String>, PositionalInsert)> {
    let lexicon = seed::lexicon();
    let normalized = crate::engine::normalize_prompt(sentence);
    // The position is the request's, not the payload's: a cue inside a quoted
    // literal ("Insert the line «… before …» after …") is text being inserted.
    let outside = crate::engine::normalize_prompt(
        &quoted_segment_spans(sentence)
            .iter()
            .rev()
            .fold(sentence.to_owned(), |text, segment| {
                [&text[..segment.start], " ", &text[segment.end..]].concat()
            }),
    );
    let after = lexicon.mentions_role(seed::ROLE_FILE_EDIT_POSITION_AFTER, &outside);
    let before = lexicon.mentions_role(seed::ROLE_FILE_EDIT_POSITION_BEFORE, &outside);
    if after == before || !lexicon.mentions_role(seed::ROLE_CODING_MEMBER_ADD_ACTION, &normalized) {
        return None;
    }
    let target = named_paths(sentence).first().map(|path| (*path).to_owned());
    let literals: Vec<QuotedLiteral> = quoted_literals(sentence)
        .into_iter()
        .filter(|literal| target.as_deref() != Some(literal.text.as_str()))
        .collect();
    let contexts = cue_occurrences(sentence, ANCHOR_CONTEXT_CUE);
    let role = if after {
        seed::ROLE_FILE_EDIT_POSITION_AFTER
    } else {
        seed::ROLE_FILE_EDIT_POSITION_BEFORE
    };
    // A position word inside the context cue ("के बाद आने वाली") is the
    // context's, not the anchor's.
    let positions: Vec<CueOccurrence> = cue_occurrences(sentence, role)
        .into_iter()
        .filter(|&(start, _, _)| {
            !contexts
                .iter()
                .any(|&(from, to, _)| start >= from && start < to)
        })
        .collect();
    // One literal is the anchor: `the following lines` speaks of the block
    // (PR #1188 G16). Without a block, the seeded blank line is the line
    // inserted beside it (G31). With a block and a seeded anchor context, a
    // second literal is the line the anchor follows (G51).
    let blank = block_text.is_none()
        && literals.len() == 1
        && contexts.is_empty()
        && lexicon.mentions_role(BLANK_LINE, &outside);
    if let Some(text) = block_text
        && literals.is_empty()
    {
        return unquoted_anchor::unquoted_anchor_insert(
            sentence, &positions, &contexts, target, text, after,
        );
    }
    if let Some(text) = block_text.or_else(|| blank.then_some("")) {
        let (anchor_at, context_at) = match (literals.len(), contexts.is_empty()) {
            // A context cue then names no second line: `the following lines`
            // speaks of the block (G16).
            (1, _) => (0, None),
            (2, false) if block_text.is_some() => {
                let anchor_at = governed_literal(&positions, &literals, &[0, 1])?;
                let context_at = governed_literal(&contexts, &literals, &[1 - anchor_at])?;
                (anchor_at, Some(context_at))
            }
            _ => return None,
        };
        let insert = PositionalInsert {
            target: String::new(),
            anchor: unescape_prose_newlines(&literals[anchor_at].text),
            inserted: text.to_owned(),
            after,
            context: context_at.map(|index| unescape_prose_newlines(&literals[index].text)),
            rebase: None,
            spans: false,
        };
        return Some((target, insert));
    }
    if literals.len() < 2 {
        return None;
    }
    let indices: Vec<usize> = (0..literals.len()).collect();
    let anchor_at = governed_literal(&positions, &literals, &indices)?;
    let context_at = if contexts.is_empty() {
        None
    } else {
        let rest: Vec<usize> = indices
            .into_iter()
            .filter(|index| *index != anchor_at)
            .collect();
        Some(governed_literal(&contexts, &literals, &rest)?)
    };
    let inserted: Vec<&QuotedLiteral> = literals
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != anchor_at && Some(*index) != context_at)
        .map(|(_, literal)| literal)
        .collect();
    if inserted.is_empty() || !joined_only(sentence, &inserted) {
        return None;
    }
    // Lines given as separate literals are already lines: an escape inside
    // one is content (PR #1188 G53).
    let lines = inserted
        .iter()
        .map(|literal| {
            if inserted.len() > 1 {
                literal.text.clone()
            } else {
                unescape_prose_newlines(&literal.text)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    // `'x' followed by an empty line`: the seeded blank line goes on the side
    // of the lines its words stand on, never dropped (PR #1188 G74).
    let first_blank = cue_occurrences(sentence, BLANK_LINE)
        .into_iter()
        .map(|(start, _, _)| start)
        .min();
    let inserted = match first_blank {
        None => lines,
        Some(blank) if blank < inserted[0].start => ["\n", lines.as_str()].concat(),
        Some(_) => [lines.as_str(), "\n"].concat(),
    };
    let insert = PositionalInsert {
        target: String::new(),
        anchor: unescape_prose_newlines(&literals[anchor_at].text),
        inserted,
        after,
        context: context_at.map(|index| unescape_prose_newlines(&literals[index].text)),
        rebase: None,
        spans: false,
    };
    Some((target, insert))
}
