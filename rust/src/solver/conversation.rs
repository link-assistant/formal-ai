//! The conversation a solve continues.
//!
//! Split from `rust/src/solver.rs` to keep it below the 900-line warning band
//! of `tests/unit/ci-cd/issue_1012.rs`. The solver records every earlier turn
//! as a `prior_turn:<role>` event before it processes the current impulse;
//! these are the turns and their roles.

/// Speaker role for [`ConversationTurn`]. The solver only inspects user
/// turns when recalling prior context; assistant turns are kept in the log
/// so the trace stays balanced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationRole {
    User,
    Assistant,
}

impl ConversationRole {
    /// Lowercase slug used in `prior_turn:<role>` event kinds.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

/// A single message in a multi-turn conversation.
///
/// The solver records every turn as a `prior_turn:<role>` event before
/// processing the current impulse so memory recall is grounded in the
/// append-only log, not in implicit state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationTurn {
    pub role: ConversationRole,
    pub content: String,
}

impl ConversationTurn {
    /// Construct a user turn.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ConversationRole::User,
            content: content.into(),
        }
    }

    /// Construct an assistant turn.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ConversationRole::Assistant,
            content: content.into(),
        }
    }
}
