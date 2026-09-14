# The run that produced both fixes

Ladder leaf L07, node `1.1.2.2.1`, driven through the Agent CLI against a real
`formal-ai serve --agent-mode` backend on 2026-09-03. Session
`ses_f9682c963fferkjQ4TKsNqd4BX`. The task:

> Edit the tracked file `src/engine_responses.rs`: add `"Good morning"` to the
> greeting list.

Fifteen tool calls, in three stages, each caused by the one before it. Replay
them with:

    python3 - <<'PY'
    import json
    for line in open("agent-stream.jsonl"):
        event = json.loads(line)
        if isinstance(event, dict) and event.get("type") == "tool_use" and event["input"]:
            print(event["name"], json.dumps(event["input"])[:200])
    PY

**1 — the node id read as a file.** The first call was
`read {"filePath": ".../1.1.2.2.1"}`, answered `Error: File not found`. The
prompt states its own node id, and `is_workspace_path` accepted it.

**2 — an edit assembled out of the prompt.** Having observed nothing, the next
call was an `edit` on `src/engine_responses.rs` whose `oldString` was a paragraph
of the prompt itself (`"only that file and keep it valid Rust.\n\nThis is
recursive binary-tree node 1.1.2.2.1 at depth 5. …"`) and whose `newString` was
another. It failed: no such text in the file.

**3 — the failure written into the source.** The error string was then written
verbatim into the proof note, then into the effects record, and finally into
`src/engine_responses.rs` itself:

    write {"content": "node_path=1.1.2.2.1\nnode_depth=5\nnode_kind=leaf\nresult=The command failed: Error: You must read the file … before overwriting it. Use the Read tool first\n",
           "filePath": ".../src/engine_responses.rs"}

Stage 1 is fixed by rejecting all-digit extensions, and stage 2 by ranking
candidate paths by shape and position; both are measured in the surveys one
directory up, and both are pinned by
`issue_1069_every_ladder_leaf_reaches_a_real_change`.

Stage 3 has a cause of its own that outlives the other two: the delivery split
in `src/agentic_coding/evidence_record.rs` claimed `src/engine_responses.rs` as
somewhere to *record an outcome*, because the leaf sentence spends a write-action
cue (`add`) on it. A file the request asks to modify is an operand, not a
destination for the run's status. That is tracked separately; it is kept here
because the evidence for it is this same run.

The offline regression that was supposed to catch all of this reported 32/32
green at the time, because it drove only the leaf's task *sentence*. The real
prompt wraps that sentence in an effect contract and a node preamble, and the
node id lives in the preamble. The test now assembles the whole prompt, and
`issue_1069_the_node_prompt_still_reads_as_the_ladder_writes_it` fails if the
ladder's wording and the harness's copy of it drift apart.
