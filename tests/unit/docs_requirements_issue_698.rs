//! Issue #698 case-study traceability: the collected data must stay in the
//! repository and stay linked to the requirements it proves.

use std::fs;
use std::path::Path;

#[test]
fn issue_698_case_study_and_external_benchmark_contract_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };

    let readme = read("docs/case-studies/issue-698/README.md");
    for expected in [
        "unmodified upstream",
        "suite=gsm8k passed=2 failed=18 total=20",
        "benchmark_unavailable",
        "Timeline",
        "media.githubusercontent.com",
    ] {
        assert!(readme.contains(expected), "README missing {expected}");
    }

    let requirements = read("docs/case-studies/issue-698/requirements.md");
    for id in [
        "R698-01", "R698-02", "R698-03", "R698-04", "R698-05", "R698-06", "R698-07", "R698-08",
        "R698-09", "R698-10", "R698-11", "R698-12",
    ] {
        assert!(requirements.contains(id), "requirements missing {id}");
    }

    let plans = read("docs/case-studies/issue-698/solution-plans.md");
    for expected in ["R698-01", "R698-06", "Rejected.", "Verified by."] {
        assert!(plans.contains(expected), "solution plans missing {expected}");
    }

    // The survey of existing harnesses the issue asks for.
    let survey = read("docs/case-studies/issue-698/survey.md");
    for expected in [
        "lm-evaluation-harness",
        "simple-evals",
        "SWE-bench",
        "EditEval",
        "coding-modification-suite.lino",
    ] {
        assert!(survey.contains(expected), "survey missing {expected}");
    }

    // The raw evidence behind the published numbers is committed.
    for evidence in [
        "docs/case-studies/issue-698/raw-data/all-suites-first-run.log",
        "docs/case-studies/issue-698/raw-data/acceptance-ignored-test.log",
        "docs/case-studies/issue-698/raw-data/humaneval-first-run.log",
    ] {
        let log = read(evidence);
        assert!(
            log.contains("total=20"),
            "{evidence} must show a real 20-case run"
        );
    }
}
