## Problem

`scripts/check-web-archive.mjs` classifies every Markdown bullet containing an
HTTP URL as broken. Lychee's Markdown report uses the same bullet syntax in its
`## Redirects per input` section for links that succeeded after redirecting.
Consequently, one real failure plus 18 healthy redirects is reported as 19
broken URLs; the helper sends all 19 to Wayback and can emit false error
annotations for successful links whose old URL has no snapshot.

The affected script blob is
`2b8244d7d76d56d9acdf88b4ea766e35c554b1fe`.

## Reproduction

Save this as `/tmp/lychee.md`:

```markdown
## Errors per input

### Errors in docs/reference.md

* [502] <https://broken.example/reference> (at 1:1) | Rejected status code: 502

## Redirects per input

### Redirects in README.md

* https://working.example/old --[301]--> https://working.example/current
```

Then run:

```sh
LYCHEE_OUTPUT=/tmp/lychee.md node scripts/check-web-archive.mjs
```

The first summary says `Found 2 broken URL(s)` and checks
`https://working.example/old`, although Lychee placed that URL in the successful
redirect section. The expected count is one.

This occurred in a real link-assistant/formal-ai#999 CI run: Lychee reported
1 error and 18 redirects, while the helper reported 19 broken URLs.

## Workarounds

- Read Lychee's own error count/list instead of the helper annotations.
- Before invoking the helper, remove the report from `## Redirects per input`
  onward. This is only safe while `Errors per input` precedes later sections.

## Suggested code fix

When `## Errors per input` exists, restrict both URL regular expressions to
that section (ending at the next level-two heading). Retain full-document
parsing only as a compatibility fallback for older/plain Lychee output. Export
the parser behind a direct-execution guard and add a `node:test` fixture with
one error and one redirect, asserting that only the error is returned.

The downstream regression test and fix were developed while auditing
link-assistant/formal-ai#999.
