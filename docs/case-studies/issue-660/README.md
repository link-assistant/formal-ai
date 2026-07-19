# Issue 660: deterministic bulk lexical grounding

The completed batch imports 208 common-noun concepts and emits 832 validated
surfaces across English, Russian, Hindi, and Chinese. The CLI reports both the
numerator and denominator: 208 accepted of 208 requested concepts and 832 of
832 expected surfaces, or 1000‰ coverage.

## Reproduction and root cause

Run the production path without network access:

```bash
cargo run --bin formal-ai -- import lexemes \
  --concepts data/lexicon-import/common-nouns.lino --offline
cargo test --test unit bulk_lexeme_import
```

The original PR validated parsing, facets, denotation, and cache closure, but
two audit claims ended at process memory: surface provenance was implicit, and
`import_rejected` events were discarded when the CLI exited. It also wrote only
the current shard names, so a smaller later batch could leave an obsolete shard
active.

The importer now carries and emits the exact Wikidata item record and JSON field
for every surface. It does not invent Wikidata Lexeme `L…` ids for item-label
data. Script validation exposed one concrete cache defect: Q188075's Hindi
label is `m`; the deterministic same-record fallback selects `डब्बा` and emits
`aliases.hi[0].value` as its source. A rejected run writes replayable
`demo_memory`, exits unsuccessfully, and leaves the previous shard set intact.
A successful run stages current files and removes only obsolete
`meanings-lexicon-import*.lino` outputs.

## Formal AI executing the learning task

`experiments/agent_cli_e2e/run_issue_660_learning.sh` starts Formal AI in agent
mode and uses it as the model provider for two real external clients:
`@link-assistant/agent` and OpenCode. With differently worded task text, both
clients derive `lexeme-import-learning-report.lino` from the persisted
associative observation network, write it through a tool call, read it back,
and must produce byte-identical output.

The artifact remains explicitly `awaiting_human_review`; the system ranks
amendments but does not promote itself. Raw JSONL transcripts and the Formal AI
request log are retained under `agent-cli-evidence/`, and CI reruns the same
dual-client protocol.
