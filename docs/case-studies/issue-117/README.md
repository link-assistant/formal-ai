# Issue 117 Case Study

## Collected Data

- GitHub issue: https://github.com/link-assistant/formal-ai/issues/117
- Pull request: https://github.com/link-assistant/formal-ai/pull/118
- Upstream repository: https://github.com/link-foundation/lino-i18n
- npm package: https://www.npmjs.com/package/lino-i18n
- Raw captured data:
  - `raw-data/issue-117.json`
  - `raw-data/issue-117-comments.json`
  - `raw-data/pr-118.json`
  - `raw-data/pr-118-review-comments.json`
  - `raw-data/pr-118-conversation-comments.json`
  - `raw-data/pr-118-reviews.json`
  - `raw-data/npm-lino-i18n.json`
  - `raw-data/upstream-repo.json`
  - `raw-data/upstream-releases.txt`
  - `raw-data/upstream-issues.json`
  - `raw-data/upstream-readme.md`
  - `raw-data/recent-merged-prs.json`

## Requirements

1. Replace the browser UI's hand-maintained JavaScript i18n catalog and interpolation fallback with `link-foundation/lino-i18n`.
2. Store UI translations in Links Notation, using nested blocks instead of flat JavaScript keys.
3. Use multiline quoted strings for long text, matching the upstream authoring style.
4. Preserve the existing supported UI languages: English, Russian, Chinese, and Hindi.
5. Ensure every supported language has the same complete set of UI translation keys.
6. Add an automated CI/CD rule that fails when a locale is missing a key or contains drift.
7. Collect issue, PR, online research, and solution notes under `docs/case-studies/issue-117/`.
8. Report upstream issues if `lino-i18n` lacks required features.

## Research Notes

- npm reports `lino-i18n@0.1.1` as the current JavaScript package, published on 2026-05-18, with the description "Universal i18n library that stores translations in Links Notation (.lino) instead of JSON."
- The upstream repository's current releases include `[JavaScript] 0.1.1` and `[Rust] 0.2.0`, both published on 2026-05-18.
- The upstream README documents nested catalog authoring, multiline quoted strings, parent `label` aliases, placeholders, plural/context key flattening, and fallback semantics.
- The browser app does not need an upstream change: `lino-i18n@0.1.1` exports `createI18n` and `parseLinoCatalogs`, which are enough to parse a single static `.lino` catalog at runtime.

No upstream issue was filed because the current JavaScript package covers the required nested authoring, multiline strings, placeholder interpolation, parent labels, and fallback behavior.

## Solution

The browser UI now keeps translations in `src/web/i18n-catalog.lino`. Each locale is a top-level block (`en`, `ru`, `zh`, `hi`), related messages are nested (`buttons.reportIssue`, `composer.placeholder.chat`, `settings.theme.dark`), and long tooltip strings use multiline quoted values.

`src/web/i18n.js` keeps the existing `window.FormalAiI18n` API but no longer embeds a local translation catalog. It imports `lino-i18n@0.1.1`, fetches `i18n-catalog.lino`, parses it with `parseLinoCatalogs`, and creates the runtime with `createI18n`. Until the runtime is ready, the API returns stable keys; `formal-ai:i18n-ready` triggers the existing React rerender path.

Parent labels use the upstream `label` convention. For example, `settings.language.label "Language"` makes `settings.language` resolve to `"Language"` while still allowing nested keys like `settings.language.auto`.

## CI Rule

`tests/e2e/scripts/check-i18n-catalog.mjs` is the catalog gate. It imports the real `lino-i18n` package, parses `src/web/i18n-catalog.lino`, and verifies:

- all four locale blocks exist;
- every required UI key exists in every locale;
- no unexpected non-label keys are present;
- no translation value is empty;
- nested blocks and multiline quoted strings are present;
- representative runtime lookups, fallback, parent labels, and interpolation work.

`.github/workflows/release.yml` runs that script in the lint job through:

```sh
npm ci --prefix tests/e2e
npm run --prefix tests/e2e check:i18n
```

## Verification

Local targeted verification:

```sh
npm run --prefix tests/e2e check:i18n
```

Result: `i18n catalog check passed for 4 locales and 104 keys`.

The Playwright Issue #94 coverage was also updated to assert the new runtime version and a nested catalog lookup path for Issue #117.
