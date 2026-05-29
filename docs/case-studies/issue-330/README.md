# Case study — Issue #330: syntax highlighting & copy buttons for chat code blocks

> The dialog quoted in the issue is already correct (fixed by predecessor
> #324). Issue #330 is a **presentation-layer** request: the fenced code blocks
> the agent already returns render as flat, monochrome `<pre><code>` with no
> highlighting and no easy way to copy them. This study reconstructs the
> context, enumerates every requirement, surveys existing libraries, records the
> chosen solution and its rationale, and lists the verification.

- **Issue:** [#330](https://github.com/link-assistant/formal-ai/issues/330) — *Issue with dialog: Сделай так, чтобы программа принимала путь как аргумент*
- **Reported version:** 0.149.0 · WASM worker · manual mode · UI language `en-US` · locale `en-US` (`Asia/Calcutta`)
- **Pull request:** [#331](https://github.com/link-assistant/formal-ai/pull/331) (branch `issue-330-64d71bce5b77`)
- **Predecessor:** [#324](https://github.com/link-assistant/formal-ai/issues/324) / PR [#325](https://github.com/link-assistant/formal-ai/pull/325) — made `write_program` answer in the prompt's language and honor follow-up modifications. The dialog in #330 is the *working output* of that fix; #330 asks to make that output **legible and copyable**.
- **Raw data:** [`raw-data/`](./raw-data/) — `issue.json`, `issue-comments.json` (`[]`), `pr-331.json`, `reproduction-dialog.md`.

---

## 1. Timeline / sequence of events

| When (UTC) | Event |
| --- | --- |
| 2026-05-28 | PR #325 lands: `write_program(rust, list_files)` answers in Russian and supports the "accept a path argument" follow-up. The agent now reliably returns fenced ` ```rust ` and ` ```text ` code blocks. |
| 2026-05-29 14:03 | In the GitHub Pages WASM worker the user replays the dialog. The answer is correct, but the code blocks render with **no syntax highlighting** and offer **no copy affordance** — to reuse the program the user must manually select the text. |
| 2026-05-29 14:06 | Issue #330 is filed (labels: `documentation`, `enhancement`, no comments). It quotes the working dialog and, in the **Description**, asks for: syntax highlighting, a per-code-block copy button, a copy-whole-message-as-Markdown button, e2e tests, run/test instructions in code examples, a check that solutions are general (not hard-coded), and a deep case study with online research and a library survey. |
| 2026-05-29 (this PR) | Self-contained syntax highlighter, per-block copy buttons, copy-as-Markdown button, i18n strings (en/ru/zh/hi), e2e tests, runnable examples, and this case study added in PR #331. |

The issue carries **no comments** (`issue-comments.json` is `[]`); every
requirement comes from the issue body.

---

## 2. Requirements (every explicit and implicit ask)

### From the Description
1. **R1 — Syntax highlighting in chat messages.** Fenced code blocks must be
   colourized by language (the dialog shows `rust` and `text`; the seed covers
   rust, python, javascript, typescript, go, c, cpp, java, csharp, ruby).
2. **R2 — Per-code-block copy button.** Each rendered code block needs a button
   that copies the **raw source** (not the surrounding Markdown fences) to the
   clipboard, with visible feedback.
3. **R3 — Copy-whole-message-as-Markdown button.** A small button on each
   message that copies the **entire message content as Markdown** (fences and
   prose preserved).
4. **R4 — E2E tests in the web app.** Browser-level tests that prove the
   highlighting renders and both copy buttons work against a freshly built
   `src/web`.
5. **R5 — Code examples with run/test instructions.** The generated program
   examples must tell the user *how to run and test them*.
6. **R6 — General solutions, not hard-coded memoization.** "Real reasoning as
   per our vision" — the highlighting/copy machinery must work for *any* code
   block and *any* language, not a memorized special case for the Rust sample.
7. **R7 — Case study + online research + library survey.** Collect issue data
   into `docs/case-studies/issue-330/`, analyze deeply, search online, list all
   requirements, and check existing components/libraries.
8. **R8 — Single PR.** Plan and execute everything in PR #331 until every
   requirement is fully addressed.

---

## 3. Root-cause analysis

This is an **absence of a feature**, not a defect, so "root cause" means
*why the current rendering falls short*.

### Cause A — Markdown renderer emits bare `<pre><code>` (causes R1)

`src/web/app.js` renders assistant content with `marked` + `DOMPurify` into a
`.markdown-body` div via `dangerouslySetInnerHTML` (`markdownHtml()`). `marked`
emits standard `<pre><code class="language-rust">…</code></pre>`. Nothing in the
pipeline tokenizes that source, so every code block is one uniform colour. There
was no highlighter wired in at all.

### Cause B — No copy affordances exist (causes R2, R3)

The rendered Markdown was display-only. There was no per-block toolbar and no
message-level action to put either the raw code or the whole message on the
clipboard. The `Message` component exposed only a "Report issue" link.

### Cause C — Examples lacked explicit run/test steps (causes R5)

The engine's `write_program` answer already appends a localized "Check command"
/ "Run command" / execution-report line (see `src/engine.rs` ~`execution_command_lines`),
but the **repository** had no standalone, runnable example showing a developer
how to compile and test the generated program end to end.

---

## 4. Solution plans (per requirement) and what was implemented

### R1 — Syntax highlighting · **done**

**Options considered** (see §5 for the survey):

- **highlight.js** — ~298 KB gzipped for the full build; automatic language
  detection; the de-facto standard.
- **Prism.js** — ~11 KB core, modular per-language; fastest.
- **Shiki** — TextMate-grammar accurate but ships a heavy WASM/regex engine.
- **Hand-rolled tokenizer** — zero dependencies, smallest possible footprint.

**Decision:** a **self-contained, dependency-free tokenizer**
(`src/web/syntax-highlight.js`) that emits highlight.js-compatible `hljs-*`
class spans. Rationale specific to this repo:

- The committed web bundle (`vendor.bundle.js`) is a binary artifact built with
  `bun`. Adding highlight.js/Prism would bloat that committed binary and couple
  the feature to a vendored build step.
- The output uses the **same `hljs-*` class contract** highlight.js uses, so the
  CSS palette and the DOM shape are forward-compatible: a future swap to
  highlight.js (or Prism with a class adapter) is a drop-in replacement that
  needs no CSS or test changes.
- All output is HTML-escaped at the tokenizer boundary, so it is XSS-safe even
  before `DOMPurify` runs.

The tokenizer (`window.FormalAiHighlight`) supports rust, python, javascript,
go, c, cpp, java, csharp, ruby, bash, json with language **aliases**
(`rs→rust`, `py→python`, `ts`/`js→javascript`, …) and handles line/block
comments, triple-quoted and single/double/backtick strings, numbers, and
keyword / literal / type / function-call classification.

### R2 — Per-code-block copy button · **done**

`enhanceCodeBlocks(root, t)` in `app.js` runs from a `useEffect` after each
render. For every `pre > code` it: reads the `language-xxx` class, runs the
highlighter, sets `code.innerHTML`, adds the `hljs` class + `data-language`, and
wraps the block in a `.code-block` shell with a `.code-block-header` containing a
language label and a `.code-copy-button` (`data-testid="code-copy-button"`).
The button copies `code.textContent` (raw source, no fences) via
`copyTextToClipboard()` (Clipboard API with a hidden-`<textarea>` +
`execCommand('copy')` fallback) and flashes a localized "Copied!" label. The
enhancement is **idempotent** (guards on the existing `.code-block` parent) so
React re-renders don't double-wrap.

### R3 — Copy-whole-message-as-Markdown button · **done**

The `Message` component gained a `.message-copy-button`
(`data-testid="copy-markdown-button"`) in the message meta row. `handleCopyMarkdown`
copies the original `message.content` (the raw Markdown, fences intact) and
flashes localized feedback. Copying the *source* Markdown — not the rendered
HTML — is what "copy as Markdown" means and is what the e2e test asserts
(`expect(clipboard).toContain('```rust')`).

### R4 — E2E tests · **done**

`tests/e2e/tests/issue-330.spec.js` (registered in
`playwright.local.config.js`) drives the real built bundle:

1. *renders a syntax-highlighted, copyable code block* — sends the Russian Rust
   prompt, asserts `.code-block`, `.code-block-lang` = `rust`, `code.hljs` with a
   visible `.hljs-keyword`, clicks the copy button, asserts `data-copied=true`
   and that the clipboard holds `fn main` / `read_dir` but **not** ` ``` `.
2. *copies the whole message as Markdown* — clicks the message button and
   asserts the clipboard holds ` ```rust ` and `fn main`.
3. *highlights every seeded program language* — Python/Go/Ruby prompts each
   render a highlighted block with the right language label.

### R5 — Code examples with run/test instructions · **done**

`examples/issue-330-code-highlighting/` contains:

- `README.md` — how the feature works and how to exercise it (build, serve,
  run the e2e suite).
- `list_files.rs` — the exact program the agent returns, with a header comment
  giving the **run** (`rustc list_files.rs && ./list_files`) and **test**
  commands, mirroring the engine's localized "Run command" line.
- `highlight-demo.mjs` — a runnable Node script that imports the tokenizer and
  prints highlighted HTML for several languages, with its own run instructions.

### R6 — General solution, not memoization · **done**

Nothing in the pipeline references the Rust sample. `enhanceCodeBlocks` walks
**every** `pre > code` in **every** message; `FormalAiHighlight.highlight`
dispatches on the fence's declared language (with alias resolution) and falls
back to a safe escaped render for unknown languages. The e2e suite proves this
generality by exercising rust **and** python/go/ruby through the same path.

### R7 — Case study + research + survey · **done** (this document + `raw-data/` + §5).

### R8 — Single PR · **done** (PR #331).

---

## 5. Existing components / libraries reviewed (R7)

Online research into web syntax highlighters (sources below):

| Library | Gzipped size | Notes | Verdict |
| --- | --- | --- | --- |
| **highlight.js** | ~298 KB full build | Industry standard, ~19M weekly npm downloads, automatic language detection, `hljs-*` class contract. | Too heavy to vendor into the committed bundle; **its class contract is adopted** so a future swap is drop-in. |
| **Prism.js** | ~2 KB core (~11 KB typical) | Fastest, modular per-language loading, plugin ecosystem (incl. a copy-to-clipboard plugin). | Strong alternative; still adds a dependency + build wiring to a bundle that is committed as a binary. |
| **Shiki** | Large (TextMate + WASM) | Most accurate (VS Code grammars). | Overkill for a chat surface; heavy WASM payload. |
| **marked** (already used) | — | Markdown→HTML; supports a `highlight` hook but ships no highlighter itself. | Reused for Markdown; highlighting layered on top in `enhanceCodeBlocks`. |
| **DOMPurify** (already used) | — | Sanitizes rendered HTML. | Reused; tokenizer also escapes at its boundary (defense in depth). |
| **clipboard.js** | ~3 KB | Wraps copy-to-clipboard with fallbacks. | Not needed — the native **Clipboard API** with an `execCommand` fallback is a few lines and adds no dependency. |

**Conclusion:** a dependency-free tokenizer emitting `hljs-*` classes gives the
visible feature (highlighting + copy) with **zero** added bundle weight while
keeping a clean upgrade path to highlight.js/Prism. The native Clipboard API
covers R2/R3 without `clipboard.js`.

Sources:
- [Comparing web code highlighters — chsm.dev (2025)](https://chsm.dev/blog/2025/01/08/comparing-web-code-highlighters)
- [Highlight.js vs Prism — PkgPulse](https://www.pkgpulse.com/compare/highlight.js-vs-prism)
- [Benchmark compare highlight.js vs Prism — peterbe.com](https://www.peterbe.com/plog/benchmark-compare-highlight.js-vs-prism)
- [highlight.js vs prism — npm trends](https://npmtrends.com/highlight.js-vs-prism-vs-prismjs-vs-rainbow-code-vs-syntax-highlighter)
- [MDN — Clipboard API](https://developer.mozilla.org/en-US/docs/Web/API/Clipboard_API)

---

## 6. Files changed

| File | Change |
| --- | --- |
| `src/web/syntax-highlight.js` | **New** — dependency-free `hljs-*`-compatible tokenizer (`window.FormalAiHighlight`). |
| `src/web/index.html` | Loads `syntax-highlight.js` before `memory.js`. |
| `src/web/app.js` | `copyTextToClipboard`, `flashCopied`, `enhanceCodeBlocks`; `Message` gains the markdown ref, copy-markdown button, and highlight `useEffect`. |
| `src/web/styles.css` | `.code-block` / header / copy-button / message-copy-button styles + `hljs-*` token palette. |
| `src/web/i18n-catalog.lino` | 6 new `message.copy*` keys × 4 locales (en/ru/zh/hi). |
| `tests/e2e/tests/issue-330.spec.js` | **New** — 3 browser tests. |
| `tests/e2e/playwright.local.config.js` | Registers the new spec. |
| `tests/e2e/scripts/check-i18n-catalog.mjs` | Adds the 6 new keys to `REQUIRED_KEYS`. |
| `examples/issue-330-code-highlighting/` | **New** — runnable examples + run/test instructions. |
| `docs/case-studies/issue-330/` | **New** — this study + raw data. |
| `docs/screenshots/issue-330-*.png` | Rendered before/after evidence. |

---

## 7. Verification

- `node tests/e2e/scripts/check-i18n-catalog.mjs` → passes (4 locales, all keys).
- `bun run build:web` → succeeds; committed `vendor.bundle.js` unchanged
  (the highlighter is a plain, unbundled script).
- `npx playwright test --config tests/e2e/playwright.local.config.js issue-330` →
  3/3 pass.
- Visual evidence: [`docs/screenshots/issue-330-message.png`](../../screenshots/issue-330-message.png).

---

## 8. Follow-up from the PR review — teach the novice (R9)

After the presentation-layer work landed, the reviewer expanded the scope in a
PR comment. The concrete, directly-actionable part:

> We need to give all the exact instructions on how to test the code snippet.
> If the message contains code, by default we should assume a novice user. We
> should also explain how the code works. So the code, the explanation, and the
> instructions on how to test it should all be provided when working with code.
> If we already provided instructions earlier in the dialog, we can omit them
> when only code changes are requested.

### R9 — Code answers must explain and instruct · **done**

The `write_program` answer in `src/engine.rs` now renders **three** parts
instead of one, all localized for the four supported response languages
(en/ru/hi/zh):

1. **The code** — the fenced program (unchanged) plus the existing localized
   execution report.
2. **"How it works"** — `program_explanation_section` / `program_explanation`
   give a plain-language description of the algorithm for each task
   (`hello_world`, `count_to_three`, `list_files`, `list_files_arg`). This is
   *reasoned from the task*, not memorized per sample: every language's template
   implements the same algorithm, so one localized explanation per task covers
   all of them (satisfies R6's "general, not hard-coded" requirement).
3. **"How to test it yourself"** — `program_test_instructions` emits numbered,
   novice-friendly steps: install the toolchain (`setup_hint`), save the snippet
   to the conventional file name (`save_as`), compile it (when the language has a
   `check_command`), run it (`run_command`), and compare the output against the
   expected-output section shown above.

Two new fields back this on the per-language catalog in
`src/engine_hello_world.rs`: `save_as` (e.g. `main.rs`, `Main.java`,
`Program.cs`) and `setup_hint` (a short, novice-friendly pointer to the
toolchain installer). They are populated for all 11 catalog languages.

**History-aware brevity.** `src/solver.rs` inspects the conversation history: if
an earlier *assistant* turn already presented a fenced code block, the answer is
built with `prior_code_response = true` and the verbose setup steps collapse to
a single concise note — e.g. *"Test the updated program the same way as before:
save the code to `main.rs` and run `…` again."* This implements the reviewer's
"if we already provided instructions earlier, we can omit them when code changes
are requested." It is detected from the dialog, not hard-coded to a turn number.

Seven unit tests in `tests/unit/specification/code_generation.rs` pin the
behavior: each of en/ru/hi/zh asserts both the explanation heading and the
test-instruction steps appear; one covers an unavailable-language path (Ruby);
one asserts a follow-up edit omits the setup steps; and one asserts the first
turn keeps the full instructions.

### Files changed (R9)

| File | Change |
| --- | --- |
| `src/engine.rs` | `write_program_answer` gains a `prior_code_response` flag and appends `program_explanation_section` + `program_test_instructions`; new localized explanation/instruction builders. |
| `src/engine_hello_world.rs` | `save_as` + `setup_hint` fields added to `ProgramLanguage` and populated for all 11 languages. |
| `src/solver.rs` | Detects a prior assistant code block in `history` and threads `prior_code_response` into the answer builder. |
| `tests/unit/specification/code_generation.rs` | 7 new tests across all languages + history-aware behavior. |

---

## 9. Deferred large-vision items from the PR comment (honest scope note)

The same PR comment also sketched a much larger, multi-month architectural
vision. These are **not** implemented in PR #331 — recording them here honestly
rather than claiming false completeness. They belong to dedicated issues:

- **Reasoning expressed as link-substitution rules over `doublets-rs`**, with a
  Turing-complete substitution-rule layer convertible to Rust / JS / WASM.
- **Isolated JavaScript evaluation** of generated snippets in the browser.
- **In-browser Rust → WASM compilation** to actually run generated Rust.
- **A Linux VM in the browser** via `link-foundation/rust-web-box`.
- **Dockerized execution** via `link-foundation/box` / `link-foundation/start`
  with detached containers, snapshots, command replay, and zip-archive
  restoration by default.
- **Actually executing the generated tests** to display real (not described)
  output, plus integration tests and coding / Q&A benchmarks that measure
  *reasoning* rather than memorization.

The R9 work above deliberately scopes to what is testable and directly serves a
novice today — *code + explanation + test instructions* — and leaves the
execution-sandbox vision to follow-up issues so each can be designed and
reviewed on its own merits.
