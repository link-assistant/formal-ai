# Issue #1591 concept-link embeddings

The JavaScript server at `c0f7515d6bfe703cdde1f2e4852ba485e7c64a7f`
returned HTTP 404 and `route not found` for the issue's request with the
`formal-ai` model and the inputs `warehouse supervisor` and `forklift
operator`. Neither server registered an embeddings route. The HTTP regression
test reproduces that request, checks the advertised dimension and verifies
similarity for English synonyms and Russian translations after the route is
registered.

## Formal AI development attempts

Agent CLI 0.27.0 selected `formalai/formal-ai` on a local JavaScript server
reporting version `0.356.1`. Each attempt used an isolated local workspace.
The captured stream events include the model selection, complete responses,
and any tool use and results.

| Ask | Expected result | Observed result |
| --- | --- | --- |
| Read `js/server/routes.mjs` and identify the handler for `GET /v1/models`. | Identify `handleModels`. | The server declined tool use without agent mode. [Stream](raw-data/route-without-tools.jsonl) |
| With agent mode enabled, read `dispatch.mjs` and identify the function registered for the models route. | Identify `handleModels` from the route dispatch table. | The agent read the file and returned its contents, including the relevant line, without identifying the function in its answer. [Stream](raw-data/route-with-tools.jsonl) |
| Write one Added changelog bullet describing the endpoint, multilingual vectors, and advertised dimensions in `changelog.md`. | Create the changelog entry. | The agent returned an explanation of its runtime. The workspace had no new file. [Stream](raw-data/changelog-draft.jsonl) |

One route-identification reasoning unit was attempted and zero were accepted.
The writing attempt produced zero accepted edits. The implementation and
changelog were authored by Codex.
