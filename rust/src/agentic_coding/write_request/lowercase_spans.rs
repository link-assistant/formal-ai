//! Lowercase cue matching must not supply offsets for slicing original UTF-8.
use super::{content_lead_close, first_prefix_lead_end};

/// An interior position in a lowercase expansion has no original character boundary.
fn raw_lowercase_boundary(request: &str, offset: usize) -> Option<usize> {
    let mut lowered = 0;
    for (original, character) in request.char_indices() {
        if offset == lowered {
            return Some(original);
        }
        lowered += character.to_lowercase().map(char::len_utf8).sum::<usize>();
        if offset < lowered {
            return None;
        }
    }
    (offset == lowered).then_some(request.len())
}

/// Return original UTF-8 byte boundaries; JavaScript mirrors use UTF-16 indices.
pub(in crate::agentic_coding) fn raw_lowercase_span(
    request: &str,
    span: Option<(usize, usize)>,
) -> Option<(usize, usize)> {
    let (start, end) = span?;
    if start > end {
        return None;
    }
    Some((
        raw_lowercase_boundary(request, start)?,
        raw_lowercase_boundary(request, end)?,
    ))
}

/// Seeded cue matching is performed on lowercase text, but returned spans are raw.
pub(in crate::agentic_coding) fn first_raw_prefix_lead_end(
    request: &str,
    role: &str,
) -> Option<(usize, usize)> {
    raw_lowercase_span(
        request,
        first_prefix_lead_end(&request.to_lowercase(), role),
    )
}

pub(in crate::agentic_coding) fn first_raw_content_lead_end(
    request: &str,
) -> Option<(usize, usize)> {
    first_raw_prefix_lead_end(request, crate::seed::ROLE_FILE_WRITE_CONTENT_LEAD)
}

/// Convert the original opener boundary into a lowercase view and map its closer back.
pub(in crate::agentic_coding) fn raw_content_lead_close(
    request: &str,
    from: usize,
) -> Option<usize> {
    let lowered_from = request.get(..from)?.to_lowercase().len();
    let close = content_lead_close(&request.to_lowercase(), lowered_from)?;
    raw_lowercase_boundary(request, close)
}
