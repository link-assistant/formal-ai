# Formal AI coding-discovery recipe evidence

This record points to the raw evidence for the Formal AI-authored
`data/meta/coding-discovery-recipe.lino` leaf of pull request #888.

- Session: `ses_f5ec49c02ffe6yDSkWK0x1t0p1`
- Model: `formal-ai/0.349.2`
- Client: external Agent CLI against the branch binary's OpenAI-compatible endpoint
- Result: the successful retry wrote the recipe and read it back exactly
- Unlisted raw trace: <https://gist.github.com/konard/dbae44b1e547bf1a9b1ba51c6178ecf7>

The unlisted gist contains the four substantive files from
`target/formal-ai-authoring/pr-888-coding-recipe/`. GitHub rejects blank gist
files, so the one-byte empty `agent-stderr.log` is recorded below by size and
hash but omitted from the gist.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `agent-stderr.log` | 1 | `01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b` |
| `agent-stream.jsonl` | 45,359 | `6c9e6f8bcd2bdb1bc8a6613749d29fc2b8b8b64f1d3f50a6d8fc65e7fabec628` |
| `formal-ai.log` | 234,567 | `0f1fbce3284484fba6d01f500a6c97b7908d9dec79b16ca89cb10f8637f61704` |
| `session-id.txt` | 83 | `64cee734025a059ee3fece36d7d041281db73fcb48484e990b3cc9d1a30eb29d` |
| `task.txt` | 930 | `47d4e8fed86d25c4ee8ae0c36f72d518679e9c5276f9f0018193387c8d8194e7` |

Before upload, the bundle was checked with the repository's pinned secret
scanner. The only API key is the deliberate local-loopback value `local`; no
reusable credential is present. The raw files remain ignored by Git and are
not committed to this repository.
