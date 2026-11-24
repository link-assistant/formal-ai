use super::MAX_ANCHOR_BYTES;

pub(super) fn inserted_at_end(source: &str, text: &str, at_end: bool) -> String {
    if source.is_empty() {
        [text, "\n"].concat()
    } else if at_end && source.ends_with('\n') {
        [source, text, "\n"].concat()
    } else if at_end {
        [source, "\n", text].concat()
    } else {
        [text, "\n", source].concat()
    }
}

/// The smallest unique run of whole lines at the insertion end, and what it
/// becomes, so the edit tool carries the change rather than the whole file.
pub(super) fn compact_end_edit(
    source: &str,
    updated: &str,
    at_end: bool,
) -> Option<(String, String)> {
    let starts = source
        .match_indices('\n')
        .map(|(index, _)| index + 1)
        .filter(|start| *start < source.len());
    let cuts: Vec<usize> = if at_end {
        std::iter::once(0).chain(starts).rev().collect()
    } else {
        starts.chain(std::iter::once(source.len())).collect()
    };
    for cut in cuts {
        let anchor = if at_end {
            &source[cut..]
        } else {
            &source[..cut]
        };
        if anchor.trim().is_empty() {
            continue;
        }
        if anchor.len() > MAX_ANCHOR_BYTES {
            break;
        }
        if source.matches(anchor).count() != 1 {
            continue;
        }
        let replacement = if at_end {
            &updated[cut..]
        } else {
            &updated[..updated.len() - (source.len() - cut)]
        };
        return Some((anchor.to_owned(), replacement.to_owned()));
    }
    None
}
