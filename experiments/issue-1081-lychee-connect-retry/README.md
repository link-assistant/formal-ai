# `--max-retries` does not apply to a reset during connect

Issue #1081. Run `34134986294` failed `Broken Link Checker` on a single link:

```text
[ERROR] https://allenai.org/data/arc (at 226:27)
        | Network error: Connection reset by peer (os error 104)
```

The page answers `200` on every attempt from anywhere else, and the same branch
had passed six minutes earlier. The workflow set `--max-retries 6
--retry-wait-time 2`, added by issue #1045 for exactly this symptom — and the
error was printed **1.5 seconds** into the step, which six retries and a
doubling wait cannot fit inside.

## What this measures

`reset-server.py` accepts a connection and closes it with `SO_LINGER` set to
zero, so the kernel sends `RST` instead of `FIN`. Two modes place that reset on
either side of the connect boundary:

* `immediate` — the reset arrives during the TLS handshake, so `reqwest`
  classifies it with `is_connect()`.
* `after-request` — the server reads the request line first, so the identical
  reset arrives on an established connection.

Each run counts the connections the server accepted. That count *is* the number
of attempts lychee made.

## Result (lychee 0.24.2, `--max-retries 5 --retry-wait-time 2`)

```text
https://127.0.0.1:8443 (immediate)     -> 1 connection attempt in 0s
http://127.0.0.1:8080  (after-request) -> 6 connection attempts in 62s
```

Same reset, same setting, same process: retried five times when it lands after
connect, never retried when it lands during it. The error text for the
`immediate` case alternates run to run between

```text
Network error: Connection reset by peer (os error 104)
I/O error (ConnectionReset). Check network connectivity and server status
```

— the same failure rendered through two branches of `analyze_error_chain`. The
attempt count does not vary.

## Why

`lychee-lib/src/retry.rs`, unchanged on `master` at the time of writing:

```rust
impl RetryExt for reqwest::Error {
    fn should_retry(&self) -> bool {
        if self.is_timeout() {
            true
        } else if self.is_connect() {
            false
        ...

fn should_retry_io(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted | io::ErrorKind::TimedOut
    )
}
```

`is_connect()` is answered before the error kind is ever examined, so a
`ConnectionReset` raised during connect or the TLS handshake takes the `false`
branch and never reaches `should_retry_io` — where that exact kind is listed as
retryable. `--max-retries` is inert for this class of failure at any value.

That branch is only half of it. Letting the connect case fall through to
`should_retry_io` changes nothing, because the io error the source chain
exposes has kind `Other`:

```text
DEBUG is_connect=true io_source=Some(Other) inner=Some("Connection reset by peer (os error 104)")
      inner_is_io=Some(true) inner_kind=Some(ConnectionReset)
      chain=["client error (Connect)", "Connection reset by peer (os error 104)"]
```

The kind that decides the question is on an `io::Error` stored *inside* that
one, reachable through `io::Error::get_ref` — and not through `source()`, whose
`io::Error` implementation forwards to the inner error's own source and skips
the inner error itself. lychee's message path already unwraps this exact
wrapper (`utils/reqwest.rs::analyze_io_other_error` calls `get_ref` and
pattern-matches the inner message); the retry path does not.

## The fix, verified

`upstream-reports/lychee-connect-phase-retry.patch` in this repository's dev log
changes both halves: the `is_connect()` branch consults `should_retry_io`, and
`should_retry_io` unwraps an inner `io::Error` before giving up. Applied to
`master` at `81cb43e1`, built, and measured with the same script:

```text
before:  https://127.0.0.1:8443 (immediate)     -> 1 connection attempt  in 0s
after:   https://127.0.0.1:8443 (immediate)     -> 6 connection attempts in 62s
         http://127.0.0.1:8080  (after-request) -> 6 connection attempts in 63s  (unchanged)
```

## Running it

```sh
curl -sSL -o lychee.tar.gz \
  https://github.com/lycheeverse/lychee/releases/download/lychee-v0.24.2/lychee-x86_64-unknown-linux-gnu.tar.gz
tar xzf lychee.tar.gz
./run.sh ./lychee-x86_64-unknown-linux-gnu/lychee
```

`run.sh` runs from a scratch directory on purpose: lychee reads `.lycheeignore`
from the working directory, and this repository's excludes `127.0.0.1`.

## What was done about it

Reported upstream, and answered here by moving the retry outside lychee:
`scripts/recheck-broken-links.mjs` asks every link that failed *without a status
code* again, directly, spread over time, and the build is failed only by the
ones that still do not answer. A link the host pronounced on — `404`, `403` — is
passed straight through and never re-checked, because that is an answer.
