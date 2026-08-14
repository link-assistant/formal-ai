## Issue #117 Lino I18n Catalog Requirements

Issue [#117](https://github.com/link-assistant/formal-ai/issues/117) asks the
browser UI to stop using its own i18n implementation and instead use
[`link-foundation/lino-i18n`](https://github.com/link-foundation/lino-i18n),
with nested Links Notation authoring, multiline quoted strings, full language
parity, CI enforcement, and case-study evidence.

| ID | Requirement | Status |
| --- | --- | --- |
| R137 | Browser UI translations must be loaded through `link-foundation/lino-i18n`, not through a hand-maintained JavaScript translation/interpolation implementation. | Implemented by `src/web/i18n.js`, which imports `lino-i18n@0.1.1`, fetches `src/web/i18n-catalog.lino` together with `src/web/i18n-catalog-permissions.lino` and `src/web/i18n-catalog-messages.lino` (split so each file stays under the Links Notation line limit), merges their per-locale keys, parses them with `parseLinoCatalogs`, and creates the runtime with `createI18n`. The import map in `src/web/index.html` is pinned to the same package version. |
| R138 | UI translation source must use nested Links Notation and multiline quoted strings for long entries. | Implemented in `src/web/i18n-catalog.lino`, where `buttons`, `titles`, `composer`, `settings`, `status`, and trace messages are nested under top-level locale blocks and long tooltip values use `"""` strings. |
| R139 | English, Russian, Chinese, and Hindi must all contain the same complete UI key surface. | Implemented by migrating all 104 existing UI keys into each locale block in `src/web/i18n-catalog.lino`, including parent-label keys such as `settings.language` via the upstream `label` convention. |
| R140 | CI/CD must fail when the i18n catalog loses a required key, adds a non-label drift key, drops a locale, or contains empty translations. | Implemented by `tests/e2e/scripts/check-i18n-catalog.mjs` and the `Check i18n catalog coverage` step in `.github/workflows/release.yml`, run as `npm run --prefix tests/e2e check:i18n`. |
| R141 | Runtime tests must prove the browser uses the published `lino-i18n` package and can resolve nested catalog entries, parent labels, interpolation, and fallback. | Implemented by updating the Issue #94 Playwright tests in `tests/e2e/tests/demo.spec.js` to expect `lino-i18n@0.1.1` and adding a nested catalog lookup test for Issue #117. |
| R142 | Compile issue #117 evidence, online research, requirements, solution plan, and verification notes under `docs/case-studies/issue-117/`. | Implemented in `docs/case-studies/issue-117/README.md` with raw captured GitHub, npm, release, and upstream README data under `docs/case-studies/issue-117/raw-data/`. |
