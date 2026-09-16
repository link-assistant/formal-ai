//! Deep formalization: segmentation, need emission, concept grounding,
//! procedure extraction and the concept graph (issue #1138, plan 04).
//!
//! The formalizer takes plan 01's `SourceLookup` rather than a transport, so it
//! owns no cache policy, no settings reading and no source list of its own. A
//! document with an unresolved need can never be reported as covered.

pub mod concept_links;
pub mod concepts;
pub mod needs;
pub mod procedures;
pub mod segment;
