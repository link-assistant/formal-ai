# Release workflow rationale

Historical release.yml rationale is retained below. Executable YAML and shell steps are unchanged by relocation. Active timing and exception policy lives in the checked data records.

    # Issue #1187 R4: an isolated e2e/<purpose>/<run>/<name> pull request
    # never triggers this repository's own pipeline.
    # Issue #1187 R3: a dispatch runs the checks only unless a release mode
    # is chosen explicitly, and every release, publish and tag job also
    # requires the main ref.
    # Set this repository variable to true only while investigating sccache
    # backend failures. The shared setup action otherwise leaves logging off.
    # TEMPORARY (issue #1111): non-Linux CI is skipped, not deleted; set this to
    # `run` to restore it. docs/case-studies/linux-only-ci/PLAN.md
    # Issue #1131: these were read from repository configuration that was never
    # set, so `configure-dockerhub-publishing.sh` disabled Docker Hub on every
    # release and the release still reported success. hive-mind names its image
    # in the workflow the same way (`IMAGE_NAME: konard/hive-mind`). A fork still
    # overrides either through `vars`, and a fork without DOCKERHUB_TOKEN skips
    # Docker Hub entirely -- the token, not the image, is what opts in.
    # Only the files a pipeline job executes (its definition, a composite action
    # it calls, or a script those run): `workflow-changed` fires for any workflow
    # file, which made a scheduled benchmark's edit pay for the whole pipeline (#1107).
    # Issue #1081: principle 16 of docs/CI-CD-BEST-PRACTICES.md -- "prove you can
    # publish before you build". Every release credential used to be exercised
    # only by the step that needed it, near the end of a 90-minute release job, so
    # a revoked crates.io token or an expired Docker Hub access token cost a full
    # build before anyone learned the release could not happen. This job asks the
    # same registries the same question in seconds, and the release jobs below
    # need it. On anything that is not a release it reports instead of failing: a
    # fork's pull request has no secrets to probe, and the code can still be
    # tested.
    # The GHCR probe opens and cancels a blob upload session, which is the
    # only question ghcr.io answers differently for "may push" and "may pull".
    # Issue #1017 (D19): one commit for every base-branch merge in this run.
    # Issue #1076 (D5) raised the one serial job's cap 15 -> 25 -> 45 minutes
    # (run 35806556194 was killed mid-gate at 25). PR #1188 runs the gates as
    # parallel lanes instead (`lane` in data/meta/ci-gates/, CI_GATE_LANE
    # below; run-ci-gates.rs --check fails a gate no lane runs); lane 1 also
    # runs the wasm and web stages. 30 holds the slowest lane (~15 min).
    # The rebuilt worker is published so a stale checked-in binary can be
    # replaced with CI's exact bytes (the toolchain is CI's stable, which a
    # contributor's machine may not match); check_wasm_worker_current below
    # still fails until it is committed.
    # Issue #1017: the cap is the backstop, the step budget the deadline (run
    # 31937348472 used 1415s of a 1500s cap). PR #1188 (R1188-U9): no job runs
    # over 30 minutes. Nothing compiles here any more -- both suites run the
    # test executables `build-artifacts` built once -- and each budgeted group
    # (the full lane's 1080s gates or 1080s tests, the specification shard's
    # 1080s) stays at 60% of the cap, inside the 70% share
    # `tests/unit/ci-cd/job_budget_fit.rs` holds.
    # Issue #1111: a reported skip, never a silent one.
    # Issue #1081: this is the ubuntu leg's first compiling step, worst
    # measured 484s over 39 runs, and it had no deadline of its own -- only
    # the job cap, whose expiry reports `cancelled` rather than `failure`
    # (issue #977).
    # Issue #1055: built once above; this only runs them. The binary comes
    # too, because `CARGO_BIN_EXE_formal-ai` bakes `rust/target/release/formal-ai`
    # into each executable at compile time -- 109 call sites across 36 test
    # files spawn it -- so the path has to exist here as well.
    # PR #1188 (R1188-U9): the specification shards run the unit executable
    # `build-artifacts` compiled once (release, all features) instead of each
    # compiling its own, which took 220-362s per shard on run 37802763478 and
    # 1201s cold on run 34095902681.
    # The existing doctest contract uses default features and the test
    # profile, unlike the release/all-feature executable producer. Prepare
    # exactly that library in the same workspace before timing the examples.
    # Issue #1017: the heaviest lane here (one archive build plus sixteen Intel
    # macOS slices at 10x the Linux rate) had no concurrency group, so a second
    # push doubled it. `main` never cancels, keeping issue #977's rule intact.
    # Issue #1017: archive and slices must merge the same base commit.
    # Issue #1076 (D5): 11.6 min worst case (the run that missed its cargo
    # cache) was 77.0% of 15; 20 puts it at 57.8%, inside the 70% share.
    # Issue #1111: `skipped` is off, not broken; `failure` still blocks.
    # Issue #1055: the binary exists; packaging needs no rebuild.
    # R1015: the JavaScript server must answer the parity corpus exactly as the
    # release binary does before that binary ships; it reuses the binary
    # build-artifacts already built (issue #1055), so no extra compile.
    # Issue #1076 (D5): 50.6 min worst case was 84.4% of the old 60m cap, and
    # the 45m GHCR budget below must fire before the cap. 90m puts it at 56.2%.
    # This job and `manual-release` are two of the four checkouts in this
    # repository that keep their credential (issue #1079); every other one
    # sets `persist-credentials: false`. `scripts/version-and-commit.rs`
    # below runs `git push` and `git push --tags` to publish the version
    # bump, and `scripts/git-config.rs` sets only `user.name`/`user.email`,
    # so that push authenticates with exactly the credential
    # `persist-credentials: false` removes. This job requires
    # `github.event_name == 'push' && github.ref == 'refs/heads/main'` and
    # `manual-release` requires `workflow_dispatch`, so neither ever
    # persists the credential on a pull request.
    # Issue #1085 (D3.5): the self-development floor no longer gates the
    # release. It is measured and reported red-until-true by
    # `.github/workflows/self-development-status.yml` on every push to
    # `main`; a release is cut on CI correctness alone, and the ledger row
    # it records says honestly what the model authored in it.
    # Re-read publication state after the version command has synchronized
    # with main. A rerun may have resumed a version prepared by the first
    # attempt, so the pre-sync `check` output can describe the wrong version.
    # Issue #1076 (D5): same shape as `auto-release` -- publishes the same
    # image with the same 45m budget, so the same 90m backstop applies.
    # No `persist-credentials: false` here, for the same reason as in
    # `auto-release` above (issue #1079): `scripts/version-and-commit.rs`
    # pushes the version bump and its tag, and nothing else in this job
    # supplies auth for that push.
    # The last of this repository's four checkouts that keep their
    # credential (issue #1079); every other one sets
    # `persist-credentials: false`. `peter-evans/create-pull-request` below
    # commits the generated fragment and pushes a new branch, and this job
    # only runs on `workflow_dispatch`, so the credential exists solely for a
    # manually triggered release and never on a pull request.
    # Issue #1138 moved the 83 lines of steps behind this call into
    # `.github/workflows/e2e-local.yml`, returning this file to the 1500-line
    # warning band; the reasoning is in that file's header. The cap moved with
    # them, because a caller job declares none.
    # Issue #1081 moved the 51 steps behind this call into their own file; the
    # reasoning is in its header. The cap moved with them, because a caller job
    # declares none.
    # Issue #1107 (item 4): the full replay is 25 of the pipeline's minutes. On
    # `main`, on a schedule and on manual dispatch it runs in full. Issue #1137
    # also requires it before merge whenever the PR changes the agentic routing
    # subsystem, because agent, OpenCode, Claude, and Codex expose different
    # tools. Other feature branches keep the held-out generalization gates.
    # Build the host binary once for the seven Box image legs. The audited run
    # compiled the same Linux binary seven times and reported 1,131 cache write
    # errors across those legs; the available logs do not include backend error
    # details, so opt-in diagnostics live in the shared sccache setup action.
    # Issue #1055: one compile per platform; consumers download the results.
    # Issue #932 (follow-up of PR #119): a generated project for a language is
    # built and run inside the link-foundation/box image that matches that
    # language, using the language's own init commands. The matrix keeps each
    # image on its own runner, so one leg only pulls the toolchain it needs.
    # A box unpacks to 4-8 GB, more than a stock runner has spare after setup.
    # The bundle and cargo doc build under their own 30-minute cap (R1188-U9).
    # A writing job states its own dispatch guard (R1187-3).
    # Covers the Pages deployment queue, whose wait the action caps at 10 minutes.

## Desktop release workflow commentary

These original YAML comments were relocated without changing executable YAML or shell content. Source SHA256 before relocation: ee7e46617498e3454bf97225cb122ad74e48ce4526869b9b1acecdc4eb3f5ef8.

Original line 1: # Cross-platform desktop release (Linux, Windows, macOS) for formal-ai.
Original line 2: #
Original line 3: # Produces the exact assets the /download page (js/download) resolves from
Original line 4: # the GitHub Releases API:
Original line 5: #   formal-ai-desktop-macos-<arch>-<version>.{dmg,zip}
Original line 6: #   formal-ai-desktop-windows-installer-<arch>-<version>.exe
Original line 7: #   formal-ai-desktop-windows-portable-<arch>-<version>.exe
Original line 8: #   formal-ai-desktop-linux-<arch>-<version>.{AppImage,deb,tar.gz}
Original line 9: #   latest.yml, latest-mac.yml, latest-linux.yml, and *.blockmap updater metadata
Original line 10: #   (Electron Builder emits x86_64/amd64 Linux x64 aliases; the build job
Original line 11: #    normalizes them to x64 before checksums/uploads and rewrites update metadata.)
Original line 12: # plus a consolidated SHA256SUMS.txt and BUILD-PROVENANCE.txt, and SLSA build
Original line 13: # provenance attestations (verifiable with `gh attestation verify`).
Original line 14: #
Original line 15: # Triggers:
Original line 16: #   * release: published     - fires for releases published manually (UI) or via
Original line 17: #                              a PAT. NOTE: the automated `[Rust] x.y.z` releases
Original line 18: #                              created by the CI/CD Pipeline use GITHUB_TOKEN, so
Original line 19: #                              GitHub suppresses the `release` event for them
Original line 20: #                              (recursion guard). The workflow_run trigger below
Original line 21: #                              covers that case automatically.
Original line 22: #   * workflow_run           - after the "CI/CD Pipeline" completes on main (with
Original line 23: #                              ANY conclusion except cancelled/skipped -- the
Original line 24: #                              release is published in an early job, so a later job
Original line 25: #                              failing must not suppress the desktop build), build
Original line 26: #                              desktop assets for the resolved release tag
Original line 27: #                              (idempotent: skips when assets already exist).
Original line 28: #   * workflow_dispatch      - manual rebuild for a specific (or latest) tag.
Original line 31: # zizmor flags every `workflow_run` trigger as fundamentally insecure, because the
Original line 32: # usual shape of one -- re-running a pull request's own head in a context that has
Original line 33: # write permissions and secrets -- is arbitrary code execution by any fork. This
Original line 34: # workflow is not that shape, on two independent counts:
Original line 35: #
Original line 36: #   1. The `resolve` job refuses any `workflow_run` whose `head_branch` is not
Original line 37: #      `main` (see its `if:` below), so a fork's pipeline run cannot reach it.
Original line 38: #   2. Nothing downstream ever checks out `workflow_run.head_sha`. The privileged
Original line 39: #      jobs check out `needs.resolve.outputs.tag` -- a release tag that
Original line 40: #      scripts/desktop-release-resolve.sh reads back from this repository's own
Original line 41: #      Releases API. Untrusted code is never the build input, so even if a fork
Original line 42: #      branch named `main` slipped past (1), the worst case is a redundant build
Original line 43: #      of an already-published tag, which the idempotency guard then skips.
Original line 44: #
Original line 45: # Issue #1076: recorded rather than silenced, so the next person to add a
Original line 46: # `workflow_run` consumer has to re-establish both counts.
Original line 59:     # Issue #1081 (D8): without this filter the event fires for EVERY pipeline
Original line 60:     # completion, including the pull_request runs of every feature branch. The
Original line 61:     # `resolve` job below then rejects them on `head_branch == 'main'`, so the
Original line 62:     # whole run concludes `skipped`. Measured over 2026-08-31..2026-09-07: 107
Original line 63:     # workflow_run-triggered runs, 102 of them skipped (95.3%), and 99 of those
Original line 64:     # 102 correlate 1:1 with a pull-request pipeline finishing on a feature
Original line 65:     # branch (dev/log/issues/1081/pulls/1082/measurements/desktop-release-workflow-run-noise.md).
Original line 66:     # Every run that did real work was triggered by a `push` pipeline on main.
Original line 67:     # `branches` on workflow_run matches the TRIGGERING workflow's branch --
Original line 68:     # "You can use the `branches` or `branches-ignore` filter to specify what
Original line 69:     # branches the triggering workflow must run on in order to trigger your
Original line 70:     # workflow" -- so this drops exactly the noise and keeps all five useful
Original line 71:     # runs. The `head_branch == 'main'` guard in `resolve` stays: it is one of
Original line 72:     # the two counts that make this `workflow_run` trigger safe (see the header
Original line 73:     # comment), and a trigger filter is not a substitute for that argument.
Original line 75:   # Issue #808: packaging and macOS signing used to run only after a release
Original line 76:   # existed, so a desktop regression was always discovered post-merge -- which is
Original line 77:   # exactly how the "unsealed contents present in the root directory of an
Original line 78:   # embedded framework" codesign failure reached main. On pull requests the same
Original line 79:   # build matrix runs in dry-run mode: everything up to and including the smoke
Original line 80:   # tests, with every publishing step (attest, release upload, SHA256SUMS)
Original line 81:   # skipped. Path-filtered so unrelated pull requests do not pay for six runners.
Original line 83:     # Issue #1187 R4: an isolated e2e/<purpose>/<run>/<name> pull request
Original line 84:     # never triggers this repository's own pipeline.
Original line 94:       # The build job runs `cargo build --manifest-path rust/Cargo.toml --release --bin formal-ai` and bundles
Original line 95:       # the result, so a Rust-only change can break desktop packaging while
Original line 96:       # touching none of the paths above. Without these entries that breakage is
Original line 97:       # a false negative on the pull request and only surfaces post-merge --
Original line 98:       # precisely the failure mode issue #808 was filed for.
Original line 134:     # For workflow_run, continue after ANY completed main-branch pipeline except
Original line 135:     # cancelled/skipped runs (issue #479). We intentionally do NOT gate on
Original line 136:     # conclusion == 'success': the auto-release publishes the GitHub release in an
Original line 137:     # EARLY pipeline job, so a LATER job failing (e.g. the E2E Pages probe timing
Original line 138:     # out) still leaves a real, asset-less release that desktop assets must heal.
Original line 139:     # Gating on full-pipeline green meant one unrelated late failure suppressed
Original line 140:     # every desktop build -> "Not available in latest release". The actual
Original line 141:     # decision to build is delegated to scripts/desktop-release-resolve.sh, whose
Original line 142:     # idempotency guard skips when the resolved release already carries desktop
Original line 143:     # assets, so running on a non-success conclusion is safe and self-healing.
Original line 144:     # Pull requests resolve nothing: there is no release to heal and no tag to
Original line 145:     # check out. The job is skipped and `build` admits the skipped dependency
Original line 146:     # explicitly, so the packaging matrix still runs against the PR head.
Original line 157:       # Check out the repo so the resolve logic lives in a unit-tested script
Original line 158:       # (scripts/desktop-release-resolve.sh) instead of being inlined here. The
Original line 159:       # script only needs to *exist*; it queries the GitHub API for everything
Original line 160:       # else, so checking out the default ref is sufficient.
Original line 182:   # Issue #1017 (D19): the six packaging legs of run 31993872684 started across
Original line 183:   # 62 minutes, and each resolved the base branch tip itself -- `linux-x64` and
Original line 184:   # `macos-arm64` merged 1858b3386 while `windows-arm64` merged d1439e557, so
Original line 185:   # six installers built from two source trees shipped as one release set with
Original line 186:   # nothing comparing them. One commit for every merge in this run.
Original line 195:   # Compile one exact selected source; package its verified target bytes twice where applicable.
Original line 453:     # `resolve` is skipped on pull requests, and a skipped dependency would
Original line 454:     # normally skip this job too -- hence the explicit `!cancelled()`.
Original line 467:     # Windows ARM64 normally takes 25-28 minutes, and runs 30573608801
Original line 468:     # (attempts 1 and 2) were both cancelled at the former 30-minute cap while
Original line 469:     # electron-builder was completing a valid installer. Keep the job bounded
Original line 470:     # while allowing for runner and antivirus/file-lock variance.
Original line 471:     #
Original line 472:     # Issue #896 added the published web-search/web-capture Rust crates. Their
Original line 473:     # current releases expose the server/browser dependency graphs
Original line 474:     # unconditionally (upstream web-search#22 and web-capture#148). On the cold
Original line 475:     # macos-15-intel runner in run 30788311906, release compilation took 33m21s
Original line 476:     # and the otherwise-valid job hit the 40-minute cap while creating the DMG.
Original line 477:     # The Windows targets moved the same way: in run 30832897812, windows-x64
Original line 478:     # compiled for 17m39s, packaged a valid installer, and was cancelled at the
Original line 479:     # cap while uploading its checksum fragment, with windows-arm64 finishing at
Original line 480:     # 36m04s. Give those three targets bounded packaging headroom until the
Original line 481:     # adapter-only upstream features let us remove the heavyweight graph; the
Original line 482:     # Linux and macOS ARM64 targets stay well under 40 minutes.
Original line 483:     #
Original line 484:     # The cap lives in the matrix (`capmin`) rather than in an expression here
Original line 485:     # so that the packaging retry guard can be derived from the *same* number:
Original line 486:     # two independent copies of "50" would drift, and a guard computed from a
Original line 487:     # stale cap is worse than no guard at all.
Original line 497:       # Issue #1017: the packaging retry must know how much of the job clock is
Original line 498:       # left, not how long packaging usually takes. `timeout-minutes` starts
Original line 499:       # before this step, so the reserve covers both the runner's own set-up and
Original line 500:       # everything that has to happen *after* packaging (artifact normalization,
Original line 501:       # the DMG mount/codesign smoke test, checksums and uploads).
Original line 577:       # Issue #990: execute the real published command-stream adapter after the
Original line 578:       # production dependency tree is installed, on every packaged platform.
Original line 590:       # Issue #1017: electron-builder fetches its 7-Zip toolset -- and, on
Original line 591:       # macOS, the dmgbuild bundle -- with a single `got` request whose only
Original line 592:       # deadline is 600 000 ms. In run 95255998673 that request stalled for the
Original line 593:       # full ten minutes on macos-15-intel: the retry underneath recovered, the
Original line 594:       # DMG, ZIP and both blockmaps were written, and the build still failed
Original line 595:       # because AsyncTaskManager.awaitTasks() rethrew the timeout it had already
Original line 596:       # recorded. Seeding the checksum-validated archive cache that
Original line 597:       # `downloadAndExtract` consults before it touches the network removes the
Original line 598:       # request itself. Every failure here is a warning only, so packaging falls
Original line 599:       # back to downloading the toolset exactly as it does today.
Original line 632:       # Issue #1055: through the retry wrapper, like the macOS legs. The
Original line 633:       # toolset download electron-builder makes is the same on every platform,
Original line 634:       # and `Build windows-x64` failed on `read ECONNRESET` mid-download while
Original line 635:       # the previous run of that job, on the same branch, had succeeded.
Original line 662:       # Issue #479: electron-builder 26 changed macOS signing control flow. Its
Original line 663:       # MacPackager.sign() now calls findSigningIdentity(), which returns null --
Original line 664:       # skipping signing entirely and NEVER invoking the custom `mac.sign` hook --
Original line 665:       # unless a real identity is found OR `mac.identity` is exactly "-" (ad-hoc).
Original line 666:       # See packages/app-builder-lib/src/targets/mac/MacTargetHelper.ts: only the
Original line 667:       # `qualifier === "-"` branch yields an Identity when no certificate exists.
Original line 668:       # Without `-c.mac.identity=-` the produced .app had no _CodeSignature, so the
Original line 669:       # pre-upload smoke test failed with "missing its signed CodeResources
Original line 670:       # envelope" and macOS assets were never published. Passing identity="-" makes
Original line 671:       # electron-builder reach doSign(), which runs our ad-hoc hook (deep ad-hoc
Original line 672:       # sign via @electron/osx-sign + an explicit root-bundle re-seal).
Original line 812:       # The macOS smoke test above is the only artifact verification this
Original line 813:       # workflow had, so the four Linux/Windows targets were uploaded to the
Original line 814:       # release with nothing checking that electron-builder actually produced
Original line 815:       # them under the expected names. A rename or a partial package silently
Original line 816:       # shipped. The deep checks are macOS-only by nature (hdiutil, codesign),
Original line 817:       # but existence and non-emptiness are checkable everywhere.
Original line 898:       # Attest before publishing, not after: uploading first left the assets
Original line 899:       # downloadable for the length of the attest step, and permanently so if
Original line 900:       # attest failed, with `gh attestation verify` unable to vouch for them.
Original line 943:   # Issue #1181 (R1-R5): ship the `formal-ai` CLI binary itself as standalone
Original line 944:   # release archives, so `curl | tar xz` (or `cargo binstall formal-ai`, whose
Original line 945:   # [package.metadata.binstall] in rust/Cargo.toml points at these names) works
Original line 946:   # without a Rust toolchain and without building the desktop app. Five
Original line 947:   # targets, one per archive:
Original line 948:   #   formal-ai-cli-x86_64-unknown-linux-musl.tar.gz   (static, no glibc dependency)
Original line 949:   #   formal-ai-cli-aarch64-unknown-linux-musl.tar.gz  (static)
Original line 950:   #   formal-ai-cli-x86_64-apple-darwin.tar.gz
Original line 951:   #   formal-ai-cli-aarch64-apple-darwin.tar.gz
Original line 952:   #   formal-ai-cli-x86_64-pc-windows-msvc.zip
Original line 953:   # Each archive holds `formal-ai-cli-<triple>/` with the binary, LICENSE and
Original line 954:   # README.md. Names are version-less because GitHub Releases namespaces assets
Original line 955:   # per tag; scripts/install.sh resolves the release by tag, so a stable name
Original line 956:   # keeps the download path (and binstall's pkg-url) independent of the version
Original line 957:   # string embedded everywhere else.
Original line 964:     # Same skipped-dependency handling as `build`; see the comment there.
Original line 1070:       # The one artifact verification this job has (mirroring the desktop
Original line 1071:       # legs' smoke tests): extract the just-built archive and run the binary
Original line 1072:       # from it. Every leg builds for its own host arch (musl included -- a
Original line 1073:       # static musl binary executes on the glibc host), so each leg is the one
