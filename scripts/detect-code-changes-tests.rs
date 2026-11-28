use super::{
    ChangeFlags, LAYER_CLASSIFIER, LAYERED_WORKFLOW, automatic_release_pending, classify_changes,
    comparison_for_event, decode_git_paths, is_documentation_path, release_pending,
};

#[test]
fn failed_git_is_not_a_negative_observation() {
    use std::{
        fs,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };
    let repo = std::env::temp_dir().join(format!(
        "formal-ai-git-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fs::create_dir(&repo).unwrap();
    let _cleanup = Cleanup(repo.clone());
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .current_dir(&repo)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    assert!(super::get_changed_files_in_repo(&repo, "push", None).is_err());
    git(&["init", "--quiet"]);
    fs::write(repo.join("source.rs"), "fn main() {}\n").unwrap();
    git(&["add", "source.rs"]);
    git(&["commit", "--quiet", "-m", "initial\n\nparent forged header"]);
    assert_eq!(
        super::get_changed_files_in_repo(&repo, "push", None).unwrap(),
        ["source.rs"]
    );
    assert!(super::get_changed_files_in_repo(&repo, "pull_request", None).is_err());
    let parent = git(&["rev-parse", "HEAD"]);
    git(&["commit", "--quiet", "--allow-empty", "-m", "empty"]);
    assert!(super::get_changed_files_in_repo(&repo, "pull_request", None).is_err());
    assert!(
        super::get_changed_files_in_repo(&repo, "push", None)
            .unwrap()
            .is_empty()
    );
    assert!(super::get_changed_files_in_repo(&repo, "push", Some(&"a".repeat(40))).is_err());
    let object = format!(".git/objects/{}/{}", &parent[..2], &parent[2..]);
    fs::remove_file(repo.join(object)).unwrap();
    assert!(super::get_changed_files_in_repo(&repo, "push", None).is_err());
}

#[test]
fn git_path_records_preserve_literal_unicode_and_linebreaks() {
    let paths = decode_git_paths("docs/中文.md\0docs/line\nbreak.md\0");
    assert_eq!(paths, vec!["docs/中文.md", "docs/line\nbreak.md"]);
    assert!(!classify_changes(&paths).any_code_changed);
    assert!(!automatic_release_pending(
        "push",
        "refs/heads/main",
        &["pending.md".to_string()],
        &paths,
    ));
    assert_eq!(
        decode_git_paths(" docs/space.md \0"),
        vec![" docs/space.md "]
    );
}

#[test]
fn documentation_only_push_never_releases_a_waiting_fragment() {
    let waiting = vec!["pending.md".to_string()];
    let docs = vec![
        "docs/guide.md".to_string(),
        "changelog.d/pending.md".to_string(),
    ];
    assert!(!automatic_release_pending(
        "push",
        "refs/heads/main",
        &waiting,
        &docs
    ));
    let mixed = vec!["docs/guide.md".to_string(), "rust/src/lib.rs".to_string()];
    assert!(automatic_release_pending(
        "push",
        "refs/heads/main",
        &waiting,
        &mixed
    ));
    assert!(is_documentation_path("README.md"));
    assert!(is_documentation_path(".md"));
    assert!(!is_documentation_path("README.MD"));
    assert!(is_documentation_path("rust/README.md"));
    assert!(!is_documentation_path("data/seed/runtime.md"));
    assert!(classify_changes(&["data/seed/runtime.md".to_string()]).any_code_changed);
    assert!(classify_changes(&["data/seed/RUNTIME.MD".to_string()]).any_code_changed);
}

#[test]
fn a_waiting_fragment_on_main_makes_the_release_pending() {
    let fragments = |names: &[&str]| {
        names
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>()
    };
    let waiting = fragments(&["README.md", "20260926_215107_issue-1149-coverage-budget.md"]);
    assert!(release_pending("push", "refs/heads/main", &waiting));
    assert!(!release_pending(
        "push",
        "refs/heads/main",
        &fragments(&["README.md"])
    ));
    assert!(!release_pending(
        "pull_request",
        "refs/heads/main",
        &waiting
    ));
    assert!(!release_pending("push", "refs/heads/feature", &waiting));
}

#[test]
fn pull_requests_compare_the_complete_base_to_head_range() {
    assert_eq!(
        comparison_for_event("pull_request", true, None),
        (
            "HEAD^".to_string(),
            "HEAD^2".to_string(),
            "complete pull request diff"
        )
    );
}

#[test]
fn pushes_compare_every_commit_in_the_event() {
    assert_eq!(
        comparison_for_event("push", true, Some("before-sha")),
        (
            "before-sha".to_string(),
            "HEAD".to_string(),
            "complete push diff"
        ),
        "a pushed merge commit must not be mistaken for GitHub's synthetic PR merge"
    );
}

#[test]
fn missing_push_before_sha_falls_back_to_the_previous_commit() {
    assert_eq!(
        comparison_for_event("push", false, Some("000000")),
        (
            "HEAD^".to_string(),
            "HEAD".to_string(),
            "previous commit diff"
        )
    );
}

#[test]
fn ignored_only_changes_set_no_flags_for_pushes_or_pull_requests() {
    for event_name in ["push", "pull_request"] {
        for path in [
            "experiments/repro.rs",
            "experiments/README.md",
            "experiments/repro.mjs",
            "dev/log/run.rs",
            "docs/case-studies/issue-846/repro.rs",
            ".gitkeep",
            "rust/src/.gitkeep",
        ] {
            assert_eq!(
                classify_changes(&[path.to_string()]),
                ChangeFlags::default(),
                "{path} must be ignored on {event_name}"
            );
        }
    }
}

#[test]
fn shipping_changes_still_set_their_flags() {
    assert_eq!(
        classify_changes(&[
            "src/lib.rs".to_string(),
            "guide.md".to_string(),
            ".github/workflows/release.yml".to_string(),
        ]),
        ChangeFlags {
            rs_changed: true,
            docs_changed: true,
            workflow_changed: true,
            pipeline_changed: true,
            any_code_changed: true,
            ..ChangeFlags::default()
        }
    );
}

/// The flag that decides whether a push pays for the heavy jobs.
///
/// Issue #1107: `workflow_changed` is true for every file under
/// `.github/workflows/`, so editing a scheduled benchmark unlocked the
/// macOS archive, the Docker check, six box images and a 25-minute agent
/// end-to-end run. `pipeline_changed` is true only for the files those
/// jobs actually execute.
#[test]
fn an_unrelated_workflow_does_not_unlock_the_pipeline_heavy_jobs() {
    let unrelated = classify_changes(&[".github/workflows/external-benchmarks.yml".to_string()]);
    assert!(
        unrelated.workflow_changed,
        "the workflow gates still have to run for it"
    );
    assert!(
        !unrelated.pipeline_changed,
        "a scheduled benchmark's workflow does not change what the pipeline jobs do"
    );
    assert!(
        !unrelated.any_code_changed,
        "a workflow is not the compiled code the heavy jobs test; counting it here is what \
         made every workflow edit pay for the macOS archive, the Docker check, six box \
         images and a 25-minute agent end-to-end run"
    );

    for path in [
        ".github/workflows/release.yml",
        ".github/actions/author-with-formal-ai/action.yml",
        "scripts/author-change-with-formal-ai.sh",
    ] {
        assert!(
            classify_changes(&[path.to_string()]).pipeline_changed,
            "{path} changes what a pipeline job runs"
        );
    }
}

#[test]
fn agentic_source_changes_request_the_four_client_replay() {
    for path in [
        "rust/src/agentic_coding/capability_router.rs",
        "rust/src/agentic_coding/planner.rs",
        "rust/src/agentic_coding/new_router.rs",
    ] {
        assert!(
            classify_changes(&[path.to_string()]).agentic_routing_changed,
            "{path} can change client-specific routing"
        );
    }

    for path in [
        "rust/src/coding/composition.rs",
        "docs/agentic_coding/README.md",
    ] {
        assert!(
            !classify_changes(&[path.to_string()]).agentic_routing_changed,
            "{path} is outside the agentic routing source boundary"
        );
    }
}

/// The js -> ts -> rust cycle's tier matrix (issue #1138, plan 16 L4).
///
/// Cumulative downward, never upward: the committed ts tree is a
/// function of the js tree and the dogfood gate binds all three, so a
/// change wakes every tier below its folder and none above it. That
/// asymmetry is the instruction's saving -- a rust iteration lets the
/// js and ts tiers stand on their last green runs.
#[test]
fn tier_gates_are_cumulative_downward_and_never_upward() {
    let rust_only = classify_changes(&["rust/src/lib.rs".to_string()]);
    assert!(rust_only.rust_changed, "a rust change wakes the rust tier");
    assert!(
        !rust_only.js_changed && !rust_only.ts_changed,
        "a rust change lets the js and ts tiers carry their last green runs forward"
    );

    for path in [
        "data/meta/server-routes.lino",
        "scripts/check-server-parity.mjs",
        "rust/tests/fixtures/server-parity/requests.lino",
    ] {
        let flags = classify_changes(&[path.to_string()]);
        assert!(
            flags.js_changed && flags.ts_changed && flags.rust_changed,
            "{path} is a server-parity input, so every tier re-runs"
        );
    }

    let js_only = classify_changes(&["js/app.js".to_string()]);
    assert!(
        js_only.js_changed && js_only.ts_changed && js_only.rust_changed,
        "a js change wakes every tier below it: the ts tree is a function of \
         the js tree, so a sleeping ts tier would carry a stale-green tier \
         past the plan 16 L3 regeneration contract"
    );

    let ts_only = classify_changes(&["ts/app.ts".to_string()]);
    assert!(
        ts_only.ts_changed && ts_only.rust_changed,
        "a ts change still reaches the tier that regenerates it"
    );
    assert!(
        !ts_only.js_changed,
        "the cycle never ascends: a ts change does not reexecute the js tier"
    );

    let seed = classify_changes(&["data/seed/tools.lino".to_string()]);
    assert!(
        seed.js_changed && seed.ts_changed && seed.rust_changed,
        "a seed consumed by all three roots wakes every tier (plan 16 risk 3)"
    );

    let docs = classify_changes(&["README.md".to_string()]);
    assert!(
        !docs.js_changed && !docs.ts_changed && !docs.rust_changed,
        "a docs-only change wakes no tier"
    );
}

/// The gating files re-run the tiers they gate, so the cycle's own
/// rules can never go stale on a green branch.
#[test]
fn the_gating_files_rerun_the_tiers_they_gate() {
    for path in [
        LAYERED_WORKFLOW,
        LAYER_CLASSIFIER,
        "scripts/check-js-rust-translation.mjs",
    ] {
        let flags = classify_changes(&[path.to_string()]);
        assert!(
            flags.js_changed && flags.ts_changed && flags.rust_changed,
            "{path} edits the tier rules, so every tier must re-run"
        );
    }

    let binary_action =
        classify_changes(&[".github/actions/formal-ai-binary/action.yml".to_string()]);
    assert!(
        binary_action.rust_changed,
        "the binary action changes what the rust tier executes"
    );
    assert!(
        !binary_action.js_changed && !binary_action.ts_changed,
        "the binary action feeds only the rust tier"
    );
}

/// A JavaScript suite is the js tier's own test, so editing one re-runs
/// that tier (issue #1017: a pinning test that never ran is a false
/// negative) without regenerating the ts tree, which does not consume it.
#[test]
fn a_javascript_suite_change_reruns_the_tier_that_executes_it() {
    let suite = classify_changes(&["rust/tests/web/translate-es.test.mjs".to_string()]);
    assert!(suite.js_changed, "the js tier runs the suite that changed");
    assert!(
        !suite.ts_changed,
        "the ts tree is a function of js/, not of its tests"
    );
}

#[test]
fn presentation_source_dependencies_rerun_all_translation_tiers() {
    for path in [
        "scripts/lib/rust-projection-presentation.mjs",
        "scripts/self-translation/lexer.mjs",
        "scripts/check-readable-code.mjs",
    ] {
        let flags = classify_changes(&[path.to_string()]);
        assert!(flags.js_changed && flags.ts_changed && flags.rust_changed);
    }
    let unrelated = classify_changes(&["docs/presentation.md".to_string()]);
    assert!(!unrelated.js_changed && !unrelated.ts_changed && !unrelated.rust_changed);
}
