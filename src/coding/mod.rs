//! Coding-task support for the universal solver.
//!
//! This module owns everything the engine needs to answer a `write_program`
//! request for a *general* coding task — not only "hello world". It is split
//! into two cohesive units following the repository's architecture principles
//! (high cohesion, clear naming, small focused files):
//!
//! - [`catalog`]: the catalog of supported programming languages, coding tasks,
//!   per-language code templates, and the lookup/alias-matching helpers that map
//!   a normalized prompt onto a concrete [`catalog::ProgramSpec`]. New languages
//!   and tasks are added here as data, so the surface grows without touching the
//!   engine.
//! - [`guidance`]: the novice-first guidance (issue #330) that accompanies every
//!   generated program — a localized "How it works" explanation and step-by-step
//!   "How to test it yourself" instructions, with history-aware brevity on
//!   follow-up edits.
//!
//! Catalog items are re-exported at the `coding` level so callers refer to them
//! as `crate::coding::ProgramSpec` rather than reaching into the submodule.

pub mod catalog;
pub mod guidance;

pub use catalog::*;
