use super::{image_references, repository_file};

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
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("run: node scripts/run-zizmor.mjs ")
        })
        .collect();
    assert_eq!(
        passes,
        ["regular", "pedantic"],
        "both complete online audit personas must run, in order (issue #1079, D3)"
    );
    let runner = repository_file("scripts/run-zizmor.mjs");
    assert!(runner.contains("`--persona=${persona}`"));
    assert!(runner.contains("`--min-confidence=${persona === 'regular' ? 'medium' : 'high'}`"));
    assert!(
        runner.contains("if (persona === 'pedantic') args.push('--min-severity=high')"),
        "only the pedantic pass may narrow severity; regular must retain all findings"
    );
    assert!(
        runner.contains("args.push('--', '.github/workflows', '.github/actions')"),
        "both passes must audit all workflow and action sources"
    );
    assert!(
        !runner.contains("--no-online-audits"),
        "online advisory checks remain authoritative"
    );
    // The fast JavaScript transport regression calls this actual runner with
    // captured process results and checks both personas' complete arguments.
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

    let runner = repository_file("scripts/run-zizmor.mjs");
    let image = runner
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("export const image = '")
                .and_then(|value| value.strip_suffix("';"))
        })
        .expect("the shared online auditor must pin its executable image");
    let (tagged, digest) = image
        .split_once("@sha256:")
        .expect("immutable auditor digest");
    assert_eq!(digest.len(), 64);
    assert!(digest.bytes().all(|value| value.is_ascii_hexdigit()));
    let version = tagged
        .strip_prefix("ghcr.io/zizmorcore/zizmor:")
        .expect("official auditor registry");
    assert!(
        documented.contains(&version),
        "the shared auditor runs {version} but the reproduction instructions name {documented:?}"
    );
    for persona in ["regular", "pedantic"] {
        assert!(
            audit
                .lines()
                .any(|line| line.trim() == format!("run: node scripts/run-zizmor.mjs {persona}")),
            "every documented audit persona must execute the same pinned runner"
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
