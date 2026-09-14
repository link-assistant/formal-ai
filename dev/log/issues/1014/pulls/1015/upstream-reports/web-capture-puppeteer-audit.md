## Reproduction

`@link-assistant/web-capture@1.11.2` declares `puppeteer: ^24.8.2`. A clean
consumer lock currently resolves Puppeteer 24 and its
`@puppeteer/browsers -> extract-zip@2.0.1` chain:

```bash
workdir="$(mktemp -d)"
cd "$workdir"
npm init -y
npm install --package-lock-only @link-assistant/web-capture@1.11.2
npm audit --package-lock-only --audit-level=high
```

The audit exits non-zero for GHSA-8jqr-rrrh-j9v7, the `extract-zip` path
traversal advisory. `extract-zip` has no release newer than 2.0.1, so updating
inside Puppeteer 24 cannot clear the finding.

## Workaround

Consumers can apply a scoped npm override for the two browser packages and run
their normal web-capture tests:

```json
{
  "overrides": {
    "@link-assistant/web-capture": {
      "puppeteer": "^25.7.0",
      "puppeteer-core": "^25.7.0"
    }
  }
}
```

Puppeteer 25 uses `@puppeteer/browsers` 3, whose archive implementation no
longer depends on `extract-zip`. link-assistant/formal-ai validated the override
with its desktop and VS Code suites while fixing #1014.

## Suggested code fix

Move `puppeteer` and `puppeteer-core` to the compatible 25.x line, exercise both
capture engines in CI, and add `npm audit --package-lock-only
--audit-level=moderate` as a required gate so future transitive advisories do
not remain install-time text with a successful exit code.

