# Issue #812 — `Build macos-*` fails on `codesign … Google Chrome for Testing Framework.framework`

Evidence base: `dev/log/issues/812/pulls/813/ci-logs/run-29752745259.log` (Desktop
Release run 29752745259, 3.6 MB, line numbers below are file line numbers),
`run-29751001867.json/.log` (CI/CD Pipeline run on the same head SHA),
`dev/log/issues/810/pulls/811/analysis.md` §3 and §8, and the packaged sources of
`@electron/osx-sign@1.3.3` / `app-builder-lib@26.15.3` fetched from unpkg.

## 0. Headline

**The failing run does not build the code that contains the fix.** Run
29752745259 is a `workflow_run` healing build; every job checks out
`ref: v0.300.0` (`run-29752745259.log:7156`, `:11854`, resolved to
`HEAD is now at 78f0800 chore: release v0.300.0`, `:7323`, `:11946`). Tag
`v0.300.0` was created 2026-07-19T18:05Z (`gh api repos/link-assistant/formal-ai/releases/latest`),
**before** the two packaging fixes that landed on `main` on 2026-07-20
(`a6f0f040`, `56fa089c`). Both fixes are absent from the tag:

| Fix | Commit | In `v0.300.0`? | Proof |
| --- | --- | --- | --- |
| `ignore` passed to `signAsync` as a *function*, not an array | `a6f0f040` | no | `git show v0.300.0:desktop/scripts/adhoc-sign-mac.cjs` line 129 still reads `ignore: signingIgnoreRules(signOptions)` where `signingIgnoreRules` returns `[...]` |
| `mac.signIgnore: ["/Contents/Resources/browser-runtime/"]` | `39fdef91` | no | `git show v0.300.0:desktop/package.json \| grep signIgnore` → no match |
| `fs.cpSync(..., { verbatimSymlinks: true })` | `56fa089c` | no | `git show v0.300.0:desktop/scripts/prepare-resources.mjs:46` → `fs.cpSync(from, to, { recursive: true })` |
| debug banner / `considered=`/`skipped=` counters | `1b1586bb` | no | the tag's hook does not even `require('node:fs')` |

That single fact answers questions 1–3 of the brief and reframes the defect: the
remaining problem is **not** a signing bug that still needs diagnosing, it is a
*release-process* bug — the healing workflow rebuilds an immutable tag whose
packaging code is known-broken, and it will keep failing forever until a new tag
is cut.

## 1. Who issues the failing `codesign`

`@electron/osx-sign@1.3.3`, from `signApplication()` in `dist/cjs/sign.js`:

- `sign.js:172` `const children = await walkAsync(getAppContentsPath(opts))` —
  everything under `Contents/`, including `Contents/Resources/browser-runtime/…`.
- `sign.js:173-174` `if (opts.binaries) children.push(...opts.binaries)` — not
  used here: the log prints `> Additional binaries: undefined`
  (`run-29752745259.log:8841`). So the framework does **not** arrive via
  `opts.binaries`.
- `sign.js:183-186` children are sorted **deepest first**; that is why the
  browser-runtime leaf files are signed at 15:09:17 (`:8844`, `:8846`) and the
  `…Framework.framework` *directory* itself — depth 1 inside the runtime — is
  signed last, at 15:10:26 (`:11760`), which is the invocation that dies
  (`:11762` `Error executing file`, `:11765` `unsealed contents present in the
  root directory of an embedded framework`).
- `sign.js:189-192` `shouldIgnoreFilePath()` is the *only* skip mechanism, and it
  is a no-op when `opts.ignore` is falsy.
- `sign.js:265` `execFileAsync('codesign', perFileArgs.concat('--entitlements', …, filePath))`
  is the literal command in the error message. No `--deep` anywhere.

There is **no second signer**. `app-builder-lib@26.15.3/out/macPackager.js:323-334`
(`doSign`) is the whole story:

```js
const customSign = await resolveFunction(this.appInfo.type, customSignOptions.sign, "sign", …);
log.info({…}, customSign ? "executing custom sign" : "signing");
return customSign ? Promise.resolve(customSign(opts, this)) : sign({ ...opts, identity: … });
```

It returns the hook's promise and does nothing afterwards — electron-builder does
**not** re-sign the outer bundle after the hook. `MacPackager.sign()`
(`macPackager.js:289-321`) calls `buildSignOptions` → `doSign` → optional MAS
installer / notarize, nothing else. So the hypothesis "electron-builder re-signs
the app bundle after the hook returns and that walk hits the framework" is
**disproved by source**.

Consistent with that, `run-29752745259.log:8831` shows exactly one
`• executing custom sign` per macOS job, and `:8832`
(`electron-osx-sign electron-osx-sign@1.3.3`) shows the single `signAsync` entry
that our hook makes.

## 2. Why the ignore rules were not honoured *in this run*

`@electron/osx-sign@1.3.3/dist/cjs/sign.js:52-56`:

```js
function validateOptsIgnore(ignore) {
    if (ignore && !(ignore instanceof Array)) {
        return [ignore];
    }
}
```

There is no `return ignore` for the array branch, so an **array** yields
`undefined` (`sign.js:70` `ignore: validateOptsIgnore(opts.ignore)`), and
`shouldIgnoreFilePath` then returns `false` for everything. The `v0.300.0` hook
hands it an array (`signingIgnoreRules()` returns `[...upstreamIgnore, fn]`),
therefore:

- every ignore rule is discarded — **including electron-builder's own**, which is
  a function built at `app-builder-lib/out/mac/MacTargetHelper.js:90-107` and
  already excludes `.kext`, `/Contents/PlugIns`, `puppeteer/.local-chromium`,
  `playwright/.local-browsers`, plus any `mac.signIgnore` regexes (`:53-71`);
- `grep -c "Skipped" run-29752745259.log` → **0**. Not one path was skipped in
  the entire run, which is exactly what "ignore === undefined" predicts;
- `grep -c "adhoc-sign-mac" …` → 6, and all six are the *step command echo*
  (`:8395`, `:8396`, `:8463`, `:13018`, `:13019`, `:13086`). There is **no**
  `[adhoc-sign-mac] hook entered` and **no** `considered=/skipped=` line, because
  the tag predates the instrumentation. `FORMAL_AI_MACOS_SIGN_DEBUG: 1` is set
  (`:8411`, `:13034`), so this is not an env problem.

Contrast with the run that *does* contain the fix — PR run 29746221627, log
`dev/log/issues/810/pulls/811/ci-logs/adhoc-sign-mac-29746221627.log`:

- `:4` `[adhoc-sign-mac] hook entered (debug=on)`;
- `:738` and `:1481` `[adhoc-sign-mac] ignore predicate: considered=1479 skipped=751`;
- and `gh run view 29746221627` reports `Build macos-x64 success` /
  `Build macos-arm64 success`.

So the function-form `ignore` fix demonstrably works end to end. Note also that
electron-builder passes `ignore` as a **function** (`MacTargetHelper.js:90`), not
an array — the §3 remark in the #810 analysis that electron-builder "forwards it
as an array" is inaccurate; what it forwards is a closure that has already
compiled `mac.signIgnore` into regexes. Simply passing `signOptions.ignore`
through untouched would already have excluded the browser runtime once
`signIgnore` existed.

Upstream status: the bug is **fixed in `@electron/osx-sign` v2**
(`@electron/osx-sign@2.6.0/dist/sign.js:22-26` reads
`return Array.isArray(ignore) ? ignore : [ignore]`). `app-builder-lib@26.15.3`
pins `@electron/osx-sign: 1.3.3` (`desktop/package-lock.json:1270`), so the
1.3.3 behaviour is what we get today. A backport request to electron-builder
(bump the pin) is the only upstream action worth filing; the osx-sign bug itself
is already resolved upstream.

## 3. What "unsealed contents present in the root directory of an embedded framework" means

`codesign` treats a *versioned* bundle (a `.framework` with a `Versions/`
directory) specially. The seal — `_CodeSignature/CodeResources` — lives inside
the versioned directory (`Versions/<v>/Resources/_CodeSignature`). Consequently
the framework's **root directory may contain only** the `Versions` directory and
symlinks that point into `Versions/Current/…`. Any real file or directory sitting
in the root cannot be covered by any seal, and when such a framework is *embedded*
in another bundle `codesign` refuses with this exact message. Apple's developer
forum thread 93914 documents the canonical case (Qt shipping `.prl` files in the
framework root) and states the required layout: only symlinks at the root, all
real content under `Versions/`, plus a `Versions/A` alias
(<https://developer.apple.com/forums/thread/93914>). The same diagnosis appears in
[sparkle-project/Sparkle#1471](https://github.com/sparkle-project/Sparkle/issues/1471),
[facebook/facebook-ios-sdk#2130](https://github.com/facebook/facebook-ios-sdk/issues/2130),
[golang/go#66406](https://github.com/golang/go/issues/66406) and
<https://indiespark.top/programming/code-signing-failure-due-to-symlink-folders/>
(symlink-shaped root entries), and Craig Hockenberry's
<https://furbo.org/2020/12/01/codesign-the-saga-continues/> for how brittle the
sealing rules are in practice.

How the Chrome for Testing framework violates it *as we ship it*:

- Its layout is `…Framework.framework/{Versions/149.0.7827.55, Versions/Current →
  149.0.7827.55, Helpers →, Libraries →, Resources →, Google Chrome for Testing
  Framework →}` — the run enumerates both `Versions/149.0.7827.55/…`
  (`run-29752745259.log:8844`) and the alias path `Versions/Current/…`
  (`:8846`), i.e. `walkAsync` traversed the symlink and produced duplicate
  children (`osx-sign/dist/cjs/util.js:150-167` — it `fs.stat`s, which follows
  links, so symlinked directories are descended into).
- At `v0.300.0` those aliases are **absolute symlinks pointing outside the app**,
  because `prepare-resources.mjs` copied the Playwright cache with
  `fs.cpSync(from, to, { recursive: true })` and Node's default
  `verbatimSymlinks: false` rewrites link targets to resolved absolute paths.
  That is proved independently by
  `dev/log/issues/808/pulls/809/2026-07-20-symlink-root-cause.md`, which records
  `codesign --verify --deep` printing `file modified:` for exactly
  `…Framework.framework/{Resources,Versions/Current,Libraries,Helpers}` and then
  `invalid destination for symbolic link in bundle`.
- Root entries whose targets resolve outside the bundle cannot be sealed by the
  framework's own `CodeResources`, which is the condition `codesign` reports as
  "unsealed contents present in the root directory of an embedded framework".

**Unproven, stated plainly:** we have no `ls -l` of the framework root from a CI
runner, so the final step of that chain (absolute-outside symlinks ⇒ *this*
particular message rather than some other) is inference from the two logs plus
Apple's documented rule, not a direct observation. Equally unproven is whether a
*correctly* copied (relative-symlink) Chrome for Testing framework could be
re-signed successfully at all — every green run we have skips it entirely, so
"do not sign it" is the only path that has ever been demonstrated to work.

Related upstream reports on the general shape of this problem:
[electron-userland/electron-builder#2010](https://github.com/electron-userland/electron-builder/issues/2010)
("Codesigning fails when adding Puppeteer as a dependency") and
[#5383](https://github.com/electron-userland/electron-builder/issues/5383) — both
are cited in electron-builder's own source comment at
`app-builder-lib/out/mac/MacTargetHelper.js:102-105`, which is why it hardcodes
skips for `puppeteer/.local-chromium` and `playwright/.local-browsers`. Our
runtime lives at `Contents/Resources/browser-runtime`, a path upstream cannot
know about, hence the need for `mac.signIgnore`.

## 4. Existing components that solve (parts of) this

| Component | Does it help? | Reference |
| --- | --- | --- |
| `@electron/osx-sign` `ignore` | Yes — a function or (in v2) an array of matchers skips paths entirely. v1.3.3 silently drops arrays. | `dist/cjs/sign.js:52`, v2.6.0 `dist/sign.js:22`; <https://www.npmjs.com/package/@electron/osx-sign> |
| electron-builder `mac.signIgnore` | Yes — compiled into the `ignore` closure it hands to osx-sign. Already set on `main`. | `MacTargetHelper.js:53-71,90-97`; <https://www.electron.build/electron-builder.Interface.MacConfiguration.html> |
| electron-builder built-in browser skips | Only for `node_modules/puppeteer/.local-chromium` and `node_modules/playwright/.local-browsers`. | `MacTargetHelper.js:99-106` |
| `asarUnpack` | Irrelevant — `browser-runtime` is `extraResources` (`desktop/package.json:59-64`), never inside the asar. | — |
| `extraResources` + archive | Yes, structurally: a `.zip`/`.tar` is one opaque file; `walkAsync` sees a non-bundle blob, `codesign` never inspects a framework. Costs a first-run extraction. | — |
| `@puppeteer/browsers` / `playwright install` at runtime | Yes, structurally: nothing browser-shaped is inside the `.app` at all; install into `app.getPath('userData')`. | <https://pptr.dev/browsers-api>, <https://www.npmjs.com/package/@puppeteer/browsers> |
| Use the user's installed Chrome (`executablePath`) | Yes, but changes product behaviour. | [puppeteer/puppeteer#4655](https://github.com/puppeteer/puppeteer/issues/4655) |

## 5. Ranked fixes

### F1 — Cut a new tag (unblocks the *actual* failure). Process fix, mandatory.

Nothing in `desktop/` needs to change for run 29752745259 to stop failing: `main`
already packages cleanly (run 29746221627, both macOS jobs `success`). What is
broken is that the only ref the healing workflow will ever build is
`needs.resolve.outputs.tag` = the latest release = `v0.300.0`
(`.github/workflows/desktop-release.yml:173`), and `Auto Release` cannot produce
a newer one: on the same head SHA, `run-29751001867.log:1418` (job `Auto Release`)
fails with

```
Error recording self-hosting release metric: self-hosting ratchet would fall from 32.68% to 18.24% for v0.301.0
```

This is structurally the same deadlock as Defect A in #810 (a gate that a release
must pass, whose only escape is a release). Until `v0.301.0` exists, Desktop
Release will keep rebuilding broken packaging code. **This is the fix to do
first**, and it belongs to the release gate, not to the signer.

### F2 — Never sign the browser runtime, by construction. Structural packaging fix.

Ship the runtime as an opaque archive and expand it on first launch, so no
`codesign` walk can ever reach a framework we did not lay out:

```diff
 // desktop/scripts/prepare-resources.mjs
-  copyDirectory(browserSource, outputBrowser);
+  // Do not stage a browser *tree* into the app: codesign walks Contents/**
+  // and refuses to re-seal Chrome's versioned framework. One opaque blob is
+  // unwalkable and preserves Google's signature byte for byte.
+  spawnSync("/usr/bin/ditto", ["-c", "-k", "--keepParent", browserSource,
+                               path.join(desktopDir, "browser-runtime.zip")]);
```

```diff
 // desktop/package.json ("build".extraResources)
-      { "from": "browser-runtime", "to": "browser-runtime", "filter": ["**/*"] }
+      { "from": "browser-runtime.zip", "to": "browser-runtime.zip" }
```

plus an idempotent first-run expansion into `app.getPath('userData')` in
`desktop/lib/…` before the browser is launched. `mac.signIgnore` and the custom
hook's browser predicate then become belt-and-braces rather than load-bearing.
This removes the whole class ("codesign co-signing a foreign bundle"), which
neither ignore tuning nor symlink flags do.

### F3 — Do not bundle a browser at all. Strongest structurally, biggest behaviour change.

Download Chrome for Testing on first use with `@puppeteer/browsers` (or
`playwright install chromium`) into `app.getPath('userData')`
(<https://pptr.dev/browsers-api>). App size drops by ~200 MB, macOS signing
becomes ordinary, and the runtime is updatable without a release. Cost: first-run
network dependency and its offline error path.

### F4 — Make the packaging contract un-regressible in the signer. Cheap, already half-done.

Keep the function-form `ignore` (on `main` since `a6f0f040`) and, in addition,
stop *rebuilding* the matcher list — compose onto whatever upstream gave us:

```diff
-  return [...upstreamIgnore, (filePath) => isBundledBrowserRuntime(filePath, appPath)];
+  // Always a single function: @electron/osx-sign@1.3.3 drops arrays
+  // (dist/cjs/sign.js:52), which silently discards electron-builder's own
+  // kext/PlugIns/signIgnore rules as well as ours.
+  return (filePath) => matchers.some(m => typeof m === "function" ? m(filePath) : !!filePath.match(m));
```

and pin the fixed upstream through npm `overrides`:

```diff
 // desktop/package.json
+  "overrides": { "@electron/osx-sign": "^2.6.0" }
```

(verify against electron-builder 26 first — `optionsForFile` gained a second
argument in v2, `sign.js:190`, which our hook ignores harmlessly).

### F5 — Stop cutting tags that cannot be packaged. Prevents recurrence of F1.

The workflow's triggers are `release`, `workflow_dispatch`, `workflow_run` on
"CI/CD Pipeline", and `pull_request` (path-filtered) —
`desktop-release.yml:36-64`. The PR dry run (added for #808, see the comment at
`:43-49`) validates the *pull request head*, not the merge result, and there is
no `push` trigger, so a tag can still be cut from a `main` on which macOS
packaging never ran. Add a `push: branches: [main]` trigger for the macOS build matrix
(dry run, `--publish never`, no upload) so a green `main` implies a packageable
tag; or, cheaper, have `desktop-release-resolve.sh` refuse to "heal" a release
whose commit predates the current packaging contract instead of retrying it
forever.

Ordering rationale: F1 is required to clear the current red build; F2 (or F3) is
the one that removes the defect class; F4 hardens the code that will still exist
either way; F5 stops the process from re-creating F1.

## 6. What remains unproven

1. Whether the Chrome for Testing framework can be re-signed *at all* once its
   symlinks are correct — never tested; all green runs skip it.
2. The precise `codesign` internal predicate that emits "unsealed contents" (see
   §3). Inferred from Apple's documented layout rule and the #808 symlink log,
   not from a direct listing of the packaged framework.
3. Whether `@electron/osx-sign@2.6.0` is drop-in compatible with
   `app-builder-lib@26.15.3` (F4's `overrides`) — not attempted here.
4. Whether the `Auto Release` ratchet failure (`run-29751001867.log:1418`) has a
   further cause beyond the changed-lines arithmetic; it is outside this
   investigation's scope but it is the gate that keeps F1 blocked.
