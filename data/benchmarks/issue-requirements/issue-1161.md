## Why

`formal-ai clients --format json` tells Hive Mind where each native CLI reads its configuration. For the Agent CLI the entry is a `json` config under `.config/link-assistant-agent/…`, which Hive Mind materialises by pointing `XDG_CONFIG_HOME` at a task-local directory (`src/formal-ai-runtime.lib.mjs:333-336`). That relocation also moves `gh`'s config (`$XDG_CONFIG_HOME/gh/hosts.yml`) and `git`'s, which is how the 2026-09-27 Scala session lost GitHub authentication and Formal AI answered `planned_not_executed` (link-assistant/hive-mind#2314).

The Agent CLI already accepts `LINK_ASSISTANT_AGENT_CONFIG_DIR` / `LINK_ASSISTANT_AGENT_CONFIG` (`js/src/config/config.ts:96-98`), so the registry can name a precise, side-effect-free mechanism.

## Requirements (what to do)

- In the client registry (`formal-ai clients`), let each `global_configs` entry declare how the client is pointed at the config: `env: LINK_ASSISTANT_AGENT_CONFIG_DIR` for the agent, `env: CODEX_HOME` for codex, etc., instead of implying a directory relocation. Keep the path for clients that have no env override.
- Document the contract in the clients registry output and in `docs/` (what a host must set; what it must not touch — `XDG_CONFIG_HOME`, `HOME`).
- Test: the registry entry for `agent` names the env var; a smoke test starts the agent CLI with that env var and a temp config dir and confirms it reaches a local `formal-ai serve`.


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

## Related (other repositories)
- hive-mind agent gh auth: https://github.com/link-assistant/hive-mind/issues/2314
- agent config-dir contract: https://github.com/link-assistant/agent/issues/314

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1161-agent-config-env.md` with each requirement of this issue; `rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md`.
- [ ] Row(s) in `docs/requirements-traceability.md`: delivered (version/commit), automated test `rust/tests/unit/issue_1161_client_registry_env.rs` (registered in `rust/tests/unit/mod.rs`), manual confirmation (the Hive Mind run below).
- [ ] Case study `docs/case-studies/issue-1161/` with the 2026-09-27 log excerpt, the root cause and the fix.
- [ ] Changelog fragment `changelog.d/<YYYYMMDD_HHMMSS>_issue-1161-agent-config-env.md`.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1161_client_registry_env` and `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green; the fix is released (check `git merge-base --is-ancestor <commit> $(git describe --tags --abbrev=0)`).
- [ ] Manual confirmation: link-assistant/hive-mind's Hello World end-to-end matrix (link-assistant/hive-mind#2324 R5) row **formal-ai × agent (Scala)** passes on the released version, linked here.

## Depends on / blocks

- Hive Mind side already done: v2.33.1 (PR #2321) no longer relocates `XDG_CONFIG_HOME` and sets `LINK_ASSISTANT_AGENT_CONFIG_DIR` / `OPENCODE_CONFIG_DIR` itself. What remains here is that `formal-ai clients --format json` declares the env var, so hosts read it from the registry instead of hard-coding it (then Hive Mind can drop its own mapping).
- Depends on: nothing. Blocks: removing that mapping from link-assistant/hive-mind.



