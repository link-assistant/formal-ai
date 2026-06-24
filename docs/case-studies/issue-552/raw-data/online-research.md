# Issue #552 Online Research Notes

## ChatGPT Shared Links

OpenAI documents shared links as unique URLs for ChatGPT conversations, created
from ChatGPT web or apps and intended to share a conversation snapshot:
https://help.openai.com/en/articles/7925741-chatgpt-shared-links-faq

Implementation note: the captured HTML for the provided URL contains streamed
React Router data with a `linear_conversation` object. The current converter is
therefore a provider-specific static extractor for this observed shape.

## Google AI Mode Sharing

Google documents AI Mode as a Search experience and documents that
AI-powered responses can be shared from Search:

- https://search.google/ways-to-search/ai-mode/
- https://support.google.com/websearch/answer/16517651?co=GENIE.Platform%3DAndroid&hl=en

Implementation note: the static capture of the provided Google AI Mode URL did
not include a transcript. It returned a Google Search interstitial/challenge
with an enable-JavaScript retry path and a fallback Search query. Browser-backed
capture support is needed before formal-ai can replay this class reliably.

## Serialization Shape

The ChatGPT capture uses an indexed serialized payload rather than plain JSON
objects. The shape is similar to devalue-style serialization where references
inside an array reconstruct object graphs:
https://github.com/sveltejs/devalue

The parser implemented here is intentionally narrow: it resolves the observed
indexed object/string references enough to recover `linear_conversation`, not a
general devalue runtime.

## web-capture

`link-assistant/web-capture` is a CLI/microservice for fetching URLs and
rendering HTML to Markdown:
https://github.com/link-assistant/web-capture

This is the correct upstream layer for provider-specific capture, especially
for Google AI Mode where static HTTP capture does not expose transcript data.

## meta-language

`link-foundation/meta-language` describes itself as a language about languages:
https://github.com/link-foundation/meta-language

formal-ai already depends on `meta-language = "0.45.0"`. A shared-dialog schema
belongs there so web-capture and formal-ai use the same source-description
model instead of drifting through local structs.
