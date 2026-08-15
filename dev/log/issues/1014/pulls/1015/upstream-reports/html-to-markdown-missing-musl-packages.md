## Reproduction

The published `@kreuzberg/html-to-markdown-node@3.7.2` metadata names two musl
optional packages that are not published at that version:

```bash
npm view @kreuzberg/html-to-markdown-node@3.7.2 optionalDependencies
npm view @kreuzberg/html-to-markdown-node-linux-x64-musl@3.7.2 version
npm view @kreuzberg/html-to-markdown-node-linux-arm64-musl@3.7.2 version
```

Both platform-package lookups return `E404`. With npm 11.17.0 this can produce
a lock that a subsequent clean install rejects:

```text
npm error `npm ci` can only install packages when your package.json and package-lock.json ... are in sync.
npm error Missing: @kreuzberg/html-to-markdown-node-linux-arm64-musl@ from lock file
npm error Missing: @kreuzberg/html-to-markdown-node-linux-x64-musl@ from lock file
```

## Workaround

Use npm 10 to generate and consume the lock on glibc targets, or pin to a
release whose complete platform package set is available. Do not hand-add lock
entries for tarballs that do not exist.

## Suggested code fix

Publish the two 3.7.2 musl artifacts, or remove them from the parent package's
3.7.2 optional dependency metadata. Add a release smoke test that queries every
declared platform package at the exact parent version and runs `npm ci` with
both the current and next npm majors.

