# Upstream report 5 - "reproduce locally with `zizmor==1.30.0`" names a version the pipeline cannot run

**Targets:** `rust`, `js`, `python`, `php`
`link-foundation/*-ai-driven-development-pipeline-template` repositories
(the four that ship a zizmor job)

**Severity:** low, and worth a two-line fix. A maintainer following the
documented reproduction runs a different analyser than CI does, so a local
green and a CI red are both believable and neither is checkable.

---

## Title

The zizmor job leaves `version:` at its default and documents a version the
action cannot install; `zizmor-action@v0.6.2`'s `latest` is frozen at 1.29.0

## Affected sites

`rust-.../.github/workflows/workflows.yml:53` and
`python-.../.github/workflows/workflows.yml:50` both say:

```
#   pipx run zizmor==1.30.0 --config .github/zizmor.yml \
#     --min-confidence medium --persona regular .github/workflows
```

and all four `zizmor` steps omit `version:`:

```yaml
      - uses: zizmorcore/zizmor-action@v0.6.2
        with:
          advanced-security: false
          annotations: true
          config: .github/zizmor.yml
          min-confidence: medium
```

## Root cause

`zizmor-action` does not query anything at runtime. It resolves the version
from a static table shipped inside the action, and refuses anything absent
from it:

```bash
# action.sh
declare -A versions
while IFS=' ' read -r version digest; do
    versions["${version}"]="${digest}"
done < "${GITHUB_ACTION_PATH}/support/versions"
...
digest="${versions[${normalized_version}]:-}"
if [[ -z "${digest}" ]]; then
    die "Unknown version: ${GHA_ZIZMOR_VERSION}"
fi
image="ghcr.io/zizmorcore/zizmor:${normalized_version}@${digest}"
```

v0.6.2's table has 37 rows. The last two are the whole story:

```console
$ curl -sL https://raw.githubusercontent.com/zizmorcore/zizmor-action/v0.6.2/support/versions | sed -n '1p;$p'
latest sha256:863026d54f91271b10b60b67ad8054cb37120167e162482597db102b3026a284
1.29.0 sha256:863026d54f91271b10b60b67ad8054cb37120167e162482597db102b3026a284
```

`latest` and `1.29.0` are the same digest. So two things follow, and the second
is the good news:

1. **The documented command is unrunnable in CI.** `version: 1.30.0` would
   `die "Unknown version"`. The comment tells a maintainer to reproduce with an
   analyser the pipeline is incapable of running.
2. **`latest` does not float.** It is pinned to whatever was current when the
   action was tagged, and it moves only when the action tag moves. That is a
   good design -- it means a zizmor release cannot turn CI red on its own -- but
   it also means the word `latest` in these workflows is misleading in the
   opposite direction from what it usually is.

The versions do differ in what they evaluate. Same tree, same config, same
flags, only the binary changed -- measured on `link-assistant/formal-ai`'s
pipeline:

```console
$ zizmor --config .github/zizmor.yml --min-confidence medium --persona regular \
         .github/workflows .github/actions
# 1.30.0: No findings to report. Good job! (99 ignored, 99 suppressed)
# 1.29.0: No findings to report. Good job! (65 ignored, 99 suppressed)
```

34 findings exist for 1.30.0 that do not exist for 1.29.0 on identical input.
They happen to be suppressed here, which is luck rather than design: the next
repository to follow the comment can just as easily get 34 findings CI will
never show it, or spend an afternoon reproducing a CI failure against a binary
that does not produce it.

## Suggested fix

Two lines, one per site:

```diff
       - uses: zizmorcore/zizmor-action@v0.6.2
         with:
           advanced-security: false
           annotations: true
           config: .github/zizmor.yml
           min-confidence: medium
+          # Named rather than left at `latest`, because `latest` here is the
+          # action's own frozen table entry (v0.6.2 -> 1.29.0), not the latest
+          # zizmor. Naming it keeps the comment above true and makes the next
+          # bump a visible line in the diff rather than a side effect of
+          # bumping the action.
+          version: 1.29.0
```

```diff
-#   pipx run zizmor==1.30.0 --config .github/zizmor.yml \
+#   pipx run zizmor==1.29.0 --config .github/zizmor.yml \
```

Bumping the action and the version together is fine and preferable; the point
is that the two numbers should be in the diff together.

## Related

- <https://github.com/zizmorcore/zizmor-action/blob/v0.6.2/support/versions>
- <https://github.com/link-assistant/formal-ai/pull/1080> -- the downstream fix.
