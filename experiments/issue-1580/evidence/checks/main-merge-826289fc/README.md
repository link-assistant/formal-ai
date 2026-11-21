# Main 826289fc integration and client-output regression

Main's complete hosted logs are retained under `../../ci/main-826289fc/`.
The live OpenCode and Qwen requests retain the actual system instructions,
assistant tool call, and tool observation. The focused replay does not change
those requests or synthesize a successful tool observation.

## Reproduction and repair

`rust/tests/web/issue-1580-client-output-constraints.test.mjs` replays each real
post-read request through the JavaScript API and checks the complete rendered
file answer, zero subsequent calls, and applied constraint provenance. Both
clients fail before the repair (`client-output-before-corrected.log.gz`).

The request-context parser previously promoted a quoted English example in
Qwen's conditional language policy into a standing output obligation. Direct
instruction boundaries now reject those fragments; an explicit same-language
instruction resolves through the existing language detector. Supported languages
remain the shared English/Russian seed inventory. This does not implement a
general interpreter for conditional language policies.

OpenCode's explicit line limit is checked against the final answer. Its explicit
code-generation exemption measures prose outside fenced blocks. Strict and
inclusive limits, final newlines, CRLF, detail requests, and over-limit prose have
negative controls. Unsupported XML/custom-schema and negated constraints still
refuse. Bare label/word-count instructions and Russian negation remain supported.

The pipeline's failing loopback greeting uses `Answer concisely.`. The shared
bounded policy accepts at most 64 words and four lines; short greetings pass and
over-limit prose refuses. These bounds are a defined repository policy rather
than a universal interpretation of brevity.

After the constraint repair, exact answer comparison exposed OpenCode's newer
path/type/content Read wrapper in the rendered source. The decoder now requires
the validated call path, consecutive numbered rows, a declared EOF total, and an
exact envelope. Slices remain incomplete; malformed/truncated frames stay opaque.
Explicit raw source metadata preserves all authored wrapper-looking bytes.
Client-absolutized path aliases use the actual call argument without upgrading
the original planned path to complete-source authority. Fifteen shared fixtures
cover the transport boundary in JavaScript and native source tests.

## Verification and limits

`client-output-path-bound-fixed.log.gz` passes all six exact replay/boundary tests.
The existing request-context suite covers all 38 shared rows, API transports,
memory isolation, and concurrency. Source provenance, provider, consumer, and
line-slice files pass 51/32/34/1 tests. Their complete logs and hashes are retained.
Two nonexistent test-name invocations in the first runner are retained as runner
errors; the correct ledger/profile filenames pass in the final focused batch.
The governed source fixture remains valid: all 34 identifier and two integrity
tests pass. Native source formatting passes; no Rust product is built or run.

The latest main pipeline has 38 native failures: all 35 previously inventoried
failures plus loopback concise greeting, a Chinese named-branch push request, and
an issue-1588 request-span fixture. JavaScript greeting/client reproduction does
not establish native runtime success. Broad template, benchmark, dependency,
historical cache, and live-delivery acceptance remains unfinished.

All supplied scripts, changes, and conclusions earn zero independent Formal AI
authorship credit. The frozen accepted score remains 0/20.
