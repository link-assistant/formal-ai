# Issue 140 Case Study: Prefilled `Report issue` URL still overflows on long Cyrillic chats

## Summary

Issue [#140](https://github.com/link-assistant/formal-ai/issues/140) is the
follow-up to [#78](https://github.com/link-assistant/formal-ai/issues/78). The
compact `U:` / `A:` dialog block from #78 was supposed to keep the prefilled
GitHub **Report issue** link under GitHub's request-URL limit, but the
reporter ran into the same `Whoops, something went wrong.` / `Whoa there!
Your request URL is too long.` GitHub error pages — this time with a longer
chat in Russian and a 20-line User Context block. The attached
[`raw-data/reported-prefill-url.txt`](./raw-data/reported-prefill-url.txt)
captured the failing URL at **15 227 bytes** — nearly twice GitHub's
documented 8192-character cap, and that was with only one short user
message (`ва`) followed by a single unknown-rule assistant reply repeated
many times in the conversation history.

The reporter (krusalovorg) noted: *"with large chats, it is not possible to
create an issue"*. The maintainer (konard) then itemised exactly what to
trim and asked for smart message truncation:

> Also we should get to know what is exactly the max size of GitHub
> prefilled issue URL, and omit first messages, except last two; if user
> and agent last messages contain multiple lines or single very long line
> we should put `... omitted X lines ...` for multiline or
> `... omitted X characters ...` for single line. So for the last two
> messages we always see start and end of the messages.

The fix ships in PR [#158](https://github.com/link-assistant/formal-ai/pull/158)
on branch `issue-140-f298777c7c24`.

## Collected Data

Fresh GitHub evidence lives in [`raw-data/`](./raw-data) so future analysts
can replay the investigation without re-querying the API:

- [`raw-data/issue-140.json`](./raw-data/issue-140.json) — full issue body
  and metadata captured with `gh issue view`.
- [`raw-data/issue-140-comments.json`](./raw-data/issue-140-comments.json) —
  conversation comments on the issue (the maintainer's punch-list of
  shrinks lives here).
- [`raw-data/reported-prefill-url.txt`](./raw-data/reported-prefill-url.txt)
  — the failing 15 227-byte URL that the reporter attached to the issue,
  preserved verbatim so we can re-measure and re-test against it.
- [`raw-data/pr-158.json`](./raw-data/pr-158.json) — pull request metadata
  for the draft that implements this case study.
- [`raw-data/pr-158-conversation-comments.json`](./raw-data/pr-158-conversation-comments.json),
  [`raw-data/pr-158-review-comments.json`](./raw-data/pr-158-review-comments.json),
  [`raw-data/pr-158-reviews.json`](./raw-data/pr-158-reviews.json) — PR
  review + discussion records.

## Prior Case Studies

This work builds on the **Report issue** scaffolding tracked in earlier
case studies and extends it; it does not replace the existing flow.

- [`../issue-10/README.md`](../issue-10/README.md) — introduced the
  original prefilled `Report issue` link.
- [`../issue-18/README.md`](../issue-18/README.md) — added the
  `## Attach full memory` block.
- [`../issue-44/README.md`](../issue-44/README.md) — added the
  `Unknown prompt: …` issue title (which is the title surface that #140
  exercises).
- [`../issue-78/README.md`](../issue-78/README.md) — switched the dialog
  transcript to a single fenced code block and moved the memory-upload
  walkthrough into [`docs/upload-memory.md`](../../upload-memory.md). #140
  is the direct sequel: the dialog block is already compact, so the
  remaining overflow comes from the User Context block plus the per-chat
  transcript volume.
- [`../issue-94/README.md`](../issue-94/README.md) — introduced the
  per-issue User Context block (UI/Browser/locale/viewport/etc.) that
  #140 trims.

## Timeline of Events

| Timestamp (UTC) | Event |
| --- | --- |
| 2026-04-19 | Issue #78 lands and switches the prefilled body to the compact `Legend: U / A` dialog block, dropping per-message Markdown subsections. |
| 2026-05-08 | Issue #94 lands and adds the per-issue `## User Context` section (20 lines: UI Language, Browser Languages, Color Scheme, Viewport, Screen, Platform, Online, Preferred Location, Location Inference, etc.) — the new dominant contributor to URL length. |
| 2026-05-19 15:06:57 UTC | krusalovorg loads the demo in Russian (`ru-RU`), types a one-character prompt (`ва`), receives the *"I can't yet answer that from local Links Notation rules"* reply, repeats the same prompt many times, and clicks **Report issue**. The browser navigates to a 15 227-byte URL. |
| 2026-05-19 15:?? UTC | GitHub responds with the *"Whoa there! Your request URL is too long."* / *"Whoops, something went wrong."* error pages — the same family of failures originally documented under #78. The reporter saves the failing URL as `url.txt`. |
| 2026-05-19 15:08:20 UTC | krusalovorg files issue #140, attaching `url.txt` and reporting *"with large chats, it is not possible to create an issue."* |
| 2026-05-19 17:58:16 UTC | konard adds the comment that distils the punch list: combine the language fields, combine the UI fields, drop UI Skin / Chat Style / Composer Style / Composer Action / Online, omit unset fields like Preferred Location, shorten Location Inference, find the exact URL cap, and smart-truncate the dialog (last two messages, `... omitted X lines ...` / `... omitted X characters ...`). |
| 2026-05-19 18:?? UTC | AI issue solver claims branch `issue-140-f298777c7c24` and opens draft PR #158. |
| 2026-05-19 19:?? UTC | Case study created under `docs/case-studies/issue-140/`; raw GitHub data and the failing URL archived. |

## Reproducing the Bug

The shortest reliable reproduction (matches what the reporter did):

1. Open [https://link-assistant.github.io/formal-ai](https://link-assistant.github.io/formal-ai)
   in a browser with a non-ASCII default locale (the original report was
   `ru-RU`; Cyrillic content triples URL bytes per character once encoded).
2. Switch to manual mode.
3. Send a short unknown-rule prompt (e.g. `ва` or `xxxxx`) at least 10
   times so the assistant keeps replying with the long Russian unknown
   answer (`Я пока не могу ответить на это по локальным правилам Links
   Notation. Добавьте факт или правило в Links Notation и повторите
   запрос.`).
4. Click **Report missing rule** on the last assistant message.

**Expected**: GitHub renders the new-issue form with title and body
prefilled.

**Observed (before this PR)**:

- desktop: *"Whoa there! Your request URL is too long."* (Apache-style
  `HTTP 414` returned by GitHub's front edge);
- mobile: *"Whoops, something went wrong!"* (GitHub's generic 5xx page).

In both cases the user cannot file the issue. With Cyrillic content this
is reached around the 3rd–5th turn; with ASCII content it stretches to
~10–12 turns but still happens for any real-world session.

To re-measure the original failing URL:

```sh
wc -c docs/case-studies/issue-140/raw-data/reported-prefill-url.txt
# 15227
```

## Root Cause Analysis

### 1. The verbose `## User Context` block

Issue #94 added the User Context block to give maintainers reproducible
locale / viewport / platform information without forcing users to attach a
full memory dump. It legitimately fixed a different problem (#94), but the
shape has grown to 20 separate Markdown bullet lines that always render
even when the underlying value is a default or unset:

```
- **UI Language**: ru
- **UI Language Preference**: auto
- **Theme Preference**: auto
- **UI Skin**: flat
- **Chat Style**: cards
- **Composer Style**: flat
- **Composer Action**: attach
- **Browser Language**: ru
- **Browser Languages**: ru, en-US, en, ru-RU
- **Locale**: ru
- **Time Zone**: Europe/Samara
- **Color Scheme**: light
- **Preferred Location**: not set
- **Guess Probability**: 100%
- **Temperature**: 0.7
- **Viewport**: 1536x730
- **Screen**: 1536x864 @1.25x
- **Platform**: Windows
- **Online**: yes
- **Location Inference**: time zone / locale only; exact geolocation was not requested
```

Each Markdown bullet costs ~5–6 extra bytes once URL-encoded (`-+**…**%3A+`),
so a 20-bullet block can carry 100–120 bytes of label-only overhead before
the values. With the Cyrillic locale label encoded as `%D1%80%D1%83…`, the
User Context block alone passes 1 800 bytes URL-encoded — and the
maintainer's punch list confirmed that most of those lines are not
informative without an accompanying full-memory upload anyway.

### 2. The full conversation history is always serialised

`createIssueReportBody` walks every message in the chat regardless of how
long the conversation has grown. The reporter's failing URL embeds a
dozen turns of the same unknown-rule exchange. Each `A (intent: unknown,
reported):` body is 132 characters URL-encoded (`%D0%AF+%D0%BF%D0%BE…` —
Cyrillic costs 4–6 bytes per character once encoded). After 5 such turns
the dialog alone exceeds the 8 KB cap.

### 3. The duplicated `User Agent` line

`User Agent` was rendered twice: once in `## Environment` (added in #94)
and once implicitly when the User Context block lists Viewport, Screen,
and Platform. The encoded user-agent string is ~250 bytes by itself.

### 4. No URL-length awareness

`createIssueUrl` was a one-shot template — it built the body and shipped
the resulting URL. Nothing checked GitHub's documented 8192-character
ceiling, so the failure mode was binary: either the URL happened to fit,
or GitHub's front edge served an error page that hid the maintainer's
actual issue.

## Requirements

Distilled from the issue body + konard's punch-list comment, extending the
matrix tracked in [REQUIREMENTS.md](../../../REQUIREMENTS.md).

| ID | Requirement |
| --- | --- |
| R140-A | The prefilled `Report issue` body must be measured against the documented GitHub `?title=&body=&labels=` URL cap (8192 bytes) and, when the candidate URL exceeds the cap, must be progressively shrunk so GitHub never serves *"Whoa there!"* / *"Whoops"* for typical sessions. |
| R140-B | Browser language metadata must collapse into one line that marks the active language, e.g. `UI languages: *ru*, en-US, en, ru-RU`, instead of three separate UI-Language / Browser-Language / Browser-Languages lines. |
| R140-C | UI environment metadata (viewport, screen, user agent, platform) must collapse into one line, e.g. `UI: 1536x730 viewport, 1536x864 @1.25x screen, … browser, Windows platform`, instead of four separate bullets. |
| R140-D | UI Skin, Chat Style, Composer Style, and Composer Action must be **omitted** from the prefilled body — those preferences only matter when the user also attaches a full-memory export. |
| R140-E | Empty / not-set fields (e.g. `Preferred Location: not set`) must be omitted entirely, not rendered as `not set`. |
| R140-F | The `Online: yes` line must be dropped — it is useless for issue triage. |
| R140-G | The `Location Inference` line must be simplified to *"inferred from `<source>`"* instead of the long explanatory sentence. |
| R140-H | The `## Environment` block must not duplicate `User Agent`; the canonical home for the UA string is the combined User Context UI line (R140-C). |
| R140-I | When the URL still overflows after the field consolidations, the dialog block must keep the **last two messages** intact in shape and replace earlier ones with a single `... omitted N earlier messages ...` marker. |
| R140-J | When even the last two messages overflow, **each** retained message must be truncated so the URL fits: a multi-line message becomes `<first>\n... omitted N lines ...\n<last>` and a single very long line becomes `<head>... omitted N characters ...<tail>`. The start and end of each preserved message must always be visible. |
| R140-K | A case study under `docs/case-studies/issue-140/` must reconstruct the timeline, the requirements, the root causes, and the solution plan, with the raw GitHub data, the reporter's failing URL, and online research about the GitHub URL cap archived under `raw-data/`. |
| R140-L | If similar problems were already solved by an off-the-shelf component, the case study must document why we are not reusing it. |
| R140-M | The e2e suite must enforce all of the above: the new compact User Context shape, the absence of dropped fields, and that the prefilled URL stays below 8192 bytes even for long dialogs. |

## Root Cause Sketch

```
  Report issue click
        │
        ▼
  createIssueUrl
        │
        ▼
  createIssueReportBody
   ├── ## Environment            (~250B URL-encoded, +250B for User Agent dup)
   ├── ## User Context           20 lines × ~80B encoded = ~1 600–2 000B
   ├── ## Dialog                 N turns × ~150B encoded; Cyrillic ~4–6× ASCII
   ├── ## Reproduction Steps     ~150B
   ├── ## Description            ~80B
   └── ## Attach full memory     ~400B (the memory upload pointer)
        │
        ▼
  URLSearchParams.toString()      → single ?body=… query parameter
        │
        ▼
  https://github.com/.../issues/new?…   ← 8192-byte cap on the request line

  When the encoded body > ~7 700 bytes (leaving room for ?title=…&labels=bug),
  GitHub's front edge rejects the request before the new-issue form ever
  renders.
```

## Solution Plan

### R140-B + R140-C + R140-D + R140-E + R140-F + R140-G + R140-H — compact User Context

- Add small formatters to [`src/web/app.js`](../../../src/web/app.js):
  - `formatUiLanguagesField(active, browserLanguagesStr)` collapses the
    UI-language fields into one line that wraps the active language in
    `*…*` (`*ru*, en-US, en, ru-RU`). The fallback path handles a missing
    active language and an empty browser-languages list.
  - `formatUiField(context)` joins viewport, screen, user agent, and
    platform with the qualifiers `viewport`, `screen`, `browser`,
    `platform` to keep the line self-describing (`1536x730 viewport,
    1536x864 @1.25x screen, Mozilla/5.0… browser, Windows platform`).
  - `formatLocaleField(context)` collapses locale + time zone:
    `ru (Europe/Samara)`.
  - `formatThemeField(context)` collapses theme preference + active
    color scheme (`auto (dark)` when the OS resolves the preference).
- Rewrite `appendUserContextBlock` to push only the non-empty entries,
  drop UI Skin / Chat Style / Composer Style / Composer Action /
  Online, drop `Preferred Location` when it is unset, and write the
  inference as `inferred from <source>` using the part of the existing
  inference sentence up to the first semicolon (i.e. reuse the same
  source description without the appended caveat).
- Add `userAgent` to the `collectUserContext` return value so the new
  `formatUiField` can read it directly, and remove the `User Agent` line
  from `## Environment` to avoid the duplicate.

### R140-A + R140-I + R140-J — URL fitter with progressive shrinks

- Add `GITHUB_URL_MAX_LENGTH = 8192` and a 16-byte safety margin
  (`URL_SAFETY_MARGIN`) to leave room for the `&labels=bug` tail and any
  encoding round-trips. The fitter targets
  `URL_BUDGET = GITHUB_URL_MAX_LENGTH - URL_SAFETY_MARGIN = 8176`.
- Add `truncateSingleLine(str, max)` and `truncateMessageContent(str, max)`
  that produce the `... omitted N characters ...` / `... omitted N lines
  ...` markers and always preserve the start and end of each preserved
  message.
- Add `buildIssueUrl(title, body, labels)` that wraps `URLSearchParams`,
  so any URL-encoded entity is counted by the JavaScript engine itself
  rather than estimated.
- Add `fitIssueUrl(context, buildBody)` that:
  1. Tries the full transcript (`strategy: full`). Returns immediately if
     the URL is already within budget.
  2. Drops every message except the last two and adds an
     `... omitted N earlier messages ...` line to the dialog code block.
  3. If the body still does not fit, halves a per-message character
     budget over [4096, 2048, 1024, 512, 256, 128, 64, 32] and applies
     `truncateMessageContent` to each retained message until the URL
     fits. The exponential backoff converges in at most eight iterations
     without an open-ended loop.
- Hand the `earlierOmitted` count through `appendDialogBlock` so the
  fitter can drive the user-visible "omitted N earlier messages" line.
- Rewrite `createIssueUrl` as a thin wrapper that delegates to
  `fitIssueUrl`.

### R140-K — case study & raw data

- Land this file at
  [`docs/case-studies/issue-140/README.md`](./README.md).
- Archive `issue-140.json` + `issue-140-comments.json` (full issue +
  comments JSON returned by `gh issue view`) and the corresponding
  PR-158 JSON files under [`raw-data/`](./raw-data/).
- Archive the reporter's failing URL verbatim as
  [`raw-data/reported-prefill-url.txt`](./raw-data/reported-prefill-url.txt).
  `wc -c` confirms it is 15 227 bytes — 1.86× the cap.

### R140-L — existing components & prior art

- We do **not** introduce a URL shortener / paste bin / compression
  layer. The maintainer's punch list is explicit about what to trim;
  re-using a third-party service would add a dependency the maintainer
  has to trust and complicate redaction (the body contains the user's
  chat). The existing dialog code-block from #78 plus the new fitter is
  enough.
- We do **not** reuse the browser's `History.scrollRestoration` /
  `Document.fragment` tricks people sometimes recommend for long
  pre-fills, because GitHub only consumes the URL query parameters and
  ignores the fragment.
- We deliberately keep `URLSearchParams` (built-in) instead of pulling
  `qs` or `query-string` — both would add ~3–5 KB of JS for the same
  output.
- We reuse the `pickDialogFence` helper introduced in #78 verbatim, so
  user content that itself contains triple-backtick code blocks does not
  break the wrapping fence.

### R140-M — regression coverage

- [`tests/e2e/tests/demo.spec.js`](../../../tests/e2e/tests/demo.spec.js):
  - The *"issue reports include UI, browser, and coarse location
    context"* test (originally added for #94) now asserts the new
    compact `**UI languages**`, `**Theme**`, `**UI**`, `**Locale**`, and
    `**Location**` shapes, and asserts the old `**UI Language**`,
    `**Browser Languages**`, `**Color Scheme**`, `**Time Zone**`,
    `**Location Inference**`, `**Online**`, `**Viewport**`,
    `**Screen**`, `**Platform**` labels are **absent**.
  - The *"unknown prompts include a prefilled missing-rule issue
    link"* test asserts the `## Environment` block no longer carries a
    standalone `**User Agent**` line (it is now inside the User Context
    `**UI**` field) and that `href.length` is below 8192.
  - A new *"prefilled issue URL stays below GitHub 8KB cap with a long
    dialog"* test sends a dozen short Cyrillic-ish prompts in manual
    mode and confirms that (a) `href.length <= 8192` and (b) the body
    contains one of the omission markers
    (`omitted N earlier messages | lines | characters`).

- Smoke: [`experiments/issue-140-prefilled-url-budget.mjs`](../../../experiments/issue-140-prefilled-url-budget.mjs)
  mirrors the JS helpers and prints `URL length`, `Strategy`, and
  `Fits under cap: YES/NO` for four fixtures (empty, 20-turn unknown
  loop, 200-turn unknown loop, multi-line + huge single-line). All four
  fit under the 8192-byte cap with strategies `full`, `last-two`,
  `last-two`, and `truncated-4096` respectively.

## Existing Components and Prior Art

- The `?title=` / `?body=` / `?labels=` URL parameters for prefilled
  GitHub issues are documented in [GitHub Docs — Creating an issue from
  a URL query](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue#creating-an-issue-from-a-url-query).
  GitHub Docs explicitly notes the **8192-character URL maximum**, which
  matches the request-line cap Apache/nginx enforce by default
  (`LimitRequestLine` is 8190 bytes). That is the cap R140-A is built
  against.
- The `URLSearchParams` Web API is the right tool to compute the encoded
  length: it round-trips correctly with GitHub's parser and counts every
  percent-encoded byte the same way the network stack will.
- The dialog code-block formatter from #78 (`pickDialogFence`,
  `appendDialogBlock`) is reused verbatim — only the new
  `earlierOmitted` parameter is added.
- The User Context formatter `collectUserContext` from #94 is preserved;
  only the rendering changes. Existing tests that exercise the data
  surface (e.g. *"auto-detects Russian UI language from browser
  preferences"*) keep passing.
- We considered using
  [`pako`](https://github.com/nodeca/pako) to gzip the body and base64
  it, but the result is opaque to maintainers and still risks the cap
  for sufficiently long dialogs. We picked the maintainer-friendly path
  (drop low-value fields, truncate with visible markers) over byte-level
  compression.

## Online Research

- [GitHub Docs — Creating an issue from a URL query](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue#creating-an-issue-from-a-url-query)
  documents the `?title=` / `?body=` / `?labels=` parameters and states
  **"The maximum size of a URL is 8192 characters."** This is the
  authoritative cap referenced from R140-A.
- [HTTP/1.1 — RFC 7230 §3.1.1](https://datatracker.ietf.org/doc/html/rfc7230#section-3.1.1)
  recommends that servers support a request line of at least 8000 bytes;
  Apache `LimitRequestLine` defaults to 8190 and nginx
  `large_client_header_buffers` defaults to 4×8KB. GitHub's edge runs at
  the conservative 8192 boundary, which is consistent with the
  *"Whoa there!"* page our reporter saw.
- [`URLSearchParams` — MDN](https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams)
  confirms that `URLSearchParams` encodes Unicode using
  `application/x-www-form-urlencoded` rules (`+` for space, `%HH` for
  bytes), which means our `body.length` test in
  `buildIssueUrl` is byte-accurate for the multi-byte Cyrillic the
  reporter triggered (`ва` → `%D0%B2%D0%B0`, 12 bytes for 2 characters).

## Upstream Reports

This is a GitHub-side cap, not a bug in any third-party library: there
is no actionable upstream issue to file. The cap is documented and
intentional. If GitHub ever relaxes the limit, the fitter's first
strategy (`full`) will start returning straight away and no further
truncation will trigger, so the fix degrades gracefully.

## Verification

- Manual:
  1. Run the demo locally
     (`npm --prefix tests/e2e run check:i18n &&
       npx serve src/web -l 0.0.0.0:4173`).
  2. Switch to manual mode, send 10+ short prompts in `ru-RU`, click
     **Report missing rule**. Confirm the GitHub new-issue form opens
     (no *"Whoa there!"*).
  3. Inspect the body: User Context has 5–7 lines, `## Environment`
     has no `**User Agent**` line, dialog ends with
     `... omitted N earlier messages ...` for long sessions.
- Automated:
  - Playwright *"issue reports include UI, browser, and coarse location
    context"* in
    [`tests/e2e/tests/demo.spec.js`](../../../tests/e2e/tests/demo.spec.js).
  - Playwright *"unknown prompts include a prefilled missing-rule issue
    link"* with the new `href.length <= 8192` assertion.
  - Playwright *"prefilled issue URL stays below GitHub 8KB cap with a
    long dialog"* — new, exercises the `last-two` /
    `truncated-<budget>` paths.
- Smoke: `node experiments/issue-140-prefilled-url-budget.mjs` prints
  `URL length`, `Strategy`, `Fits under cap: YES/NO` for four fixtures.
  All four fit.
- Live browser: after driving the demo through 15 turns of the short
  Cyrillic prompt `ва` in manual mode, the **Report issue** link on the
  last assistant message resolved to a **2 270-byte** URL whose body
  begins with `... omitted 32 earlier messages ...` and ends with the
  last user/assistant pair, intact. Captured in
  [`screenshots/after-15-turn-cyrillic.png`](./screenshots/after-15-turn-cyrillic.png).

## Status

Implemented in PR [#158](https://github.com/link-assistant/formal-ai/pull/158).
