# External Agent CLI evidence

The external `@link-assistant/agent` 0.25.3 client executed the reference
procedure against the release `formal-ai serve` OpenAI-compatible endpoint.
Agent session `ses_0559382b9ffeZg7gTXqS6kySQU` completed four requests:

1. Formal AI compiled the natural-language impulse and asked Agent to write
   `compiled-procedure.lino`.
2. Formal AI asked Agent to read the same path back with `cat`.
3. Formal AI verified the artifact and asked Agent to execute
   `formal-ai procedure conformance` against the persisted file.
4. Formal AI verified the complete `procedure_run` and returned the artifact,
   execution evidence, and source-cited step restatement.

`agent-authored-compiled-procedure.lino` is the file from the Agent workspace.
The replay driver used `cmp` to prove it is byte-identical to
`data/meta/issue-674-compiled-procedure.lino`, and it rejected a stream without
the public conformance command.

Evidence files:

- `agent-stream.jsonl` — raw structured output from the external Agent client,
  including session id, provider `formalai`, tool loop, and final answer;
- `formal-ai.log` — matching Formal AI server request/planner trace;
- `session.json` — deterministic in-repository replay of the same task;
- `agent-authored-compiled-procedure.lino` — exact file written by Agent.

Reproduce with:

```bash
cargo build --release --bin formal-ai
experiments/issue-674-agent-cli/run.sh
```

The Rust implementation and manually reviewed tests are not claimed as
Formal-AI-authored work. Only the artifact leaf produced by this external
session carries self-authorship evidence.
