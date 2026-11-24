# Real connection resets at both request phases

Rust template practice 50 preserves a historical lychee report distinguishing
a reset before an HTTPS request from a reset after an HTTP request. Historical
metadata and a successful simulated fetch do not establish current transport
behavior.

The new maintained controls call the existing recheckAll helper with real Node
fetch against finite loopback TCP servers. An immediate reset on the HTTPS
listener precedes the HTTP request; the HTTP listener waits for a request line
before resetting. Both exhausted cases report unreachable after exactly three
probes and three physical connections. A third case resets the first HTTP
request and answers the second with HTTP 200: only that later success recovers
the URL.

All three focused controls pass locally. Each test has a five-second ceiling,
three probes, a 500 ms request deadline, a three-second total helper budget,
an 8 KiB request-header limit and explicit socket cleanup. The existing
permanent-status and HTTP recovery controls remain unchanged. This Node
recheck test runs entirely on loopback.

The complete maintained owner passes all 444 observations in 216.826 seconds;
all eleven affected JavaScript gates pass. The [receipt](../../../experiments/issue-1580/evidence/checks/link-transport-reset-20261123/manifest.json)
retains complete outputs, source identities and the frozen observation scope.
On 2026-11-24 the workflow's `lychee-action@v2` default was v0.24.2.
Run `python3 experiments/issue-1580/probe-lychee-reset.py /path/to/lychee`
with the official Linux x86-64 archive. The replay checks its published
SHA-256 `1f4e0ef7f6554a6ed33dd7ac144fb2e1bbed98598e7af973042fc5cd43951c9a`
before extraction. A single Markdown link drives each phase with two retries
configured. HTTPS connects once and reports a reset, with the exact diagnostic
varying with handshake timing. HTTP sends three complete request lines and
reports `Connection failed`.
Both scans report one error and exit two. The initial direct-URL probe
exited one for its HTTPS failure. The Markdown input matches the workflow's
scan and produces its per-link report. No Rust compilation is needed.

Actual malformed-lock rejection by Cargo audit and updater engine
compatibility remain unfinished.

All wider acceptance remains active in
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581), kept draft and
unmerged. Supplied test code receives zero independent authorship or reasoning
credit.
