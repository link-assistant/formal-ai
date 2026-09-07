# A connection reset during connect or the TLS handshake is never retried, at any `--max-retries`

**Filed:** [lycheeverse/lychee#2297](https://github.com/lycheeverse/lychee/issues/2297)
**Reproduced against:** lychee 0.24.2 (binary release) and `master` at `81cb43e1`
**Reproduction:** `experiments/issue-1081-lychee-connect-retry/` in this repository
**Verified patch:** `lychee-connect-phase-retry.patch` (built and measured, below)

## What happens

`--max-retries N` does not apply to a connection reset that arrives before the
request is written. The same reset on an already-established connection is
retried `N` times. The URL is healthy in both cases; only the moment the reset
lands differs.

This is not a tuning problem. No value of `--max-retries` or
`--retry-wait-time` changes the connect-phase result, because the retry
decision for that class is a constant.

## Reproduction

A server that answers every connection with `RST` instead of `FIN`
(`SO_LINGER` with `l_onoff = 1, l_linger = 0`), in two modes: reset on accept
(the client fails during connect / the TLS handshake) and reset after reading
the request line (the client fails on an established connection). It prints one
line per accepted connection, so the number of lines is the number of attempts
lychee actually made.

```python
# reset-server.py  (full copy in experiments/issue-1081-lychee-connect-retry/)
server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
server.bind(("127.0.0.1", port))
server.listen(16)
print(f"listening on 127.0.0.1:{port}", flush=True)

while True:
    connection, _ = server.accept()
    attempts += 1
    print(f"attempt {attempts} at +{time.monotonic() - started:.2f}s", flush=True)
    if mode == "after-request":
        connection.settimeout(5)
        try:
            connection.recv(4096)
        except OSError:
            pass
    # l_onoff = 1, l_linger = 0: close() sends RST rather than FIN.
    connection.setsockopt(socket.SOL_SOCKET, socket.SO_LINGER, struct.pack("ii", 1, 0))
    connection.close()
```

```bash
python3 reset-server.py 8443 immediate &      # reset during the TLS handshake
python3 reset-server.py 8080 after-request &  # the same reset, post-connect

printf '[a](https://127.0.0.1:8443/)\n' > input.md
lychee --no-progress --verbose --max-retries 5 --retry-wait-time 2 --timeout 10 input.md
```

Run it from a directory without a `.lycheeignore`: lychee reads one from the
working directory, and a rule excluding localhost makes the URL vanish from the
report rather than fail.

### Measured

```text
lychee 0.24.2, --max-retries 5 --retry-wait-time 2 (five retries cost at least 30s)

https://127.0.0.1:8443 (reset on accept)         -> 1 connection attempt  in 0s
    [ERROR] https://127.0.0.1:8443/ (at 1:1) | Network error: Connection reset by peer (os error 104)
http://127.0.0.1:8080  (reset after the request) -> 6 connection attempts in 62s
    [ERROR] http://127.0.0.1:8080/ (at 1:1) | Connection failed. Check network connectivity and firewall settings
```

One attempt against six, for the same reset from the same server with the same
flags. The 62 seconds in the second row is the retry budget being spent; the
0 seconds in the first is it being skipped.

## Root cause: two things, and the first one alone is not enough

### 1. `should_retry` decides by phase before it looks at the error

`lychee-lib/src/retry.rs`, `impl RetryExt for reqwest::Error`:

```rust
fn should_retry(&self) -> bool {
    if self.is_timeout() {
        true
    } else if self.is_connect() {
        false                      // <-- everything below is unreachable for connect errors
    } else if self.is_body() || self.is_decode() || self.is_builder() || self.is_redirect() {
        false
    } else if self.is_request() {
        // ... walks the source chain, downcasts to io::Error, calls should_retry_io
    }
    ...
}
```

`should_retry_io` already names `ConnectionReset` as retryable; the
`is_connect()` branch means a connect-phase reset never reaches it.

### 2. `should_retry_io` cannot see the kind it classifies on

Fixing only the branch above still does not retry. Measured, with an
`eprintln!` in that branch on `master`:

```text
DEBUG is_connect=true io_source=Some(Other) inner=Some("Connection reset by peer (os error 104)")
      inner_is_io=Some(true) inner_kind=Some(ConnectionReset)
      chain=["client error (Connect)", "Connection reset by peer (os error 104)"]
```

The io error reachable through the source chain has kind **`Other`**. The real
kind — `ConnectionReset` — is on an `io::Error` stored *inside* it, reachable
through `io::Error::get_ref` and not through `source()`: `io::Error`'s
`Error::source` implementation forwards to the inner error's own source and
skips the inner error itself, so `get_source_error_type` walks straight past
it.

This is the same wrapper lychee's *message* path already deals with.
`utils/reqwest.rs::analyze_io_error` routes `ErrorKind::Other` to
`analyze_io_other_error`, which calls `io_error.get_ref()` and pattern-matches
on the inner message. So one half of the crate unwraps this wrapper by string
matching, and the other half returns `false` without unwrapping it at all.

## Suggested fix

Both halves; either alone leaves the behaviour unchanged.

```diff
         } else if self.is_connect() {
-            false
+            // A reset or an abort during connect or the TLS handshake is as
+            // transient as the same error one byte later; classify it by its
+            // io kind rather than by the phase it happened in.
+            get_source_error_type::<io::Error>(self).is_some_and(should_retry_io)
         } else if self.is_body() || self.is_decode() || self.is_builder() || self.is_redirect() {

 /// Classifies an `io::Error` into retryable or not.
 fn should_retry_io(error: &io::Error) -> bool {
-    matches!(
+    if matches!(
         error.kind(),
         io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted | io::ErrorKind::TimedOut
-    )
+    ) {
+        return true;
+    }
+
+    // The transport layer hands us the OS error wrapped in another `io::Error`,
+    // whose kind is `Other`; the kind that decides this is on the error inside.
+    // `io::Error::source` skips that inner error, so the chain walk above
+    // cannot see it -- unwrap it here.
+    if let Some(inner) = error
+        .get_ref()
+        .and_then(|inner| inner.downcast_ref::<io::Error>())
+    {
+        return should_retry_io(inner);
+    }
+
+    false
 }
```

`get_source_error_type` and `should_retry_io` are both already in the file and
both already used by the `is_request()` branch, so nothing new is introduced.
A connect error with no io source, and one whose kind is genuinely not
retryable (DNS failures, `ConnectionRefused`, certificate rejections), still
answers `false` — the change is limited to the three kinds `should_retry_io`
was written to name. The `get_ref` unwrap also benefits the `is_request()`
branch, which has the same blind spot for the same reason.

### Verified

Applied to `master` at `81cb43e1`, built with `cargo build --release`, and the
reproduction re-run against the resulting binary:

```text
before:  https://127.0.0.1:8443 (reset on accept) -> 1 connection attempt  in 0s
after:   https://127.0.0.1:8443 (reset on accept) -> 6 connection attempts in 62s
         http://127.0.0.1:8080  (control)         -> 6 connection attempts in 63s  (unchanged)
```

The connect-phase case now spends the same retry budget as the post-connect
case, and the control is unaffected. On the patched tree
`cargo test -p lychee-lib --lib` reports 497 passed, 0 failed, 1 ignored, and
`cargo clippy -p lychee-lib --lib --all-features` is clean. The patch is attached as
`lychee-connect-phase-retry.patch`.

A regression test in the existing `mod tests` needs a real `reqwest::Error`,
which needs a reset server; the reproduction above is the smallest one we
found.

## Why it matters

This is a false positive in CI, and an expensive one because the report cannot
be told apart from a real broken link. In `link-assistant/formal-ai`, run
`https://github.com/link-assistant/formal-ai/actions/runs/34134986294` failed
on a healthy URL:

```text
[ERROR] https://allenai.org/data/arc (at 226:27) | Network error: Connection reset by peer (os error 104)
```

1.5 seconds into the step, with `--max-retries 6 --retry-wait-time 2`
configured — six retries with a growing wait cannot fit in 1.5 s, which is how
we noticed that no retry was running. The same host answers 200 from a
workstation and answered 200 on the next run. An earlier round of the same
failure had been answered by raising `--max-retries` from the default to 6; the
setting has no effect on this class, so the failure returned.

Because a reset carries no status code, none of the existing escape hatches can
name it: `--accept` takes status codes, `--cache-exclude-status` takes status
codes, and `--exclude` would suppress the link permanently rather than retry
it.

If #1797 (migrate to `reqwest`'s own retries) lands first, the same two
questions apply to whichever classifier replaces this one: is a connect-phase
error classified by its io kind, and is the `ErrorKind::Other` wrapper unwrapped
before that kind is read.

## Workaround

Retry outside lychee, and only for failures where no host answered. In
`link-assistant/formal-ai` this is `scripts/recheck-broken-links.mjs`: it parses
the lychee markdown report, separates failures that carry a status code (a host
answered — `404` stays `404`, and is never re-checked) from failures that do
not (`[ERROR]`, `[TIMEOUT]`, `[UNKNOWN]`), and re-requests only the latter with
the same accept list and user agent lychee was given. The workflow fails only
if a link is still unanswered afterwards.

Raising `--max-retries` is not a workaround for this class — that is the
finding.
