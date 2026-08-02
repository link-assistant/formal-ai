# Root-cause analysis

## Observed control flow

The agentic planner receives the latest user message and the client's advertised
tool names. It probes bounded recipes, then write/edit/read, and finally fetch and
search intent. A probe must both classify intent and compose schema-compatible
arguments; returning prose or `None` means the CLI never gets a tool call.

## Failure classes

1. **Fetch had two disconnected concepts.** HTTP-fetch cues were considered,
   while the existing URL-navigation vocabulary was ignored by the agentic route.
   The request already contained the strongest argument evidence: a normalized
   HTTP(S) URL.
2. **Search conflated vocabulary with sentence templates.** The first draft added
   the three issue sentences to `web_search_explicit_prefix`. This passed the
   report but could not generalize. The real frames are (a) strong search action
   plus topic, (b) interrogative plus external source plus topic connective, and
   (c) external source plus recency plus topic connective.
3. **Edit target extraction assumed one word order.** It required a target cue
   before the path and selected the first edit action. Leading-action requests
   instead put the action next to the path and use a later action to introduce
   old/new spans.
4. **Write classification and composition could disagree.** The write parser was
   seed-driven, but file-read exclusion had a separate hardcoded verb table. The
   declarative `contents:` surface was absent, so read could claim the filename.

## Corrective invariants

- Classification is accepted only when the same parser can produce the tool's
  typed arguments.
- Lexical growth is a seed edit; Rust/JavaScript implement role-and-slot logic.
- Source evidence uses boundary-preserving seed forms (`web`, not `webhook`).
- Native and browser paths mirror the same semantic frames.
- Learned amendments are persistent and ranked, but promotion is review-gated
  by the original matrix plus unseen paraphrases.

## Rejected alternatives

- Adding all reported sentences as explicit prefixes: correct only for the
  finite report and violates the issue's requested generalization.
- Routing every message containing a URL to fetch: would steal informational
  references where no navigation/fetch action is requested.
- Automatically appending failed phrases to the seed: a single noisy observation
  could broaden permissions. The learning report proposes a reusable semantic
  amendment and waits for tests and human review.
