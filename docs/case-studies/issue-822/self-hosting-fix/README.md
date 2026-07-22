# Self-hosting repair evidence for issue 822

This artifact records the Formal AI Agent CLI session that authored the
follow-up correctness repair for issue 822 after the initial pull request
failed the repository's self-hosting ratchet.

## Provenance

- Formal AI session: `ses_07660a750ffeEbAnHNXH44gtkJ`
- Pull request: <https://github.com/link-assistant/formal-ai/pull/823>
- Failing workflow run: <https://github.com/link-assistant/formal-ai/actions/runs/29912605458>
- Failing job: `Self-Hosting Evidence Check` (`88898896939`)
- Local server: the release `formal-ai serve` binary built from this branch
- Agent provider/model: `formalai/formal-ai`
- Agent package: `@link-assistant/agent` 0.25.0

The failing check reported `0.00% (0/4929 changed lines; 0/23 commits)` and a
projected ratchet decrease from 17.14% to 15.36%. That result was accurate:
the original commits contained no self-authorship trailers because the first
live self-coding attempt had failed before a Formal AI session started.

The repair therefore did not manufacture attribution for the earlier work.
It opened a new local session, used that session to apply each new regression
and implementation patch, and limits the corresponding commit trailer to the
follow-up changes documented here.

## Reproduction authored through the Agent CLI

The first Agent request applied
`experiments/issue_822_complete_transcript_regression.patch`. Its focused test
proved two independent losses in the context exported by the existing code:

1. Two exchanges with the same millisecond timestamp were reordered by request
   id instead of retaining physical JSONL append order.
2. Only the most recent request's messages were exported, so incremental
   clients lost earlier turns and the final server assistant response was
   absent.

The red assertion expected this complete sequence:

```text
system, user, assistant, user, assistant/tool-call, tool, assistant
```

The implementation returned six entries, ending with the tool result rather
than the final assistant response. The append-order assertion also saw
`request-a` before the earlier `request-z` record.

A second Agent request applied
`experiments/issue_822_shared_lino_serializer_regression.patch`. That fixture
created a read-only OpenCode SQLite session containing the literal string
`"true"`, exported it as JSON and LiNo, and compared the latter with the public
arbitrary-JSON conversion path. It failed because the embedded Python script
had copied the serializer instead of sharing it:

- its object fields used a different order;
- it emitted the string `"true"` as bare `true`, changing its apparent type;
- future quoting fixes would have required synchronized Python and Rust edits.

These failures were captured before either production change was applied.
They are minimal integration regressions rather than assertions against an
implementation detail: one feeds real dialog JSONL, while the other invokes
the installed Python extractor over a real temporary SQLite database.

## Repair authored through the Agent CLI

The Agent applied
`experiments/issue_822_complete_transcript_fix.patch`, which changed canonical
conversation reconstruction to:

- preserve the append order already guaranteed by the JSONL recorder;
- merge all request histories by maximal exact suffix/prefix overlap, covering
  both full-history and incremental clients without duplicate turns;
- append assistant messages from OpenAI Chat Completions responses;
- reconstruct streamed SSE content and indexed fragmented tool calls.

The Agent then applied
`experiments/issue_822_shared_lino_serializer_fix.patch`. The Python helper is
now solely a read-only structured-data adapter. It emits JSON, and the Rust CLI
parses that value and routes it through `render_server_context`, the same
serializer used by arbitrary JSON and server-dialog export.

A third regression, in
`experiments/issue_822_response_envelopes_regression.patch`, expanded response
coverage for every non-streaming protocol envelope handled by the server:

- OpenAI Chat Completions `choices[].message`;
- OpenAI Responses `output[]` message items;
- Gemini `candidates[].content`;
- direct response objects containing `role` and `content`.

That regression deliberately includes an OpenAI Responses reasoning item. The
normalizer ignores it because reasoning metadata is evidence in `server_logs`,
not a conversation message. Unknown envelopes remain present verbatim in the
same raw exchange list, so normalization never destroys transport evidence.

Finally, the Agent applied
`experiments/issue_822_context_export_docs.patch` to document complete capture,
source selection, API formats, merge behavior, supported response envelopes,
the common LiNo contract, read-only OpenCode extraction, privacy, and failure
diagnostics. This closes the public-documentation gap exposed while tracing
the incomplete export.

## Commands and outcomes

All substantive patches were passed to the resumed session with this shape:

```text
agent --model formalai/formal-ai \
  --permission-mode auto \
  --output-format stream-json \
  --compact-json \
  --verbose \
  --no-summarize-session \
  --compaction-model same \
  --resume ses_07660a750ffeEbAnHNXH44gtkJ \
  --no-fork --disable-stdin \
  --prompt 'Run git apply --recount <patch>'
```

The session applied the following artifacts in order:

| Artifact | Purpose | Outcome |
| --- | --- | --- |
| `issue_822_complete_transcript_regression.patch` | Red complete-transcript tests | Applied; tests failed on the missing/reordered messages |
| `issue_822_complete_transcript_fix.patch` | Request merge and streamed-response reconstruction | Applied; focused tests passed |
| `issue_822_shared_lino_serializer_regression.patch` | Red shared-serializer fixture | Applied; comparison exposed the divergent Python output |
| `issue_822_shared_lino_serializer_fix.patch` | Route OpenCode JSON through Rust | Applied; focused tests passed |
| `issue_822_response_envelopes_regression.patch` | Add complete non-streaming response coverage | Applied; focused tests passed |
| `issue_822_context_export_docs.patch` | Publish operator and API guidance | Applied |
| `issue_822_self_hosting_evidence.patch` | Reconcile the case study and record provenance | Applied |
| `issue_822_clippy_test_helpers.patch` | Borrow fixture values instead of consuming them | Applied after Clippy identified three warnings |
| `issue_822_verification_evidence.patch` | Record the completed local validation | Applied after the full suite passed |

The first verbose request included explanatory prose after the patch command;
the shell applied the patch and then rejected the trailing words. Subsequent
requests used exact command-only prompts, producing clean zero exits. This
detail is retained because evidence should describe the observed tool result,
not only its intended result.

## Generated artifacts and verification boundary

The Agent also requested the repository's deterministic self-AST generator.
Its shell tool reached a fixed 60-second client timeout while Cargo was still
compiling. The already-requested command was then run directly to completion:
it reported 292 documents, three rewritten, and none removed. Those generated
`.lino` changes correspond only to the two modified Rust source files and the
index.

`cargo fmt` was run directly as a deterministic mechanical formatter after the
Agent-applied patches. No authored logic was added by formatting or self-AST
generation. Codex prepared the patch artifacts and orchestrated the local
server; the Formal AI session inspected and applied the substantive repository
changes described above.

Focused verification after the repair covered all issue-822 integration and
unit tests: six integration tests and eleven unit tests passed. The full local
`cargo test --all-features --verbose` run subsequently passed every test binary,
including 172 integration tests and 1,966 unit tests with two intentional
ignores; its doc-test phase also passed. Formatting, Clippy with warnings denied,
file-size, associative-terminology, hardcoded-language, worker-line-budget, and
WASM-size checks passed locally.

The exact self-hosting ratchet result is recorded in the pull request check
associated with the self-authored commit because the metric reads committed
history. This evidence file intentionally states that boundary instead of
implying every generated byte or orchestration command was authored by the
model.

## Attribution contract

The commit containing this evidence may carry:

```text
Formal-AI-Session: ses_07660a750ffeEbAnHNXH44gtkJ
Formal-AI-Evidence: docs/case-studies/issue-822/self-hosting-fix/README.md
```

The evidence path is committed in that same change, contains the exact session
identifier, and describes the actual Formal AI Agent CLI activity. No prior
commit is rewritten or retroactively attributed.
