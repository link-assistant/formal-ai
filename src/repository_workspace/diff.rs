//! The unified diff a repository task is finally judged by (#1138 B7).
//!
//! The diff is computed from the bytes the protocol observed, not from a second
//! `git` call, and it must round-trip: applying it to a second clone at the same
//! base commit reproduces the edited tree byte for byte.
//!
//! Wave T lands the shapes only; wave I7 leaf 03-L5 fills the bodies in.

/// A unified diff between the base commit and the current tree.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnifiedDiff(pub String);

impl UnifiedDiff {
    /// The diff text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the tree is untouched.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Compute the unified diff between `before` and `after` for one path.
///
/// A whole-file hunk rather than a minimal one: the diff has to *apply*, and a
/// hunk that names every line it replaces applies to a clean clone at the same
/// base commit without depending on a context heuristic agreeing with git's.
#[must_use]
pub fn unified_diff(relative_path: &str, before: &str, after: &str) -> UnifiedDiff {
    use std::fmt::Write as _;

    if before == after {
        return UnifiedDiff::default();
    }
    let before_lines = split_lines(before);
    let after_lines = split_lines(after);

    let mut out = String::new();
    let _ = writeln!(out, "--- a/{relative_path}");
    let _ = writeln!(out, "+++ b/{relative_path}");
    let _ = writeln!(
        out,
        "@@ -1,{} +1,{} @@",
        before_lines.len(),
        after_lines.len()
    );
    for line in before_lines {
        let _ = writeln!(out, "-{line}");
    }
    for line in after_lines {
        let _ = writeln!(out, "+{line}");
    }
    UnifiedDiff(out)
}

/// The lines of a file, without a phantom empty line for a trailing newline.
fn split_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.strip_suffix('\n')
            .unwrap_or(text)
            .split('\n')
            .collect()
    }
}
