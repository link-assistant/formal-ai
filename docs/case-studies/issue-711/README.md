# Issue 711: consumed changelog fragments

Issue: [link-assistant/formal-ai#711](https://github.com/link-assistant/formal-ai/issues/711)<br>
Pull request: [link-assistant/formal-ai#718](https://github.com/link-assistant/formal-ai/pull/718)

## Outcome

The automatic release script now removes changelog fragments only after it has
successfully written `CHANGELOG.md`, and stages those removals with `git add -A`.
A regression test proves that a second collection is empty and that
`changelog.d/README.md` survives.

The polluted changelog was rebuilt from Git release trees. At the time of the
fix it shrank from 609,927 to 5,261 lines and assigned all 391 fragments that
had reached a release tree exactly once. After merging releases v0.290.0 through
v0.293.0, the same reconstruction contains 395 fragments in 5,337 lines. The
388 stale files present when issue 711 was reported were removed; unreleased
fragments remain pending for the next release.

## Root cause

There were two release paths:

- `scripts/collect-changelog.rs`, used by the manual path, wrote the changelog,
  deleted consumed fragments, and staged the directory correctly.
- The inline collector in `scripts/version-and-commit.rs`, used by automatic
  releases, wrote the changelog but neither deleted nor staged fragments.

Because release detection treats any fragment as a pending release, every
successful automatic release left its own trigger behind. Every later release
then collected the complete surviving directory again. The defect existed in
the repository's initial commit, `6f8d4a8a05770adfd2fe33fdf3c6c586efb103af`.

## Reconstruction method

`experiments/issue_711_rebuild_changelog.mjs` treats Git as the source of truth:

1. Preserve the pre-fragment 0.1.0 section verbatim.
2. Match fragments in the initial import to the earliest original section that
   contains their exact, frontmatter-free body.
3. Walk release commits reachable from the selected ref in semantic-version
   order, including releases merged from another parent of a feature branch.
4. Assign every previously unseen fragment path to the first release tree in
   which it appears and read its content from that tree.
5. Omit releases with no new fragment and emit releases in descending order.

The committed [fragment-release map](fragment-release-map.tsv) records the
result. CI runs the generator with `--check` and requires byte-for-byte equality
with both `CHANGELOG.md` and the map.

One branch-only fragment for issue 468 was added and removed before reaching a
release tree. It is deliberately excluded: it was never released and was
superseded within its pull request. Three formerly released fragments that were
subsequently deleted are recovered from release trees, producing 391 released
fragments from the 388 stale files found on `main` at the time of the fix. The
map grows normally as later releases consume new fragments.

## Timeline

- **2026-05-12:** the initial repository import includes the faulty inline
  collector and already-retained historical fragments.
- **2026-06-08:** the Rust pipeline template independently fixes the same defect
  in [template PR 66](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/pull/66).
- **2026-07-13 19:06 UTC:** v0.283.0 is published with 117,865 bytes of notes.
- **2026-07-14 17:22 UTC:** v0.289.0 is published with the same 117,865-byte
  replayed fragment set (release-specific comparison links differ).
- **2026-07-14 19:52 UTC:** issue 711 reports 388 stale fragments and a
  609,927-line changelog.
- **2026-07-14:** PR 718 adds the consuming collector, deterministic cleanup,
  regression coverage, template audit, research, and this evidence archive.

## Evidence layout

- [requirements.md](requirements.md) maps every requested outcome to evidence.
- [template-audit.md](template-audit.md) compares all four pipeline templates.
- [online-research.md](online-research.md) records established fragment-lifecycle
  practices and alternatives.
- `agent-evidence/` contains the generated plan and externally authored output.
- `raw-data/` preserves issue/PR/API payloads, release payloads, Git histories,
  template diffs, CI queries, build logs, and before/after test logs.

The documented live self-coding wrapper was attempted first. It rejected the
configured `formal-ai` model before launching the agent and automatically
posted [this diagnostic issue comment](https://github.com/link-assistant/formal-ai/issues/711#issuecomment-4974321419).
The documented direct Agent CLI fallback then completed three real rounds
against a locally built Formal AI server; its raw stream and server trace are
preserved under `raw-data/`.
