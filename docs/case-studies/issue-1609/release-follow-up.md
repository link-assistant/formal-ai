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

## Self-use attribution

A read-only attempt asked the Agent CLI against the local JavaScript Formal AI
server to independently identify the two assertions that can fail after issue
creation. Formal AI repeated a code search three times and stopped without a
diagnosis. One reasoning task was attempted and zero were accepted. Codex
authored the implementation and regression checks.
[Captured tool and message events](raw-data/release-follow-up-dogfood.jsonl).
