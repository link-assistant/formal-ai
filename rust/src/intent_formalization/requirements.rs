/// One ordered source requirement found during structural decomposition.
///
/// This small value is shared by the recursive problem-frame builder and the
/// arbitrary-procedure compiler. Keeping byte spans here makes decomposition a
/// reusable formalization primitive instead of making each downstream handler
/// split natural language independently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedRequirementSpan {
    pub source_text: String,
    pub source_span: (usize, usize),
}

/// Decompose `text` on language-neutral punctuation and caller-supplied,
/// seed-derived coordinating surfaces while preserving original byte spans.
#[must_use]
pub fn ordered_requirement_spans(
    text: &str,
    coordinating_surfaces: &[&str],
) -> Vec<OrderedRequirementSpan> {
    const PUNCTUATION: &[char] = &[
        ',', ';', ':', '.', '!', '?', '，', '、', '；', '。', '！', '？', '।',
    ];

    let (lower, offsets) = lower_with_offsets(text);
    let operands = requirement_operand_spans(text);
    let outside_operand =
        |index: usize| !operands.iter().any(|span| span.contains(&offsets[index]));
    let source_boundary = |index: usize| index == 0 || offsets[index] != offsets[index - 1];
    let mut cuts: Vec<(usize, usize)> = lower
        .char_indices()
        .filter(|(index, character)| {
            PUNCTUATION.contains(character)
                && !surrounded_by_numeric_characters(&lower, *index, character.len_utf8())
                && !(*character == '.'
                    && lower[*index + 1..]
                        .chars()
                        .next()
                        .is_some_and(char::is_alphanumeric))
                && outside_operand(*index)
        })
        .map(|(index, character)| (index, index + character.len_utf8()))
        .collect();
    for surface in coordinating_surfaces {
        let surface = surface.to_lowercase();
        if surface.is_empty() {
            continue;
        }
        for (start, _) in lower.match_indices(&surface) {
            let end = start + surface.len();
            if source_boundary(start)
                && source_boundary(end)
                && standalone_surface(&lower, start, end)
                && outside_operand(start)
            {
                cuts.push((start, end));
            }
        }
    }
    for marker in requirement_list_spans(&lower) {
        if outside_operand(marker.end - 1) {
            cuts.push((marker.start, marker.end));
        }
    }
    cuts.sort_by(|left, right| left.0.cmp(&right.0).then(right.1.cmp(&left.1)));

    let mut lower_spans = Vec::new();
    let mut cursor = 0usize;
    for (start, end) in cuts {
        if start < cursor {
            continue;
        }
        push_requirement_span(&lower, cursor, start, &mut lower_spans);
        cursor = end;
    }
    push_requirement_span(&lower, cursor, lower.len(), &mut lower_spans);

    lower_spans
        .into_iter()
        .map(|(start, end)| {
            let source_span = (offsets[start], offsets[end]);
            OrderedRequirementSpan {
                source_text: text[source_span.0..source_span.1].to_owned(),
                source_span,
            }
        })
        .collect()
}

/// Literal slots and source addresses are operands, not clause syntax. Reused
/// by the sentence-level frame pass so it cannot damage them before this reader.
pub fn requirement_operand_spans(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut spans = crate::normal_markov::quoted_segment_spans(text)
        .into_iter()
        .map(|slot| slot.start..slot.end)
        .collect::<Vec<_>>();
    let mut cursor = 0;
    for token in text.split_whitespace() {
        let start = cursor + text[cursor..].find(token).unwrap_or(0);
        cursor = start + token.len();
        let Some((scheme, address)) = token.split_once("://") else {
            continue;
        };
        if !address.is_empty()
            && scheme.starts_with(|character: char| character.is_ascii_alphabetic())
            && scheme.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
            })
        {
            let operand = token.trim_end_matches(['.', ',', ';', '!', '?', '。', '；']);
            spans.push(start..start + operand.len());
        }
    }
    spans
}

/// Markdown list syntax belongs to the boundary, never to the requirement's
/// payload. Wrapped continuation lines stay in the same source span.
pub fn requirement_list_spans(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut line_start = 0;
    let mut spans = Vec::new();
    for line in text.split_inclusive('\n') {
        if let Some(prefix_len) = list_marker_end(line) {
            spans.push(line_start..line_start + prefix_len);
        }
        line_start += line.len();
    }
    spans
}

fn list_marker_end(line: &str) -> Option<usize> {
    let item = line.trim_start_matches([' ', '\t']);
    let marker_len = if item.starts_with(['-', '*', '+']) {
        1
    } else {
        let digits = item.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || !item[digits..].starts_with(['.', ')']) {
            return None;
        }
        digits + 1
    };
    item[marker_len..]
        .starts_with([' ', '\t'])
        .then_some(line.len() - item.len() + marker_len + 1)
}

fn surrounded_by_numeric_characters(text: &str, start: usize, separator_len: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[start + separator_len..].chars().next();
    before.is_some_and(char::is_numeric) && after.is_some_and(char::is_numeric)
}

fn lower_with_offsets(text: &str) -> (String, Vec<usize>) {
    let mut lower = String::with_capacity(text.len());
    let mut offsets = Vec::with_capacity(text.len() + 1);
    for (index, character) in text.char_indices() {
        let before = lower.len();
        lower.extend(character.to_lowercase());
        offsets.extend((before..lower.len()).map(|_| index));
    }
    offsets.push(text.len());
    (lower, offsets)
}

fn standalone_surface(lower: &str, start: usize, end: usize) -> bool {
    if lower[start..end]
        .chars()
        .any(|character| matches!(character, '\u{3400}'..='\u{9fff}' | '\u{f900}'..='\u{faff}'))
    {
        return true;
    }
    let before = lower[..start].chars().next_back();
    let after = lower[end..].chars().next();
    !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
}

fn push_requirement_span(lower: &str, start: usize, end: usize, spans: &mut Vec<(usize, usize)>) {
    let slice = &lower[start..end];
    let trimmed = slice.trim();
    if trimmed.is_empty() {
        return;
    }
    let offset = start + (slice.len() - slice.trim_start().len());
    spans.push((offset, offset + trimmed.len()));
}
