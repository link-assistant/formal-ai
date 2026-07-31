# Research notes

Captured 2026-07-30 from primary project/specification sources.

| Source | Observation | Decision |
| --- | --- | --- |
| [Issue 687](https://github.com/link-assistant/formal-ai/issues/687) and [PR 688](https://github.com/link-assistant/formal-ai/pull/688) | A real Agent CLI trace showed ordinary environment actions falling outside the existing recipe vocabulary. | Add a shared primitive vocabulary and test the actual external client boundary. |
| [Issue 654](https://github.com/link-assistant/formal-ai/issues/654) and [PR 677](https://github.com/link-assistant/formal-ai/pull/677) | General planning was bounded to explicit repository-file changes. | Reuse the agentic planner shape, extending it with typed non-visual computer steps. |
| [Issue 671](https://github.com/link-assistant/formal-ai/issues/671) and [PR 814](https://github.com/link-assistant/formal-ai/pull/814) | Matrix replay established the repository pattern for recorded client evidence. | Record and replay the same ten plans through fresh external sessions. |
| [Issue 870](https://github.com/link-assistant/formal-ai/issues/870) and [PR 871](https://github.com/link-assistant/formal-ai/pull/871) | Recent external-agent integration work uses real client sessions and committed evidence. | Preserve native Agent session ids and exact streams. |
| [GAIA dataset card](https://huggingface.co/datasets/gaia-benchmark/GAIA) | Gated access; no explicit redistribution license observed. | Exclude GAIA content. |
| [AgentBench repository](https://github.com/THUDM/AgentBench) | Apache-2.0; deterministic OS interaction is relevant prior art. | Borrow only the isolated deterministic test shape; copy no tasks. |
| [MCP tools](https://modelcontextprotocol.io/specification/2025-06-18/server/tools) | Tools expose schemas and return content, structured content, and explicit errors. | Advertise all primitives with JSON schemas and return a typed step record. |
| [WHATWG Fetch](https://fetch.spec.whatwg.org/) | Fetch defines structured request/response behavior. | Record method, URL, status, cache path, and content digest. |
| [WHATWG DOM](https://dom.spec.whatwg.org/) and [Selectors API](https://www.w3.org/TR/selectors-api/) | Structured DOM selection is separable from visual rendering. | Support bounded HTML selection and name rendering as an explicit gap. |

The GitHub issue image associated with issue 687 was downloaded with
authenticated redirects, validated as a PNG before inspection, and used only as
research evidence. It is not redistributed here because the linked issue
retains the canonical attachment.
