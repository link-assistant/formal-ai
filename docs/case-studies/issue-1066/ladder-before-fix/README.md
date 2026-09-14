# The 32-leaf ladder as it ran before the request-block fixes (#1066)

This directory is the *before* half of a comparison. It is the real
`experiments/issue_1028_agent_cli_ladder/run.sh` at `TREE_DEPTH=5` — thirty-two
Agent CLI sessions against a real `formal-ai serve`, one per leaf — captured
while `stated_request::request_blocks` did not yet exist, so every route read the
node's task and the note that places the worker as one string.

Nothing here is synthesised. `run.log`, `selected.tsv`, `tree.tsv`, `leaves.tsv`
and everything under `nodes/` and `proofs/` are the bytes the run left behind;
`verdicts.tsv` and `grep-patterns.tsv` are derived from them by the commands
below.

| Path | What it is |
| --- | --- |
| `run.log` | The runner's own per-leaf transcript: thirty-two `PASS depth=5` lines |
| `tree.tsv`, `leaves.tsv` | The decomposition tree the runner built and the leaf tasks it handed out |
| `selected.tsv` | The node id, depth, task and completion criterion each session was given |
| `nodes/<id>/agent-stream.jsonl` | The Agent CLI's own event stream for that session |
| `nodes/<id>/agent-stderr.log` | That session's standard error |
| `proofs/node-<id>-proof.md` | The proof file each session wrote, unedited |
| `verdicts.tsv` | `experiments/issue_1066_ladder_offline/judge-proof.py` re-run over `proofs/` |
| `grep-patterns.tsv` | Every `grep` pattern the sessions planned, counted |

The server-side `formal-ai.log` of each session is not kept here — 7.2 MB of
request logging that carries nothing the stream does not. The *after* run under
`docs/case-studies/issue-1028/agent-tree-run/` keeps its copy.

## What it shows

Thirty-two of thirty-two leaves pass the runner's mechanical criterion — exit 0
and a proof file opening with `node_path=<id>`. Judged on what is *under* that
marker, twenty-four are `ok` and eight are `hollow_reported_failure`: a whole
sentence reporting that a lookup returned no content, which proves exactly as
much as an empty file.

`grep-patterns.tsv` shows why. Twenty-nine `grep` calls were planned across the
thirty-two sessions, and twenty of them searched for `binary_tree` — a word from
the note that places the worker, not from any task. `task_decomposition`, which
eight of the tasks are explicitly about, accounts for two.

Both effects are traced to their general cause, and to the fixes that removed
it, in [the case study](../README.md) — see "The note that places the worker is
not the request".

## Regenerating the two derived tables

`verdicts.tsv`:

```sh
cd docs/case-studies/issue-1066/ladder-before-fix
printf 'node\tverdict\n'
for p in proofs/node-*-proof.md; do
  id=${p#proofs/node-}; id=${id%-proof.md}
  printf '%s\t%s\n' "$id" \
    "$(python3 ../../../../experiments/issue_1066_ladder_offline/judge-proof.py "$p" "$id")"
done
```

`grep-patterns.tsv` counts one row per distinct `tool_use_id`, because the Agent
CLI emits each `tool_use` event twice — once as the call opens with an empty
`input`, once with the arguments filled in:

```sh
python3 - <<'PY'
import collections, json, pathlib
calls = {}
for p in sorted(pathlib.Path("docs/case-studies/issue-1066/ladder-before-fix/nodes").glob("*/agent-stream.jsonl")):
    for line in p.read_text(errors="replace").splitlines():
        if not line.strip():
            continue
        event = json.loads(line)
        if event.get("type") != "tool_use" or event.get("name") != "grep":
            continue
        pattern = (event.get("input") or {}).get("pattern")
        if pattern is not None:
            calls[(p.parent.name, event["tool_use_id"])] = pattern
counts = collections.Counter(calls.values())
print("count\tpattern")
for pattern, n in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
    print(f"{n}\t{pattern}")
PY
```
