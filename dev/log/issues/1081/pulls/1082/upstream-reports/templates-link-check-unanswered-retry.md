# The link checker relies on `--max-retries`, which does not cover the one failure it was added for: a connection reset during connect

**Filed against:** all five `link-foundation/*-ai-driven-development-pipeline-template` repositories
**Reproduced at:** the snapshotted commits in `../references/templates/*-template.HEAD`
**Upstream cause:** `lycheeverse/lychee` — see `lychee-connect-phase-reset-not-retried.md`

## What happens

`.github/workflows/links.yml` runs lychee with `--max-retries 3` and fails the
workflow on any remaining failure. A healthy external URL that answers a
`RST` during connect or the TLS handshake — a normal event for a rate-limiting
or load-shedding host seen from a CI runner's address range — is reported as a
broken link **without being retried once**, and reddens the run.

The retry setting is not a partial mitigation here. It has no effect on this
class at any value, so a maintainer who sees this failure and raises
`--max-retries` gets the same failure back.

## Affected files

Identical shape in all five templates:

| template | `links.yml` | `--max-retries` | archive fallback | re-check for unanswered links |
|---|---|---|---|---|
| rust | `.github/workflows/links.yml` | line 71 | `scripts/check-web-archive.mjs` | none |
| js | `.github/workflows/links.yml` | line 62 | `scripts/check-web-archive.mjs` | none |
| python | `.github/workflows/links.yml` | line 51 | `scripts/check_web_archive.py` | none |
| php | `.github/workflows/links.yml` | line 48 | `scripts/check-web-archive.php` | none |
| csharp | `.github/workflows/links.yml` | line 60 | `scripts/check-web-archive.mjs` | none |

The Web Archive step is not a substitute: it looks for a *snapshot* of a link
it treats as broken, and suggests replacing the URL. A link that is fine and
was merely reset does not want to be replaced with an archive.org copy — it
wants to be asked again.

## Why `--max-retries` cannot cover it

`lychee-lib/src/retry.rs`:

```rust
fn should_retry(&self) -> bool {
    if self.is_timeout() {
        true
    } else if self.is_connect() {
        false                 // <-- a connect-phase error is never retried
    } else if ...
    // ... the branch below downcasts the source chain to io::Error and calls
    //     should_retry_io, which *does* list ConnectionReset as retryable
}
```

The reset is classified by the phase it happened in rather than by its io kind,
so it never reaches the classifier that would retry it.

## Reproduction

A server that resets every connection, in two modes: reset on accept (the
client fails during connect) and reset after the request line (the client fails
post-connect). The full scripts are in `experiments/issue-1081-lychee-connect-retry/`
of `link-assistant/formal-ai`; the essential part is the socket option:

```python
# l_onoff = 1, l_linger = 0: close() sends RST rather than FIN.
connection.setsockopt(socket.SOL_SOCKET, socket.SO_LINGER, struct.pack("ii", 1, 0))
connection.close()
```

```bash
printf '[a](https://127.0.0.1:8443/)\n' > input.md
lychee --no-progress --verbose --max-retries 5 --retry-wait-time 2 --timeout 10 input.md
```

Measured on lychee 0.24.2, counting the server's accepted connections:

```text
https://127.0.0.1:8443 (reset on accept)        -> 1 connection attempt  in 0s
http://127.0.0.1:8080  (reset after the request) -> 6 connection attempts in 62s
```

Run it from a directory with no `.lycheeignore`; lychee reads one from the
working directory and a localhost exclusion makes the URL disappear from the
report instead of failing.

## What it looks like in a real run

`link-assistant/formal-ai` run 34134986294:

```text
[ERROR] https://allenai.org/data/arc (at 226:27) | Network error: Connection reset by peer (os error 104)
```

1.5 seconds into a step configured with `--max-retries 6 --retry-wait-time 2`.
Six retries with a growing wait cannot fit inside 1.5 seconds, which is how the
missing retry became visible. The URL answered 200 from a workstation at the
same moment and 200 on the next run.

## Suggested fix

Re-check, outside lychee, only the failures where **no host answered**, and let
the workflow fail only on what is still unanswered afterwards. The rule that
keeps this from hiding real breakage: a failure carrying a status code means a
host answered, and that answer is final — a `404` is never re-checked.

`links.yml`:

```yaml
      - name: Re-check links that never got an answer
        if: steps.lychee.outputs.exit_code != 0
        id: recheck
        run: node scripts/recheck-broken-links.mjs   # or the template's language
        env:
          LYCHEE_OUTPUT: lychee/out.md
          RECOVERED_OUTPUT: lychee/recovered.txt

      - name: Check broken links against the Web Archive
        if: >-
          steps.lychee.outputs.exit_code != 0
          && steps.recheck.outputs.all_recovered != 'true'
        ...
        env:
          LYCHEE_OUTPUT: lychee/out.md
          RECOVERED_URLS: lychee/recovered.txt

      - name: Fail if broken links were found
        if: >-
          ${{ !cancelled()
          && steps.lychee.outputs.exit_code != 0
          && steps.recheck.outputs.all_recovered != 'true' }}
```

Note the `!= 'true'` rather than `== 'false'`: a step that was skipped or that
crashed leaves the output empty, and only the `!=` form fails safe.

A reference implementation is `scripts/recheck-broken-links.mjs` in
`link-assistant/formal-ai` (with `scripts/recheck-broken-links.test.mjs`, 14
cases). It parses the lychee markdown report, splits failures into *answered*
(numeric marker, or `Rejected status code` in the detail) and *unanswered*
(`[ERROR]`, `[TIMEOUT]`, `[UNKNOWN]`), and re-requests only the unanswered set,
round-robin across URLs with a doubling wait and a total budget that expires
before the job cap. It re-uses the same `--accept` list and `--user-agent`
lychee was given, so a link the checker would have accepted is accepted here
too — a unit test there reads both the workflow and the script, so the two
cannot drift apart. The script exits 0 in every case: it downgrades failures,
it never raises them, so a bug in the re-check cannot turn a green run red.

The php and python templates carry their archive fallback in php and python
rather than node; the same 60-line parser translates directly, and the split
between answered and unanswered is the whole of the logic.

## Workaround until then

None that works through configuration. `--accept` and `--cache-exclude-status`
both take status codes and a reset has none; `--exclude` suppresses the link
permanently rather than re-checking it; raising `--max-retries` is the setting
this report is about.
