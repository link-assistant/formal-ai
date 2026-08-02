# Online research (issue #796)

## npm deprecation warnings in CI

- `npm warn deprecated ...` is written to **stderr** but npm exits 0. Any CI
  step treating non-empty stderr as failure breaks on it.
- npm's only control is `loglevel`
  ([docs](https://docs.npmjs.com/cli/v11/using-npm/config#loglevel)):
  `silent, error, warn, notice (default), http, info, verbose, silly`.
  `--loglevel=error` suppresses deprecation lines wholesale.
- **No per-package or transitive-only suppression exists.** Open request:
  [npm/cli#7633](https://github.com/npm/cli/issues/7633). pnpm has the same gap
  ([pnpm#9720](https://github.com/pnpm/pnpm/issues/9720),
  [pnpm#4343](https://github.com/pnpm/pnpm/issues/4343)).
- `NODE_OPTIONS=--no-deprecation` is a **Node** flag for runtime
  `process.emitWarning` DeprecationWarnings; it does not affect npm's
  registry-metadata deprecation lines
  ([nodejs/node#40940](https://github.com/nodejs/node/issues/40940)).
- **No existing library or GitHub Action classifies npm stderr diagnostics.**
  `npm audit --json` covers vulnerabilities only; `npm ls --json` and
  `npm query` expose no `deprecated` field (`npm query ":deprecated"` errors
  with `EQUERYNOPSEUDO`). A regex over stderr, or a registry lookup per
  resolved version, remains the only mechanism -- which is why this repo keeps
  its own classifier rather than adopting a component.

### The glob deprecation specifically

glob deprecated everything except 12.x/13.x. Critically, `10.5.0` and `11.1.0`
are themselves the patch releases for CVE-2025-64756 yet are still flagged
deprecated -- filed as
[isaacs/node-glob#644](https://github.com/isaacs/node-glob/issues/644).
Downstream fallout: [mocha#5779](https://github.com/mochajs/mocha/issues/5779),
[jest#15910](https://github.com/jestjs/jest/issues/15910),
[gemini-cli#18327](https://github.com/google-gemini/gemini-cli/issues/18327),
[vscode#267530](https://github.com/microsoft/vscode/issues/267530).
So the warning is unavoidable noise, not an actionable vulnerability.

Dependency chain verified against this repo's committed lockfiles (not assumed):

```
@link-assistant/web-capture@^1.10.10 -> archiver@7.0.1
  -> archiver-utils@5.0.2 (glob ^10.0.0) -> glob@10.5.0
```

`@vscode/vsce@3.9.2` already uses `glob ^13.0.6`; electron-builder pulls only
`glob@7.2.3`. Because web-capture is first-party, this is fixable at source.

## git trailers

From [git-interpret-trailers](https://git-scm.com/docs/git-interpret-trailers),
verbatim:

> "Existing trailers are extracted from the input by looking for a group of one
> or more lines that (i) is all trailers, or (ii) contains at least one
> Git-generated or user-configured trailer and consists of at least 25%
> trailers. The group must be preceded by one or more empty (or
> whitespace-only) lines. The group must either be at the end of the input or
> be the last non-whitespace lines before a line that starts with `---`."

This confirms the root cause exactly: only **one** group is recognised and it
must be at the **end** of the message. A blank line between two trailers splits
them into two paragraphs; only the final paragraph is parsed, so the earlier
trailer is invisible to both `git interpret-trailers` and `%(trailers)`. The
25% heuristic is a second failure mode: a trailer adjacent to enough prose can
be swallowed.

`--no-divider` does **not** help -- it concerns `---` handling in
format-patch/email input, and is already the default. There is no option making
git scan earlier paragraphs.

### Existing tools evaluated

- **commitlint** has `signed-off-by` and `footer-*` rules
  ([rules](https://commitlint.js.org/reference/rules.html)); the documented
  `trailer-exists` rule is
  [not actually recognised](https://github.com/conventional-changelog/commitlint/issues/3033).
- commitlint and [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)
  both define their **own** footer parsers, not git's trailer algorithm, so
  neither can catch the blank-line bug -- they would accept a message git parses
  differently.

**Conclusion:** no off-the-shelf component validates against git's actual
semantics, so the fix is implemented in-repo. `git interpret-trailers --parse`
(`--only-trailers --only-input --unfold`) is the robust primitive for reading
trailers; scanning the raw body for trailer-shaped lines absent from the parsed
output is what detects the orphaned-trailer failure mode.
