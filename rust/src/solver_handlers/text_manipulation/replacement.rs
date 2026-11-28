use crate::solver_handlers::text_edit_ops::normalized_word_spans;

pub(super) fn replace_text(input: &str, from: &str, to: &str) -> String {
    let direct = input.replace(from, to);
    if from.is_empty() {
        return direct;
    }
    let exact_ranges = exact_match_ranges(input, from);
    if !exact_ranges.is_empty() {
        if normalized_word_spans(from).len() > 1 {
            let mut ranges = exact_ranges.clone();
            ranges.extend(word_sequence_match_ranges(input, from, &exact_ranges));
            return replace_ranges(input, &ranges, to).unwrap_or(direct);
        }
        return direct;
    }
    replace_word_sequence(input, from, to).unwrap_or(direct)
}

fn replace_word_sequence(input: &str, from: &str, to: &str) -> Option<String> {
    let ranges = word_sequence_match_ranges(input, from, &[]);
    replace_ranges(input, &ranges, to)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReplacementRange {
    start: usize,
    end: usize,
}

fn exact_match_ranges(input: &str, from: &str) -> Vec<ReplacementRange> {
    if from.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative_start) = input[cursor..].find(from) {
        let start = cursor + relative_start;
        let end = start + from.len();
        ranges.push(ReplacementRange { start, end });
        cursor = end;
    }
    ranges
}

fn word_sequence_match_ranges(
    input: &str,
    from: &str,
    excluded: &[ReplacementRange],
) -> Vec<ReplacementRange> {
    let needle = normalized_word_spans(from)
        .into_iter()
        .map(|span| span.word)
        .collect::<Vec<_>>();
    if needle.is_empty() {
        return Vec::new();
    }

    let haystack = normalized_word_spans(input);
    if haystack.len() < needle.len() {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index + needle.len() <= haystack.len() {
        let matches = needle
            .iter()
            .enumerate()
            .all(|(offset, word)| haystack[index + offset].word == *word);
        if matches {
            let start = haystack[index].start;
            let end = haystack[index + needle.len() - 1].end;
            let range = ReplacementRange { start, end };
            if !excluded
                .iter()
                .any(|excluded| ranges_overlap(range, *excluded))
            {
                ranges.push(range);
            }
            index += needle.len();
        } else {
            index += 1;
        }
    }
    ranges
}

const fn ranges_overlap(left: ReplacementRange, right: ReplacementRange) -> bool {
    left.start < right.end && right.start < left.end
}

fn replace_ranges(input: &str, ranges: &[ReplacementRange], to: &str) -> Option<String> {
    if ranges.is_empty() {
        return None;
    }

    let mut ranges = ranges.to_vec();
    ranges.sort_by_key(|range| (range.start, range.end));

    let mut out = String::new();
    let mut cursor = 0usize;
    let mut replaced = false;
    for range in ranges {
        if range.start < cursor {
            continue;
        }
        out.push_str(&input[cursor..range.start]);
        out.push_str(to);
        cursor = range.end;
        replaced = true;
    }
    if !replaced {
        return None;
    }
    out.push_str(&input[cursor..]);
    Some(out)
}
