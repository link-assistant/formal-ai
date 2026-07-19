//! User-requested self-change through the same human-gated repair loop (issue #558).
//!
//! Issue #558 ("Auto learning") asks that *"users must be able to ask for changes in
//! the AI system through this mechanism"* (`R558-07`): a natural-language change
//! request should flow through the *same* human-gated repair loop that the
//! self-healing slices use — producing a requirement, a test, and a patch, offered as
//! a reviewable pull request that only merges when the tests pass and the user
//! accepts.
//!
//! This module composes that flow deterministically. A raw request plus the owned
//! module it targets becomes a [`ChangeRequest`]: a normalised *requirement*, a
//! proposed *test* name, and an ordered *patch plan* whose target module is grounded
//! against the owned manifest ([`crate::self_source_links::owned_manifest`]), so a
//! request can never target source the repository does not ship. The whole thing
//! serialises to Links Notation — the reviewable pull request a human reads.
//!
//! Crucially it is **proposal-only and human-gated**, exactly like
//! [`crate::self_healing`] and [`crate::learning_ledger`]: [`ChangeRequest::review`]
//! reuses the same two acceptance conditions — a green [`BenchmarkGateReport`] *and*
//! an explicit [`HumanApproval`] — and refuses every case that is not both, so no
//! user request is ever applied automatically. Neural inference stays a NON-GOAL: the
//! requirement, test, and patch plan are deterministic functions of the request and
//! its grounded target, and the *patch* is a plan a human or Agent CLI executes, not
//! generated code.

use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::learning_ledger::HumanApproval;
use crate::self_improvement::BenchmarkGateReport;
use crate::self_source_links::owned_manifest;

/// A natural-language request to change Formal AI itself, turned into a structured,
/// reviewable, human-gated proposal.
///
/// Every field is a deterministic function of the raw request and the owned module it
/// targets, so the whole proposal — and its content id — is reproducible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRequest {
    /// Stable content-addressed id of the request.
    pub id: String,
    /// The raw natural-language request, verbatim.
    pub request: String,
    /// The owned `src/**/*.rs` module the change targets (grounded against the manifest).
    pub target_module: String,
    /// The manifest content id of the target module — provenance for the review.
    pub target_content_id: String,
    /// The requirement derived from the request — the "requirements" half of the loop.
    pub derived_requirement: String,
    /// The name of the test the change must add before it is reviewable.
    pub proposed_test: String,
    /// The ordered patch plan a human or Agent CLI executes — the reviewable "patch".
    pub patch_plan: Vec<String>,
}

impl ChangeRequest {
    /// Turn `request` into a reviewable change proposal targeting `target_module`.
    ///
    /// # Panics
    ///
    /// Panics if `target_module` is not an owned `src/**/*.rs` file, or if `request`
    /// is empty once trimmed. Both are deliberate: a change request must target real,
    /// content-addressed source (never a path the repository does not ship), and an
    /// empty request has no requirement to derive.
    #[must_use]
    pub fn for_module(request: &str, target_module: &str) -> Self {
        let trimmed = request.trim();
        assert!(
            !trimmed.is_empty(),
            "a change request must carry a non-empty request"
        );
        let Some(digest) = owned_manifest()
            .into_iter()
            .find(|digest| digest.path == target_module)
        else {
            panic!("a change request targets a module that is not in the owned manifest: {target_module}")
        };

        let derived_requirement = derive_requirement(trimmed);
        let proposed_test = derive_test_name(trimmed);
        let patch_plan = patch_plan(target_module, &proposed_test, &derived_requirement);
        let id = stable_id(
            "change_request",
            &format!("{trimmed}:{target_module}:{}", digest.content_id),
        );
        Self {
            id,
            request: trimmed.to_owned(),
            target_module: target_module.to_owned(),
            target_content_id: digest.content_id,
            derived_requirement,
            proposed_test,
            patch_plan,
        }
    }

    /// Whether applying this change stays a human decision. Always `true`: the request
    /// is proposal-only by construction, mirroring the self-healing loop.
    #[must_use]
    pub const fn is_human_gated(&self) -> bool {
        true
    }

    /// Review the request against the same repair-loop gate as the learning ledger.
    ///
    /// Succeeds only when the benchmark gate is green (*"when tests … accept"*) *and*
    /// `approval` is granted (*"and the user accept\[s\]"*), returning the merged
    /// [`AcceptedChange`]. Otherwise returns the [`ChangeRejected`] reason, so no user
    /// request is ever applied without both the tests and the human accepting.
    ///
    /// # Errors
    ///
    /// Returns [`ChangeRejected::TestsNotGreen`] if the gate does not permit adoption,
    /// or [`ChangeRejected::HumanDeclined`] if the human withheld approval.
    pub fn review(
        &self,
        gate: &BenchmarkGateReport,
        approval: &HumanApproval,
    ) -> Result<AcceptedChange, ChangeRejected> {
        if !gate.permits_adoption() {
            return Err(ChangeRejected::TestsNotGreen);
        }
        if !approval.is_granted() {
            return Err(ChangeRejected::HumanDeclined);
        }
        Ok(AcceptedChange {
            change_id: self.id.clone(),
            request: self.request.clone(),
            target_module: self.target_module.clone(),
            requirement: self.derived_requirement.clone(),
            test: self.proposed_test.clone(),
            benchmark_suite: gate.suite_id.clone(),
            benchmark_passed: gate.passed,
            reviewer: approval.reviewer().to_owned(),
        })
    }

    /// A one-line human-readable summary of the proposed change.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Change request `{}` maps onto {} (content id {}); it derives the requirement \"{}\", proposes test `{}`, and a {}-step patch plan — human-gated, reviewable as a pull request.",
            self.id,
            self.target_module,
            self.target_content_id,
            self.derived_requirement,
            self.proposed_test,
            self.patch_plan.len(),
        )
    }

    /// Render the whole change request as Links Notation — the reviewable pull request
    /// a human reads before any merge. Ends trimmed of trailing whitespace.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("change_request\n");
        field(&mut out, "id", &self.id);
        field(&mut out, "request", &self.request);
        field(&mut out, "human_gated", "true");
        field(&mut out, "target_module", &self.target_module);
        field(&mut out, "target_content_id", &self.target_content_id);
        field(&mut out, "derived_requirement", &self.derived_requirement);
        field(&mut out, "proposed_test", &self.proposed_test);
        out.push_str("  reviewable_pull_request\n");
        for step in &self.patch_plan {
            nested(&mut out, "step", step);
        }
        out.trim_end().to_owned()
    }

    /// A stable content id over the request's Links Notation.
    #[must_use]
    pub fn content_id(&self) -> String {
        stable_id("change_request", &self.links_notation())
    }
}

/// Why a [`ChangeRequest`] could not be merged.
///
/// Both variants are guardrails: a user-requested change stays proposal-only until
/// *both* the tests and the human accept, mirroring [`crate::learning_ledger`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeRejected {
    /// The benchmark gate did not pass — "tests" did not accept.
    TestsNotGreen,
    /// The human withheld approval — "the user" did not accept.
    HumanDeclined,
}

impl ChangeRejected {
    /// A stable, human-readable slug for the rejection reason.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::TestsNotGreen => "tests_not_green",
            Self::HumanDeclined => "human_declined",
        }
    }
}

/// A user-requested change that passed the tests and was approved — a merged pull
/// request. Deterministic: every field comes from the request, gate, and approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedChange {
    /// The originating change-request id (provenance back to the request).
    pub change_id: String,
    /// The raw natural-language request that was fulfilled.
    pub request: String,
    /// The owned module the accepted change targets.
    pub target_module: String,
    /// The requirement the change satisfies.
    pub requirement: String,
    /// The test that pinned the requested behaviour.
    pub test: String,
    /// The benchmark suite that gated acceptance.
    pub benchmark_suite: String,
    /// Passing case count from the gate run that green-lit the merge.
    pub benchmark_passed: usize,
    /// The human who approved the merge.
    pub reviewer: String,
}

impl AcceptedChange {
    /// A one-line human-readable summary of the accepted change.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Accepted change `{}` for request \"{}\": requirement \"{}\", test `{}`, mapped onto {}, gated by {} passing case(s), approved by {}.",
            self.change_id,
            self.request,
            self.requirement,
            self.test,
            self.target_module,
            self.benchmark_passed,
            self.reviewer,
        )
    }
}

/// Build the canonical, fully-worked change request.
///
/// A concrete user request ("let users ask Formal AI to reverse-sort program output")
/// targeting the deterministic planner — the module that would route the new
/// capability. Deterministic and self-contained — used by the agentic recipe, the
/// example, and the tests.
#[must_use]
pub fn canonical_change_request() -> ChangeRequest {
    ChangeRequest::for_module(
        "Please add a new capability to Formal AI: let users ask it to reverse-sort a program's output.",
        "src/agentic_coding/planner.rs",
    )
}

/// Derive a requirement statement from a raw request.
///
/// Strips leading politeness, trailing punctuation, and lower-cases the first word, so
/// "Please add X." becomes "The system must add X." — a deterministic transform, not
/// paraphrase.
fn derive_requirement(request: &str) -> String {
    let mut core = collapse_whitespace(request);
    for prefix in [
        "please ",
        "can you please ",
        "can you ",
        "could you please ",
        "could you ",
        "i want you to ",
        "i would like you to ",
        "i want ",
        "i would like ",
        "we need to ",
        "we need ",
    ] {
        if let Some(stripped) = core.to_lowercase().strip_prefix(prefix) {
            core = core[core.len() - stripped.len()..].to_owned();
            break;
        }
    }
    let core = core.trim().trim_end_matches(['.', '!', '?']).trim();
    format!("The system must {}.", decapitalize(core))
}

/// Derive a `snake_case` test name from a raw request.
fn derive_test_name(request: &str) -> String {
    let mut slug = String::new();
    let mut last_was_sep = true;
    for ch in request.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            slug.push('_');
            last_was_sep = true;
        }
    }
    // Keep the name readable: at most the first eight words of the request.
    let words: Vec<&str> = slug.trim_matches('_').split('_').take(8).collect();
    format!("user_requested_change_{}", words.join("_"))
}

/// The fixed, ordered patch plan a human or Agent CLI executes to fulfil the request.
///
/// It is a *plan* (requirement → failing test → grounded edit → green gate → reviewed
/// merge), not generated code — honouring the NON-GOAL of neural inference while still
/// producing the "requirements, tests, patches, and a reviewable PR" the issue asks
/// for.
fn patch_plan(target_module: &str, test: &str, requirement: &str) -> Vec<String> {
    vec![
        format!("Record the derived requirement for review: {requirement}"),
        format!("Add the failing test `{test}` that pins the requested behaviour"),
        format!("Edit the grounded target module `{target_module}` until the test passes"),
        "Run the benchmark gate; only a green run is reviewable".to_owned(),
        "Open a reviewable pull request; a human approves before it merges".to_owned(),
    ]
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decapitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_lowercase().collect::<String>() + chars.as_str()
    })
}

fn field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "  {key} \"{}\"", quote(value));
}

fn nested(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "    {key} \"{}\"", quote(value));
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
