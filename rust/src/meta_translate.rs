//! Any-direction translation through the meta language as the single pivot
//! (issue #1138, plan 16 L2).
//!
//! `translate(X → Y)` is always `extract(X → meta)` followed by
//! `render(meta → Y)`; the dispatcher holds no per-pair code. A direction
//! opens by carrying a leg, and a leg that is not materialized yet reports
//! the plan-16 leaf that owes it — the same honesty the capability table
//! practices: a gap is stated, never papered over with a silent no-op.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// The four trees the pivot connects; `Meta` is the pivot itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceRoot {
    Rust,
    JavaScript,
    TypeScript,
    Meta,
}

impl SourceRoot {
    /// Parse the CLI spelling of a root, accepting the long language name
    /// beside the directory spelling.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "js" | "javascript" => Some(Self::JavaScript),
            "ts" | "typescript" => Some(Self::TypeScript),
            "meta" | "lino" => Some(Self::Meta),
            _ => None,
        }
    }

    /// The canonical CLI spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::JavaScript => "js",
            Self::TypeScript => "ts",
            Self::Meta => "meta",
        }
    }
}

/// What one translate call produced: the rendered target, or the honest gap
/// naming the plan-16 leaf that owes the missing half of the leg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationOutcome {
    /// The pivot carried the source into the target language.
    Rendered { target: String },
    /// The leg is registered but not materialized; the leaf owes it.
    Pending { plan_leaf: &'static str },
}

/// The plan-16 leaf that owes a leg, or `None` when the leg is live.
///
/// Live today: `rust → meta`, the self-AST projection that renders one Rust
/// module as its meta-language census document (the format committed under
/// `data/meta/self-ast/`). Every other direction runs through the pivot in
/// principle and is owed by exactly one leaf: the projection renderers and the
/// advertised `js → ts` command by L2, the dogfood back-translation into Rust
/// by L3, and the CST-equal round trip — including the full `meta → rust`
/// inverse and the js/ts extractors into the pivot — by L5. `L0` marks a
/// same-root non-direction: no leaf owes it because it is not a translation,
/// and callers reject it before listing.
#[must_use]
pub const fn pending_leg(from: SourceRoot, to: SourceRoot) -> Option<&'static str> {
    match (from, to) {
        (SourceRoot::Rust, SourceRoot::Meta) => None,
        (SourceRoot::Rust, SourceRoot::JavaScript | SourceRoot::TypeScript)
        | (SourceRoot::JavaScript | SourceRoot::TypeScript, SourceRoot::Meta)
        | (SourceRoot::Meta, SourceRoot::Rust) => Some("L5"),
        (SourceRoot::Meta, SourceRoot::JavaScript | SourceRoot::TypeScript)
        | (SourceRoot::JavaScript, SourceRoot::TypeScript)
        | (SourceRoot::TypeScript, SourceRoot::JavaScript) => Some("L2"),
        (SourceRoot::JavaScript | SourceRoot::TypeScript, SourceRoot::Rust) => Some("L3"),
        (SourceRoot::Rust, SourceRoot::Rust)
        | (SourceRoot::JavaScript, SourceRoot::JavaScript)
        | (SourceRoot::TypeScript, SourceRoot::TypeScript)
        | (SourceRoot::Meta, SourceRoot::Meta) => Some("L0"),
    }
}

/// Translate one document from `from` into `to` through the meta pivot.
///
/// `display_path` names the source for the rendered document's target header,
/// and `source` is its text. Calling this on a pending leg returns the gap
/// rather than failing, so a caller can list and translate through one shape.
#[must_use]
pub fn translate(
    from: SourceRoot,
    to: SourceRoot,
    display_path: &str,
    source: &str,
) -> TranslationOutcome {
    if let Some(plan_leaf) = pending_leg(from, to) {
        return TranslationOutcome::Pending { plan_leaf };
    }
    match (from, to) {
        (SourceRoot::Rust, SourceRoot::Meta) => TranslationOutcome::Rendered {
            target: crate::agentic_coding::self_ast::render_ast_document(display_path, source),
        },
        _ => TranslationOutcome::Pending {
            plan_leaf: pending_leg(from, to).unwrap_or("L2"),
        },
    }
}

/// Every ordered pair of distinct roots with its leg status, in a stable
/// order, so the CLI can state what the cycle covers and what it still owes.
#[must_use]
pub fn directions() -> Vec<(SourceRoot, SourceRoot, Option<&'static str>)> {
    let roots = [
        SourceRoot::Rust,
        SourceRoot::JavaScript,
        SourceRoot::TypeScript,
        SourceRoot::Meta,
    ];
    let mut listed = Vec::new();
    for from in roots {
        for to in roots {
            if from != to {
                listed.push((from, to, pending_leg(from, to)));
            }
        }
    }
    listed
}

/// The one-line CLI rendering of a leg row; the row text is carried by the
/// seed (`translate_leg_live` / `translate_leg_pending`).
#[must_use]
pub fn describe_leg(from: SourceRoot, to: SourceRoot, pending: Option<&'static str>) -> String {
    pending
        .map_or_else(
            || {
                crate::seed::render_response(
                    "translate_leg_live",
                    "en",
                    &[("from", from.name()), ("to", to.name())],
                )
            },
            |leaf| {
                crate::seed::render_response(
                    "translate_leg_pending",
                    "en",
                    &[("from", from.name()), ("to", to.name()), ("leaf", leaf)],
                )
            },
        )
        .unwrap_or_else(|| "translate_leg".to_string())
}
