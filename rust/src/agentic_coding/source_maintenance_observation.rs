//! Observe existing source without granting edit authority or copying normalization.
use crate::coding_research_generated::{SourceLayoutResult, normalize_trailing_layout};
use crate::es_meta::{SourceLanguage, extract};
use crate::es_tokenizer::{Tree, tokenize};
use std::ops::Range;

fn lexical_intervals(source: &str, trees: &[Tree<'_>]) -> Result<Vec<Range<usize>>, String> {
    let mut intervals = Vec::new();
    let mut pending: Vec<_> = trees.iter().collect();
    while let Some(tree) = pending.pop() {
        let span = tree.span();
        if span.start > span.end
            || span.end > source.len()
            || !source.is_char_boundary(span.start)
            || !source.is_char_boundary(span.end)
        {
            return Err("SourceMaintenanceUnsupportedLexicalSource".to_owned());
        }
        match tree {
            Tree::Group { trees, .. } => {
                if span.end - span.start < 2 {
                    return Err("SourceMaintenanceUnsupportedLexicalSource".to_owned());
                }
                intervals.push(span.start..span.start + 1);
                intervals.push(span.end - 1..span.end);
                pending.extend(trees);
            }
            // Templates remain whole and opaque, including interpolation bodies.
            Tree::Leaf(_) | Tree::Template { .. } => intervals.push(span),
        }
    }
    intervals.sort_by_key(|span| span.start);
    if intervals.windows(2).any(|pair| pair[0].end > pair[1].start) {
        return Err("SourceMaintenanceUnsupportedLexicalSource".to_owned());
    }
    Ok(intervals)
}

fn non_layout_gap(source: &str, start: usize, end: usize) -> bool {
    source.as_bytes()[start..end]
        .iter()
        .any(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
}

fn observed_characters(source: &str, intervals: &[Range<usize>]) -> (Vec<String>, Vec<bool>) {
    let mut characters = Vec::new();
    let mut protection = Vec::new();
    let mut interval_index = 0;
    let mut gap_protected = non_layout_gap(
        source,
        0,
        intervals.first().map_or(source.len(), |span| span.start),
    );
    for (byte_offset, character) in source.char_indices() {
        while interval_index < intervals.len() && byte_offset >= intervals[interval_index].end {
            let gap_start = intervals[interval_index].end;
            interval_index += 1;
            let gap_end = intervals
                .get(interval_index)
                .map_or(source.len(), |span| span.start);
            gap_protected = non_layout_gap(source, gap_start, gap_end);
        }
        let token_protected = intervals
            .get(interval_index)
            .is_some_and(|span| byte_offset >= span.start && byte_offset < span.end);
        characters.push(character.to_string());
        protection.push(token_protected || gap_protected);
    }
    (characters, protection)
}

pub(super) fn derive_source_maintenance(
    property: &str,
    language: &str,
    source: &str,
) -> Result<String, String> {
    let Some(language_kind) = SourceLanguage::parse(language) else {
        return Err("SourceMaintenancePropertyUnsupported".to_owned());
    };
    if property != "trailing-horizontal-whitespace" {
        return Err("SourceMaintenancePropertyUnsupported".to_owned());
    }
    if source.len() > 131_072 {
        return Err("SourceMaintenanceInputLimit".to_owned());
    }
    if crate::coding::cst::validated_program_cst(language, source).is_none() {
        return Err("SourceMaintenanceInvalidSource".to_owned());
    }
    let trees =
        tokenize(source).map_err(|_| "SourceMaintenanceUnsupportedLexicalSource".to_owned())?;
    let intervals = lexical_intervals(source, &trees)?;
    let (characters, protection) = observed_characters(source, &intervals);
    let normalized = match normalize_trailing_layout(characters, protection) {
        SourceLayoutResult::Refused(reason) => return Err(reason),
        SourceLayoutResult::Normalized(normalized, _) => normalized,
    };
    // The maintained pivot owns span-free token, group and template equality.
    let before = extract("", language_kind, source)
        .map_err(|_| "SourceMaintenanceUnsupportedLexicalSource".to_owned())?;
    let after = extract("", language_kind, &normalized)
        .map_err(|_| "SourceMaintenanceUnsupportedLexicalSource".to_owned())?;
    if before.trees != after.trees {
        return Err("SourceMaintenanceTokenMismatch".to_owned());
    }
    if !source
        .bytes()
        .filter(|byte| matches!(byte, b'\r' | b'\n'))
        .eq(normalized
            .bytes()
            .filter(|byte| matches!(byte, b'\r' | b'\n')))
    {
        return Err("SourceMaintenanceLinebreakMismatch".to_owned());
    }
    Ok(normalized)
}
