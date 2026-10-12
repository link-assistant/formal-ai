TASK (tag READABLE): deliver R1188-U23, a strict owner requirement.

Every regular code file in `js/`, `ts/` and `rust/` is human-readable multi-line code. Only real distribution bundles (built output such as `js/vendor.bundle.js`, `js/*.bundle.js`, wasm glue) may be exempt, and each exemption is listed with its reason.

Known offenders:
- `scripts/translate-es.mjs` writes every `ts/**/*.mts` / `*.ts` twin as one line of space-joined tokens.
- The JS → Rust translation projections (`rust/tests/fixtures/js-rust-translation/**`) may hold very long single lines; measure them.

Do:
1. **The gate:** `scripts/check-readable-code.mjs`, registered in `data/meta/ci-gates/`.
   - Fail any tracked file under `js/`, `ts/`, `rust/` (and `scripts/`, `packages/`) with code extensions that has fewer than N lines for its size, or any line over a limit (e.g. 300 characters), unless it appears in `data/meta/readable-code-exemptions.lino` with a reason.
   - Seed the exemption list only with true bundles and generated data that is not code.
   - Add a test in `rust/tests/web/`.
2. **Make `translate-es` emit readable TypeScript.**
   - Best: keep the JS source's own layout, whitespace and comments, and change only what differs between the languages (type annotations, `.mts` import paths). The translator is token-based, so carry each token's original leading whitespace through the pivot.
   - If that isn't possible, run a deterministic formatter over the output. A local dev dependency is fine if it is pinned and already present (check `package.json`); otherwise write a small printer.
   - The `--check` mode must stay byte-exact against the regenerated output, and `ts/` must stay in parity with `js/`.
3. **Regenerate `ts/`** (341 files).
4. **Fix every other offender** the gate finds:
   - For a generated file, fix the generator to emit multiple lines.
   - For the translation projections, the projection text comes from meta-language. If it is one line, format it in our wrapper, or file an upstream issue on link-foundation/meta-language and exempt it with a link to that issue.
5. **Update R1188-U23** honestly.

Rules:
- **No cargo and no rust-script locally.**
- Run only the tests next to your change: the translate-es and parity tests and your new test. CI runs the rest.
- Mind the disk: no repository copies.
- Claim `scripts/translate-es.mjs`, `ts/` and the new files in `claims.md`. Other agents:
  - SPANISH edits `data/seed/multilingual-responses*.lino`.
  - CIFIX-LOOP commits CI fixes. If it touches `js/`, you will need `translate-es --write` after it.
- Use Formal AI for the small edits (ledger rows T500–T519).
- Do not commit; LEAD integrates.
