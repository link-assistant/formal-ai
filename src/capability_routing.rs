//! Capability is a function of the object in the request, the act asked for,
//! and where the effect lands (#1138 B10, plan 10 Architecture 1-2).
//!
//! Verbs select the *act*; they never select the capability. A triple with no
//! row in `data/seed/capability-routing.lino` asks rather than guesses, so a
//! silent UNKNOWN is unreachable by construction.
//!
//! Wave T skeleton: the shapes the tests name exist, the behaviour does not.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// What the request is *about*, derived structurally with no natural-language
/// vocabulary in the derivation (plan 10 Architecture 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ObjectType {
    /// A token parsing as an absolute URL.
    Url,
    /// A token with a path separator, a known extension, or a filesystem noun.
    Path,
    /// A filesystem scope noun (`desktop`, `home`, the current directory).
    PathScope,
    /// A balanced quoted span longer than one token.
    QuotedContent,
    /// A clock time, a date, or a weekday/relative-day reference.
    TimeExpression,
    /// A term resolving to a registered language in `data/seed/languages.lino`.
    LanguageName,
    /// An interrogative whose expected answer is a magnitude.
    QuantityQuestion,
    /// One or more content tokens that are none of the above.
    BareTerm,
    /// The subject is the assistant's own UI, answer or behaviour.
    SelfSurface,
    /// No content token survives.
    #[default]
    None,
}

impl ObjectType {
    /// The seed slug, so the decision table is data.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        panic!("plan 10 leaf 4 -- ObjectType slugs")
    }

    /// Read an object type back from its seed slug.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        let _ = slug;
        todo!("plan 10 leaf 4 -- ObjectType slugs")
    }
}

/// The eight acts, one meaning per act, five languages, seeded in
/// `data/seed/meanings-acts.lino`. The act vocabulary is the one place
/// per-language surfaces remain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Act {
    Retrieve,
    Enumerate,
    Transform,
    Compose,
    Schedule,
    Explain,
    Demonstrate,
    Record,
    #[default]
    Unresolved,
}

impl Act {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        panic!("plan 10 leaf 6 -- act slugs")
    }

    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        let _ = slug;
        todo!("plan 10 leaf 6 -- act slugs")
    }
}

/// Where the effect lands. Derived, never stated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Locus {
    Workspace,
    Web,
    Dialogue,
    SelfSurface,
    /// Unresolved: the table's `ask` outcome applies.
    #[default]
    Unresolved,
}

impl Locus {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        panic!("plan 10 leaf 7 -- locus slugs")
    }
}

/// One row of `data/seed/capability-routing.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteRow {
    pub object: ObjectType,
    pub act: Act,
    pub locus: Locus,
    /// The preferred capability, e.g. `web_fetch`.
    pub capability: String,
    /// The capability used when the preferred one is not advertised (#758's
    /// specialized-first, bash-as-universal-fallback policy, stated as data).
    pub fallback: Option<String>,
    /// The written reason the row exists.
    pub because: String,
}

/// The four exhaustive outcomes (plan 10 Architecture 5). Every path ends in
/// one of them, so "silent UNKNOWN" is unreachable by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingOutcome {
    /// The triple has a row and the capability's tool is advertised.
    Routed { capability: String },
    /// The preferred capability is not advertised and the row names a fallback.
    Lowered {
        preferred: String,
        capability: String,
    },
    /// The triple has a row but neither the capability nor its fallback is
    /// advertised: the answer names what was needed and what was missing.
    HonestGap { needed: String, missing: String },
    /// The triple has no row, or two rows tie: one question naming both readings.
    Ask { readings: Vec<String> },
}

/// Every object the prompt carries, ranked; the table is consulted for the
/// highest-ranked first.
#[must_use]
pub fn object_type(prompt: &str) -> Vec<ObjectType> {
    let _ = prompt;
    todo!("plan 10 leaf 4 -- structural object derivation")
}

/// The act the prompt asks for, resolved through `data/seed/meanings-acts.lino`.
#[must_use]
pub fn act(prompt: &str) -> Act {
    let _ = prompt;
    todo!("plan 10 leaf 6 -- eight acts, five languages")
}

/// Where the effect lands, derived from the objects and the scope nouns.
#[must_use]
pub fn locus(prompt: &str) -> Locus {
    let _ = prompt;
    todo!("plan 10 leaf 7 -- derive the locus, possessive-insensitive")
}

/// The shipped decision table.
#[must_use]
pub fn routing_table() -> Vec<RouteRow> {
    todo!("plan 10 leaf 8 -- read data/seed/capability-routing.lino")
}

/// Parse a routing table document, so a fixture row can change routing with no
/// Rust edit.
///
/// # Errors
/// Returns the parse failure when the document is not a routing table.
pub fn routing_table_from(text: &str) -> Result<Vec<RouteRow>, String> {
    let _ = text;
    todo!("plan 10 leaf 8 -- parse data/seed/capability-routing.lino")
}

/// Resolve one prompt against the table, given the capabilities the client
/// advertised.
#[must_use]
pub fn route(prompt: &str, advertised: &[&str]) -> RoutingOutcome {
    let _ = (prompt, advertised);
    todo!("plan 10 leaves 8-10 -- route over the decision table")
}

/// Resolve one triple against a supplied table.
#[must_use]
pub fn route_with(
    table: &[RouteRow],
    object: ObjectType,
    act: Act,
    locus: Locus,
    advertised: &[&str],
) -> RoutingOutcome {
    let _ = (table, object, act, locus, advertised);
    todo!("plan 10 leaves 8-10 -- route over the decision table")
}
