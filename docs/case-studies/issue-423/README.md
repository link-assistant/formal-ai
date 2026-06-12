# Case study: issue #423

> Source issue: <https://github.com/link-assistant/formal-ai/issues/423>
> Branch: `issue-423-85026fdc955a` - PR: #424
> Raw data: [`raw-data/`](./raw-data) (issue JSON, comments, PR JSON, PR comments,
> review comments, reviews, and a top-50 GitHub repository snapshot)

## Summary

Issue #423 asks the assistant to convert GitHub project installation
instructions between these surfaces:

- a `README.md` installation or deployment guide;
- a POSIX shell script (`sh` / `bash`);
- a PowerShell script;
- and the reverse path from an installation script back to `README.md`.

Before this change, prompts such as "Convert this README.md installation guide
into both sh and PowerShell scripts" were either unclaimed (`unknown`) or were at
risk of being treated as a generic "write a script" request. The solver had no
intermediate representation for installation steps, so it could not preserve
the source commands and render multiple target formats from the same meaning.

This PR adds a deterministic `installation_conversion` handler in the Rust
solver and the browser worker mirror. The handler extracts ordered command-like
install/deploy steps, stores them in a small install-step IR, and renders the
requested target format(s) from that single IR.

## Research and raw inputs

Raw GitHub inputs were archived under [`raw-data/`](./raw-data):

- `issue-423.json` and `issue-423-comments.json`
- `pr-424.json`, `pr-424-conversation-comments.json`,
  `pr-424-review-comments.json`, and `pr-424-reviews.json`
- `github-top-50-repositories.json`, captured with GitHub repository search
  sorted by stars on 2026-06-12

There were no issue comments or PR review comments when the work started. The
top-50 repository snapshot is summarized in
[`raw-data/online-research.md`](./raw-data/online-research.md) and is used as
the basis for the 50-project regression matrix in
`tests/unit/installation_conversion.rs`.

## Requirements

| # | Requirement | Status |
|---|-------------|--------|
| R1 | Convert README installation instructions to `sh` scripts. | Done. |
| R2 | Convert README installation instructions to PowerShell scripts. | Done. |
| R3 | Convert installation scripts back to a `README.md` guide. | Done. |
| R4 | Use a general algorithm rather than one-off per-project answers. | Done: one install-step IR feeds every renderer. |
| R5 | Cover at least 50 popular GitHub project cases. | Done: 50 repository prompts from the captured top-starred GitHub snapshot. |
| R6 | Cover the deployed browser worker as well as the Rust solver. | Done: `src/web/formal_ai_worker.js` mirrors the handler and dispatch order. |
| R7 | Preserve commands verbatim so conversions can round-trip. | Done: commands are stored unchanged in ordered `InstallStep` records. |
| R8 | Document the investigation, requirements, solution, and tests. | Done: this case study plus archived raw data. |

## Root cause

The existing dispatch had software-project planning and generic write-script
paths, but no handler for "convert these existing installation instructions."
That matters because conversion is not the same as generation: the correct
answer must keep the original commands, preserve ordering, and render one or
more target surfaces from the same source meaning.

The first reproducing tests confirmed the gap:

- README-to-script prompts returned `unknown`.
- Script-to-README prompts returned `unknown`.
- 50 popular-project conversion prompts were claimed by the wrong generic path
  or not claimed at all.

## Implementation

The new handler lives in `src/solver_handlers/installation_conversion.rs` and is
exported through `src/solver_handlers/mod.rs`. `src/solver_dispatch.rs` registers
it before generic write-script behavior, because an installation conversion
request is more specific than "write a script."

The handler uses this pipeline:

1. Recognize conversion language plus both installation-document and script
   surfaces (`README`, `markdown`, `install guide`, `deployment script`, `sh`,
   `bash`, `PowerShell`, `ps1`, and related forms).
2. Detect source format. Explicit script source phrases win over target
   mentions, so "this shell script back to README.md" is not misread as a
   markdown source because the target says `README.md`.
3. Detect target format(s): markdown, shell script, PowerShell script, or both
   shell and PowerShell.
4. Extract ordered install/deploy commands:
   - shell or PowerShell fenced blocks inside README content;
   - inline backtick commands;
   - bullet/list commands;
   - script lines, ignoring shebangs, comments, and strict-mode boilerplate.
5. Build `InstallStep { id, description, command }` records.
6. Render every target from the same ordered step list.

The rendered answer includes a formalized meaning block:

```lino
installation_conversion_request
  source_format markdown
  target_format shell_script
  target_format powershell_script
  project "react/react"
  validation "ordered_commands_preserved"
  validation "single_ir_renders_markdown_shell_powershell"
  step "S1"
  description "Clone the repository"
  command "git clone https://github.com/react/react.git"
```

The browser worker mirror implements the same recognizer, IR extraction, and
renderers in `src/web/formal_ai_worker.js`. Its dispatch entry runs before the
generic `write_program` handler so "convert this README to a script" keeps the
source instructions instead of producing a starter script.

## Reproducing tests

`tests/unit/installation_conversion.rs` covers:

- README guide to both Bash and PowerShell scripts.
- A wrapped `markdown` README containing nested shell fences, which is common
  when users paste README content into a prompt.
- Shell installation script back to a `README.md` guide.
- 50 popular GitHub project conversion prompts built from the captured
  top-starred repository snapshot.

`experiments/issue-423-js-installation-conversion.mjs` loads the browser worker
in a Node VM and verifies:

- the direct worker handler claims README-to-script and script-to-README prompts;
- rendered output preserves commands;
- full `solve()` dispatch routes through `tryInstallationConversion` before
  `write_program`.

## Verification

Focused checks run during development:

```text
node --check src/web/formal_ai_worker.js
node experiments/issue-423-js-installation-conversion.mjs
cargo test --test unit installation_conversion -- --nocapture
```

The final PR validation should also include formatting, source mirror checks,
clippy, and the repository file-size check.

## Limitations and next steps

The handler intentionally does not execute installation commands or validate
external project setup. It performs deterministic text conversion only. The
command detector is conservative and favors common installation commands; future
work can widen the command vocabulary through seed data if the project later
wants multilingual or ecosystem-specific installation phrase coverage.
