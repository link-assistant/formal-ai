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
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Compute the unified diff between `before` and `after` for one path.
#[must_use]
pub fn unified_diff(_relative_path: &str, _before: &str, _after: &str) -> UnifiedDiff {
    todo!("plan 03 leaf L5")
}
