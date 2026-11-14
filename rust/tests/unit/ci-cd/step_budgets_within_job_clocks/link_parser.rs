use super::super::workflow_fixtures::{job_block, unwrapped, workflow_step_block};
use super::repository_file;

/// Each trigger owns parser coverage; both execution jobs retain the real tests.
pub(super) fn assert_contract() {
    let links = repository_file(".github/workflows/links.yml");

    let live = job_block(&links, "link-checker");
    let mandatory = job_block(&links, "link-parser-checks");
    let parser_step = "Test Web Archive report parser";
    let live_parser = workflow_step_block(live, parser_step);
    let mandatory_parser = workflow_step_block(mandatory, parser_step);
    let tests = [
        "scripts/check-web-archive.test.mjs",
        "scripts/recheck-broken-links.test.mjs",
        "scripts/recheck-transient-http.test.mjs",
    ];
    for step in [live_parser, mandatory_parser] {
        assert!(
            step.contains("node --test"),
            "the named parser step must execute unit tests"
        );
        for test in tests {
            assert!(step.contains(test), "the parser step must execute {test}");
        }
    }
    assert!(
        !mandatory
            .lines()
            .any(|line| line.starts_with("    if:") || line.starts_with("    needs:")),
        "the cheap parser job must remain unconditional and independent"
    );
    let node_test = live
        .find(live_parser)
        .expect("the live parser step belongs to the live job");
    let lychee = live
        .find("lycheeverse/lychee-action")
        .expect("the live job must retain lychee");
    assert!(
        node_test < lychee,
        "the live parser tests must run before lychee"
    );
    let triggers = links
        .split_once("\njobs:\n")
        .expect("the workflow has jobs")
        .0;
    for event in ["push", "pull_request"] {
        let trigger = job_block(triggers, event);
        let paths = trigger
            .split_once("    paths:\n")
            .expect("each trigger owns a paths filter")
            .1;
        let paths: Vec<_> = paths
            .lines()
            .take_while(|line| line.trim().is_empty() || line.starts_with("      "))
            .collect();
        for test in tests {
            let entry = format!("      - '{test}'");
            assert!(
                paths.contains(&entry.as_str()),
                "{event} paths must independently retain {test}"
            );
        }
    }
    assert!(
        unwrapped(live).contains("!cancelled() && steps.lychee.outputs.exit_code != 0"),
        "a cancelled link check must not append a `broken links` error to a run \
         that never finished checking them"
    );
    assert!(
        !links.contains("if: ${{ always()"),
        "the guard above is only load-bearing while nothing in links.yml has \
         widened back to `always()`"
    );
    assert!(
        !repository_file("scripts/check-web-archive.test.mjs").is_empty(),
        "the parser test file must exist"
    );
}
