# Case study: restoring docs CI/CD for issue #742

- Issue: [link-assistant/formal-ai#742](https://github.com/link-assistant/formal-ai/issues/742)
- Pull request: [link-assistant/formal-ai#743](https://github.com/link-assistant/formal-ai/pull/743)
- Failed documentation build: [docs.rs build 3878209](https://docs.rs/crate/formal-ai/0.296.4/builds/3878209)
- Related upstream reports: [lindera#750](https://github.com/lindera/lindera/issues/750), [meta-language#181](https://github.com/link-foundation/meta-language/issues/181)
- Investigation date: 2026-07-17 (UTC)

## Executive summary

The red README badge was not a GitHub Actions failure. The branch CI run was green,
and the repository already generated Rust API documentation and copied it into the
GitHub Pages artifact under `/docs/api`. The failed badge linked to docs.rs, where
every `formal-ai` 0.296.4 documentation build failed inside the transitive
`lindera-jieba` build script.

The exact dependency chain is:

```text
formal-ai 0.296.4
└── meta-language 0.45.0
    └── lindera 3.0.7
        └── lindera-jieba 3.0.7
            └── lindera-dictionary::assets::fetch (DOCS_RS branch)
```

That upstream `DOCS_RS` branch creates a dummy dictionary directory with the
non-idempotent `std::fs::create_dir`. The directory already exists on the second
build-script invocation, so documentation aborts with `File exists (os error 17)`.
The defect and minimal upstream code fix were already reported in lindera#750;
meta-language#181 tracks making the tokenizer dependency optional for downstream
consumers.

This PR applies the safe downstream workaround recommended by docs.rs itself:
`meta-language` is optional but remains in `default`, preserving every ordinary
build, while `[package.metadata.docs.rs] no-default-features = true` excludes the
broken build script only from hosted documentation. CI now runs the exact same
profile with `DOCS_RS=1` and `RUSTDOCFLAGS=-D warnings`, so the badge cannot regress
silently. Feature-disabled fallbacks compile without pretending parsing occurred.

## Requirements and disposition

| ID | Requirement extracted from #742 | Result |
|---|---|---|
| R1 | Fix the failing documentation badge and all associated warnings/errors | Fixed with a docs.rs-specific dependency profile; exact local reproduction now passes without warnings. |
| R2 | Avoid false positives and false negatives | CI validates both the normal full-feature API and the hosted-docs profile before release. Structural tests pin the complete validation-to-Pages chain. |
| R3 | If generated-doc CI is absent, add it and publish at `/docs` | It was already present. `release.yml` builds rustdoc, copies it to `src/web/docs/api`, uploads `src/web`, and deploys Pages. No duplicate workflow was added. |
| R4 | Compare the complete workflow/script trees with all four templates | Completed at pinned commits; findings are in the template audit below. |
| R5 | Preserve logs/data and reconstruct the timeline, requirements, root causes, and solutions | Completed in this directory and `raw-data/`. |
| R6 | Add diagnostics when evidence is insufficient | Not needed: docs.rs exposed the precise failing process, path, and OS error. The new CI step makes future failures directly observable. |
| R7 | Report related upstream defects with reproductions, workarounds, and code fixes | lindera#750 and meta-language#181 already contain those details and remain open with no maintainer response as of this investigation. Duplicate issues were not filed. |
| R8 | Apply the fix everywhere the defect occurs | Both automatic and manual releases share the same lint gate, so one added validation step protects both release paths. |
| R9 | Complete the work in PR #743 | Completed. |

## Evidence and timeline

| Time (UTC) | Event and evidence |
|---|---|
| 2026-07-16 16:37 | lindera#750 and meta-language#181 are opened after the same failure is isolated during issue #736. |
| 2026-07-17 11:14 | Issue #742 is opened with a screenshot showing CI/CD green and docs.rs failing. The issue has no comments; see `raw-data/issue.json` and `comments.json`. |
| 2026-07-17 11:51 | Prepared branch run [29578259319](https://github.com/link-assistant/formal-ai/actions/runs/29578259319) starts at placeholder SHA `250867e3` and passes. This proves GitHub Actions was not the red badge shown in the screenshot. |
| Investigation | docs.rs build 3878209 for released version 0.296.4 is downloaded and traced to `lindera-jieba v3.0.7`: `Failed to create dummy input directory ... File exists (os error 17)`. |
| Red test | A regression test initially fails because `meta-language` is mandatory, docs.rs metadata is absent, and CI does not exercise the hosted profile. |
| Fix verification | `DOCS_RS=1 RUSTFLAGS=-Dwarnings RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --lib --no-default-features` succeeds and does not compile Lindera. Default-feature CST, summarization, and agentic-coding tests also pass. |

The issue screenshot is preserved as `raw-data/readme-badges.png`. Its PNG signature
was verified before visual inspection. The docs.rs download is retained as HTML,
matching what the endpoint actually returned; it contains the complete rendered
build log rather than being mislabeled as plaintext.

## Root cause

### Why docs.rs fails

The released manifest makes `meta-language` unconditional. Cargo therefore runs
all transitive build scripts even though rustdoc needs only this crate's public
API. In Lindera 3.0.7, `lindera-dictionary/src/assets.rs` has a special `DOCS_RS`
branch intended to avoid downloading dictionaries, but it calls
`fs::create_dir(&input_dir)`. That call rejects an already-existing directory.
docs.rs invokes the build in a state where that dummy directory exists, causing
errno 17 and preventing rustdoc from reaching `formal-ai` at all.

The best upstream correction is `create_dir_all`, or explicit acceptance of
`ErrorKind::AlreadyExists`, together with honoring `src_subdir`. The best
meta-language correction is to make Lindera an opt-in tokenizer feature. Until a
fixed release reaches the dependency graph, a downstream documentation-only
feature profile is the only workaround formal-ai can ship independently.

### Why ordinary CI was green

The existing lint step used `cargo doc --no-deps --lib` with default features.
That is valuable because it validates the API users receive, but local/default
builds do not reproduce docs.rs's `DOCS_RS` environment. A single profile could
not validate both contracts. The workflow now runs both:

1. default features, ensuring the complete API remains documented; and
2. `DOCS_RS=1` plus no default features, ensuring hosted generation is viable.

This closes the false-positive gap without weakening normal documentation checks.

### Why the Pages pipeline did not need replacement

The existing `deploy-pages` job already:

1. depends on successful validation and package build;
2. renders default-feature rustdoc;
3. copies `target/doc/.` into `src/web/docs/api/`;
4. uploads the whole `src/web` site with `actions/upload-pages-artifact`; and
5. deploys it with `actions/deploy-pages`, followed by a live route check.

The website's `/docs` route and `/docs/api` reference therefore already satisfy
R3. Adding a second docs workflow would create competing Pages deployments and a
new race instead of solving the badge failure.

## Complete workflow and script template audit

The audit enumerated every file returned by `rg --files .github/workflows scripts`
in formal-ai and in four shallow template checkouts. This covered 49 formal-ai
files and 98 template files, not only similarly named release workflows. Exact
template revisions:

| Template | Commit | Workflow/script files | Relevant conclusion |
|---|---|---:|---|
| C# | `c6ea17b108f1f0add7a1df615c0192ce16c2e607` | 19 | Separate `docs.yml` is appropriate for DocFX/NuGet but does not model docs.rs metadata. No shared defect. |
| JavaScript | `727f22a0cfcccb401b3c99812807c515bcba9e8e` | 33 | Pages deploys an example app; npm registry wait/smoke logic is ecosystem-specific. No shared docs.rs defect. |
| Python | `c484a816fb2e2e653c38151674a668e3b13a4b7e` | 13 | Separate docs workflow and PyPI smoke test are ecosystem-specific. No shared docs.rs defect. |
| Rust | `bd217bf40c8ce3bd9974b855b6cae84caa006a11` | 20 | Closest comparison: validates rustdoc and deploys API docs through Pages, but lacks a `DOCS_RS` dependency profile and therefore would not catch this class of transitive hosted-build failure. |

Cross-file findings:

- The formal-ai Pages implementation already follows the Rust template's
  configure/upload/deploy pattern and goes further with live verification and a
  stamped artifact manifest.
- The Rust template runs `cargo doc --all-features`; formal-ai deliberately keeps
  that default/full-profile check and adds a second docs.rs check. Replacing the
  first with the second would create a false negative for feature-complete APIs.
- The templates contain published-package smoke tests, a Cargo.lock guard, and
  resilient Buildx setup. Formal-ai already has analogous registry waiting,
  committed `Cargo.lock`, crate-size checks, runtime Docker verification, and
  disk diagnostics. The remaining implementation differences are release
  hardening opportunities, not causes of the observed documentation failure;
  introducing them without a reproducing failure would exceed this bug's scope.
- The same Lindera-specific defect does not occur in the JS, Python, or C#
  dependency graphs. Filing template issues there would be a false report.
- The Rust template has no Lindera dependency, so it is not currently broken.
  A generic docs.rs profile cannot be safely prescribed for every template user:
  the appropriate feature selection is crate-specific.

No new shared template defect was found. The actual shared upstream impact is all
crates depending unconditionally on `meta-language`, which is why meta-language#181
is the correct central report.

## Solution design and alternatives

| Option | Assessment |
|---|---|
| Wait for Lindera | Leaves every current release undocumented for an unbounded time. Rejected. |
| Patch/fork Lindera in `Cargo.toml` | Couples production builds to a Git revision and changes ordinary runtime resolution. Too invasive for documentation. |
| Remove meta-language | Breaks default functionality. Rejected. |
| Hide the docs.rs badge | Converts a real failure into a false positive. Rejected. |
| Make meta-language optional, retain it in defaults, and disable defaults only on docs.rs | Selected: ordinary users retain identical defaults while hosted rustdoc avoids the defective transitive build. |

The fallback behavior is explicit and conservative. Without `meta-language`,
document conversions return `None`, recognition returns `false`, repository-file
formalization omits parser evidence, and CST validation reports unavailable rather
than inventing successful evidence. Default-feature behavior is unchanged.

## Verification protocol

The patch is complete only when all of these pass:

```bash
cargo test --test unit issue_742
DOCS_RS=1 RUSTFLAGS=-Dwarnings RUSTDOCFLAGS='-D warnings' \
  cargo doc --no-deps --lib --no-default-features
cargo check --lib --no-default-features
cargo test --test source cst
cargo test --test source summarization
cargo test --test unit agentic_coding
cargo test --test unit issue_425
```

The first test pins manifest metadata and the full validation-to-deployment
contract. The exact docs.rs simulation proves the broken transitive build script
is absent. The remaining tests prove the normal/default feature path is unchanged.
The PR's full repository CI remains the final integration check.

## Raw evidence inventory

`raw-data/` contains:

- `issue.json` and `comments.json`: complete issue snapshot and empty comment set;
- `readme-badges.png`: the original visual report;
- `docsrs-build-3878209-formal-ai-0.296.4.html`: rendered complete docs.rs build log;
- `branch-runs.json` and `main-runs.json`: timestamped workflow run lists;
- `pr-run-29578259319.json` and `pr-run-29578259319-jobs.json`: prepared-branch run and all jobs.

No failed GitHub Actions log exists for the prepared SHA because that run passed.
The failing external docs.rs build is preserved instead; treating the green branch
run as the failure would have produced the wrong fix.

## Online references

- [docs.rs metadata documentation](https://docs.rs/about/metadata) documents
  `no-default-features`, `features`, and other hosted-build controls.
- [docs.rs build documentation](https://docs.rs/about/builds) describes its
  sandboxed rustdoc builds and resource constraints.
- [GitHub's custom Pages workflow guide](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
  documents the configure/upload/deploy architecture already used here.
- [Rust `create_dir_all`](https://doc.rust-lang.org/std/fs/fn.create_dir_all.html)
  is the idempotent standard-library primitive appropriate for the upstream fix.
