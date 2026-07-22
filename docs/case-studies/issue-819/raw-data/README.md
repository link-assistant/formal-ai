# Raw evidence inventory

This directory retains the source material and machine evidence used for issue
#819. JSONL dialog records contain complete request and response bodies and may
include large client system prompts; the smaller `dialog-sequence.json` files
are verifier-generated projections of the four required turns.

## GitHub

`github/` contains the issue, all issue comments and timeline events, the
prepared pull request, all PR conversation comments, all inline review comments,
and all submitted reviews. List endpoints were fetched with pagination.

## Red/green tests

`tests/reproduction-before-fix.log` is the pre-implementation failure.
`focused-unit.log` includes the exact prompt, fuzzy execution, language examples,
negative web controls, and full seed-vocabulary sweep. The native-protocol and
release-build logs are retained alongside it.

## Real clients and TUI

`e2e/four-client-and-tui.log` is the successful aggregate harness output. Each
client subdirectory contains the full raw server dialogs and normalized
user/tool-call/tool-result/final-answer sequence. `opencode-tui/` additionally
contains the command-stream/xterm frame transcript from the real TUI.

The fixture root is a fresh temporary directory. Its random suffix is expected
to differ between runs and prevents the test from searching a developer's real
Desktop.

## Self-authoring

`self-authoring/` preserves a real Agent CLI stream, Formal AI server trace,
exact dialog JSONL, and the generated Links Notation plan. That session created
`../../agent-authored-requirement.lino` and verified it by reading it back.

Full request logging is enabled only inside this controlled harness. Production
dialog logging remains opt-in because real prompts and tool results may contain
private information.
