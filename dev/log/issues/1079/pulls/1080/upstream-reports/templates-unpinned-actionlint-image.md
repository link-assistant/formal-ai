# Upstream report 2 - the `'*': hash-pin` policy does not reach `docker://rhysd/actionlint:1.7.7`, because `unpinned-images` never runs

**Targets:** all five
`link-foundation/*-ai-driven-development-pipeline-template` repositories
(`rust`, `js`, `python`, `php`, `csharp`)

**Severity:** a mutable third-party Docker tag executes with the repository
checked out on every push and pull request, under a written policy that says it
must not, and no configured gate can say so.

---

## Title

`.github/zizmor.yml` declares `'*': hash-pin`, but the actionlint container is
pinned by tag; `unpinned-uses` only covers *action* references and the
`unpinned-images` audit that covers container references is Pedantic-persona

## Affected sites

```
rust-ai-driven-development-pipeline-template    .github/workflows/workflows.yml:31  uses: docker://rhysd/actionlint:1.7.7
js-ai-driven-development-pipeline-template      .github/workflows/workflows.yml:46  uses: docker://rhysd/actionlint:1.7.7
python-ai-driven-development-pipeline-template  .github/workflows/workflows.yml:42  uses: docker://rhysd/actionlint:1.7.7
php-ai-driven-development-pipeline-template     .github/workflows/workflows.yml:42  uses: docker://rhysd/actionlint:1.7.7
csharp-ai-driven-development-pipeline-template  .github/workflows/workflows.yml:42  uses: docker://rhysd/actionlint:1.7.7
```

The `js` template carries a second, unrelated instance of the same class,
suppressed by the same persona split:

```
js-ai-driven-development-pipeline-template      .github/workflows/example-app.yml:231  image: mcr.microsoft.com/playwright:v1.59.1-noble
```

`rust`, `js`, `python` and `php` each declare, in `.github/zizmor.yml`:

```yaml
rules:
  unpinned-uses:
    config:
      policies:
        actions/*: ref-pin
        github/*: ref-pin
        docker/*: ref-pin
        astral-sh/*: ref-pin
        lycheeverse/*: ref-pin
        zizmorcore/*: ref-pin
        '*': hash-pin
```

`rhysd/*` is not on the allow-list, so the declared intent is that this
reference must carry a hash. It does not, and the audit that would say so is
not the one being run.

(The `csharp` template has no zizmor job at all -- reported separately -- so
there the reference is simply unchecked.)

## Reproducible example

zizmor 1.30.0, against the rust template at
`4d444d976f6bb46168b9f4a7001c775bf7c0ef6b`:

```console
$ zizmor --config .github/zizmor.yml --min-confidence medium \
         --persona regular .github/workflows/workflows.yml
No findings to report. Good job! (1 suppressed)
$ echo $?
0
```

That is exactly the invocation the job runs -- `zizmorcore/zizmor-action@v0.6.2`
with `min-confidence: medium` and the default persona, as the comment above the
step documents:

```
#   pipx run zizmor==1.30.0 --config .github/zizmor.yml \
#     --min-confidence medium --persona regular .github/workflows
```

Raise only the persona, change nothing else:

```console
$ zizmor --config .github/zizmor.yml --min-confidence medium \
         --persona pedantic .github/workflows/workflows.yml
error[unpinned-images]: unpinned image references
  --> .github/workflows/workflows.yml:31:24
   |
31 |       - uses: docker://rhysd/actionlint:1.7.7
   |                        ^^^^^^^^^^^^^^^^^^^^^^ container image is not pinned to a SHA256 hash
   |
   = note: audit confidence → High
   = help: audit documentation → https://docs.zizmor.sh/audits/#unpinned-images

1 finding: 0 informational, 0 low, 0 medium, 1 high
```

**High severity, High confidence** -- suppressed purely by persona. The same
command reproduces on all five templates.

## Root cause

Two separate audits, and the intuitive reading maps them the wrong way round:

| audit | covers | persona |
|---|---|---|
| `unpinned-uses` | `uses:` **action** references (`owner/repo@ref`) | Regular |
| `unpinned-images` | `uses: docker://` and `container:` **image** references | Pedantic |

`unpinned-uses` is the audit the `policies:` block configures, and it is the
one that runs by default -- but it does not look at container references at
all. `unpinned-images` does, and it never runs, because the job uses the
default `regular` persona.

So the policy block reads as though it governs every third-party reference in
the file, and in practice governs only some of them. Nothing in the
configuration hints at the split; the templates' own comment ("third-party
actions must be pinned to a commit hash, because a mutable tag or branch of a
repository we do not control is arbitrary code execution in any job that holds
credentials") states the reasoning that applies just as much to the image.

## Why this matters here specifically

The actionlint step runs `actions/checkout` first, so the image executes with
the repository's working tree mounted. `1.7.7` is a mutable Docker tag: whoever
controls the `rhysd/actionlint` repository -- or anyone who obtains a
credential for it -- can change what it resolves to without any change landing
in these repositories, and every fork of every template picks it up on the next
run. That is precisely the threat model the `hash-pin` policy was written for.

## Suggested fix

Two independent changes; either alone is an improvement, and both are cheap.

**1. Pin the image.** Digests resolved from Docker Hub on 2026-09-07:

```
rhysd/actionlint:1.7.7   sha256:887a259a5a534f3c4f36cb02dca341673c6089431057242cdc931e9f133147e9
rhysd/actionlint:1.7.12  sha256:b1934ee5f1c509618f2508e6eb47ee0d3520686341fec936f3b79331f9315667
```

```diff
-      - uses: docker://rhysd/actionlint:1.7.7
+      # Re-resolve when bumping:
+      #   docker pull rhysd/actionlint:<tag> &&
+      #   docker inspect --format='{{index .RepoDigests 0}}' rhysd/actionlint:<tag>
+      - uses: docker://rhysd/actionlint@sha256:b1934ee5f1c509618f2508e6eb47ee0d3520686341fec936f3b79331f9315667
         with:
           args: -color
```

`1.7.12` is the current release; `1.7.7` dates from the templates' original
adoption. Bumping and pinning in one step is one review rather than two.

The `csharp` template needs one more line changed in the same commit, because
its own test asserts the tag form and would start failing:

```js
// scripts/workflow-injection-policy.test.mjs:117
expect(workflow).toContain('uses: docker://rhysd/actionlint:');
```

```diff
-    expect(workflow).toContain('uses: docker://rhysd/actionlint:');
+    expect(workflow).toContain('uses: docker://rhysd/actionlint@sha256:');
```

Keeping that assertion (adjusted) is worth doing: it is what stops a future
edit quietly reverting to a native binary, and after the change it also pins
the *pinning*.

**2. Make the policy able to see the reference.** The obvious-looking
configuration does not exist, so this is worth stating precisely rather than
leaving as an exercise. zizmor 1.30.0 has **no per-rule persona override**:

```console
$ cat zizmor-persona.yml
rules:
  unpinned-images:
    persona: regular
$ zizmor --config zizmor-persona.yml --persona regular .github/workflows
fatal: no audit was performed
error: configuration error in zizmor-persona.yml
Caused by:
    2: rules.unpinned-images: unknown field `persona`, expected one of
       `disable`, `ignore`, `config`, `remap` at line 3 column 5
```

`remap` is not a way around it either -- it accepts only `severity`, and
severity is not what filters the finding:

```console
$ cat zizmor-remap.yml
rules:
  unpinned-images:
    remap:
      severity: high
$ zizmor --config zizmor-remap.yml --persona regular .github/workflows
... (unpinned-images still absent)
5 findings (3 suppressed, 1 unsafe fixes): 0 informational, 0 low, 1 medium, 1 high
```

The finding is one of the three suppressed. Persona is applied before severity,
so no `rules:` entry can promote a Pedantic audit into a Regular run.

What does work, and costs one step, is a **second, narrow pass** beside the
existing one -- pedantic, but filtered to high severity *and* high confidence,
which is where `unpinned-images` sits and where the pedantic persona's usual
noise does not:

```yaml
      # The job above runs `--persona regular`, which is the right default:
      # pedantic surfaces dozens of stylistic findings. But two audits that
      # matter here are Pedantic-only, `unpinned-images` among them, so the
      # `'*': hash-pin` policy declared in `.github/zizmor.yml` was never
      # actually enforced against container references. Narrowing the pedantic
      # run to high severity *and* high confidence keeps that enforcement
      # without importing the noise.
      - name: 'Audit the pipeline for pedantic-only high-severity findings'
        run: |
          pipx run zizmor==1.30.0 --config .github/zizmor.yml \
            --persona pedantic --min-severity high --min-confidence high \
            .github/workflows .github/actions
```

Measured on each template at the commits listed above -- total findings the
pedantic persona produces, versus how many this filter actually surfaces:

| template | pedantic findings | surfaced by the narrow pass | what surfaces |
|---|---:|---:|---|
| `rust` | 54 | 1 | `workflows.yml:31` actionlint image |
| `js` | 59 | 2 | `workflows.yml:46` actionlint image; `example-app.yml:231` `mcr.microsoft.com/playwright:v1.59.1-noble` |
| `python` | 39 | 1 | `workflows.yml:42` actionlint image |
| `php` | 20 | 1 | `workflows.yml:42` actionlint image |
| `csharp` | 73 | 7 | actionlint image, plus 4 `template-injection` and 2 `excessive-permissions` that no zizmor job is currently running to catch |

So on four of the five templates the entire cost of closing this gap is one
step that reports exactly the defect this issue is about, and nothing else.
After fix 1 lands it reports nothing at all, which is the point: it is then a
gate that stays silent until someone reintroduces a mutable image.

(The `js` row shows why the fix is worth making general rather than editing the
one line: the same class of defect is already present a second time in that
template, in a container the actionlint change would not have touched. The
`csharp` row is the subject of a separate report -- that template has no zizmor
job, so those six other findings are not suppressed by persona, they are simply
never looked for.)

## Workaround, until this lands

A consumer of the template can add the digest locally, which is what
`link-assistant/formal-ai` did
([PR #1080](https://github.com/link-assistant/formal-ai/pull/1080)):
both references to the image -- the lint step and the ShellCheck canary that
proves the lint step is real -- carry the same digest, and a test asserts they
stay identical and stay digest-pinned.

Consumers can also see the gap without changing their job, by running the
narrow pedantic pass out of band:

```console
$ zizmor --config .github/zizmor.yml \
         --persona pedantic --min-severity high --min-confidence high \
         .github/workflows .github/actions
```

`link-assistant/formal-ai` measured that on its own pipeline: 198 pedantic
findings, exactly one surfaced -- and after the digest pin, the only remaining
one is a deliberately-floating base image carrying a written exception. A
consumer can therefore adopt fix 2 before the template does, and get a green
check on the first run.

## Related

- <https://docs.zizmor.sh/audits/#unpinned-images> -- the audit and its
  Pedantic default.
- <https://docs.zizmor.sh/audits/#unpinned-uses> -- the audit the `policies:`
  block configures.
