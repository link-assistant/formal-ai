//! Recompile Formal AI and reattach the improved worker to the UI (issue #558, `R558-06`).
//!
//! Issue #558 ("Auto learning") asks that Formal AI be able to *"recompile and reattach
//! the improved code to the UI"* — the last step of the self-change loop, after a change
//! is accepted. The web UI runs Formal AI as a WebAssembly worker
//! (`src/web/formal_ai_worker.wasm`, loaded by `src/web/formal_ai_worker.js` and mounted
//! by `src/web/index.html`); "recompile and reattach" therefore means: rebuild the
//! crate, regenerate that worker, reattach it to the UI, hot-swap the local server, and
//! verify the UI uses the accepted version.
//!
//! This module composes that as a deterministic, human-gated [`RebuildPlan`]: it is
//! *derived from an already-accepted change* ([`crate::change_request::AcceptedChange`],
//! which only exists once a benchmark gate is green *and* a human approved), and it emits
//! an ordered pipeline of steps that are each observable, testable, and reversible. Every
//! UI artifact the plan reattaches is grounded — the crate manifest, the server entry,
//! the worker glue, and the UI entry are embedded from the real repository, so the plan
//! can never reference a fabricated artifact.
//!
//! Nothing here rebuilds or restarts anything: the plan is the reviewable *product*, so
//! the "recompile and reattach" guardrail (observable, testable, reversible,
//! human-approved) is preserved. Neural inference stays a NON-GOAL — the plan is a
//! deterministic function of the accepted change and the embedded artifacts, and the
//! rebuild is a script a human or Agent CLI runs, never executed automatically.

use std::fmt::Write as _;

use crate::change_request::AcceptedChange;
use crate::engine::stable_id;
use crate::self_source_graph::owned_manifest;

/// A UI artifact the rebuild reattaches, grounded by content-addressing its real bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReattachArtifact {
    /// Repository-relative path of the artifact.
    pub path: String,
    /// The artifact's role in the reattach pipeline.
    pub role: String,
    /// Content-addressed id over the artifact's embedded bytes.
    pub content_id: String,
}

/// One observable, reversible step of the rebuild-and-reattach pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebuildStep {
    /// 1-based position in the pipeline.
    pub ordinal: usize,
    /// What the step does.
    pub action: String,
    /// The command a human or Agent CLI runs to perform it.
    pub command: String,
    /// What proves the step happened (observability).
    pub observable: String,
    /// How to roll the step back (reversibility).
    pub reversible: String,
}

/// A deterministic, human-gated plan to recompile Formal AI and reattach it to the UI.
///
/// Derived from an [`AcceptedChange`], so it can only follow the same green-gate +
/// human-approval that the change request and learning ledger enforce. It is a *plan*,
/// not an executed rebuild: a human or Agent CLI runs the steps, and every step is
/// reversible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebuildPlan {
    /// Stable content-addressed id of the plan.
    pub id: String,
    /// The accepted change this rebuild delivers (provenance).
    pub change_id: String,
    /// The requirement the accepted change realised.
    pub requirement: String,
    /// The human who approved the change.
    pub reviewer: String,
    /// The grounded UI artifacts the rebuild reattaches.
    pub artifacts: Vec<ReattachArtifact>,
    /// The ordered, reversible rebuild-and-reattach steps.
    pub steps: Vec<RebuildStep>,
    /// The automated verification that must pass before the swap is kept.
    pub verification: String,
}

impl RebuildPlan {
    /// Compose the rebuild-and-reattach plan for an already-accepted change.
    ///
    /// Deterministic: the artifacts are content-addressed from the embedded repository
    /// bytes, and the steps are a fixed pipeline (recompile → regenerate worker →
    /// reattach → hot-swap → verify). The id is content-addressed over the accepted
    /// change and every artifact id, so any change to the reattached UI surface changes
    /// the plan id.
    #[must_use]
    pub fn for_accepted_change(accepted: &AcceptedChange) -> Self {
        let artifacts = grounded_artifacts();
        let steps = pipeline_steps();
        let artifact_ids = artifacts
            .iter()
            .map(|artifact| artifact.content_id.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let id = stable_id(
            "rebuild_plan",
            &format!("{}:{artifact_ids}", accepted.change_id),
        );
        Self {
            id,
            change_id: accepted.change_id.clone(),
            requirement: accepted.requirement.clone(),
            reviewer: accepted.reviewer.clone(),
            artifacts,
            steps,
            verification:
                "The rebuilt binary must pass the whole test suite, the UI smoke test must load the \
                 regenerated worker, and the version the UI reports must match the accepted change \
                 before the hot-swap is kept; otherwise the prior binary is restored."
                    .to_owned(),
        }
    }

    /// Whether keeping the rebuild stays a human decision. Always `true`: the plan is
    /// derived from an approved change, is proposal-only, and every step is reversible.
    #[must_use]
    pub const fn is_human_gated(&self) -> bool {
        true
    }

    /// A one-line human-readable summary of the plan.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Rebuild-and-reattach plan for accepted change `{}` (approved by {}): a {}-step, reversible pipeline that recompiles Formal AI, regenerates the WebAssembly worker, reattaches it to {} grounded UI artifacts, hot-swaps the local server, and verifies the UI uses the accepted version. Human-gated, proposal-only.",
            self.change_id,
            self.reviewer,
            self.steps.len(),
            self.artifacts.len(),
        )
    }

    /// Render the whole plan as Links Notation — the auditable artifact a human reviews.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("rebuild_plan\n");
        field(&mut out, "id", &self.id);
        field(&mut out, "change_id", &self.change_id);
        field(&mut out, "requirement", &self.requirement);
        field(&mut out, "reviewer", &self.reviewer);
        field(&mut out, "human_gated", "true");
        out.push_str("  reattached_artifacts\n");
        for artifact in &self.artifacts {
            let _ = writeln!(out, "    artifact \"{}\"", quote(&artifact.path));
            sub(&mut out, "role", &artifact.role);
            sub(&mut out, "content_id", &artifact.content_id);
        }
        out.push_str("  rebuild_and_reattach_pipeline\n");
        for step in &self.steps {
            let _ = writeln!(out, "    step \"{}\"", step.ordinal);
            sub(&mut out, "action", &step.action);
            sub(&mut out, "command", &step.command);
            sub(&mut out, "observable", &step.observable);
            sub(&mut out, "reversible", &step.reversible);
        }
        field(&mut out, "verification", &self.verification);
        out.trim_end().to_owned()
    }

    /// A stable content id over the plan's Links Notation.
    #[must_use]
    pub fn content_id(&self) -> String {
        stable_id("rebuild_plan", &self.links_notation())
    }
}

/// The grounded UI artifacts the rebuild reattaches, content-addressed from the real
/// embedded repository bytes so the plan can never reference a fabricated artifact.
///
/// The regenerated worker (`src/web/formal_ai_worker.wasm`) is deliberately *not* listed
/// here: it is the rebuild's *output* (regenerated by the pipeline), so content-addressing
/// its pre-rebuild bytes would be meaningless. The stable inputs — the crate manifest,
/// the server entry, the worker glue that loads the worker, and the UI entry that mounts
/// it — are what is grounded.
fn grounded_artifacts() -> Vec<ReattachArtifact> {
    // The crate manifest recompiled into the new binary/worker.
    let cargo_toml = include_str!("../Cargo.toml");
    // The worker glue that loads the recompiled WebAssembly and reattaches it to the UI.
    let worker_glue = include_str!("web/formal_ai_worker.js");
    // The UI entry that mounts the worker.
    let ui_entry = include_str!("web/index.html");

    // The server/CLI entry that serves the reattached UI, grounded against the owned
    // manifest exactly like a change request's target — a fabricated entry cannot appear.
    let manifest = owned_manifest();
    let server_entry = manifest
        .iter()
        .find(|digest| digest.path == "src/main.rs")
        .expect("the server entry src/main.rs must be in the owned manifest");

    vec![
        ReattachArtifact {
            path: "Cargo.toml".to_owned(),
            role: "crate manifest recompiled into the new binary and worker".to_owned(),
            content_id: stable_id("reattach_artifact", cargo_toml),
        },
        ReattachArtifact {
            path: server_entry.path.clone(),
            role: "server/CLI entry that serves the reattached UI".to_owned(),
            content_id: server_entry.content_id.clone(),
        },
        ReattachArtifact {
            path: "src/web/formal_ai_worker.js".to_owned(),
            role: "worker glue that loads the recompiled WebAssembly and reattaches it to the UI"
                .to_owned(),
            content_id: stable_id("reattach_artifact", worker_glue),
        },
        ReattachArtifact {
            path: "src/web/index.html".to_owned(),
            role: "UI entry that mounts the reattached worker".to_owned(),
            content_id: stable_id("reattach_artifact", ui_entry),
        },
    ]
}

/// The fixed, ordered rebuild-and-reattach pipeline. Every step is observable and
/// reversible, so keeping the swap stays a reviewable human decision.
fn pipeline_steps() -> Vec<RebuildStep> {
    vec![
        RebuildStep {
            ordinal: 1,
            action: "Recompile Formal AI from the accepted source.".to_owned(),
            command: "cargo build --release --locked".to_owned(),
            observable: "a fresh target/release/formal-ai binary whose build is reproducible from the accepted source".to_owned(),
            reversible: "the prior release binary is retained until the swap is kept".to_owned(),
        },
        RebuildStep {
            ordinal: 2,
            action: "Regenerate the WebAssembly worker the browser UI runs.".to_owned(),
            command: "cargo build --release --target wasm32-unknown-unknown && wasm-bindgen (project worker build)".to_owned(),
            observable: "a new src/web/formal_ai_worker.wasm whose content id differs from the shipped one".to_owned(),
            reversible: "git restore src/web/formal_ai_worker.wasm rolls back to the prior worker".to_owned(),
        },
        RebuildStep {
            ordinal: 3,
            action: "Reattach the regenerated worker to the UI.".to_owned(),
            command: "serve src/web/index.html with the worker glue pointing at the new formal_ai_worker.wasm".to_owned(),
            observable: "the UI loads the new worker and reports the rebuilt version".to_owned(),
            reversible: "revert the worker glue to the prior worker reference".to_owned(),
        },
        RebuildStep {
            ordinal: 4,
            action: "Hot-swap the local server/worker to the rebuilt binary.".to_owned(),
            command: "restart `formal-ai serve` with the new binary (or hot-swap the running worker)".to_owned(),
            observable: "the server's version endpoint reports the accepted change id".to_owned(),
            reversible: "restart the prior binary to restore the previous version".to_owned(),
        },
        RebuildStep {
            ordinal: 5,
            action: "Verify the UI uses the accepted version, then keep or roll back.".to_owned(),
            command: "run the UI smoke test and compare the reported version to the accepted change".to_owned(),
            observable: "the UI smoke test passes and the reported version matches the accepted change".to_owned(),
            reversible: "if any check fails, restore the prior binary and worker — the swap is never kept on a red check".to_owned(),
        },
    ]
}

/// The canonical rebuild-and-reattach plan: the plan for the canonical accepted change,
/// used by the agentic recipe, the example, and the tests.
///
/// The canonical change is accepted through the same human gate the change request and
/// learning ledger enforce — a green benchmark gate *and* an explicit approval — so the
/// rebuild plan can only ever follow an approved change.
#[must_use]
pub fn canonical_rebuild_plan() -> RebuildPlan {
    use crate::change_request::canonical_change_request;
    use crate::learning_ledger::HumanApproval;
    use crate::self_improvement::BenchmarkGateReport;

    let request = canonical_change_request();
    let green = BenchmarkGateReport::issue_362_from_counts(4, 0);
    let accepted = request
        .review(&green, &HumanApproval::granted("maintainer"))
        .expect("the canonical change is green and approved, so it accepts");
    RebuildPlan::for_accepted_change(&accepted)
}

fn field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "  {key} \"{}\"", quote(value));
}

fn sub(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "      {key} \"{}\"", quote(value));
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
