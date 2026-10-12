# Upstream issue drafts (PR #1188)

Each draft opens with two comments that name the target repository and the title. Drafts 02 and 04 mention "Draft 1" and "Draft 3". Replace those with the issue URLs once 01 and 03 are filed.

```bash
for f in docs/case-studies/pull-request-1188/upstream-issue-drafts/[0-9]*.md; do
  repo=$(sed -n 's/^<!-- repo: \(.*\) -->$/\1/p' "$f")
  title=$(sed -n 's/^<!-- title: \(.*\) -->$/\1/p' "$f")
  tail -n +4 "$f" > /tmp/body.md
  gh issue create -R "$repo" --title "$title" --body-file /tmp/body.md
done
```

| Draft | Repository | Verified |
| --- | --- | --- |
| 01 | link-foundation/links-notation | JS reproduction re-run 2026-10-07 against links-notation 0.13.0 |
| 02 | link-foundation/links-notation | JS `Link.toString()` quoting re-checked 2026-10-07 |
| 03 | link-foundation/lino-objects-codec | JS reproduction re-run 2026-10-07 against lino-objects-codec 0.4.0 |
| 04 | link-foundation/lino-objects-codec | JS `formatIndented` output re-checked 2026-10-07 |
| 05 | link-foundation/meta-language | crates.io 0.58.2 dependency list checked 2026-10-08 |
| 06 | link-assistant/agent | Reproduced 2026-10-08 on @link-assistant/agent 0.26.0 (exit 1) and 0.26.11 (summary dropped); `gh search issues --repo link-assistant/agent` found only the related #304 (unhandled summary rejection, fixed) |
| 07 | link-foundation/meta-language | `parity/self-translation/expected/arithmetic-to-rust.rs` and the prelude constant read at PR #196 head `ddcde32` on 2026-10-08; lint behaviour from the clippy and rustc lint definitions (no upstream build run); `gh search issues --repo link-foundation/meta-language` found no existing report |
| 08 | link-foundation/meta-language | Release request. Checked 2026-10-08: PR #196 merged as `679a3b3c` on 2026-10-06; the latest GitHub/crates.io release is `v0.58.2` (2026-08-20), and `js/src/self-translation.js` and `rust/src/self_translation.rs` do not exist at that tag; `npm view meta-language versions` lists only `0.46.0`; #199 lists the release as remaining scope |
| 09 | link-foundation/meta-language | The `twice`/`quad` reproduction run with `selfTranslate` and `translateProgram` at `679a3b3c` on 2026-10-08; the counts are formal-ai's census (`data/meta/js-rust-translation.lino`) |
| 10 | link-foundation/meta-language | Each row of the table run through `translateProgram` at `679a3b3c` on 2026-10-08 |
| 11 | link-foundation/meta-language | `pluralize` and `escape` signatures from formal-ai's first measurement at `679a3b3c` (before JSDoc types were added); the 17/9/5/4 counts are that measurement's |
| 12 | link-foundation/meta-language | `translateGroup` and `generate-self-translation-report.mjs` read at `679a3b3c` on 2026-10-08 |
| 13 | link-foundation/meta-language | Timings measured 2026-10-08 at `679a3b3c` on node 20 (busy 12-core laptop); the scaling is linear, the absolute numbers depend on load |
- Filed 2026-10-08: draft 08 as a comment on [meta-language#199](https://github.com/link-foundation/meta-language/issues/199#issuecomment-6055834620); draft 09 as [#202](https://github.com/link-foundation/meta-language/issues/202) (sibling items) and [#203](https://github.com/link-foundation/meta-language/issues/203) (imports); draft 10 as [#211](https://github.com/link-foundation/meta-language/issues/211) and [#205](https://github.com/link-foundation/meta-language/issues/205); drafts 11, 12 and 13 as [#213](https://github.com/link-foundation/meta-language/issues/213), [#214](https://github.com/link-foundation/meta-language/issues/214) and [#215](https://github.com/link-foundation/meta-language/issues/215).
