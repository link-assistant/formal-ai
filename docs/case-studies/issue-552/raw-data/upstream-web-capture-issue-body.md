# Context

formal-ai issue https://github.com/link-assistant/formal-ai/issues/552 asks for automated replay of shared AI dialogs, including:

- https://chatgpt.com/share/6a3825b9-8de4-83ee-9c24-52fd1eb38d24
- https://share.google/aimode/VG0HhpnAXrBkC0QgP

The formal-ai PR can parse the captured ChatGPT static HTML and export it as `demo_memory`, but this is provider-specific code living in the wrong layer. Google AI Mode static capture currently returns an interstitial/challenge page rather than transcript data.

Case-study data is preserved in formal-ai at `docs/case-studies/issue-552`.

# Request

Add a browser-backed shared-dialog capture mode to web-capture that normalizes supported shared AI conversation URLs into a structured source description.

Suggested CLI shape:

```bash
web-capture shared-dialog <url> --format meta-language
web-capture shared-dialog <url> --format demo-memory
```

# Desired Output

For supported captures, emit structured data with:

- provider (`chatgpt`, `google_ai_mode`, etc.)
- source URL
- capture method (`static_http`, `browser`, etc.)
- conversation id/title when available
- ordered turns with ids, roles, content, visibility, and source evidence
- capture diagnostics and warnings

For unsupported captures, emit a structured reason instead of guessing:

- provider challenge/interstitial
- login required
- deleted/expired share
- no transcript in captured DOM
- unsupported provider format

# Observed Behavior

- ChatGPT static HTML includes streamed data containing `linear_conversation`; visible user/assistant turns can be extracted.
- Google AI Mode static capture of the provided URL does not include a transcript and needs browser-backed capture or an explicit unsupported-capture diagnostic.

# Acceptance Criteria

- Capturing the ChatGPT URL returns the four visible turns preserved in formal-ai issue #552.
- Capturing the Google AI Mode URL either returns a transcript from a browser-backed capture or a structured unsupported diagnostic that names the actual blocker.
- Output uses or maps cleanly to the shared schema requested in link-foundation/meta-language.
- Capture results include enough evidence for formal-ai to build replay tests without memoizing one URL.
