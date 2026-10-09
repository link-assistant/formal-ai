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
