# Issue 687 — online research

Research was refreshed on 2026-07-15. Primary sources were used because the
example is time-sensitive and the architecture depends on precise client/tool
behavior.

## Election fact used by the scenario

The Federal Election Commission's 2026 congressional calendar states that the
general election date is **November 3, 2026**. It identifies state election
offices, statutes, and state parties as its sources and notes that dates can
change. This is the official fact the end-to-end research workflow is expected
to discover and cite:

- [FEC: 2026 congressional primary dates and filing deadlines](https://www.fec.gov/documents/5910/2026pdates.pdf)

The caveat matters: Formal AI should not freeze this date into its local rules.
It should search and fetch at request time, prefer the official source, and cite
what it fetched. The deterministic tests therefore inject tool results rather
than encoding the election date as planner knowledge.

## GitHub report action

GitHub's documentation says `gh issue create` is the CLI command for creating an
issue and that `--title` plus `--body` make it non-interactive. The CLI manual
also documents `--repo OWNER/REPO`, which is how the generated action targets
Formal AI explicitly:

- [GitHub Docs: Creating an issue with GitHub CLI](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue#creating-an-issue-with-github-cli)
- [GitHub CLI manual: `gh issue create`](https://cli.github.com/manual/gh_issue_create)

The implementation uses the existing shell capability instead of adding a
GitHub client library. User/conversation text is POSIX-quoted, and the E2E
replaces only the `gh` executable on `PATH` so execution is verified safely.

## Agent CLI capabilities and risk model

The Link Assistant Agent repository describes the JavaScript implementation as
OpenCode-JSON-compatible and documents `websearch`, `webfetch`, and `bash` among
its enabled tools. It also explicitly warns that the agent is unrestricted and
should be used only in an isolated environment:

- [Link Assistant Agent README](https://github.com/link-assistant/agent#readme)

That supports two design choices in this PR:

1. Formal AI emits standard client tool calls instead of owning HTTP/search
   implementations.
2. The E2E runs in a temporary directory and substitutes a non-mutating `gh`
   executable. It proves the action boundary without relying on network-side
   cleanup or creating test issues.

## Component decision

| Need | Existing component | Decision |
| --- | --- | --- |
| Intent vocabulary | Links Notation seed and embedded role registries | Store report/recall/research language here, not in Rust arrays. |
| Context and learning | `solve_with_history` plus associative memory from #686 | Reuse one interpretation path for recall and contextual research. |
| Web access | Agent/OpenCode `websearch` and `webfetch` | Orchestrate advertised tools; add no HTTP dependency. |
| Trust selection | URL host/suffix classification | Prefer government/education sources deterministically, then cite fetched URL. |
| GitHub write | Agent shell plus official `gh issue create` | Reuse authenticated CLI; shell-escape all generated arguments. |
| UI control | Existing React setters/normalizers and seed loader | Add a typed catalog and generic command mapping, not another library. |

The result stays within Formal AI's symbolic/GOFAI design: external tools gather
facts and perform effects; seed semantics and deterministic state machines decide
which capability is appropriate and when its evidence is sufficient.
