# electron-userland/electron-builder — a recovered toolset-download timeout still fails the build

Filed as <https://github.com/electron-userland/electron-builder/issues/10091>
against `electron-builder` 26.15.7 on 2026-08-17. Distinct from
[#9750](https://github.com/electron-userland/electron-builder/issues/9750)
("Slow electron downloads are now forcibly aborted after 600000ms"), which was a
proxy-detection regression closed by
[#9754](https://github.com/electron-userland/electron-builder/pull/9754): here
the download **completes**, every artifact is written, and the build fails
afterwards over the request that was already got past.

---

## Issue body

> Distinct from #9750 ("Slow electron downloads are now forcibly aborted after
> 600000ms"), which was a proxy-detection regression closed by #9754: here the
> download **completes**, every artifact is written, and the build fails
> afterwards over the request that was already got past.

### Summary

On a GitHub-hosted `macos-15-intel` runner, `electron-builder --mac --publish never`
produced a complete DMG, a complete ZIP and both blockmaps, emitted **both**
`artifactBuildCompleted` events — and then exited non-zero with

```
⨯ Timeout awaiting 'request' for 600000ms  failedTask=build
```

Nothing was missing from `release/`. The build was successful and was reported as
failed.

### Environment

- `electron-builder` 26.15.7 — `app-builder-lib` 26.15.7, `dmg-builder` 26.15.7,
  `builder-util` 26.15.3, `builder-util-runtime` 9.7.0 (as resolved by the lockfile)
- `got` 11.8.6 (transitively, via `@electron/get`)
- runner: GitHub-hosted `macos-15-intel`, image `20260727.x`
- command: `npx electron-builder --mac --publish never`, with
  `DEBUG=electron-builder,electron-osx-sign*`

### Observed sequence

Timestamps are from the runner log, in order, with nothing omitted between them:

```
01:40:46.375  • building        target=macOS zip …
01:40:46.376  • building        target=DMG …
01:40:46.392  • downloading     file=7zip-darwin-x86_64.tar.gz
01:40:46.444  • downloading     file=dmgbuild-bundle-x86_64-75c8a6c.tar.gz
01:40:46.793  • downloaded      file=7zip-darwin-x86_64.tar.gz progress=100%
01:50:46.8669 • async task error  error=Timeout awaiting 'request' for 600000ms
01:50:46.8671 • async task error  error=Timeout awaiting 'request' for 600000ms
01:50:49.012  • downloaded      file=dmgbuild-bundle-x86_64-75c8a6c.tar.gz progress=100%
01:50:54.397  • executing       file=…/dmgbuild …
01:52:39.139  • done            file=…/dmgbuild
01:53:10.610  • building block map
01:53:10.610  • artifactBuildCompleted  … .dmg
01:53:10.610  • artifactBuildCompleted  … .zip
01:53:10.800  ⨯ Timeout awaiting 'request' for 600000ms  failedTask=build
```

with the stack

```
RequestError: Timeout awaiting 'request' for 600000ms
    at ClientRequest.<anonymous> (…/got/dist/source/core/index.js:970:65)
    at Timeout.timeoutHandler [as _onTimeout] (…/got/dist/source/core/utils/timed-out.js:36:25)
```

Three things are worth reading off that sequence.

1. The request stalls for **exactly** its whole deadline: 01:40:46.4 → 01:50:46.87.
2. The rejection is recorded **twice**, 0.2 ms apart — one rejection object
   reaching two `AsyncTaskManager` instances, not two independent timeouts (the
   two downloads started 52 ms apart).
3. The very same artifact then reports `progress=100%` **2.1 s later** — one
   `builder-util-runtime` `retry({interval: 2000})` after the rejection — and
   `dmgbuild` runs to completion on it. The toolset was fine; only the record of
   the earlier attempt was not.

### Why the failure is terminal (`builder-util/out/asyncTaskManager.js`)

```js
addTask(promise) {
    …
    this.tasks.push(promise.catch(it => {
        log.debug({ error: it.message || it.toString() }, "async task error");
        this.errors.push(it);
        return Promise.resolve(null);
    }));
}

async awaitTasks() {
    …
    const checkErrors = () => {
        if (this.errors.length > 0) {
            this.cancelTasks();
            throwError(this.errors);
            return;
        }
    };
```

`this.errors` is append-only: nothing in the class removes from it, so there is
no path by which a task that later succeeds can retract a rejection that was
already recorded. `awaitTasks()` runs *after* the targets finish, which is why
the artifacts exist by the time `throwError` fires. And because the record is
written with `log.debug`, a build without `DEBUG=electron-builder` shows only the
final `⨯` line, with no hint that the operation it names had succeeded.

### Why the stall lasts the full ten minutes (`app-builder-lib/out/util/electronGet.js:290`)

```js
const downloadOptions = {
    timeout: { request: 10 * 60 * 1000 }, // prevent indefinite hang on stalled connections
    …
```

`got`'s `request` timeout is a **total** deadline. With no `lookup`, `connect`,
`socket` or `response` sub-timeout, a connection that produces no bytes at all
still consumes the entire 600 s before `shouldRetry` is ever consulted. On a CI
job with a wall-clock cap, a single silent socket therefore costs ten minutes
even though the failure mode is detectable in seconds.

### The part I cannot settle from the log

`downloadArtifactToFile` wraps the request in

```js
retry(() => get.downloadArtifact(configWithProgress), {
    retries: 3, interval: 2000, backoff: 2000,
    shouldRetry: e => … ["ENOTFOUND", "ETIMEDOUT", "ECONNRESET", "EPIPE", "ENOENT"].includes(e.code),
})
```

and `got`'s `TimeoutError` does carry `code = 'ETIMEDOUT'`
(`got/dist/source/core/utils/timed-out.js:8–15`, preserved by `RequestError`'s
constructor at `core/index.js:128`), so the retry *should* have swallowed this
one — and something clearly did retry, 2.1 s later. What I cannot tell from the
outside is which promise rejected into the two task managers despite that. Two
candidates, either of which would explain it:

- a caller that awaits the toolset promise outside the `retry` wrapper, so the
  first attempt's rejection reaches it even though the retry succeeds; or
- two concurrent `downloadBuilderToolset` calls for the same file (`dmg-builder`'s
  `getDmgVendorPath()` calls it directly and does not go through the
  `versionToPromise` memo in `binDownload.js::getBin`), where one loses.

I'd rather ask than guess: **which path can push a timeout into
`AsyncTaskManager.errors` while the operation it belongs to goes on to succeed?**

### Reproduction

The stall itself is a network condition, so the reproducible half is the
consequence, which needs no network at all — inject a rejection into a task
manager whose work then succeeds:

```js
// npm i builder-util@26.15.3 builder-util-runtime@9.7.0
const { AsyncTaskManager } = require("builder-util")
const { CancellationToken } = require("builder-util-runtime")

const manager = new AsyncTaskManager(new CancellationToken())
manager.addTask(Promise.reject(new Error("Timeout awaiting 'request' for 600000ms")))
manager.addTask(Promise.resolve("artifact written"))

manager.awaitTasks().then(
  r => console.log("resolved", r),
  e => console.log("rejected:", e.message),   // ← always this branch
)
```

```
rejected: Timeout awaiting 'request' for 600000ms
```

The second task resolved. There is no output in which that matters.

The ten-minute half reproduces the same way, against a listener that accepts the
connection and then writes nothing — the stalled socket of the CI run:

```js
// npm i got@11.8.6
const net = require("net"), got = require("got")
const server = net.createServer(() => {})            // accepts, never responds
server.listen(0, "127.0.0.1", async () => {
  const url = `http://127.0.0.1:${server.address().port}/dmgbuild-bundle.tar.gz`
  for (const timeout of [{ request: 3000 }, { request: 3000, socket: 1000 }]) {
    const t = Date.now()
    await got(url, { timeout, retry: 0 }).catch(e =>
      console.log(`${Date.now() - t} ms  ${e.message}`))
  }
  server.close()
})
```

```
3024 ms  Timeout awaiting 'request' for 3000ms
1009 ms  Timeout awaiting 'socket' for 1000ms
```

Same dead socket, same `got`; only the second configuration notices in time to
do anything about it. Both halves are packaged as one runnable script (which
also asserts the behaviour, so it fails if a future release fixes either) as
[`experiments/issue-1017-electron-builder-stale-error/run.mjs`](https://github.com/link-assistant/formal-ai/blob/issue-1017-fcfc7331ff54/experiments/issue-1017-electron-builder-stale-error/run.mjs);
`node run.mjs` installs the three packages into a scratch directory and needs no
further setup.

### Workaround in use

`downloadAndExtract` consults a predictable, checksum-validated archive cache
before it touches `@electron/get`:

```js
const archiveCachePath = path.join(getCacheDirectory({ allowEnvVarOverride: true }), releaseName, filenameWithExt)
```

so seeding `<cacheDir>/<releaseName>/<filenameWithExt>` before invoking
electron-builder removes the request entirely. We do this in CI with a small
prefetch script that fetches each required toolset with a 30-second stall
deadline and four attempts, verifies the SHA-256 electron-builder itself
verifies, and downgrades **every** failure to a warning so the prefetch can never
become a new reason for a build to fail. It is a workaround, not a fix: it only
helps for toolsets a caller knows about in advance.

### Suggested fixes

1. **Give the download a stall deadline, not just a total one.** Adding
   `lookup`/`connect`/`response` (or `socket`) sub-timeouts to `downloadOptions`
   lets a silent connection fail in seconds, where the existing
   `retry`/`shouldRetry` machinery can actually do its job:

   ```js
   timeout: { request: 10 * 60 * 1000, lookup: 30_000, connect: 30_000, socket: 60_000 },
   ```

   The total deadline stays as the backstop; the sub-timeouts turn a ten-minute
   dead wait into a two-second recovery. Measured above: 3024 ms → 1009 ms on the
   same dead socket, i.e. the detection time becomes the sub-timeout you choose
   rather than the whole budget.
2. **Let a success retract its own recorded rejection**, or scope the recorded
   error to the task rather than the manager — e.g. keep `errors` keyed by task
   and drop an entry when a subsequent attempt for the same key resolves. As it
   stands, `retry` cannot protect anything that has already been recorded.
3. **Raise the record above `debug`.** If `async task error` is going to decide
   the exit status, it should be visible in a default build. A `warn` here would
   have made this a five-minute diagnosis instead of a log archaeology exercise.

Happy to open a PR for (1) and (3) if that would help — (2) is a design call I'd
rather leave to the maintainers.
