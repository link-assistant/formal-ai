# Issue #552 Solution Plan

## Architecture

Shared-dialog support is split into three layers:

1. Capture: fetch the source page or transcript artifact and preserve source
   evidence. This belongs primarily in `web-capture`.
2. Normalize: convert provider-specific shape into a shared dialog model:
   title, conversation id, turn ids, roles, content, visibility, source URL,
   and capture diagnostics.
3. Replay: export the normalized dialog to formal-ai memory/test formats and
   run solver regressions against representative turns.

This PR implements layer 2 and 3 for ChatGPT static captures and Markdown
transcripts. It records the layer-1 gap for Google AI Mode.

## Format Decisions

- `demo_memory` is used as the immediate replay format because formal-ai already
  imports and exports it.
- Every generated event includes:
  - `role`
  - `content`
  - `demoLabel`
  - `conversationId`
  - `conversationTitle`
  - `evidence`
- Multi-line content is escaped inside quoted Links Notation values so code
  fences and shell snippets round-trip.

## Parser Decisions

ChatGPT shared pages currently embed the conversation in streamed React Router
data. The parser:

- extracts `window.__reactRouterContext.streamController.enqueue(...)` chunks;
- decodes JSON string literals from each chunk;
- resolves the indexed-array/object-reference shape;
- finds `linear_conversation`;
- emits visible user/assistant messages only.

Markdown transcripts are supported as a provider-neutral fallback using compact
speaker prefixes such as `User:`, `U:`, `Assistant:`, and `A:`.

Google AI Mode is intentionally not guessed from the static interstitial. The
converter returns a structured unsupported-format error when it sees a Google
AI Mode capture without replayable transcript data.

## Solver Decisions

The captured dialog exercises shell-command transformation, not shell
execution. In chat mode the correct behavior is to produce the command text.

The solver handler:

- extracts shell commands from terminal prompt lines such as `host:~$ command`;
- wraps command text as `while true; do <command>; done`;
- uses prior assistant history to resolve follow-up references like "that line";
- converts `screen -R <name>` follow-ups into detached execution form:
  `screen -dmS <name> bash -c '<loop command>'`;
- returns plain one-line commands without code fences for one-line requests.

## Next Work

- Add `web-capture shared-dialog` once the upstream capture/schema issues are
  accepted.
- Convert the shortest public ChatGPT and Google AI Mode samples into a
  provider regression corpus.
- Add capture diagnostics for provider challenge pages, deleted/expired shared
  links, login walls, empty transcripts, and visibility-filtered turns.
- Replace the local normalized struct with the shared meta-language schema when
  available.
