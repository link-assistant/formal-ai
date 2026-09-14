//! Regression gates for the link checker's remaining false positive.
//!
//! Run 34134986294 failed `Broken Link Checker` on one link:
//!
//! ```text
//! [ERROR] https://allenai.org/data/arc (at 226:27)
//!         | Network error: Connection reset by peer (os error 104)
//! ```
//!
//! The page answers 200 on every attempt from anywhere else, and the same
//! branch had passed six minutes earlier. Issue #1045 met this symptom on four
//! links and answered it with `--max-retries 6 --retry-wait-time 2`, reasoning
//! that a reset carries no status code, so retrying is the only lever left.
//!
//! The retries never ran. lychee reported the failure 1.5 s into the step, and
//! six retries with a doubling wait cannot fit in 1.5 s. lychee's own source
//! says why -- `lychee-lib/src/retry.rs`:
//!
//! ```text
//! } else if self.is_connect() {
//!     false
//!
//! fn should_retry_io(error: &io::Error) -> bool {
//!     matches!(error.kind(),
//!         io::ErrorKind::ConnectionReset | ConnectionAborted | TimedOut)
//! }
//! ```
//!
//! A reset during connect or the TLS handshake takes the first branch and is
//! never retried, while the identical `ConnectionReset` reaching the body path
//! is listed as retryable. `--max-retries` is inert for the exact class of
//! failure it was raised for, at any value.
//!
//! The invariant pinned here: **the build is failed by a link, never by a
//! moment.** A failure that never got a status code is asked again, by this
//! repository, outside lychee -- and a failure that did get one is not, because
//! 404 is an answer.

use std::fs;

fn workflow() -> String {
    super::repository_file(".github/workflows/links.yml")
}

fn recheck_script() -> String {
    super::repository_file("scripts/recheck-broken-links.mjs")
}

/// Read a lychee argument out of the workflow, without its quotes.
fn lychee_arg(name: &str) -> String {
    workflow()
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix(&format!("--{name} "))
                .map(|value| value.trim().trim_matches('\'').to_string())
        })
        .unwrap_or_else(|| panic!("the link checker sets --{name}"))
}

/// A reset has to be re-checked by something that will actually retry it.
#[test]
fn a_failure_with_no_status_code_is_asked_again_outside_lychee() {
    let workflow = workflow();

    assert!(
        workflow.contains("node scripts/recheck-broken-links.mjs"),
        "lychee refuses to retry a connect-phase reset -- `is_connect()` \
         short-circuits `should_retry` to false before `should_retry_io` is \
         ever consulted -- so `--max-retries` cannot answer run 34134986294. \
         The retry has to happen outside it."
    );

    let recheck = workflow
        .find("node scripts/recheck-broken-links.mjs")
        .expect("the re-check step runs the script");
    let fail = workflow
        .find("Broken live links were detected")
        .expect("the workflow has a step that fails the build");

    assert!(
        recheck < fail,
        "a re-check that runs after the build has already been failed is not \
         a re-check"
    );
}

/// The verdict of the re-check is what decides the build.
///
/// Both of the steps that act on a broken link have to consult it, or the run
/// still reports an error for a link that has since answered: before this, the
/// Web Archive step annotated `https://allenai.org/data/arc` with a 2024
/// snapshot as a suggested replacement for a page that was serving fine.
#[test]
fn a_recovered_link_fails_nothing_and_suggests_nothing() {
    let workflow = workflow();

    let guarded = workflow
        .matches("steps.recheck.outputs.all_recovered != 'true'")
        .count();

    assert!(
        guarded >= 2,
        "both the Web Archive step and the failing step have to be gated on \
         the re-check, otherwise a link that answered is still annotated or \
         still fails; found {guarded} of 2"
    );
}

/// An unset output must read as "not recovered".
///
/// The comparison is against `'true'`, never against `'false'`: a step that
/// was skipped, or crashed before writing its outputs, leaves the value empty,
/// and `'' != 'true'` fails the build while `'' == 'false'` would pass it. A
/// missing verdict is not an acquittal.
#[test]
fn a_missing_verdict_does_not_clear_a_link() {
    let workflow = workflow();

    assert!(
        !workflow.contains("all_recovered == 'false'"),
        "comparing against 'false' passes the build whenever the re-check \
         produced no verdict at all"
    );

    let script = recheck_script();
    assert!(
        script.contains("setOutput('all_recovered', 'false')"),
        "every path that cannot prove a recovery has to say so explicitly"
    );
}

/// Two components deciding what "broken" means is how a link passes one gate
/// and fails the next. The re-check reads lychee's own accept list.
#[test]
fn the_recheck_accepts_exactly_what_lychee_accepts() {
    let accept = lychee_arg("accept");
    let script = recheck_script();

    assert!(
        script.contains(&format!("'{accept}'")),
        "the re-check's ACCEPTED_STATUS has drifted from the workflow's \
         --accept {accept}; the two have to name the same statuses or a link \
         resolves for one of them and not the other"
    );
    assert!(
        !accept.contains("404"),
        "a missing page is a missing page on the re-check too"
    );
}

/// A host that rate-limits or blocks by user agent has to see the same client
/// twice, or the re-check is asking a different question than the one lychee
/// asked.
#[test]
fn the_recheck_introduces_itself_the_way_lychee_did() {
    let user_agent = lychee_arg("user-agent");
    let script = recheck_script();

    assert!(
        script.contains(&format!("'{user_agent}'")),
        "the re-check's USER_AGENT has drifted from the workflow's \
         --user-agent {user_agent}"
    );
}

/// The parser and the re-check are both tested by `node --test`, and both test
/// files have to be named there: a suite that nothing runs is not coverage.
#[test]
fn both_link_checker_suites_run_before_the_links_are_checked() {
    let workflow = workflow();

    for suite in [
        "scripts/check-web-archive.test.mjs",
        "scripts/recheck-broken-links.test.mjs",
    ] {
        assert!(
            workflow.contains("node --test") && workflow.contains(suite),
            "{suite} is not run by the workflow"
        );
        assert!(
            fs::metadata(format!("{}/{suite}", env!("CARGO_MANIFEST_DIR"))).is_ok(),
            "{suite} is named by the workflow but does not exist"
        );
    }

    let tests = workflow.find("node --test").expect("the suites run");
    let lychee = workflow
        .find("lycheeverse/lychee-action")
        .expect("lychee runs");
    assert!(
        tests < lychee,
        "a parser bug should fail in seconds, before the network work"
    );
}

/// A change to either script has to re-run the workflow that depends on it.
#[test]
fn the_workflow_watches_the_scripts_it_runs() {
    let workflow = workflow();

    for script in [
        "scripts/check-web-archive.mjs",
        "scripts/check-web-archive.test.mjs",
        "scripts/recheck-broken-links.mjs",
        "scripts/recheck-broken-links.test.mjs",
    ] {
        assert_eq!(
            workflow.matches(&format!("- '{script}'")).count(),
            2,
            "{script} has to appear in both the push and the pull_request \
             path filters, or a change to it goes unchecked on one of them"
        );
    }
}

/// The re-check has to be bounded, because it runs inside a job cap that
/// reports its own expiry as `cancelled` rather than as a failure (issue #977).
#[test]
fn the_recheck_cannot_outlast_the_job_that_contains_it() {
    let script = recheck_script();

    assert!(
        script.contains("RECHECK_BUDGET_SECONDS"),
        "a re-check with no total budget can spend the job's whole cap on \
         however many links happened to fail"
    );
    assert!(
        script.contains("budgetMs = 180000"),
        "the default budget is what runs in CI; a change to it should be a \
         change to this test too"
    );

    let cap: u64 = workflow()
        .lines()
        .find_map(|line| line.trim().strip_prefix("timeout-minutes: "))
        .and_then(|value| value.trim().parse().ok())
        .expect("the link-checker job sets a cap");
    assert!(
        cap * 60 > 180 * 2,
        "the job cap ({cap} min) leaves no room for the re-check budget plus \
         the link check it follows"
    );
}
