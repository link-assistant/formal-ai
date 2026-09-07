//! Regression coverage for issue #1079: the false positives, false negatives,
//! warnings and errors found by auditing every CI/CD run at `main` head
//! `f971b8205`, and by comparing this repository's whole file tree against the
//! five `link-foundation/*-ai-driven-development-pipeline-template`
//! repositories.
//!
//! Two of the defects were gates that could not fail, which is the worst kind
//! of green check -- it is indistinguishable from a gate that passed.
//!
//! * **D2.** `scripts/check-rust-dependencies.sh` ran `cargo audit` with no
//!   `--deny warnings`. `cargo audit` classifies `unmaintained`, `unsound` and
//!   `yanked` as warnings, and a warning does not move the exit status: it
//!   prints `warning: N allowed warnings found` and exits 0. The gate was green
//!   for the whole time `Cargo.lock` pinned `chacha20 0.10.1`, a release its
//!   own authors had yanked. A yanked crate carries no advisory ID, so it
//!   cannot be silenced by an `ignore` entry either -- the only way past the
//!   line is to change the lockfile.
//! * **D3.** `.github/zizmor.yml` has declared `'*': hash-pin` since issue
//!   #1076, and that policy had never once been applied to a container image.
//!   It configures `unpinned-uses`, which reads *action* references only;
//!   images belong to the separate `unpinned-images` audit, which zizmor
//!   classifies as Pedantic, and the job ran `--persona regular`. So
//!   `docker://rhysd/actionlint:1.7.12` was a mutable third-party tag executing
//!   with the repository checked out, under a written policy forbidding exactly
//!   that.
//! * **D6.** The zizmor job left `version:` at its default while two comments
//!   told a maintainer to reproduce with `zizmor==1.30.0`. `zizmor-action`
//!   resolves versions from a static table shipped inside the action, and
//!   v0.6.2's table stops at 1.29.0 -- its `latest` row is literally the same
//!   digest as its `1.29.0` row. "Reproduce locally with X" is only true when
//!   CI runs X.
//!
//! D3 and D6 reproduce unchanged in all five templates and are filed against
//! each; the reports and the filed issue URLs are in
//! `dev/log/issues/1079/pulls/1080/upstream-reports/README.md`. The full
//! reconstruction is in `dev/log/issues/1079/pulls/1080/README.md`.

use std::fs;
use std::path::Path;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// Strip a YAML line's trailing comment, keeping `#` that a quoted scalar owns.
///
/// Only the unquoted case matters here: every image reference in these
/// workflows is a plain scalar, and the marker this sweep looks for lives in
/// the comment it removes -- so the comment is read separately rather than
/// thrown away.
fn code_and_comment(line: &str) -> (&str, &str) {
    match line.split_once(" #") {
        Some((code, comment)) => (code, comment),
        None => (line, ""),
    }
}

/// Every YAML file anywhere under `.github/`, in a stable order.
///
/// The sweeps below read the directory rather than a hand-written list: a
/// defect that only holds for the files someone remembered to enumerate is
/// not an invariant, and every one of these defects was originally introduced
/// by a file nobody thought to look at.
fn github_yaml_files() -> Vec<std::path::PathBuf> {
    let mut stack = vec![std::path::PathBuf::from(format!(
        "{}/.github",
        env!("CARGO_MANIFEST_DIR")
    ))];
    let mut files = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("readable .github directory") {
            let path = entry.expect("directory entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Every container image `.github/` runs, as `(file, line number, reference,
/// trailing comment)`.
///
/// Three spellings reach a registry: a `docker://` step, a job-level
/// `container:`, and a service's `image:`. zizmor's `unpinned-images` audit
/// covers all three, so this sweep does too -- a fix applied only to the
/// spelling that happened to be in the tree is not a fix.
fn image_references() -> Vec<(String, usize, String, String)> {
    let mut found = Vec::new();

    for path in github_yaml_files() {
        let name = path
            .strip_prefix(env!("CARGO_MANIFEST_DIR"))
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let body = fs::read_to_string(&path).expect("readable workflow");
        for (index, line) in body.replace("\r\n", "\n").lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            let (code, comment) = code_and_comment(trimmed);
            let code = code.trim();
            let reference = if let Some(rest) = code.strip_prefix("- uses: docker://") {
                rest
            } else if let Some(rest) = code.strip_prefix("uses: docker://") {
                rest
            } else if let Some(rest) = code.strip_prefix("container:") {
                rest.trim()
            } else if let Some(rest) = code.strip_prefix("image:") {
                rest.trim()
            } else {
                continue;
            };
            if reference.is_empty() || reference.starts_with("${{") {
                continue;
            }
            found.push((
                name.clone(),
                index + 1,
                reference.to_owned(),
                comment.trim().to_owned(),
            ));
        }
    }

    found
}

/// D3. The policy `.github/zizmor.yml` writes down, applied to the references
/// it was never applied to.
///
/// An image may be unpinned only by carrying the suppression marker on the very
/// line that pins it -- which is the same fact zizmor's narrow pedantic pass
/// enforces in CI, expressed here so it also fails on a machine with no Docker
/// and no network.
#[test]
fn every_container_image_is_digest_pinned_or_explicitly_excepted() {
    let references = image_references();
    assert!(
        references.len() >= 2,
        "expected to reach at least the actionlint image and the stock Rust \
         container, reached {}",
        references.len()
    );

    let mut excepted = 0_usize;
    for (file, line, reference, comment) in &references {
        if reference.contains("@sha256:") {
            continue;
        }
        assert!(
            comment.contains("zizmor: ignore[unpinned-images]"),
            "{file}:{line}: `{reference}` is a mutable tag executing with this \
             repository checked out, and `.github/zizmor.yml` declares \
             `'*': hash-pin`. Pin it by digest, or -- if a mutable tag is the \
             point, as it is for the stock-image install proof -- say so on the \
             line with `# zizmor: ignore[unpinned-images]` and a comment giving \
             the reason (issue #1079, defect D3)."
        );
        excepted += 1;
    }

    assert!(
        excepted <= 1,
        "{excepted} images opt out of the hash-pin policy. Exactly one is \
         justified -- `stock-rust-install.yml` exists to prove `cargo install` \
         works on the image users actually pull, so pinning its digest would \
         stop it testing that. A second exception means the policy is being \
         worn down rather than applied (issue #1079)."
    );
}

/// The gate above is only as good as its enforcement in CI, where a
/// contributor sees it. A `--persona regular` run cannot report
/// `unpinned-images` at all, so the repository runs a second, narrow pass.
///
/// It is narrow on purpose: the pedantic persona produces 164 findings on this
/// tree, nearly all stylistic, and a gate that cries wolf is a gate people
/// learn to merge past. Filtering to high severity *and* high confidence leaves
/// exactly the class this is about -- 0 reported on the clean tree, 2 when the
/// digest pin and the exception marker are removed. The transcript is
/// `dev/log/issues/1079/pulls/1080/analysis/zizmor-narrow-pedantic-gate.md`.
///
/// There is no narrower way to express it. zizmor 1.29 and 1.30 both reject
/// `rules.<audit>.persona` outright ("unknown field `persona`, expected one of
/// `disable`, `ignore`, `config`, `remap`"), and `remap` rewrites severity,
/// which is not what the persona filter reads.
#[test]
fn a_pedantic_pass_enforces_the_hash_pin_policy_on_images() {
    let audit = repository_file(".github/workflows/workflows.yml");
    let passes: Vec<&str> = audit
        .split("uses: zizmorcore/zizmor-action@")
        .skip(1)
        .collect();
    assert_eq!(
        passes.len(),
        2,
        "workflows.yml must run zizmor twice: `--persona regular` for the \
         findings everyone should see, and a narrow pedantic pass for \
         `unpinned-images`, which the regular persona cannot report at all \
         (issue #1079, defect D3)"
    );

    let regular = passes[0];
    assert!(
        !regular.contains("persona:"),
        "the first zizmor pass must stay on the default persona"
    );
    assert!(
        !regular.contains("min-severity"),
        "the first zizmor pass must not filter by severity -- that hides real \
         findings rather than noisy ones (hive-mind CI/CD best practices, \
         principle 14). Severity is the narrowing device for the *second* pass \
         only, where it substitutes for the per-rule persona override zizmor \
         does not have."
    );

    let pedantic = passes[1];
    for (key, value) in [
        ("persona:", "pedantic"),
        ("min-severity:", "high"),
        ("min-confidence:", "high"),
    ] {
        assert!(
            pedantic.contains(&format!("{key} {value}")),
            "the second zizmor pass must set `{key} {value}`: without the \
             persona it cannot see `unpinned-images`, and without both floors \
             it reports 164 findings instead of the one class it is for \
             (issue #1079)"
        );
    }
}

/// D6. Both passes name the version they run, and it is the version the
/// reproduction comment tells a maintainer to install.
///
/// `zizmor-action@v0.6.2` resolves versions from `support/versions`, a static
/// 37-row table shipped inside the action, and `die`s on anything absent. Its
/// `latest` row carries the same digest as its `1.29.0` row, so leaving
/// `version:` at its default does not track releases -- it freezes, one minor
/// release behind what this file's comment claimed. The measurement is
/// `dev/log/issues/1079/pulls/1080/analysis/zizmor-action-v0.6.2-versions.txt`.
#[test]
fn every_zizmor_pass_pins_the_version_its_comment_documents() {
    let audit = repository_file(".github/workflows/workflows.yml");

    let documented: Vec<&str> = audit
        .lines()
        .filter_map(|line| {
            line.split_once("zizmor==")
                .map(|(_, rest)| rest.split_whitespace().next().unwrap_or(rest))
        })
        .collect();
    assert!(
        !documented.is_empty(),
        "workflows.yml must keep a local reproduction command for the zizmor \
         audit; a finding nobody can reproduce is a finding nobody fixes"
    );

    let pinned: Vec<String> = audit
        .split("uses: zizmorcore/zizmor-action@")
        .skip(1)
        .map(|pass| {
            pass.lines()
                .find_map(|line| line.trim().strip_prefix("version:"))
                .unwrap_or_else(|| {
                    panic!(
                        "every zizmor step must set `version:`. Left at its \
                         default, `zizmor-action@v0.6.2` installs whatever its \
                         own static version table calls `latest` -- 1.29.0, \
                         frozen -- so the version CI runs is a property of the \
                         action rather than of this file (issue #1079, defect \
                         D6)"
                    )
                })
                .trim()
                .to_owned()
        })
        .collect();

    for version in &pinned {
        assert!(
            documented.contains(&version.as_str()),
            "workflows.yml runs zizmor {version} but tells a maintainer to \
             reproduce with {documented:?}. The comment said 1.30.0 while CI \
             ran 1.29.0 and could not have run 1.30.0 -- the action's version \
             table stops there (issue #1079, defect D6)."
        );
    }
}

/// D3, the other half: the same image, pinned twice.
///
/// `workflows.yml` names the actionlint image in two places -- the lint step
/// and the `ShellCheck` canary that proves the lint step can still see inside a
/// `run:` block. If they drift apart, the canary stops proving anything about
/// the image that actually runs, which is a false negative wearing a green
/// check.
#[test]
fn the_actionlint_image_is_pinned_once_and_used_everywhere() {
    let audit = repository_file(".github/workflows/workflows.yml");

    let digests: Vec<&str> = audit
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| line.split_once("rhysd/actionlint@sha256:"))
        .map(|(_, rest)| {
            rest.split(|c: char| !c.is_ascii_alphanumeric())
                .next()
                .unwrap_or(rest)
        })
        .collect();

    assert_eq!(
        digests.len(),
        2,
        "workflows.yml must reference the digest-pinned actionlint image \
         exactly twice -- the lint step and the ShellCheck canary that proves \
         it -- found {}",
        digests.len()
    );
    assert_eq!(
        digests[0], digests[1],
        "the canary lints a different actionlint image than the one that lints \
         these workflows, so it proves nothing about the lint that actually \
         runs (issue #1079)"
    );
    assert_eq!(
        digests[0].len(),
        64,
        "a sha256 digest is 64 hex characters; `{}` is not one",
        digests[0]
    );
}

/// D2. The audit that could not fail.
#[test]
fn the_rust_dependency_audit_treats_a_warning_as_a_finding() {
    let script = repository_file("scripts/check-rust-dependencies.sh");

    let invocation = script
        .lines()
        .find(|line| line.trim_start().starts_with("cargo audit"))
        .expect("check-rust-dependencies.sh must invoke `cargo audit`");
    assert!(
        invocation.contains("--deny warnings"),
        "`{}` exits 0 on `unmaintained`, `unsound` and `yanked` findings -- it \
         prints \"warning: N allowed warnings found\" and returns success. This \
         gate was green for the whole time Cargo.lock pinned the yanked \
         `chacha20 0.10.1`. Add `--deny warnings` (issue #1079, defect D2).",
        invocation.trim()
    );
    assert!(
        !invocation.contains("--no-yanked"),
        "`--no-yanked` turns the yanked-crate check off entirely, which is the \
         one class `[advisories] ignore` cannot silence -- a yanked release \
         carries no advisory ID. Suppressing it here would put the gate back \
         where issue #1079 found it."
    );
}

/// Every ignored advisory carries exactly one proof, and a `blocked-upstream`
/// proof names a report that exists.
///
/// `check-rust-dependencies.sh` re-derives each proof from `cargo tree` on
/// every run, so the property is already enforced where it matters. This test
/// covers the part `cargo tree` cannot: that the *shape* of each entry is one
/// of the two honest forms. An ignore with no proof, or with both, would be
/// caught by the script -- but only on a machine that can resolve the whole
/// dependency graph, and a config defect should fail in the unit suite.
#[test]
fn every_ignored_advisory_carries_exactly_one_proof() {
    let config = repository_file(".cargo/audit.toml");

    // The same reading `scripts/check-rust-dependencies.sh` does: collect every
    // quoted value inside `ignore = [...]`, which TOML allows to be written on
    // one line or spread over several.
    let mut ignored: Vec<String> = Vec::new();
    let mut collecting = false;
    for line in config.lines() {
        if line.trim_start().starts_with("ignore") && line.contains('=') {
            collecting = true;
        }
        if !collecting {
            continue;
        }
        let code = line.split_once('#').map_or(line, |(code, _)| code);
        ignored.extend(
            code.split('"')
                .skip(1)
                .step_by(2)
                .filter(|value| value.starts_with("RUSTSEC-"))
                .map(str::to_owned),
        );
        if code.contains(']') {
            break;
        }
    }

    assert!(
        !ignored.is_empty(),
        ".cargo/audit.toml must keep its `ignore` list parseable by the same \
         reading `scripts/check-rust-dependencies.sh` uses"
    );

    for advisory in &ignored {
        let unreachable = config.contains(&format!("# {advisory} unreachable = \""));
        let blocked = config.contains(&format!("# {advisory} blocked-upstream = \""));

        assert!(
            unreachable || blocked,
            ".cargo/audit.toml ignores {advisory} without a proof line. Write \
             one of:\n  # {advisory} unreachable = \"<crate>@<version>\"\n  \
             # {advisory} blocked-upstream = \"<crate>@<version>\" \
             report = \"<url>\"\nAn advisory that can be neither proven \
             unreachable nor traced to a filed upstream report must be fixed, \
             not ignored (issue #1079)."
        );
        assert!(
            !(unreachable && blocked),
            ".cargo/audit.toml proves {advisory} both ways. An advisory is \
             either unreachable or blocked upstream, never both -- the two \
             forms expire in opposite directions, so an entry claiming both \
             can never expire at all (issue #1079)."
        );

        if blocked {
            let report = config
                .lines()
                .find(|line| line.contains(&format!("# {advisory} blocked-upstream = \"")))
                .and_then(|line| line.split_once("report = \""))
                .and_then(|(_, rest)| rest.split('"').next())
                .unwrap_or("");
            assert!(
                report.starts_with("https://github.com/") && report.contains("/issues/"),
                ".cargo/audit.toml ignores {advisory} as blocked upstream, but \
                 `report = {report:?}` is not a filed report. \"Someone should \
                 fix this upstream\" is not a report; a URL is (issue #1079)."
            );
        }
    }
}

/// The evidence bundle is part of the deliverable, not a by-product of it.
///
/// Issue #1079 asks for the data to be collected into
/// `dev/log/issues/1079/pulls/1080/`, for every defect to be traced to a root
/// cause, and for a report to be filed upstream wherever a template shares the
/// defect. A claim about an upstream repository that nobody can check is worth
/// less than no claim, so each report keeps its reproduction here and the
/// index records where it was filed.
#[test]
fn the_upstream_reports_are_recorded_with_the_issues_they_were_filed_as() {
    let bundle = "dev/log/issues/1079/pulls/1080";
    let index = repository_file(&format!("{bundle}/upstream-reports/README.md"));

    for template in ["rust", "js", "python", "php", "csharp"] {
        assert!(
            index.contains(&format!(
                "{template}-ai-driven-development-pipeline-template"
            )),
            "{bundle}/upstream-reports/README.md must record what the sweep \
             found in the {template} template -- issue #1079 asks for all five \
             to be compared, and \"nothing found\" is a result that has to be \
             written down too"
        );
    }

    // Every report body in the directory is linked from the index, so a report
    // cannot be written, filed and then lost.
    let dir = format!("{}/{bundle}/upstream-reports", env!("CARGO_MANIFEST_DIR"));
    let mut bodies = 0_usize;
    for entry in fs::read_dir(&dir).expect("upstream-reports directory") {
        let path = entry.expect("directory entry").path();
        if path.extension().is_none_or(|extension| extension != "md") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name == "README.md" {
            continue;
        }
        bodies += 1;
        assert!(
            index.contains(&name),
            "{name} is an upstream report the index does not list; add it to \
             {bundle}/upstream-reports/README.md with the issue it was filed as"
        );
    }
    assert!(
        bodies >= 8,
        "expected the eight report bodies this sweep produced -- six against \
         the pipeline templates and two against the Agent CLI (D11) -- found \
         {bodies}"
    );
}

/// `actions/checkout` writes the job's token into `.git/config` as an
/// `http.extraheader`, where it stays for the rest of the job. Every later
/// step -- and every tool any of them shells out to -- can read it, and
/// anything that archives the workspace carries it out of the run. zizmor
/// calls this `artipacked`, and reported it 46 times here before this sweep.
///
/// All five `link-foundation/*-ai-driven-development-pipeline-template`
/// repositories set `persist-credentials: false`; this one did not, at 46 of
/// its 48 checkouts. It is a Low-confidence finding, so neither of the two
/// live gates was ever going to report it -- the default pass floors
/// confidence at medium and the pedantic pass at high. That is exactly why it
/// needs a test: an invariant no gate enforces is a comment.
///
/// Four checkouts keep the credential because a later step in the same job
/// pushes with it. Each says so in a comment, and the cap below is what stops
/// a fifth from being added silently.
///
/// The sweep runs in both directions, because the first pass of it broke two
/// releases: `auto-release` and `manual-release` call
/// `scripts/version-and-commit.rs`, which ends in `git push` and
/// `git push --tags`, and `scripts/git-config.rs` sets only the committer
/// identity, so those two jobs push with exactly the credential the input
/// removes. Dropping a credential a job needs fails at release time, on
/// `main`, in the one workflow no pull request exercises -- so the direction
/// that matters more is the one that checks that a job which pushes still has
/// something to push with.
#[test]
fn every_checkout_drops_its_credential_unless_it_pushes() {
    let mut exceptions = Vec::new();
    let mut swept = 0_usize;

    for path in github_yaml_files() {
        let name = path
            .strip_prefix(env!("CARGO_MANIFEST_DIR"))
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let body = fs::read_to_string(&path).expect("readable workflow");
        let body = body.replace("\r\n", "\n");
        let lines: Vec<&str> = body.lines().collect();

        for (index, line) in lines.iter().enumerate() {
            if !line.contains("actions/checkout@") || line.trim_start().starts_with('#') {
                continue;
            }

            // A `- uses:` line opens the step, so its sibling keys sit one
            // level deeper; a bare `uses:` is already at that level.
            let indent = line.len() - line.trim_start().len();
            let key_indent = if line.trim_start().starts_with("- ") {
                indent + 2
            } else {
                indent
            };

            let mut drops_credential = false;
            for following in &lines[index + 1..] {
                if following.trim().is_empty() {
                    continue;
                }
                let following_indent = following.len() - following.trim_start().len();
                if following_indent < key_indent || following.trim_start().starts_with("- ") {
                    break;
                }
                if following.contains("persist-credentials: false") {
                    drops_credential = true;
                    break;
                }
            }

            if drops_credential {
                swept += 1;
                continue;
            }

            // An exception has to be argued for where it is written, not in a
            // list somewhere else that drifts out of date.
            let reason: String = lines[..index]
                .iter()
                .rev()
                .take_while(|previous| previous.trim_start().starts_with('#'))
                .map(|previous| previous.trim_start().trim_start_matches('#'))
                .collect();
            assert!(
                reason.contains("#1079") && reason.contains("persist-credentials"),
                "{name}:{} keeps its checkout credential without a comment \
                 saying why. Set `persist-credentials: false`, or, if a later \
                 step in this job pushes with it, write the reason directly \
                 above the step and name issue #1079 (issue #1079)",
                index + 1
            );
            exceptions.push(format!("{name}:{}", index + 1));
        }
    }

    assert!(
        swept >= 44,
        "expected the 44 swept checkouts, found {swept}: a checkout that \
         stopped being read by this sweep is a checkout that stopped being \
         checked"
    );
    assert!(
        exceptions.len() <= 4,
        "only the four pushing jobs may keep their checkout credential, found \
         {}: {exceptions:?}",
        exceptions.len()
    );
}

/// Anything that writes to the remote over git -- rather than over the API,
/// which reads its token from the environment instead of from `.git/config`.
const REMOTE_GIT_WRITES: [&str; 3] = [
    "git push",
    "scripts/version-and-commit.rs",
    "peter-evans/create-pull-request",
];

/// The converse of the sweep above, and the more expensive direction to get
/// wrong: a job that drops a credential a later step pushes with fails on
/// `main` at release time, in a workflow no pull request runs.
///
/// Rather than trusting a list of which jobs push, this reads each job body
/// and asks whether anything in it writes to the remote over git. A job that
/// does must keep its checkout credential, because nothing else in this
/// repository supplies one: `scripts/git-config.rs` sets `user.name` and
/// `user.email` and no credential helper, and no workflow configures one.
#[test]
fn every_job_that_pushes_still_has_a_credential_to_push_with() {
    let mut pushing_jobs = Vec::new();

    for path in github_yaml_files() {
        let name = path
            .strip_prefix(env!("CARGO_MANIFEST_DIR"))
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let body = fs::read_to_string(&path).expect("readable workflow");
        let body = body.replace("\r\n", "\n");
        let lines: Vec<&str> = body.lines().collect();

        // Job bodies, split at the two-space job headers under `jobs:`.
        let mut starts: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                line.starts_with("  ")
                    && !line.starts_with("   ")
                    && line.trim_end().ends_with(':')
                    && !line.trim_start().starts_with('#')
            })
            .map(|(index, _)| index)
            .collect();
        starts.push(lines.len());

        for window in starts.windows(2) {
            let (start, end) = (window[0], window[1]);
            let job = &lines[start..end];
            let job_name = lines[start].trim().trim_end_matches(':');

            // A comment explaining why a credential is kept names the push it
            // is kept for, so it must not be read as the push itself.
            let writes = job.iter().any(|line| {
                !line.trim_start().starts_with('#')
                    && REMOTE_GIT_WRITES.iter().any(|write| line.contains(write))
            });
            if !writes {
                continue;
            }

            let checkouts = job
                .iter()
                .filter(|line| {
                    line.contains("actions/checkout@") && !line.trim_start().starts_with('#')
                })
                .count();
            if checkouts == 0 {
                continue;
            }

            pushing_jobs.push(format!("{name}:{job_name}"));
            assert!(
                !job.iter()
                    .any(|line| line.contains("persist-credentials: false")
                        && !line.trim_start().starts_with('#')),
                "{name}: job `{job_name}` writes to the remote over git but its \
                 checkout sets `persist-credentials: false`, which removes the \
                 only credential the push has. Nothing else supplies one -- \
                 `scripts/git-config.rs` sets the committer identity and no \
                 credential helper (issue #1079)"
            );
        }
    }

    assert_eq!(
        pushing_jobs.len(),
        4,
        "expected the four jobs that write to the remote over git, found \
         {pushing_jobs:?}. A new one must keep its checkout credential; one \
         that stopped pushing should drop it"
    );
}

/// Every `experiments/**.sh` path a workflow names, in the order they appear.
///
/// The reference is textual on purpose: a workflow may name a harness in a
/// `run:` block, in an `if:` guard or inside a heredoc, and a sweep that only
/// understood one of those spellings would silently measure less than it
/// claims.
fn experiment_script_references(body: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut start = 0;
    while let Some(index) = body[start..].find("experiments/") {
        let begin = start + index;
        let end = body[begin..]
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '`' || c == ')')
            .map_or(body.len(), |offset| begin + offset);
        if Path::new(&body[begin..end])
            .extension()
            .is_some_and(|extension| extension == "sh")
        {
            found.push(body[begin..end].to_string());
        }
        start = end.max(begin + 1);
    }
    found
}

/// The sibling libraries a script `source`s, resolved against its own
/// directory.
///
/// `experiments/agentic_cli_matrix/run_leg.sh` reaches the Agent CLI through
/// `lib.sh`, so a sweep that stopped at the file the workflow names would
/// declare that harness clean without ever reading the line that launches the
/// client.
fn sourced_siblings(path: &str, body: &str) -> Vec<String> {
    let Some((directory, _)) = path.rsplit_once('/') else {
        return Vec::new();
    };
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed
                .strip_prefix("source ")
                .or_else(|| trimmed.strip_prefix(". "))?;
            let token = rest.split_whitespace().next()?.trim_matches('"');
            let name = token.rsplit('/').next()?;
            Path::new(name)
                .extension()
                .is_some_and(|extension| extension == "sh")
                .then(|| format!("{directory}/{name}"))
        })
        .collect()
}

/// Every shell script CI reaches, whether a workflow names it directly or
/// another script it names does.
fn shell_scripts_ci_runs() -> Vec<(String, String)> {
    let mut queue: Vec<String> = Vec::new();
    for path in github_yaml_files() {
        let body = fs::read_to_string(&path).expect("readable workflow");
        queue.extend(experiment_script_references(&body));
    }

    let mut seen: Vec<String> = Vec::new();
    let mut scripts: Vec<(String, String)> = Vec::new();
    while let Some(path) = queue.pop() {
        if seen.contains(&path) {
            continue;
        }
        seen.push(path.clone());
        let Ok(body) = fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))) else {
            continue;
        };
        let body = body.replace("\r\n", "\n");
        queue.extend(experiment_script_references(&body));
        queue.extend(sourced_siblings(&path, &body));
        scripts.push((path, body));
    }
    scripts.sort();
    scripts
}

/// Issue #1079, D11. `Proactive failure report Agent CLI E2E` failed twice in a
/// row on this branch with a provider error from a service the harness never
/// configured:
///
/// ```text
/// Error from provider (Console): Request is missing x-opencode-session and cannot be routed efficiently
/// ```
///
/// The harness points the Agent CLI at a local `formal-ai serve`, and that is
/// the only provider in its config. But `--summarize-session` defaults to true
/// and the summarizer does not use the session's model: it loads the head of
/// the default compaction cascade, `opencode/big-pickle`, over the hosted
/// gateway. `agent-stream.raw.log` records the decision --
/// `"service":"session.summary","providerID":"opencode","modelID":"big-pickle"`
/// -- and the rejection arrives as an `UnhandledRejection` on stderr, which
/// `scripts/classify-agent-cli-stderr.sh` refuses to hide.
///
/// This repository already knew: commit f9cee7b68, "fix(ci): disable hosted
/// Agent summarization", added `--no-summarize-session` in July 2026 -- to one
/// harness, pinned by a gate that reads that one file. Fourteen of the
/// twenty-five harnesses CI runs never got it.
///
/// The compaction half is spelled `--compaction-models "(same)"` rather than
/// `--compaction-model same`, because the singular flag is silently ignored:
/// `src/cli/model-config.js` takes the cascade branch whenever
/// `argv['compaction-models']` is set, and yargs always sets it to the default
/// cascade. Reproduced against `@link-assistant/agent` 0.26.0: passing
/// `--compaction-model same` still logs
/// `models: ["opencode/big-pickle", "kilo/minimax-m2.5-free", "same"],
/// source: "default"`, while `--compaction-models "(same)"` logs
/// `models: ["same"], source: "cli"`. Filed upstream; the report is in
/// `dev/log/issues/1079/pulls/1080/upstream-reports/`.
#[test]
fn every_agent_cli_harness_ci_runs_keeps_the_session_local() {
    let harnesses: Vec<(String, String)> = shell_scripts_ci_runs()
        .into_iter()
        // `--disable-stdin` is an Agent CLI flag; no other client in these
        // harnesses accepts it.
        .filter(|(_, body)| body.contains("--disable-stdin"))
        .collect();

    assert!(
        harnesses
            .iter()
            .any(|(path, _)| path == "experiments/agent_cli_e2e/run_issue_864.sh"),
        "the sweep must reach the harness that failed, found {:?}",
        harnesses.iter().map(|(path, _)| path).collect::<Vec<_>>()
    );

    for (path, body) in &harnesses {
        assert!(
            body.contains("--no-summarize-session"),
            "{path} drives the Agent CLI against a local provider but leaves \
             `--summarize-session` at its default, so the client calls the \
             hosted `opencode/big-pickle` summarizer between turns and the run \
             dies on a gateway this harness never configured (issue #1079)"
        );
        assert!(
            body.contains("--compaction-models \"(same)\""),
            "{path} must pin compaction to the session's own model with \
             `--compaction-models \"(same)\"`. The singular `--compaction-model \
             same` reads as the fix but is ignored: the CLI takes the cascade \
             branch whenever the cascade argument is set, and it is always set \
             to a default that begins with hosted models (issue #1079)"
        );
    }
}

/// Every job of a workflow, keyed by name.
///
/// `workflow_fixtures::job_block` answers "give me this one job of
/// `release.yml`"; a sweep needs the opposite -- every job of every workflow,
/// without naming any of them, so a job added tomorrow is measured too.
fn workflow_jobs(body: &str) -> Vec<(String, String)> {
    let Some(jobs_at) = body.find("\njobs:\n") else {
        return Vec::new();
    };
    let mut jobs: Vec<(String, String)> = Vec::new();
    for line in body[jobs_at + "\njobs:\n".len()..].lines() {
        let is_job_header = line.starts_with("  ")
            && !line.starts_with("   ")
            && line.trim_end().ends_with(':')
            && !line.trim_start().starts_with('#');
        if is_job_header {
            jobs.push((line.trim().trim_end_matches(':').to_string(), String::new()));
        } else if let Some((_, collected)) = jobs.last_mut() {
            collected.push_str(line);
            collected.push('\n');
        }
    }
    jobs
}

/// Issue #1079, D11, second layer. The flags on the harness only bind the
/// invocations that harness spells out; a job that also launches the client
/// some other way -- a ladder that shells out per node, a step that resumes a
/// session -- would still reach for the hosted summarizer.
///
/// `LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION=false` binds the whole job, and the
/// CLI reports the result in its first stream event
/// (`{"type":"config","summarizeSession":false,...}`), so the belt is
/// observable rather than assumed. `release.yml`'s `test-agent-cli-e2e` job
/// has carried it since issue #819; the two workflows that grew their own
/// Agent CLI job afterwards did not inherit it, which is the same
/// one-file-gate shape as the missing flags.
#[test]
fn every_ci_job_that_launches_the_agent_cli_disables_hosted_summarization() {
    let agent_harnesses: Vec<String> = shell_scripts_ci_runs()
        .into_iter()
        .filter(|(_, body)| body.contains("--disable-stdin"))
        .map(|(path, _)| path)
        .collect();

    let mut checked = 0;
    for path in github_yaml_files() {
        let body = fs::read_to_string(&path)
            .expect("readable workflow")
            .replace("\r\n", "\n");
        for (job, job_body) in workflow_jobs(&body) {
            let launches_agent = experiment_script_references(&job_body)
                .iter()
                .any(|script| agent_harnesses.contains(script));
            if !launches_agent {
                continue;
            }
            checked += 1;
            assert!(
                job_body.contains("LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION: \"false\""),
                "job `{job}` of {} runs an Agent CLI harness but does not set \
                 LINK_ASSISTANT_AGENT_SUMMARIZE_SESSION=false, so any client \
                 the job starts outside the harness command line still calls \
                 the hosted `opencode/big-pickle` summarizer (issue #1079)",
                path.display()
            );
        }
    }

    assert!(
        checked >= 3,
        "the sweep must find the jobs that run the Agent CLI, found {checked}"
    );
}

/// Issue #1079, D11, third place. The same wrong flag is baked into the
/// product: `formal-ai with agent ...` appends `no_summarize_args` from the
/// seed to every wrapped invocation, so every user of the wrapper -- not just
/// CI -- was sending a flag the CLI ignores and getting the hosted summarizer
/// they asked to be rid of.
///
/// The value is `(same)` rather than `same` because `--compaction-models`
/// parses a parenthesized list; `formal-ai with` passes argv directly, so the
/// parentheses that a shell would quote away belong in the argument itself.
#[test]
fn the_wrapper_pins_agent_compaction_to_the_session_model() {
    let agent = formal_ai::seed::client_integrations()
        .into_iter()
        .find(|integration| integration.id == "agent")
        .expect("the seed registry must describe the Agent CLI");

    assert_eq!(
        agent.invocation.no_summarize_args,
        vec![
            "--no-summarize-session".to_string(),
            "--compaction-models".to_string(),
            "(same)".to_string(),
        ],
        "`formal-ai with agent` must disable summarization *and* pin \
         compaction to the session's own model. `--compaction-model same` is \
         silently ignored whenever the plural default is set, which it always \
         is (issue #1079)"
    );
}
