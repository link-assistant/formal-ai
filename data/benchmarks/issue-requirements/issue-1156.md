## What happened

Kotlin run, backend 0.352.1. The issue says "The program should print exactly: `Hello, World!`" and shows the expected output block once. Formal AI committed (`4337bf1c`, PR https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/pull/2):

```kotlin
fun main() {
    println("Hello, World!\nHello, World!")
}
```

and a `tests/verify-output.sh` that pins the doubled output:

```sh
printf '%s\n' 'Hello, World!
Hello, World!' > "$verification_dir/expected"
```

**Root cause** (`rust/src/coding/program_contract.rs:25-43`, `explicit_stdout`): every quoted literal whose preceding clause carries the `print_stdout` meaning is collected, and the list is joined with `"\n"`. The issue has two such clauses for the same output: "The program should print exactly: `Hello, World!`" (requirement 2) and "Code prints "Hello, World!" exactly" (Definition of Done), so the program prints the line twice.

So the program is wrong **and** the verification agrees with the wrong program: the harness reported "Created and verified" and Hive Mind converted the PR to ready-for-review. On 2026-09-15/16 (backend 0.351.0) the Scala run wrote `println("Hello, World!")` once, so this is a regression in the 0.352 line (candidates: `fe6c54fad feat(coding): derive programs through sourced discovery`, `fd88a1745 feat(coding): fund the literal-clean rotation program`).

Earlier on the same PR (2026-09-15, backend 0.349.x/0.351.0) the executor produced `Main.java` with `System.out.println("Main.kt")` and later a `Main.java` whose `println` contains the entire Kotlin source (commit `302f35a2`) — the literal-file composer took the *file name* / the program *source* as the text to print. That file is still on the branch.

## Requirements (what to do)

- In `explicit_stdout`, mentions of the same literal in different print clauses are one output obligation (coreference); only literals that the request orders as a sequence ("first print A, then B", a fenced multi-line expected output) form multiple lines. Replace the lexical clause test with the formalized obligations of #1166 (E131).
- The verification script must be derived from the requirement ("print exactly"), never from the produced program's output.
- Add the three real issue bodies (Kotlin, Scala, Rust — identical template) as fixtures. For each: program prints exactly `Hello, World!\n`, verify script expects exactly that, workflow name is meaningful (issue asks for e.g. `test-hello-world.yml`; today `run.yml`), and the language matches the request (no `Main.java` for a Kotlin task).
- Add a regression test for the "file name/source as output" mis-composition of 2026-09-15.

## How to test

`cargo test` with the fixtures; then the Hive Mind Kotlin run produces a `Main.kt` printing once, and `sh tests/verify-output.sh` passes against the *issue's* expected output.


## Shared evidence (2026-09-27 runs)

| Repo / tool | PR | Session start | Outcome |
|---|---|---|---|
| Kotlin, `--tool claude` | https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/pull/2 | 15:02:53Z | `Main.kt` committed, then 2 identical failed restarts, stop `no_progress_between_sessions`, PR left **draft** |
| Scala, `--tool agent` | https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2 | 15:09:06Z | `gh` unauthenticated inside the agent session → Formal AI `planned_not_executed`, PR left **draft** |
| Rust, `--tool codex` | https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2 | 15:15:15Z | 78 identical `gh issue view` calls (6.2M input tokens), no change, stop `draft_pull_request` after 3 fake "restores", PR left **draft** |

Runtime for all three: `solve v2.32.0`, task image `konard/hive-mind-dind:2.32.0`, Formal AI serving backend `0.352.1` (local wrapper `0.351.0`), `@link-assistant/agent` 0.26.5. Command (identical except `--tool`):

```
solve <issue-url> --model formal-ai --tool <claude|agent|codex> --attach-logs --verbose --no-tool-check --disable-report-issue --language en
```
(`--auto-restart-until-mergeable` defaults to `true` in `src/solve.config.lib.mjs:283-286`.)

Full logs (gists uploaded by solve):
- Kotlin main session: https://gist.github.com/konard/b0b660367fcd7e5a4c21a833cff4c5d4 · restart 1: https://gist.github.com/konard/f2683d22d41085bbe01bb45ab768b6de · restart 2: https://gist.github.com/konard/71bbc9af9e04dce7f98e11929bc30fa0 · final: https://gist.github.com/konard/bdca731664778563b104343fe3696335
- Scala (agent): https://gist.github.com/konard/8a196a1d179ecb105304232a46e09ede
- Rust (codex): https://gist.github.com/konard/513141ac5f144b15e9593f05f86362e8

Task issues are identical Hello World specs (print exactly `Hello, World!`, add a GitHub Actions workflow), e.g. https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/issues/1.

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1156-single-output-literal.md` with each requirement of this issue; `rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md`.
- [ ] Row(s) in `docs/requirements-traceability.md`: delivered (version/commit), automated test `rust/tests/unit/issue_1156_explicit_stdout_coreference.rs` (registered in `rust/tests/unit/mod.rs`), manual confirmation (the Hive Mind run below).
- [ ] Case study `docs/case-studies/issue-1156/` with the 2026-09-27 log excerpt, the root cause and the fix.
- [ ] Changelog fragment `changelog.d/<YYYYMMDD_HHMMSS>_issue-1156-single-output-literal.md`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1156_explicit_stdout_coreference` and `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green; the fix is released (check `git merge-base --is-ancestor <commit> $(git describe --tags --abbrev=0)`).
- [ ] Manual confirmation: link-assistant/hive-mind's Hello World end-to-end matrix (link-assistant/hive-mind#2324 R5) row **formal-ai × claude (Kotlin)** passes on the released version, linked here.

## Depends on / blocks

- Depends on: nothing for the coreference fix; the general replacement of lexical clause matching is #1166 (E131).
- Blocks: the claude row of the Hive Mind matrix.



