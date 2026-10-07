# Upstream issue drafts (PR #1188)

Each draft opens with two comments that name the target repository and the title. Drafts 02 and 04 mention "Draft 1" and "Draft 3". Replace those with the issue URLs once 01 and 03 are filed.

```bash
for f in docs/case-studies/pull-request-1188/upstream-issue-drafts/0*.md; do
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
