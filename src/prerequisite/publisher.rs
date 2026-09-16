//! Finding the *official* setup procedure, not the best-ranked page (#1138 B6).
//!
//! Ranking is not authority: a `.gov`/`.edu` preference does not identify a
//! compiler's official source. `data/seed/setup-publishers.lino` pins which host
//! is authoritative for which program, and a lookalike host is refused with a
//! recorded reason.
//!
//! Wave T lands the shapes only; wave I6 leaf 06-L6 fills the bodies in.

use std::path::PathBuf;

use super::{PrerequisiteNeed, Platform};
use crate::source_walk::{LookupBounds, SourceLookup};

/// An install procedure, with the provenance that makes it trustable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupProcedure {
    /// The program this procedure installs.
    pub program: String,
    /// The trusted publisher the procedure came from, by `sources_registry` id.
    pub source_id: String,
    /// The exact URL fetched.
    pub source_url: String,
    /// Content id of the bytes retrieved.
    pub content_id: String,
    /// Platform this procedure is valid for.
    pub platform: Platform,
    /// Ordered steps, each with its own postcondition.
    pub steps: Vec<SetupStep>,
    /// The probe that must pass afterwards. Without it the procedure is refused.
    pub postcondition: Option<super::probe::ToolchainProbe>,
}

/// One ordered step of a setup procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupStep {
    /// The command line exactly as the publisher documents it.
    pub command: String,
    /// Where the step is allowed to write. A step outside the workspace root is
    /// refused before execution, whatever the fetched text says.
    pub writes_under: PathBuf,
    /// Expected artifact digest, when the publisher documents one.
    pub digest: Option<String>,
}

/// One row of `data/seed/setup-publishers.lino`: which host is authoritative for
/// which program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupPublisher {
    /// The program the row is authoritative for.
    pub program: String,
    /// The `sources-registry` id of the publisher.
    pub source_id: String,
    /// The host the publisher actually serves from.
    pub host: String,
}

/// Every publisher row declared in seed, in file order.
#[must_use]
pub fn seed_publishers() -> Vec<SetupPublisher> {
    todo!("plan 06 leaf L6")
}

/// Why a candidate procedure was refused, recorded rather than dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherRefusal {
    /// The host that was refused.
    pub host: String,
    /// The program it claimed to publish.
    pub program: String,
    /// Why it was refused.
    pub reason: String,
}

/// What the publisher search observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherSearch {
    /// The procedure found, when one carried a postcondition.
    pub procedure: Option<SetupProcedure>,
    /// Registry ids consulted, in consultation order.
    pub consulted: Vec<String>,
    /// Every candidate refused, with its reason.
    pub refusals: Vec<PublisherRefusal>,
    /// The dependency cycle detected, when one was.
    pub cycle: Vec<String>,
}

/// Find `program`'s official setup procedure through the trusted-source
/// registry, deepest-first over the source kinds declared in
/// `data/seed/sources-registry.lino`.
///
/// Bounded by evidence, not by a budget: the search ends when a procedure with a
/// postcondition is found, when the publisher list is exhausted, or when a cycle
/// is detected.
pub fn discover_setup_procedure<L: SourceLookup>(
    _need: &PrerequisiteNeed,
    _lookup: &mut L,
    _bounds: &LookupBounds,
) -> Option<SetupProcedure> {
    todo!("plan 06 leaf L6")
}

/// The same search, reporting everything it observed rather than only its result.
pub fn search_setup_procedure<L: SourceLookup>(
    _need: &PrerequisiteNeed,
    _lookup: &mut L,
    _bounds: &LookupBounds,
) -> PublisherSearch {
    todo!("plan 06 leaf L6")
}
