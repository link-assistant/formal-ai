# [js template] `actions/cache/restore@v5` and `actions/cache/save@v5` are a major behind (v6), and no Dependabot config catches it

Target repo: `link-foundation/js-ai-driven-development-pipeline-template` (HEAD `8e72538`)
Found while auditing: link-assistant/formal-ai#1637 (same defect class in formal-ai:
`actions/setup-node@v6` and `actions/setup-python@v6` a major behind v7, which makes
formal-ai's own dependency-currency gate fail on main).

Duplicate check (2026-11-25): no open issue. Related closed #234 ("checkout/setup-node
a major behind") — fixed those two actions only; the cache actions were introduced
later (e48623e, 2026-11-13) and were not covered. Open #312 and #315 are unrelated.

## Problem

`.github/workflows/links.yml`:

```yaml
L88:   uses: actions/cache/restore@v5
L138:  uses: actions/cache/save@v5
```

The current major is `actions/cache` **v6** (latest v6.1.0). Every other template that
uses the cache actions (rust, python) is already on `actions/cache*@v6`.

Root cause of recurrence: js is the only template **without**
`.github/dependabot.yml`, so action majors are only bumped when someone notices
(#234 was the previous instance).

## Reproduction

```bash
git clone --depth 1 https://github.com/link-foundation/js-ai-driven-development-pipeline-template t && cd t
grep -rnE 'uses: actions/cache(/restore|/save)?@v5' .github
gh release view --repo actions/cache --json tagName -q .tagName      # v6.1.0
ls .github/dependabot.yml                                            # No such file
```

## Workaround (downstream)

Replace `@v5` with `@v6` in both lines (the restore/save inputs used here —
`path`, `key`, `restore-keys` — are unchanged in v6).

## Suggested fix

1. Bump both uses to `actions/cache/restore@v6` / `actions/cache/save@v6` (or to the
   v6.1.0 commit SHA with a `# v6.1.0` comment, consistent with guide §14).
2. Add `.github/dependabot.yml` with at least:
   ```yaml
   version: 2
   updates:
     - package-ecosystem: github-actions
       directory: /
       schedule: { interval: weekly }
       groups: { actions: { patterns: ["*"] } }
     - package-ecosystem: npm
       directory: /
       schedule: { interval: weekly }
   ```
3. Optionally extend the existing dependency check (`7bdabe9`, "bounded dependency
   checks") to compare each `uses: owner/repo@vN` against the latest release major
   and fail on PRs that touch `.github/**`.
