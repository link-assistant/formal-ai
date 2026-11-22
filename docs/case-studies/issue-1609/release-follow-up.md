# Released client verification follow-up

PR #1611 merged the [original client fixes](README.md), including the
[OpenCode report and export fixes for #1610](../issue-1610/README.md).
The maintainer reopened #1609 at 07:17 UTC on November 22, 2026, after the new
pre-publication checks blocked user downloads. The release job prepared and
tagged v0.356.3, and the staged package verification failed before any publish
job could use the package. The public download workflow skipped its download
jobs. [Maintainer comment](https://github.com/link-assistant/formal-ai/issues/1609#issuecomment-7075826829),
[download verification](https://github.com/link-assistant/formal-ai/actions/runs/44971340331).

| Version and verification time (UTC) | Result | Evidence |
| --- | --- | --- |
| v0.356.1, November 21 at 22:11–22:14 | Codex reported a successful issue submission before its temporary marketplace clone raced deletion of the test directory. Gemini and Qwen also failed the gate. | [Job](https://github.com/link-assistant/formal-ai/actions/runs/44953310700/job/137000650349), [log](raw-data/release-verify-44953310700.log.gz) |
| v0.356.2, November 21 at 23:11–23:13 | Codex passed. Gemini and Qwen still failed. | [Job](https://github.com/link-assistant/formal-ai/actions/runs/44958138655/job/137015939493), [log](raw-data/release-verify-44958138655.log.gz) |
| v0.356.3, November 22 at 01:11–01:16 | The 26 protocol tests, all 16 stock greetings and the six other stock issue-report replays passed. Gemini and Qwen failed the verification step. | [Job](https://github.com/link-assistant/formal-ai/actions/runs/44968089085/job/137047298875), [log](raw-data/release-verify-44968089085.log.gz), [client artifact](https://github.com/link-assistant/formal-ai/actions/runs/44968089085/artifacts/14733975172) |

## Causes and regression checks

- **Gemini CLI 0.66.0.** The report shell received the explicit
  `gemini_release_report` session argument. Gemini's strict redaction in
  GitHub Actions removed the environment variables that tell the report CLI
  which client and capture directory to use. The resulting command tried to
  resolve the Gemini session as an OpenCode transcript. The generated settings
  for both Gemini and Vertex append three report variables to the supported
  allowlist. The release replay sets `GITHUB_SHA` for the same strict behavior
  locally and additionally allows the path used by the recording GitHub CLI.
  The focused native test verifies the settings emitted by the real wrapper.
  The stock rerun with the staged v0.356.3 binary submitted the report with
  9 harness and 16 server records.
  [Hosted failure receipt](raw-data/release-44968089085-gemini-receipt.jsonl),
  [local success receipt](raw-data/replay-0.356.3-gemini-receipt.jsonl),
  [Gemini's redaction implementation](https://github.com/google-gemini/gemini-cli/blob/v0.66.0/packages/core/src/services/environmentSanitization.ts).
- **Qwen Code 0.26.4.** The recording GitHub CLI returned the issue URL and Qwen
  confirmed it. The rendered issue body was 52,151 bytes. The body limiter
  omitted the reasoning trace, which held the source counts, and 36 older
  messages. Its links still pointed to the complete export of 53 records and
  689,430 bytes. The recording GitHub CLI now checks both positive source counts
  from the merged Lino file passed to `gh gist create`. The replay separately
  rejects a zero harness count and a zero server count. The stock Qwen rerun
  with the staged v0.356.3 binary produced counts of 8 harness and 45 server
  records and returned the issue URL. Its preview still failed the former
  Markdown check. [Hosted success receipt](raw-data/release-44968089085-qwen-receipt.jsonl),
  [local success receipt](raw-data/replay-0.356.3-qwen-receipt.jsonl),
  [GitHub CLI upload contract](https://cli.github.com/manual/gh_gist_create).
- **Codex CLI 0.171.0.** The v0.356.1 verifier log records a successful issue
  receipt followed by a removal failure in `.tmp/plugins-clone-*/plugins`.
  The stock release replay disables the optional plugin feature for this
  loopback test so the default marketplace refresh cannot race cleanup.
  Publication and receipt assertions run as before. The PR policy check
  pins the setting to the isolated Codex publication replay. The stock rerun
  with the staged v0.356.3 binary submitted the report with 7 harness and
  7 server records and completed cleanup.
  [Codex plugin feature](https://github.com/openai/codex/blob/rust-v0.171.0/codex-rs/features/src/lib.rs#L1929),
  [first verification log](raw-data/release-verify-44953310700.log.gz),
  [local replay log](raw-data/replay-0.356.3-codex.log).

The [release replay](../../../experiments/issue_1609_client_reports/verify-release.sh)
still exercises stock clients against the selected binary before the automatic
and manual workflow paths can publish.
[Staged workflow](../../../.github/workflows/release-staged.yml),
[recorded capture counts](raw-data/replay-0.356.3-capture-counts.log).

## v0.356.4 publication on November 22, 2026

The release selected source `09711c000cca319d7269b4b0fa15d15339baf881`.
At 08:39 UTC, the selected package passed 28 protocol checks, all 16
English and Russian greetings, and issue report replays for Agent, Aider,
Claude, Codex, Gemini, Grok, OpenCode and Qwen. The reports use a recording
GitHub CLI with fixture issue and Gist URLs, so their receipts demonstrate the
submission and export behavior in isolation. The complete workflow artifact
includes the OpenCode terminal recording and the retained captures.
[Verification job](https://github.com/link-assistant/formal-ai/actions/runs/45001543202/job/137149097233),
[hosted log](raw-data/release-verify-45001543202.log.gz),
[downloaded artifact](raw-data/release-client-reports-auto-45001543202-1.zip),
[artifact checksum and metadata](raw-data/release-client-artifact-14745894801.json).

| Surface | Result for v0.356.4 |
| --- | --- |
| crates.io | The crate was published at 08:46:30 UTC. Registry availability and installation smoke passed. [Registry record](raw-data/crates-io-0.356.4.json), [hosted job results](raw-data/release-45001543202-jobs.json). |
| Full GHCR image | `0.356.4` and `latest` both resolved anonymously to `sha256:c9d1bbfa30a92358daed777f7cc6edb4d0360278cdd8f76f1432cfbd7edde270`. Its privileged server smoke failed after the push. [Registry response](raw-data/ghcr-0.356.4-manifests.log), [image job](https://github.com/link-assistant/formal-ai/actions/runs/45001543202/job/137156873419). |
| Slim GHCR image and Docker Hub mirror | The full image error stopped the remaining publish steps. [Hosted job results](raw-data/release-45001543202-jobs.json). |
| GitHub Release and downloads | GitHub Release creation skipped. The subsequent desktop, browser, source archive and public-download wrappers skipped their delivery or verification jobs. [Desktop](https://github.com/link-assistant/formal-ai/actions/runs/45006837300), [browser](https://github.com/link-assistant/formal-ai/actions/runs/45006837288), [source archive](https://github.com/link-assistant/formal-ai/actions/runs/45006837320), [download verification](https://github.com/link-assistant/formal-ai/actions/runs/45006926920). |

### Server startup failure

At 09:16:48 UTC, the inner Docker execution printed `formal-ai-dind-ok`.
The native server then exited with `PermissionDenied` before opening
`/health`. The full image inherited the Box entrypoint, which switches
to user `box` for the command, while the Dockerfile assigned that command a
memory file under `/root/.formal-ai`. The server opens its memory store
before binding the HTTP listener. The failed path is reproduced in the replay
below. The production error appears at lines 4325 and 4340–4341 of the archived
log. [Hosted log](raw-data/release-publish-images-45001543202.log.gz),
[Box entrypoint](https://github.com/link-foundation/box/blob/v2.10.2/ubuntu/24.04/dind/dind-entrypoint.sh#L767-L795),
[server transport](../../../rust/src/server/transport.rs).

```sh
docker run --rm --privileged \
  ghcr.io/link-assistant/formal-ai@sha256:c9d1bbfa30a92358daed777f7cc6edb4d0360278cdd8f76f1432cfbd7edde270 \
  verify-formal-ai-dind
```

The full image now pre-creates `/home/box/.formal-ai` with owner `box:box`
and mode `0700` before declaring it as a volume. The slim image already
uses the same path. Compose and Desktop bind the existing host directory at
that container path. A temporary override to a writable directory can unblock
an existing image, and deployments using a bind or named volume can reuse the
same data at the new path. The smoke reports the server exit status and memory
path when startup fails. [Docker volume initialization](https://docs.docker.com/engine/storage/volumes/#mounting-a-volume-over-existing-data),
[deployment instructions](../../configuration/docker.md).

Under user `box`, the hosted v0.356.4 executable failed with the old image
path and passed health, chat, agent routing, greetings and issue forms when
pointed at a private `box` directory. Both invocations used the same
executable, SHA-256 `faa25eed783bc5c417e2872417466099dd02eb4a9f0d64a670cd5fdc9c7a2e5a`.
The replay exercises the published native program and server smoke on the
host. The hosted container workflow exercises the actual image and inner
Docker daemon. [Source selection](raw-data/release-0.356.4-source-selection.json),
[executable receipt](raw-data/release-0.356.4-native-executable-receipt.json),
[local replay](raw-data/replay-0.356.4-dind-memory.log),
[Docker runtime regression test](../../../rust/tests/web/container-node-runtime.test.mjs).

### Tagging and release acceptance

The original build command assigned the public release tags before running
the image. The full and slim staged publishers now build to a run-specific
candidate tag, test its immutable digest, and promote that same manifest to
the release tags. Both check the final digest of each tag. The full publisher
also checks source authority again immediately before promotion. The
regressions cover smoke failure and a release tag that resolves to a
different digest. [Full publisher test](../../../rust/tests/web/prepared-release-factory.test.mjs),
[cache and slim publisher test](../../../rust/tests/web/native-workflow-cache-bindings.test.mjs),
[Buildx manifest promotion](https://docs.docker.com/reference/cli/docker/buildx/imagetools/create/).

The retained v0.356.4 observations explain the maintainer's report that the
downloads were missing. Release acceptance still requires a guarded main run
with the fix, successful full and slim image verification and the corresponding
public crate, GitHub Release and downstream download checks. The changelog
fragment queues a new version through the normal release workflow.
[Issue discussion](raw-data/issue-release-follow-up.json), [PR #1626](https://github.com/link-assistant/formal-ai/pull/1626).

### Native verification of the pull request

The first PR native job compiled the test CLI at optimization level zero. In
both server modes, the same classification request reached its 10-second
deadline and the server reached its 90-second fixture lifetime before the
last ten assertions. The failure stopped the later reporting steps. The job
now inherits Cargo's test profile at optimization level two, with assertions
and overflow checks enabled. The request and server deadlines still apply.
The same replay passed 84 assertions against the optimized v0.356.4 native
executable, with the JavaScript-only worker assertion skipped by design.
[Hosted native job](https://github.com/link-assistant/formal-ai/actions/runs/45008981956/job/137166953914),
[captured prompt-data log](raw-data/pr-native-prompt-data-45008981956.log.gz),
[optimized executable replay](raw-data/replay-0.356.4-prompt-data.log),
[Cargo test profile](../../../rust/Cargo.toml),
[native verification workflow](../../../.github/workflows/native-type-check.yml).

The next pull request run's dependency audit flagged `url` 2.5.8. The
application and the standalone parser experiment now resolve the patched
2.5.9 release and its required `form_urlencoded` 1.2.3 dependency.
A local replay with the CI audit tool reported the advisory on the previous
lockfile. The repository audit then passed on the updated lockfiles.
[Hosted audit](https://github.com/link-assistant/formal-ai/actions/runs/45010582620/job/137171807186),
[RustSec advisory](https://github.com/rustsec/advisory-db/blob/main/crates/url/RUSTSEC-2026-0592.md).

## v0.356.5 publication and Pages build on November 22, 2026

The guarded release selected `43b31443025d6b632d3352f86512e01abb8f658a`.
The package passed 28 protocol checks and completed 16 stock greetings and
eight stock reports before the crate and container publishers ran. The
OpenCode terminal replay verified the issue body and separate harness, server
and combined export links. The recording GitHub CLI returned fixture URLs
for the report submissions. [Selected release](https://github.com/link-assistant/formal-ai/releases/tag/v0.356.5),
[stock verifier job](https://github.com/link-assistant/formal-ai/actions/runs/45020746043/job/137214149077),
[verifier log](raw-data/release-verify-45020746043.log.gz),
[stock client capture artifact](https://github.com/link-assistant/formal-ai/actions/runs/45020746043/artifacts/14754788232),
[archived capture](raw-data/release-client-reports-auto-45020746043-1.zip),
[archive checksum and metadata](raw-data/release-client-artifact-14754788232.json).

| Time (UTC) | Surface and observation |
| --- | --- |
| 13:14–13:32 | The crate published, became visible in the crates.io index, and passed install and runtime smoke. [Registry and job observations](raw-data/release-0.356.5-public-observations.json). |
| 13:36 | The maintainer reported another apparent delivery failure while image and GitHub publication were still in progress. [Maintainer comment](https://github.com/link-assistant/formal-ai/issues/1609#issuecomment-7079928805). |
| 13:55 | Full GHCR, slim GHCR and Docker Hub mirror publication completed after the runtime checks. Full and mirror receipts record the successful inner Docker command and server smoke. The slim smoke checked health, chat, agent routing, greetings and issue forms. [Image job](https://github.com/link-assistant/formal-ai/actions/runs/45020746043/job/137225130068), [image log](raw-data/release-publish-images-45020746043.log.gz). |
| 13:59 onward | GitHub published the release. The downstream jobs attached native and desktop packages, the installable browser engine tarball, VSIX and source archives. The public downloads workflow verified the bytes, the packaged browser worker, and macOS packages on arm64 and x64. The browser receipt records npm registry publication as optional and unconfigured. [Release](https://github.com/link-assistant/formal-ai/releases/tag/v0.356.5), [public download checks](https://github.com/link-assistant/formal-ai/actions/runs/45037053420), [browser receipt](https://github.com/link-assistant/formal-ai/releases/download/v0.356.5/browser-publication.json). |
| 14:04 | The Pages artifact job failed during public Rust API documentation generation. The browser greeting and report smoke in that job could not run. The public site continued to return v0.355.34 from the November 5 deployment. [Pages job](https://github.com/link-assistant/formal-ai/actions/runs/45020746043/job/137232338452), [deployment job](https://github.com/link-assistant/formal-ai/actions/runs/42062109916/job/127429052754). |

The public observation records the crate checksum, immutable image receipts,
and anonymous manifest digests for all three image tags. The full and slim
GHCR version tags are multi-platform indexes. Their digests identify the
public indexes, while the smoke receipts identify the candidates used by
the staged image job. [Public observations](raw-data/release-0.356.5-public-observations.json),
[platform index assembly](https://github.com/link-assistant/formal-ai/actions/runs/45020746043/job/137239081940).

### Root cause and regression replay

The Pages build runs rustdoc with `-D warnings`. Two links in the public
documentation for `bound_output_literals` pointed to private functions.
The `parse_language_grammar` documentation rendered the `<id>` filename
placeholder as an unclosed HTML tag. The job stopped with exit code 101.
The archived extract preserves the original log line numbers 58676–58709.
[Failed job log](raw-data/pages-rustdoc-45020746043.log),
[rustdoc private link lint](https://doc.rust-lang.org/rustdoc/lints.html#private_intra_doc_links),
[rustdoc invalid HTML tag lint](https://doc.rust-lang.org/rustdoc/lints.html#invalid_html_tags).

Inline code preserves the helper names and the filename example in the
public documentation. The fast pull-request checks reject both original
patterns. Each failed on the original comments and passed on the corrected
comments. An isolated `rustdoc -D warnings` replay using those comments
reported all three errors before the change and generated HTML with the
literal filename after it. [Fast replay](../../../rust/tests/web/default-branch-build-policy.test.mjs),
[main regression ledger](../../../data/meta/main-only-regressions.lino).

The Pages artifact job still requires the strict API docs build and the
artifact greeting and issue-report smoke before deployment. A Playwright
replay on the currently deployed v0.355.34 answered `Hello` and `Привет`
through the WASM worker and included both turns in the GitHub issue form
link. Live acceptance for this release requires a trusted main release that
includes the markup fix, a successful Pages deployment, and the same checks
on that deployed source. [Public observations](raw-data/release-0.356.5-public-observations.json),
[Pages artifact workflow](../../../.github/workflows/pages-artifact.yml),
[deployment and live smoke workflow](../../../.github/workflows/release.yml).

### v0.356.6 partial publication on November 22, 2026

The next run passed the stock package verifier and installed the public
crate. Its full GHCR candidate passed the privileged runtime smoke at 16:43:42
UTC and was promoted to the version tag and `latest`. An anonymous HEAD
request to the version tag returned the verified candidate's manifest digest
`sha256:27bad63622a2686009a67b8157f9683b4dc64ae6d0a55a13e65a0555f2939ae3`.
[Stock verification](https://github.com/link-assistant/formal-ai/actions/runs/45038778921/job/137265736854),
[installed crate smoke](https://github.com/link-assistant/formal-ai/actions/runs/45038778921/job/137273012981),
[image publication job](https://github.com/link-assistant/formal-ai/actions/runs/45038778921/job/137276086927).

Main advanced from the selected v0.356.6 source `ffd50eb404a9` to
`0367553b776a` at 16:45:02 UTC. The optional cache export reported its
bounded timeout at 16:45:17 UTC. The subsequent source check rejected the
changed main SHA and stopped the image stage before the slim sidecar and
Docker Hub mirror. GitHub Release creation and the Pages artifact build were
skipped. The strict API docs check in the same run had already reproduced
all three markup errors from v0.356.5. The next trusted release must select
fresh main source. [Main update](https://github.com/link-assistant/formal-ai/pull/1623),
[image log](raw-data/release-publish-images-45038778921.log.gz),
[API docs errors](raw-data/api-docs-45038778921.log),
[skipped Pages build](https://github.com/link-assistant/formal-ai/actions/runs/45038778921/job/137279408287).

## Self-use attribution

A read-only attempt asked the Agent CLI against the local JavaScript Formal AI
server to independently identify the two assertions that can fail after issue
creation. Formal AI repeated a code search three times and stopped without a
diagnosis. One reasoning task was attempted and zero were accepted. Codex
authored the implementation and regression checks.
[Captured tool and message events](raw-data/release-follow-up-dogfood.jsonl).

For the v0.356.4 follow-up, the Agent CLI made one read-only attempt to
identify the Docker startup failure and did not return a confirmed cause.
Zero of one reasoning tasks were accepted. Codex authored the follow-up.
[Captured events](raw-data/release-dind-dogfood.jsonl).

For the Pages follow-up, the read-only Agent CLI research attempt ended
with a provider connection error. The JavaScript dogfood driver read the
grammar source and returned without changing either file. Zero of one
reasoning tasks and zero of one editing tasks were accepted. Codex authored
the documentation corrections, regression checks and release analysis.
[Requests and captured events](raw-data/pages-rustdoc-dogfood.log.gz).
