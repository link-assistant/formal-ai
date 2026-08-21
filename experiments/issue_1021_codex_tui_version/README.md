# Codex TUI startup regression — bisect and reproduction

Issue #1021's pull request went red on `E2E Tests (agent CLI <-> formal-ai)`
without touching anything the Codex TUI startup path reads. This directory is
the evidence that the defect was upstream and the tooling that says whether a
future pin bump is safe.

## What broke

`@openai/codex@0.148.0` was published on 2026-08-18T22:30:14Z — after the last
green run on `main` (2026-08-18T11:13Z, codex 0.147.0) and before the first red
run on the branch (2026-08-19T06:02Z, codex 0.148.0). `release.yml` installed
`@openai/codex` unpinned, so the client changed under the harness.

From 0.148.0 on, an ENTER written to the pty the moment Codex's first-run
"Do you trust the contents of this directory?" screen renders is dropped. The
harness's keystroke was delivered — the failing CI artifact records
`interactionCount: 1` — but the dialog never advanced, and `formal-ai.log`
shows only `GET /health`: no prompt ever reached the server the test exists to
exercise.

Reported upstream as <https://github.com/openai/codex/issues/39487>.

## Reproductions

`codex_trust_dialog_probe.py` removes Formal AI from the picture entirely: a
bare `codex` in a pseudo terminal under a throwaway `HOME`, no config, no
wrapper. It installs the version it is asked for.

```
$ python3 codex_trust_dialog_probe.py 0.148.0 enter-now
[probe] codex 0.148.0 strategy=enter-now         bytes= 141221 marker_rendered=True still_on_dialog=True
```

| version | keystroke | bytes written to the pty | still on the trust dialog after 20 s |
|---|---|---|---|
| 0.147.0 | ENTER at marker + 0 s | 5 986 | no |
| 0.147.0 | ENTER at marker + 3 s | 27 838 | no |
| 0.148.0 | ENTER at marker + 0 s | 141 221 | **yes** |
| 0.148.0 | ENTER at marker + 3 s | 28 445 | no |
| 0.149.0-alpha.1 | ENTER at marker + 0 s | 9 680 | **yes** |
| 0.149.0-alpha.1 | ENTER at marker + 3 s | 4 026 | no |

`run.sh` is the same finding through the real harness: it runs the Codex leg of
`experiments/agent_cli_e2e/run_issue_819.sh` once per version, against the same
`formal-ai` binary, back to back.

```
$ cargo build --release --bin formal-ai
$ experiments/issue_1021_codex_tui_version/run.sh 0.147.0 0.148.0
== codex 0.147.0: PASS
!! codex 0.148.0: FAIL (exit 1)
== issue #819 codex E2E OK: user -> find -> result -> final ==
!! codex TUI transcript failed
```

The non-TUI Codex leg passes on both versions — only the terminal path is
affected.

## Why the harness was not made to work around it

The obvious harness fix is "answer the dialog once the screen stops moving":
`command-stream`'s interaction driver takes a per-interaction
`idleMilliseconds`, which fires only after the terminal has been quiet for that
long. That does not work here, and the measurement is the reason:

Codex's trust screen animates. It emits a full-screen repaint about every
80 ms, in 0.147.0 as well as 0.148.0, for as long as the screen is up. An idle
window therefore never opens, and setting `idleMilliseconds: 1500` on the
startup interaction made the Codex leg fail on **0.147.0 too** — the ENTER was
never sent at all. Every workaround that does clear the dialog on 0.148.0
(ENTER after 3 s, ENTER twice, `1` then ENTER, arrow keys then ENTER) needs a
wall-clock delay, which the driver has no way to express.

So the change that landed is the pin, which is what
`experiments/agentic_cli_matrix/clients.lock` already asks for in its own
words: "Versions are pinned rather than floating so a matrix leg fails because
our server changed, not because an upstream CLI shipped overnight."

## Lifting the pin

When the upstream issue closes, run `run.sh <candidate-version>`. A `PASS` is
the evidence that the pin in `.github/workflows/release.yml` can move; the
assertions in `tests/unit/ci-cd/issue_1021.rs` name the version, so the bump is
a deliberate commit rather than an overnight drift.
