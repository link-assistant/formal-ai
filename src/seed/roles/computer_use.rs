//! Role constants for the non-visual computer-use capability layer (issue #707).
//!
//! The generalized planner never names an English (or Russian, Hindi, Chinese)
//! verb in Rust. It asks the [lexicon](crate::seed::lexicon) which meanings
//! carry an operation, resource, or capability-gap role and uses each meaning's
//! *slug* as the language-independent identifier. Adding a new operation or a
//! new resource is therefore a pure data change in
//! `data/seed/meanings-computer-use.lino` (issue #386 convention).
//!
//! Re-exported flat through [`super`] so every constant stays reachable as
//! `crate::seed::roles::ROLE_*` and `crate::seed::ROLE_*`.

/// Semantic role: a computer-use operation a request asks for.
///
/// Carried by `computer_use_*` action meanings (fetch, filter, count, unique,
/// extract, selector query, form submission, directory listing, archive
/// pack/unpack, move, process status). The induction pass in
/// [`crate::computer_use::learned`] learns which primitive step each operation
/// slug denotes by aligning the recognised operations of every seeded benchmark
/// prompt with that task's recorded plan.
pub const ROLE_COMPUTER_USE_OPERATION_CUE: &str = "computer_use_operation_cue";

/// Semantic role: the data resource a computer-use request operates on.
///
/// Carried by `computer_use_resource_*` object meanings. The induction pass
/// learns each resource's materialisation steps (a seeded `fs.write`, or an
/// `http.fetch` of a committed fixture) from the corpus, so an unseen request
/// naming a known resource can be planned without a new hardcoded task.
pub const ROLE_COMPUTER_USE_RESOURCE_CUE: &str = "computer_use_resource_cue";

/// Semantic role: evidence that a request needs a capability we do not have.
///
/// Carried by `computer_use_gap_*` meanings, one per named capability gap (the
/// meaning slug's `computer_use_gap_` suffix *is* the capability name). Any
/// phrasing mentioning such a surface is answered with the honest, localized
/// `capability_gap` response instead of a plan.
pub const ROLE_COMPUTER_USE_CAPABILITY_GAP_CUE: &str = "computer_use_capability_gap_cue";
