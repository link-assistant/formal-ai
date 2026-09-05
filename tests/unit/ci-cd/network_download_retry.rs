//! Regression coverage for issue #1076 (D19): a dropped connection must not
//! fail a build.
//!
//! `Agentic CLI Matrix` run 33967170904 went red on a commit that changed no
//! shell script -- the 345 MB VS Code tarball its `opencode-vscode` leg
//! installs stopped arriving mid-transfer. The rules here hold every network
//! download in the repository to a retry, and sweep for one that was added
//! without it. The full reconstruction is in
//! `dev/log/issues/1076/pulls/1077/README.md` §4.3.

use std::{fs, path::Path};

/// Reads a repository file relative to the crate root.
fn repository_file(path: &str) -> String {
    let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    fs::read_to_string(&full).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

/// D19. `Agentic CLI Matrix` run 33967170904 failed on a commit that changed no
/// shell script: the 345 MB VS Code tarball the `opencode-vscode` leg installs
/// stopped arriving mid-transfer, and the leg reported
///
/// ```text
/// curl: (18) transfer closed with 344439862 bytes remaining to read
/// !! downloading VS Code 1.135.0 failed
/// ```
///
/// A dropped connection is not a defect in the change under test, so a build
/// that goes red for it is a false positive of exactly the kind issue #1076
/// asks to remove. Every network install in this repository therefore retries,
/// and `--retry` alone does not do it: measured against curl 8.20.0 by
/// `experiments/issue-1076/repro-curl-truncated-download.sh`, a truncated
/// transfer is retried only under `--retry-all-errors` -- with plain `--retry`
/// curl still exits 18 on the first attempt.
///
/// The count is part of the rule. A file that grows a fourth download without
/// the flags fails here rather than on the day the connection drops.
const DOWNLOADS_THAT_MUST_RETRY: &[(&str, usize)] = &[
    ("experiments/agentic_cli_matrix/install_client.sh", 3),
    ("experiments/issue-1021-laravel/run.sh", 1),
    ("scripts/install.sh", 2),
];

/// Remote fetches that carry a retry loop of their own, and so must *not* be
/// rewritten to retry inside curl: each already counts its attempts, and
/// `--retry-all-errors` would only multiply every miss by four.
const FETCHES_WITH_THEIR_OWN_RETRY_LOOP: &[(&str, &str)] = &[
    (
        "scripts/wait-for-pages-deployment.sh",
        "polls until the deployed marker names the expected SHA; a miss is the \
         expected outcome, not a failure",
    ),
    (
        "scripts/verify-ghcr-visibility.sh",
        "`while :` with an attempt counter that distinguishes 401 (private) from \
         000/5xx (retryable), which a curl-level retry would flatten",
    ),
    (
        "experiments/issue-892/fetch-query.sh",
        "40 attempts against a Wikidata endpoint that rate-limits the shared \
         runner IP with HTTP 403",
    ),
    (
        "experiments/issue-1076/repro-curl-truncated-download.sh",
        "the reproduction: its first call omits the flags on purpose, and its \
         second one shows what they change",
    ),
    (
        "experiments/issue-1076/repro-npm-tarball-truncation.sh",
        "the same reproduction aimed at the js template's piped `tar` install, \
         behind js#168; its cases compare the upstream command against retried \
         and file-first ones, all against a local server",
    ),
];

/// Splits a shell script into its `curl` invocations, line continuations
/// joined, skipping comments and the `echo`s that quote a command rather than
/// run one.
fn curl_invocations(script: &str) -> Vec<String> {
    script
        .replace("\\\n", " ")
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| {
            !line.starts_with('#') && !line.starts_with("echo") && line.contains("curl -")
        })
        .collect()
}

/// True for an invocation that pulls an artifact off the public network -- it
/// writes the response to a file or feeds it to an interpreter, and does not
/// name the loopback address every health check in this repository names.
fn is_remote_download(invocation: &str) -> bool {
    let fetches_an_artifact = invocation.contains(" -o ")
        || invocation.contains("| bash")
        || invocation.contains("| sh")
        || invocation.contains("| tar");
    fetches_an_artifact && !invocation.contains("127.0.0.1") && !invocation.contains("localhost")
}

#[test]
fn every_network_install_survives_a_dropped_connection() {
    for (path, expected) in DOWNLOADS_THAT_MUST_RETRY {
        let invocations = curl_invocations(&repository_file(path));
        assert_eq!(
            invocations.len(),
            *expected,
            "{path} has {} curl invocations, not the {expected} this rule was written \
             against; if a download was added, give it the retry flags and update the count",
            invocations.len()
        );

        for invocation in &invocations {
            assert!(
                invocation.contains("--retry ") && invocation.contains("--retry-all-errors"),
                "{path}: `{invocation}` downloads over the network without retrying. One \
                 truncated transfer then fails the build for a reason the change under \
                 test did not cause (run 33967170904). Pass `--retry N --retry-delay N \
                 --retry-all-errors`; plain `--retry` does not cover curl exit 18."
            );
        }
    }
}

#[test]
fn no_remote_download_is_left_out_of_that_rule() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut swept = 0;

    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            name != "target" && name != "node_modules" && name != ".git"
        })
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "sh") {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(script) = fs::read_to_string(path) else {
            continue;
        };

        for invocation in curl_invocations(&script) {
            if !is_remote_download(&invocation) {
                continue;
            }
            swept += 1;
            let listed = DOWNLOADS_THAT_MUST_RETRY
                .iter()
                .any(|(known, _)| *known == relative)
                || FETCHES_WITH_THEIR_OWN_RETRY_LOOP
                    .iter()
                    .any(|(known, _)| *known == relative);
            assert!(
                listed,
                "{relative}: `{invocation}` downloads from the network, and this rule has \
                 never seen it. Either give it `--retry N --retry-delay N \
                 --retry-all-errors` and list it in DOWNLOADS_THAT_MUST_RETRY, or -- if it \
                 already sits in a retry loop of its own -- say so in \
                 FETCHES_WITH_THEIR_OWN_RETRY_LOOP."
            );
        }
    }

    assert!(
        swept >= 10,
        "expected to sweep every remote download in the repository's shell scripts, swept {swept}"
    );
}
