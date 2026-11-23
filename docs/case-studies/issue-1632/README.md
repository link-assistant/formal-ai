# Issue #1632 · Verified instructions for a short Russian how-to

On November 23, 2026, the reporter sent `привет` and then `Как какать?` to Formal AI 0.356.10 through OpenCode on macOS. The greeting worked. The next reply described the internal lookup pipeline and the reasoning text claimed the answer had been found. The reporter identified the relevant Russian wikiHow article and asked for a general solution in one pull request. GitHub listed no required sub-issues. [Issue report](https://github.com/link-assistant/formal-ai/issues/1632) · [Maintainer feedback](https://github.com/link-assistant/formal-ai/issues/1632#issuecomment-7112155513) · [Article](https://ru.wikihow.com/хорошо-покакать)

The report links the raw [OpenCode transcript](https://gist.github.com/konard/e9fb4adae6934c8cd18377bd8e18e912), [server transcript](https://gist.github.com/konard/0c6d9535710f21dc91c4de8bf6700266) and [merged context](https://gist.github.com/konard/250d60390b0c6aefbdf56d716711da9a). The [GitHub metadata and comments](raw-data/) record the initial report, follow-up and empty sub-issue list. The earlier automatic drafting attempt ended without a verified draft. Its log reported a shared diagnostic use-site check, and provided no successful user-facing replay. [Draft attempt](https://github.com/link-assistant/formal-ai/actions/runs/45252675121)

## Why the reply failed

The exact-title MediaWiki request returned `missingtitle`. The next request, MediaWiki full-text search, returned HTTP 500. The prior regression supplied a successful full-text response, so it exercised an existing candidate and archive path while skipping the publisher failure. Live probing showed that wikiHow's `Special:LSearch` endpoint still returned the article and its structured parse still contained the instructions. Under the old extraction order, the real article's first section also supplied constipation remedies to an unqualified question. The everyday instructions are in a later section.

The repair builds on the localized source resolution from [PR #1398](https://github.com/link-assistant/formal-ai/pull/1398) and bounded publisher verification from [PR #1622](https://github.com/link-assistant/formal-ai/pull/1622). It reuses the source registry, title relevance rules, capture transport, browser capture component and guide provenance. The language's existing aspect policy expands the search while relevance checks keep the original task. Publisher results contribute steps only after a same-origin candidate's structured article independently passes verification. Section selection uses shared language policy and preserves top-level steps, including supporting details as part of their parent step.

## Acceptance and verification

| Requirement | Verification |
| --- | --- |
| Keep the greeting and answer the next turn using the reported source | `issue-1396-native.test.mjs` replays both turns over Chat Completions in JavaScript and the native CLI. It checks the greeting, `[DONE]`, complete final Markdown, and tool-call behavior. |
| Find instructions when the exact title and structured search fail | `procedural-runtime-parity.test.mjs` uses the observed API failure and checks the ordered request sequence, publisher URL, capture hash, offline cache and source opt-out. The native replay uses only the publisher result and article capture. |
| Generalize without a stored prompt or a stored article title | A held-out Russian verb with a required subject checks discovery and rejects foreign, negated and unrelated titles before fetching. The browser component test covers a blocked direct fetch. |
| Use the appropriate method for the request | Both runtime suites cover a remedy section before everyday instructions and an explicit request that selects the scoped section. The nested detail remains within its own numbered step. |
| Keep chat readable and honest about missing evidence | Chat emits instructions and their citation when verified, or the localized evidence gap. Technical retrieval details remain in trace records. `procedural-runtime-parity.test.mjs` replays the EN, RU, HI, ZH and ES assertions against JavaScript. |
| Verify the live client path and deliver together | The [OpenCode experiment](../../../experiments/issue-1632-opencode-replay.mjs) runs both requests in one OpenCode session against the JavaScript server with live source access. Its assertions identify the everyday method from the publisher. The native replay runs in the pull request's native check. [PR #1633](https://github.com/link-assistant/formal-ai/pull/1633) |

The [live streaming HTTP replay](raw-data/live-http.json) preserved the greeting and returned three everyday steps with the article citation. It recorded the exact-title response, the HTTP 500 from MediaWiki search, and successful responses from the public publisher search and the verified article parse.

Run the deterministic browser and HTTP replays one file at a time.

```bash
node --test --test-concurrency=1 rust/tests/web/procedural-runtime-parity.test.mjs
node --test --test-concurrency=1 rust/tests/web/issue-1396-native.test.mjs
```

`FORMAL_AI_TEST_NATIVE_CLI` points the HTTP suite to a built native executable. The live experiment additionally requires OpenCode on `PATH` and publisher access. The committed fixture provides controlled HTML methods and reuses the earlier checked-in Russian steps.
