# Issue #552 Shared Dialog Replay Case Study

## Source Material

- Issue: https://github.com/link-assistant/formal-ai/issues/552
- PR: https://github.com/link-assistant/formal-ai/pull/553
- ChatGPT shared dialog: https://chatgpt.com/share/6a3825b9-8de4-83ee-9c24-52fd1eb38d24
- Google AI Mode shared dialog: https://share.google/aimode/VG0HhpnAXrBkC0QgP
- Raw GitHub exports, captured HTML, search samples, and generated replay data are saved in `raw-data/`.

## Summary

Issue #552 asks for automated scripts and CLI commands that can turn shared AI
dialog URLs into replayable test data, then use that data to improve the solver
without memorizing one transcript. The first useful slice is now implemented:
formal-ai can convert captured ChatGPT shared-page HTML or a Markdown
transcript into `demo_memory` Links Notation, and the solver can replay the
captured shell-command dialog by deriving readable one-line shell commands.

The Google AI Mode URL could not be converted from a static HTTP capture. The
saved response is a Google Search interstitial/challenge page, not a transcript.
That dependency gap is documented for `web-capture` and `meta-language`.

## Evidence

- `raw-data/issue-552.json` and `raw-data/issue-552-comments.json`: original issue and comments.
- `raw-data/pr-553*.json`: PR state, conversation comments, review comments, and reviews at investigation start.
- `raw-data/chatgpt-share-6a3825b9.html`: static ChatGPT shared-page capture.
- `raw-data/chatgpt-share-6a3825b9.demo_memory.lino`: generated replay memory from the ChatGPT capture.
- `raw-data/google-ai-mode-VG0HhpnAXrBkC0QgP.html`: static Google AI Mode capture; this is an interstitial, not replayable dialog data.
- `raw-data/github-code-search-*-public.json`: first 100 public GitHub code-search results for each shared-link family.
- `raw-data/github-code-search-*-shortest.json`: extracted unique links from those files, sorted by shortest URL.
- `raw-data/online-research.md`: source notes for ChatGPT shared links, Google AI Mode sharing, web-capture, meta-language, and the serialization shape seen in the ChatGPT capture.
- `requirements.md` and `solution-plan.md`: requirement breakdown and implementation plan.

## Captured Dialog

The ChatGPT shared page exposes a `linear_conversation` stream in the HTML. The
converter extracts visible user and assistant turns only, preserving multi-line
content with escaped newlines in `demo_memory`.

The visible conversation has four turns:

1. User asks to turn `sleep 30m && hive-cleanup -f` into an infinite loop and to answer with one line.
2. Assistant replies with `while true; do sleep 30m && hive-cleanup -f; done`.
3. User asks to run that line inside `screen -R auto-cleanup` as one line.
4. Assistant replies with `screen -dmS auto-cleanup bash -c 'while true; do sleep 30m && hive-cleanup -f; done'`.

The solver tests assert the same command shape while checking that answers stay
single-line and human-readable.

## Corpus Search

Broader GitHub code search was used to avoid treating the provided URLs as the
only examples:

| Family | Files fetched | Unique links extracted | Shortest sample file |
| --- | ---: | ---: | --- |
| ChatGPT shared links | 100 | 243 | `raw-data/github-code-search-chatgpt-share-shortest.json` |
| Google AI Mode shared links | 100 | 135 | `raw-data/github-code-search-google-ai-mode-shortest.json` |

The first ten shortest links for each family are saved in the raw-data files.
Those samples are deliberately not wired into solver fixtures yet; they define
the next corpus for provider-specific replay tests once capture support can
fetch transcripts reliably.

## Implemented Solution

- Added `formal-ai shared-dialog convert` with format selection for
  `auto`, `chatgpt-share-html`, and `markdown-transcript`.
- Added a shared-dialog parser that reconstructs visible ChatGPT turns from the
  streamed React Router payload and converts Markdown transcript snippets.
- Extended memory export/import escaping so multi-line turn content round-trips
  through `demo_memory` records.
- Added solver support for shell-command transforms:
  - wrap a command in a readable `while true; do ...; done` loop;
  - use prior assistant context to wrap that loop in a detached `screen`
    session.
- Added unit and integration coverage for conversion, memory round-tripping,
  CLI output, and the captured dialog replay.

## Dependency Findings

`web-capture` is the right layer for provider-specific page capture. It needs a
browser-backed shared-dialog mode that can distinguish "transcript extracted"
from "provider challenge/no transcript". Static Google AI Mode capture currently
falls into the latter case.

`meta-language` should define the shared-dialog/source-description schema so
formal-ai, web-capture, and future capture providers agree on turn ids, roles,
source URLs, capture method, evidence, and unsupported-capture reasons.

Upstream reports:

- web-capture: https://github.com/link-assistant/web-capture/issues/141
- meta-language: https://github.com/link-foundation/meta-language/issues/168

## Verification

Focused checks passed during implementation:

- `cargo test --test unit shared_dialog -- --nocapture`
- `cargo test --test source memory -- --nocapture`
- `cargo test --test integration cli_shared_dialog_convert_chatgpt_share_writes_demo_memory -- --nocapture`
- `cargo test --test unit shared_dialog_replay -- --nocapture`

Final local checks are saved under `raw-data/local-checks/`:

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features`: passed.
- `cargo test`: passed, including 1,024 unit tests in the largest target and doc tests.
- `cargo test --doc`: passed.
- `rust-script scripts/check-file-size.rs`: passed. It reports existing line-count warnings for files already near repository thresholds, with no violations.
