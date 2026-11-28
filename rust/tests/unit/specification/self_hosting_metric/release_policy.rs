use super::{Path, fs};

pub(super) fn release_pipeline_and_ledger_remain_pinned_to_the_metric() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let workflow = crate::ci_gates::staged_release_operations::release_operation_workflow();
    let version_script = fs::read_to_string(root.join("scripts/version-and-commit.rs"))
        .expect("version script must be readable");
    let release_script = fs::read_to_string(root.join("scripts/create-github-release.rs"))
        .expect("release script must be readable");
    // Issue #977: both release jobs now build this argv through one shared
    // script instead of duplicating it inline, so the pin lives there.
    let invoke_script = fs::read_to_string(root.join("scripts/invoke-create-github-release.sh"))
        .expect("release invocation script must be readable");
    let ledger = crate::release_ledger::read(&root.join("data/meta/self-hosting-ledger.lino"))
        .expect("self-hosting ledger must be readable");

    assert!(
        workflow.contains("scripts/invoke-create-github-release.sh"),
        "the release jobs must reach the release step through the shared script"
    );
    assert!(invoke_script.contains("--self-hosting-ledger"));
    assert!(
        !workflow.contains("--check-ratchet") && !invoke_script.contains("--check-ratchet"),
        "issue #846: an honest manual contribution must not be blocked for omitting false \
         Formal-AI attribution"
    );
    assert!(version_script.contains("self-hosting-metric.rs"));
    // Issue #1085 (D3.5): the reviewed-contribution floor is reported by
    // `.github/workflows/self-development-status.yml`, red until true, and no
    // longer decides whether a release is cut. The record itself is kept.
    assert!(
        !version_script.contains("ensure_self_development_release("),
        "the release path must record the share, not gate on it"
    );
    assert!(
        version_script.contains("record_release_with_policy"),
        "every release still records its self-hosting row"
    );
    let status_workflow =
        fs::read_to_string(root.join(".github/workflows/self-development-status.yml"))
            .expect("self-development status workflow must be readable");
    assert!(
        status_workflow.contains("scripts/check-self-development-release.rs"),
        "the floor is still checked, from its own workflow"
    );
    assert!(
        !workflow.contains("check-self-development-release.rs"),
        "the release workflow no longer runs the floor as a gate"
    );
    assert!(
        version_script.contains("RatchetPolicy::Report"),
        "issue #812: a release must never be blocked by immutable history"
    );
    assert!(
        ledger.contains("current_metric_version \"3\""),
        "issue #812: the ledger must name the definition its newest rows use"
    );
    assert!(
        ledger.contains("model_trailer \"Formal-AI-Model\""),
        "issue #1085: the ledger must name the trailer that carries the model"
    );
    assert!(ledger.contains("pull_request_trailer \"Formal-AI-Pull-Request\""));
    assert!(ledger.contains("release_cycle_floor \"1\""));
    assert!(ledger.contains("release_cycle_unit \"merged session-backed Formal AI pull request\""));
    // Issue #1069: the header has to say what "session-backed" scopes over, so a
    // reader is not left inferring the all-or-nothing rule the gate dropped.
    assert!(ledger.contains("release_cycle_attribution \"issue #1069: a merged pull request"));
    assert!(release_script.contains("## Self-hosting"));
    assert!(ledger.contains("tag \"v0.296.0\""));
    assert!(ledger.contains("percentage_basis_points \"0\""));
}
