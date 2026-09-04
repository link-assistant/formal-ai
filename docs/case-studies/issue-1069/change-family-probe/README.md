# Three change families, delivered and attributed by the loop

Issue #1069 records that "attribution is never written" as the first blocker on
a legitimate release: R924-6 counts a pull request only when *every* non-merge
commit it introduces carries valid session evidence, and nothing in the ladder
so far had shown the self-development loop writing those trailers onto a change
to a tracked source. The issue #1028 ladder does not show it either. All 32 of
its leaves are Inspect/Verify/Record tasks, and every one of them is satisfiable
by writing a self-describing side file -- an *added* artifact that says the work
happened, never a *diff* to a file the repository already had.

This run closes that gap for three change shapes. Each is one delegation through
the real `@link-assistant/agent` CLI against `formal-ai serve --agent-mode`, into
a throwaway Git repository seeded with a single tracked source copied out of this
one, and each is judged by a verifier that reads only that tracked file.

## Result

```
member-insertion	PASS	src/orchestration/workspace.rs
literal-replacement	PASS	src/orchestration/runner.rs
identifier-rename	PASS	src/orchestration/attribution.rs
```

| family | session | wall time | effect |
| --- | --- | --- | --- |
| member-insertion | `ses_f94ec692bffe0vH0riuozWaHwl` | 27.2 s | `node_modules` added to the `matches!` arm in `fn ignored` |
| literal-replacement | `ses_f94ec01efffeAFbpVtBglB429M` | 22.1 s | `DEFAULT_BASE_URL` port `8080` → `8090` |
| identifier-rename | `ses_f94eba8e8ffehPXqKzSERq05Hm` | 21.7 s | `SESSION_TRAILER` → `AGENT_SESSION_TRAILER`, definition and use |

Every session reports `"kind": "modified"` with a `before_sha256` differing from
its `after_sha256`, and every commit the loop wrote is a **modification** of the
tracked source, not an addition beside it:

```
formal-ai: apply verified agent effect

Formal-AI-Session: ses_f94ec692bffe0vH0riuozWaHwl
Formal-AI-Evidence: .formal-ai-orchestration/sessions/000-agent.json
Formal-AI-Pull-Request: https://github.com/link-assistant/formal-ai/pull/1070
```

```
A	.formal-ai-orchestration/sessions/000-agent.json
M	src/orchestration/workspace.rs
```

The three trailers are written by `commit_verified_effect` in
`src/orchestration/attribution.rs`; no step of this run hand-wrote one. That is
the point of the run: the trailers R924-6 counts are produced by the loop, on a
commit whose content is a source diff.

## Reproducing

```bash
RUSTUP_TOOLCHAIN=1.98.0 cargo build --bin formal-ai
bash experiments/issue_1069_change_shaped_ladder/probe-families.sh
```

`families.tsv` (copied here beside the evidence) is the whole input: one row per
family, giving the task text and the three fields of the change contract --
`change_path`, `change_marker`, `change_guard`. The task text is ordinary prose;
nothing in the router matches on it, which is what makes a family a family rather
than a special case.

## What the verifier demands

`experiments/issue_1069_change_shaped_ladder/verify.sh` runs inside the delegated
workspace, so it is deliberately git-free and compiler-free: the workspace walk
in `src/orchestration/workspace.rs` strips `.git` and `target/`, so neither is
available to a verifier shipped with the fixture. It compares the target against
a pristine `.baseline/` copy and passes only when:

1. the marker is **absent** from the baseline -- otherwise a contract could name
   text the file already contains, and doing nothing would pass;
2. the tracked file actually differs from the baseline -- an added side file is
   not a modification, which is the escape this verifier exists to close;
3. the difference contains the marker that was asked for;
4. the anchor survives -- a file replaced wholesale by one line holding the
   marker is not the requested edit;
5. `rustfmt --edition 2024` still parses a `.rs` target, so a syntactically
   broken edit cannot pass;
6. nothing else under the baseline is disturbed.

The probe then asserts the same outcome a second time, independently of anything
`dispatch` reports, by reading the workspace's Git history directly: a
modification commit for the contract's path, carrying all three trailers, adding
nothing but the orchestrator's own evidence file.

## A verifier that contaminated its own evidence

The first revision of check 5 redirected `rustfmt`'s stderr to `parse.err` in
the working directory. The loop -- correctly -- committed that file as part of
the verified effect, so all three attributed commits read:

```
A	.formal-ai-orchestration/sessions/000-agent.json
A	parse.err
M	src/orchestration/workspace.rs
```

Check 6 could not see it, because `.baseline/` holds only files that existed
before the run and so has nothing to compare a new file against; the dispatch
report could not see it either, since it reports the change the agent made, not
what the verifier left behind. Only the workspace history showed it. The
verifier now writes that output to a scratch file outside the tree it is
judging, and the probe's own assertion fails on any added file outside
`.formal-ai-orchestration/`, so the contamination cannot return unnoticed.

## Scope

Three families passing is evidence that the change routes deliver a diff to a
tracked source under an adversarial verifier, and that the loop attributes it.
It is not a rewritten ladder: the issue #1028 ladder's 32 leaves are still
Inspect/Verify/Record tasks. A family that had failed here would not have been a
reason to soften the contract -- per R924-7 the failure would have become the
next thing to fix, with a fix that generalises rather than one that recognises
the probe's wording.
