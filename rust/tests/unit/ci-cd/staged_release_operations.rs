//! Checked operation provenance for native release fixtures; physical readers remain physical.
use std::path::Path;
use std::process::Command;

fn source_view() -> serde_json::Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root");
    let output = Command::new("node")
        .arg(root.join("scripts/native-staged-release-operation-view.mjs"))
        .arg("--source-view")
        .current_dir(root)
        .output()
        .expect("checked source view Node process must be available");
    assert!(
        output.status.success(),
        "checked source view refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let view: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("complete source view receipt");
    assert_eq!(view["productionAuthority"], false);
    assert_eq!(view["inventory"]["operations"], 52);
    assert_eq!(view["inventory"]["bindings"], 104);
    view
}

pub fn release_operation_workflow() -> String {
    source_view()["originalSource"]
        .as_str()
        .expect("original operation source")
        .to_owned()
}

pub fn assert_release_delivery_budget(job_name: &str) {
    assert!(matches!(job_name, "auto-release" | "manual-release"));
    let view = source_view();
    if view["mode"] == "inline" {
        let minutes = view["physicalCaller"]["jobs"][job_name]["timeout-minutes"]
            .as_u64()
            .expect("inline deadline");
        assert!(
            minutes >= 60,
            "original inline Create Release starvation floor"
        );
        return;
    }
    assert_eq!(view["mode"], "deployed");
    let mode = if job_name == "auto-release" {
        "auto"
    } else {
        "manual"
    };
    let stages = [
        "prepare-source",
        "compile-release",
        "verify-package",
        "publish-crate",
        "registry-availability",
        "published-crate-smoke",
        "publish-verify-images",
        "create-release",
    ];
    let caller = &view["physicalCaller"]["jobs"][job_name];
    assert!(caller.get("timeout-minutes").is_none());
    assert_eq!(caller["with"]["mode"], mode);
    assert_eq!(caller["permissions"]["actions"], "read");
    assert_eq!(
        caller["concurrency"]["group"],
        "formal-ai-repository-writes"
    );
    assert_eq!(caller["concurrency"]["queue"], "max");
    for stage in stages {
        let job = &view["physicalStages"]["jobs"][format!("{mode}_{stage}")];
        assert_eq!(job["timeout-minutes"], 30);
        assert!(job.get("concurrency").is_none());
    }
    let needs = &view["physicalStages"]["jobs"][format!("{mode}_create-release")]["needs"];
    let expected = stages[..7]
        .iter()
        .map(|stage| serde_json::Value::String(format!("{mode}_{stage}")))
        .collect::<Vec<_>>();
    assert_eq!(
        needs.as_array().expect("all prerequisite stages"),
        &expected
    );
}

pub fn assert_image_delivery_budget(job_name: &str) {
    assert!(matches!(job_name, "auto-release" | "manual-release"));
    let view = source_view();
    if view["mode"] == "inline" {
        assert_eq!(
            view["physicalCaller"]["jobs"][job_name]["timeout-minutes"],
            90
        );
        let steps = view["physicalCaller"]["jobs"][job_name]["steps"]
            .as_array()
            .expect("inline steps");
        let caps = steps
            .iter()
            .filter_map(|step| step["timeout-minutes"].as_u64())
            .collect::<Vec<_>>();
        assert_eq!(caps, vec![45, 15, 2, 20]);
        return;
    }
    assert_release_delivery_budget(job_name);
    let mode = if job_name == "auto-release" {
        "auto"
    } else {
        "manual"
    };
    let job = &view["physicalStages"]["jobs"][format!("{mode}_publish-verify-images")];
    assert_eq!(job["timeout-minutes"], 30);
    let steps = job["steps"].as_array().expect("image operations");
    let publish = steps
        .iter()
        .find(|step| step["name"] == "Publish Docker image to GHCR")
        .expect("same original publishing operation");
    assert_eq!(publish["timeout-minutes"], 21);
    assert!(21 * 100 <= 30 * 70);
    let command = publish["run"].as_str().expect("bounded original command");
    for operand in [
        "TEST_BUDGET_ENFORCE=true",
        "TEST_BUDGET_GRACE_SECONDS=5",
        "TEST_BUDGET_POLL_SECONDS=1",
        "run-with-budget-warning.sh 1250",
        "scripts/release-image-factory.mjs publish prepared-release",
        "$RELEASE_VERSION",
        "$GHCR_IMAGE",
    ] {
        assert!(
            command.contains(operand),
            "missing bounded publishing operand {operand}"
        );
    }
    let anonymous = steps
        .iter()
        .find(|step| step["name"] == "Verify anonymous access to the immutable published manifest")
        .expect("original anonymous immutable visibility operation");
    assert_eq!(anonymous["timeout-minutes"], 2);
    assert!(
        anonymous["run"]
            .as_str()
            .expect("immutable verifier")
            .contains("scripts/verify-anonymous-image-manifest.mjs")
    );
}
