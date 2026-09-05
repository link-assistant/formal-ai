# js template: the npm tarball fallback has no retry, and a truncation leaves a half-populated npm directory

Filed against `link-foundation/js-ai-driven-development-pipeline-template`.

## What happens

`scripts/setup-npm.mjs` installs npm through four strategies in order. The
second one downloads the release tarball and pipes it straight into `tar`:

```js
await $`curl -fsSL "${npmRelease.tarballUrl}" | tar xz --strip-components=1 -C "${tempNpmDir}" && rm -rf "${globalNpmDir}" && mv "${tempNpmDir}" "${globalNpmDir}"`;
```

There is no `--retry`. A mid-transfer truncation -- the server promises a
`Content-Length` it does not deliver and hangs up, which is curl exit 18 --
aborts the strategy, and `tar` has already written part of the archive into
`tempNpmDir` by then.

This is the same defect class that took a build of ours red on a commit that
changed no code: a 345 MB vendor tarball stopped arriving mid-transfer and
`curl -fsSL` reported it as final
(`curl: (18) transfer closed with 344439862 bytes remaining to read`).

## Reproduction

`experiments/issue-1076/repro-npm-tarball-truncation.sh` in
link-assistant/formal-ai serves a real gzipped tar, truncating the first
response and answering honestly afterwards. Nothing touches the network.

```text
upstream:
    curl: (18) end of response with 1166 bytes missing
    gzip: stdin: unexpected end of file
    tar: Unexpected EOF in archive
    tar: Error is not recoverable: exiting now
  exit=2  files=1

file-first:
  exit=0  files=2
```

`files=1` on the failing run is the second half of the finding: `tempNpmDir` is
left with part of an npm.

## Workaround

None is needed at the call site -- strategy 3 (`npx npm@11 install -g npm@11`)
covers the failure, which is why this is a resilience defect rather than an
outage. It does cost the strategy-2 attempt and its full download every time a
transfer drops.

## Suggested fix

Download to a file first, then extract, and retry the download:

```js
const tarball = '/tmp/setup-npm-package.tgz';
await $`curl -fsSL --retry 3 --retry-delay 2 --retry-all-errors "${npmRelease.tarballUrl}" -o "${tarball}"`;
await $`rm -rf "${tempNpmDir}" && mkdir -p "${tempNpmDir}"`;
await $`tar xz --strip-components=1 -C "${tempNpmDir}" -f "${tarball}"`;
```

Two details worth keeping:

- `--retry` on its own does **not** cover curl exit 18. Measured against curl
  8.20.0, `curl -fsSL --retry 3 --retry-delay 1` still exits 18 on a truncated
  response; `--retry-all-errors` is what retries it.
- Retry belongs on a download that writes to a file, not on one piped into an
  extractor: a retried transfer restarts from the beginning, so the extractor
  can be handed the head of the file twice.
