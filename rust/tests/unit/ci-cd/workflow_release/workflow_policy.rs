use std::fs;

use super::super::workflow_fixtures::{
    job_block, minimum_job_timeout, pages_artifact_workflow, release_workflow, workflow_job_names,
};

#[test]
fn release_workflow_jobs_have_explicit_timeouts() {
    let workflow = release_workflow();
    let expected_timeouts = [
        ("hive-source-provenance", 0),
        ("detect-changes", 5),
        ("changelog", 10),
        // Issue #808's evidence-check job now lives in
        // `.github/workflows/evidence-check.yml`; its timeout is asserted by
        // `evidence_check_workflow_caps_its_job_and_installs_rust_script_with_the_retry_wrapper`.
        ("docker-build", 30),
        ("docker-slim", 0),
        // PR #1523: 597s checkout left only 5s for the clean scanner before cancellation.
        // Keep full history/tree and reserve five bounded minutes after acquisition.
        ("secrets-scan", 15),
        ("version-check", 5),
        // Issue #1081 (D15): probes every publishing credential with a write
        // before the pipeline spends 90 minutes building what it cannot
        // publish. Five probes over HTTP against five registries; the cap is a
        // backstop for a hung TLS handshake, not a work budget.
        ("release-preflight", 5),
        // Issue #1017: resolves the base-branch commit once so `lint`, `test`
        // and the macOS lane all merge the same one instead of each resolving
        // the tip at its own start time. A reusable workflow, so it owns its
        // own cap -- and so `release.yml` does not pay lines for it against the
        // 1500-line band this same file pins below.
        ("base", 0),
        // Issue #812: raised from 10; the job grew from ~3.3 to ~7.8 minutes.
        // Issue #1076 (D5): raised from 15. Over 400 `main` runs the worst
        // case was 12.7 min, 84.4% of that cap and trending up (6.0 -> 12.7
        // across the window) -- the deadline, not the backstop, and a cap kills
        // as `cancelled`. No single step dominates here, so the cap is the only
        // mechanism available; 25 puts the same worst case at 50.7%.
        // Issue #1138: raised from 25. The gate set kept growing and run
        // 35806556194 consumed the full 25 minutes, killed mid-gate inside
        // repository_workspace_protocol and reported as `cancelled` -- the red
        // hidden behind a cancelled job, every gate that did run passing. The
        // measured demand of at least 25.2 min sits under 56% of 45, back
        // below the 70% warn line of check-job-headroom.rs.
        // PR #1188: instead of a fourth raise, the gates run as four parallel
        // lanes (`lane` in data/meta/ci-gates/); 30 holds the slowest (~15).
        ("lint", 30),
        // Issue #812: raised from 15 after run 29767811026 was killed 1.1 s
        // after the suite passed. See
        // `test_job_budget_exceeds_the_measured_suite_cost_and_warns_before_it_is_eaten`.
        // Issue #1012 partitions the macOS core suite so every slice retains
        // this baseline rather than extending a monolithic timeout.
        // Issue #1017 raised this from 25: 455s of the job runs outside the
        // budgeted step, so a 20-minute budget under a 25-minute cap could
        // never expire first.
        // Issue #1081 raised this from 35. The job is a matrix, and its two
        // legs are budgeted separately: the `full` leg now declares 2640s
        // across four steps and the `specification` leg 2460s across two, after
        // the compile that was 86% of `Run specification tests` was split into
        // a step of its own. One cap covers both legs, so it has to hold the
        // larger sum at or under the 70% share
        // `job_budget_fit::the_budgets_a_job_can_spend_together_fit_inside_its_cap`
        // enforces: 2640/3900 is 67.7%.
        // The same fix raised the cap from 65 to 90: the spec lane's worst
        // budgeted group is 3600s, and 3600/5400 leaves it at 66% of the cap
        // instead of 92% of the old one -- the budget, not the cap, stays the
        // real deadline.
        // Issue #1138 raised the full leg's `Run tests` budget from 1440 to
        // 2400 (measured demand ~1520s: unit 1251-1315s + integration ~200s),
        // which lifts that leg's sum to 3600s -- the same 66% of this cap.
        // PR #1188 (R1188-U9) lowered it to 30: both suites run prebuilt
        // executables in shards (full five, specification four), and each
        // budgeted group is 1080s, 60% of the cap.
        ("test", 30),
        // Issue #1014 compiles one nextest archive and fans it out to five
        // macOS runners. The reusable workflow owns both internal timeouts.
        ("doc-tests", 30),
        ("macos-core-tests", 0),
        // Issue #896: raised from 10; the published web-search/web-capture
        // graphs moved the job from ~4-5 to 7.2 minutes, and a cold release
        // build after a Cargo.lock change hit the former cap.
        // Issue #1076 (D5): raised from 15. The run that missed its cargo
        // cache took 11.6 min, 77.0% of the cap; 20 puts it at 57.8%.
        ("build", 20),
        // R1015: the JavaScript server answers the parity corpus as the
        // release binary does; it reuses build-artifacts' binary, so no compile.
        ("server-parity", 30),
        // Issue #1076 (D5): raised from 60. Worst case 50.6 min over 400
        // `main` runs, 84.4% of that cap -- and the 45-minute GHCR budget that
        // owns the real deadline could never have fired inside it, since the
        // rest of the job costs ~18 min. A budget the cap pre-empts is not a
        // budget: the run still ends `cancelled`.
        ("auto-release", 0),
        // Same image, same steps, same budgets, same backstop (issue #1076).
        ("manual-release", 0),
        ("changelog-pr", 10),
        // Issue #1138 moved the steps, and the 40-minute cap with them, into
        // `.github/workflows/e2e-local.yml`; what is left here is the call.
        ("test-e2e-local", 0),
        // Issue #538: real Agent CLI ↔ formal-ai OpenAI-compatible round-trip.
        // Boots `formal-ai serve`, drives it with `@link-assistant/agent`, and
        // asserts the CLI writes the enriched meaning file. The extra headroom
        // over test-e2e-local absorbs a cold cargo build of the release binary.
        // Raised from 20 (PR #965 review): 16m16s and 17m30s green left ~2
        // minutes of headroom, and run 31097339962 tipped over into a
        // *cancelled* job that looked like a regression but was only variance.
        // Raised from 32 (issue #1069): 19m54s green, of which the two
        // computer-use steps cost 5m32s, is 39m24s at their 900s+600s budgets.
        // Issue #1081 moved those 324 steps, and the 45-minute cap with them,
        // into `.github/workflows/agent-cli-e2e.yml`; what is left here is the
        // call. See that file's header for why.
        ("test-agent-cli-e2e", 0),
        // Issue #1012: the shared release binary is built once before the seven
        // Box image legs, avoiding seven identical cache restores and builds.
        ("build-artifacts", 25),
        ("native-response-observations", 0),
        // Issue #932: per-language matrix leg that pulls one link-foundation/box
        // image, generates the project from solver answers and runs the
        // language's traditional init commands inside it. The budget covers a
        // cold release build plus the image pull.
        ("box-language-projects", 30),
        // The Pages build runs `cargo doc` for the /docs/api reference (issue
        // #479) in pages-artifact.yml under its own 30-minute cap (R1188-U9).
        ("build-pages", 0),
        // deploy-pages only waits in the GitHub Pages deployment queue, which
        // can stall for many minutes (PR #965 review) — see
        // `pages_deploy_uses_the_actions_maximum_supported_wait`.
        ("deploy-pages", 15),
        ("test-e2e-pages", 15),
        // Issue #977: the terminal gate that turns a silently-`cancelled` run
        // (the shape a `timeout-minutes` kill takes) into a red failure.
        ("native-container-images", 0),
        ("pipeline-status", 5),
    ];
    // Preserve known budget contracts without requiring a second job inventory.
    // Every newly declared job must also own a deadline or delegate to a local workflow.
    for job_name in workflow_job_names(&workflow) {
        let job = job_block(&workflow, job_name);
        assert!(
            minimum_job_timeout(job).is_some()
                || job
                    .lines()
                    .any(|line| { line.trim().starts_with("uses: ./.github/workflows/") }),
            "{job_name} must own a positive deadline or delegate to a local workflow"
        );
    }

    for (job_name, timeout_minutes) in expected_timeouts {
        let job = job_block(&workflow, job_name);
        if timeout_minutes == 0 {
            // A reusable call declares no cap of its own; the called workflow's
            // jobs own them. Read whichever workflow this job actually calls,
            // so a second delegating job cannot be checked against the first
            // one's file (issue #1017 added `base`).
            let called = job
                .lines()
                .find_map(|line| line.trim().strip_prefix("uses: ./"))
                .unwrap_or_else(|| {
                    panic!("{job_name} is listed as delegating but calls no reusable workflow")
                })
                .trim();
            let reusable =
                fs::read_to_string(format!("{}/../{called}", env!("CARGO_MANIFEST_DIR")))
                    .unwrap_or_else(|error| panic!("read {called}: {error}"));
            for inner_job in workflow_job_names(&reusable) {
                assert!(
                    minimum_job_timeout(job_block(&reusable, inner_job)).is_some()
                        || job_block(&reusable, inner_job).contains("uses: ./.github/workflows/"),
                    "{inner_job} should declare an explicit timeout or delegate"
                );
            }
            continue;
        }
        let expected = format!("    timeout-minutes: {timeout_minutes}\n");
        assert!(
            job.contains(&expected),
            "{job_name} should declare {expected:?}"
        );
    }
}

/// Issue #479: the docs hub links to a Rust API reference at `/docs/api/`. The
/// deploy-pages job generates it with `cargo doc` and copies it into the Pages
/// artifact — and the copy must run *after* stamping (rustdoc HTML carries no
/// version placeholders, so copying post-stamp keeps the large generated tree
/// out of the placeholder scan).
#[test]
fn pages_deploy_generates_api_docs_and_copies_them_after_stamping() {
    let workflow = pages_artifact_workflow();
    let deploy = job_block(&workflow, "build");

    assert!(
        deploy.contains("bash scripts/install-rust-script.sh"),
        "the Pages build should install rust-script with the retry wrapper"
    );
    assert!(
        deploy.contains("bash scripts/build-rust-api-docs.sh"),
        "deploy-pages should invoke the API-docs builder"
    );
    assert!(
        deploy.contains("cp -R target/doc/. js/docs/api/"),
        "deploy-pages should copy the generated docs into js/docs/api/"
    );

    let stamp_pos = deploy
        .find("Stamp GitHub Pages artifact")
        .expect("deploy-pages should stamp the Pages artifact");
    let copy_pos = deploy
        .find("Copy Rust API docs into the Pages artifact")
        .expect("deploy-pages should copy the API docs");
    assert!(
        stamp_pos < copy_pos,
        "the API-docs copy must run after the stamp step so rustdoc HTML is not scanned for placeholders"
    );

    // cargo doc emits the crate root under target/doc/formal_ai/ (lib name
    // formal_ai); a redirect at the doc root keeps /docs/api/ from 404ing.
    let docs_script = std::fs::read_to_string(format!(
        "{}/../scripts/build-rust-api-docs.sh",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("build-rust-api-docs.sh");
    assert!(
        docs_script.contains("cargo doc --no-deps --lib"),
        "the API-docs builder should run cargo doc"
    );
    assert!(
        docs_script.contains("url=formal_ai/index.html"),
        "a redirect should point /docs/api/ at the crate root"
    );
}
