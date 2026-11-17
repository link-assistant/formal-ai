## Reproduction

At `a5b9a1a46686cfe844f9afa2ace030094969e766`, two valid pending changesets can disappear while `scripts/merge-changesets.mjs` reports success. One input has a name the merger can generate: `merged-bright-apple.md`; the other is `other.md`. Their front matter requests `patch` and `major`, and both have nonempty descriptions.

Force the random name only to make the existing collision deterministic, then execute the unchanged source:

```sh
node --max-old-space-size=64 --import 'data:text/javascript,Math.random%3D()%3D%3E0' scripts/merge-changesets.mjs
```

Observed: exit 0, `Changeset merge completed successfully`, and an empty `.changeset` directory. Both descriptions and the major bump are lost. This requires valid inputs; it is distinct from the invalid-parse loss previously addressed by #78/#83.

## Cause and candidate guard

Lines 213-217 choose a random output name and use ordinary `writeFileSync`. If that name equals a pending input path, the merged output overwrites the input. Lines 222-225 then unlink every parsed input, including that output. No merged file survives for versioning.

The disposable candidate changes only the output write to `writeFileSync(mergedFilePath, mergedContent, {flag: 'wx'})`. A collision then raises `EEXIST` and exits 1 with both original inputs byte-for-byte intact. A full repair could reserve a distinct name with exclusive creation and a finite collision retry; it must never delete any input until a distinct output is safely established. The tested candidate rejects collisions; automatic name recovery is not tested or claimed.

## Executable evidence

- [Content-addressed pinned source and license](https://github.com/link-assistant/formal-ai/tree/EVIDENCE_COMMIT/experiments/issue-1580/fixtures/csharp-template).
- [Four-case regression](https://github.com/link-assistant/formal-ai/blob/EVIDENCE_COMMIT/experiments/issue-1580/template-csharp-changeset-integrity.test.mjs).
- [Before/after log and identity index](https://github.com/link-assistant/formal-ai/blob/EVIDENCE_COMMIT/experiments/issue-1580/evidence/checks/csharp-changeset-integrity.json).

Run `node --test --test-concurrency=1 experiments/issue-1580/template-csharp-changeset-integrity.test.mjs` in that checkout. It executes unchanged pinned helpers in small temporary directories, then applies the candidate only to disposable copies. The preservation assertion fails against the original (3 pass / 1 fail) and all four cases pass after the guard is inserted. Positive controls retain the major bump and both descriptions in chronological order; malformed-input controls retain the existing fatal, nonmutating behavior. The maintained entrypoint also passes all 25 tests, without skips.

No .NET build, Git repository, provider API, package publication or hosted template workflow is involved. All random-source injection, candidate changes and conclusions are supplied by Codex; Formal AI shell execution is not counted as independent reasoning. The C# repository review remains partial.
