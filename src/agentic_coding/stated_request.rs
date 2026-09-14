//! Which part of a prompt states the work.
//!
//! A request handed to a worker often arrives in two parts: the work, and a
//! note that says where the worker is and how to report. Issue #1066 measured
//! what happens when a route reads both as one string — twenty of the ladder's
//! twenty-nine searches took their subject from the note, because the note's
//! longest hyphenated word outscored the one the request was actually about.
//!
//! The rule is the one [`crate::task_decomposition`] already applies when it
//! narrows a prompt to the block that asks: a block that only places the worker
//! is not work of its own, so nothing is read out of it. It is stated here
//! separately because the agentic routes narrow per route — each one asks which
//! block carries *its* act — while decomposition narrows once for the whole
//! request.

/// The prompt's blocks, in order, or the whole prompt when it has only one.
///
/// A blank line is the separator, matching the block split decomposition reads.
/// A prompt with a single block is returned whole rather than trimmed, so a
/// route that narrows sees byte-identical input to what it saw before narrowing
/// existed — the change is meant to be visible only where a second block is.
pub(super) fn request_blocks(prompt: &str) -> Vec<&str> {
    let blocks: Vec<&str> = prompt
        .split("\n\n")
        .map(str::trim)
        .filter(|block| !block.is_empty())
        .collect();
    if blocks.len() < 2 {
        vec![prompt]
    } else {
        blocks
    }
}
