# `should_retry` answers `is_connect()` with a flat `false`, so a connection reset during connect or the TLS handshake is never retried at any `--max-retries`

**Filed:** `lycheeverse/lychee` (see `README.md` for the issue link)
**Reproduced against:** lychee 0.24.2 (binary release) and `master` at the time of filing
**Reproduction:** `experiments/issue-1081-lychee-connect-retry/` in this repository

## What happens

`--max-retries N` does not apply to a connection reset that arrives before the
request is written. The same reset on an already-established connection is
retried `N` times. The URL is healthy in both cases; only the moment the reset
lands differs.

This is not a tuning problem. No value of `--max-retries` or
`--retry-wait-time` changes the connect-phase result, because the retry
decision for that class is a constant.

## Where

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

fn should_retry_io(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted | io::ErrorKind::TimedOut
    )
}
```

`should_retry_io` already names `ConnectionReset` as retryable. The
`is_connect()` branch means a connect-phase reset never reaches it.

The strongest evidence that the information is available at that point is
inside lychee itself. `lychee-lib/src/utils/reqwest.rs` builds the error
message by walking the very same source chain:

```rust
fn analyze_error_source_chain(error: &reqwest::Error) -> Option<String> {
    let mut source = error.source();
    while let Some(err) = source {
        if let Some(io_error) = err.downcast_ref::<std::io::Error>() {
            return Some(analyze_io_error(io_error));
        }
        ...
```

and `analyze_io_error` reaches its `format!("I/O error ({kind_name}) ...")`
fallback with `kind_name == "ConnectionReset"` — which is exactly the message
the reproduction below prints. So for the same error object, the *message* path
successfully determines that the io kind is `ConnectionReset`, while the
*retry* path returns `false` without ever asking.

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

Point lychee at it with retries that would be impossible to miss:

```bash
python3 reset-server.py 8443 immediate &      # reset during the TLS handshake
python3 reset-server.py 8080 after-request &  # the same reset, post-connect

printf '[a](https://127.0.0.1:8443/)\n' > input.md
lychee --no-progress --verbose --max-retries 5 --retry-wait-time 2 --timeout 10 input.md
```

Run it from a directory without a `.lycheeignore`: lychee reads one from the
working directory, and a rule excluding localhost makes the URL vanish from the
report rather than fail.

### Measured, lychee 0.24.2

```text
--max-retries 5 --retry-wait-time 2 (so five retries cost at least 30s)

https://127.0.0.1:8443 (immediate)     -> 1 connection attempt(s) in 0s
    [ERROR] https://127.0.0.1:8443/ (at 1:1) | I/O error (ConnectionReset). Check network connectivity and server status
http://127.0.0.1:8080  (after-request) -> 6 connection attempt(s) in 62s
    [ERROR] http://127.0.0.1:8080/ (at 1:1) | Connection failed. Check network connectivity and firewall settings
```

One attempt versus six, for the same reset from the same server, with the same
flags. The 62 seconds in the second row is the retry budget being spent; the
0 seconds in the first is it being skipped.

## Why it matters

This is a false positive in CI, and an expensive one because it is
indistinguishable from a real broken link in the report. In
`link-assistant/formal-ai`, run
`https://github.com/link-assistant/formal-ai/actions/runs/34134986294` failed
on a healthy URL:

```text
[ERROR] https://allenai.org/data/arc (at 226:27) | Network error: Connection reset by peer (os error 104)
```

1.5 seconds into the step, with `--max-retries 6 --retry-wait-time 2`
configured — six retries with a growing wait cannot fit in 1.5 s, which is how
we noticed that the retries were not running at all. The same host answers 200
from a workstation and answered 200 on the next run. An earlier round of the
same failure was "fixed" by raising `--max-retries` from the default to 6; the
setting had no effect on this class, so the failure returned.

Because a reset carries no status code, none of the existing escape hatches
can name it: `--accept` takes status codes, `--cache-exclude-status` takes
status codes, and `--exclude` would suppress the link permanently rather than
retry it.

## Suggested fix

Let a connect error whose source chain contains an `io::Error` reach the
classifier that already exists for it, instead of short-circuiting:

```rust
 fn should_retry(&self) -> bool {
     if self.is_timeout() {
         true
     } else if self.is_connect() {
-        false
+        // A reset or an abort during connect or the TLS handshake is as
+        // transient as the same error one byte later; classify it by its io
+        // kind rather than by the phase it happened in.
+        get_source_error_type::<io::Error>(self).is_some_and(should_retry_io)
     } else if self.is_body() || self.is_decode() || self.is_builder() || self.is_redirect() {
```

`get_source_error_type` and `should_retry_io` are both already in the file, and
both already used by the `is_request()` branch below. A connect error with no
io source (DNS failures, `ConnectionRefused`, certificate rejections) keeps
answering `false`, so the change is limited to `ConnectionReset`,
`ConnectionAborted` and `TimedOut` — the three kinds `should_retry_io` was
written to name.

A test in the existing `mod tests` would need a reset server to construct a
real `reqwest::Error`; the reproduction above is the smallest one we found.

If #1797 (migrate to `reqwest`'s own retries) lands first, the same question
applies to whichever classifier replaces this one: connect-phase resets should
be retried on their io kind, not excluded by their phase.

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
