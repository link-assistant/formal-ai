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
permanent-status and HTTP recovery controls remain unchanged. No Rust build,
lychee binary, external site or unbounded reset server is run.

The complete maintained owner passes all 444 observations in 216.826 seconds;
all eleven affected JavaScript gates pass. The [receipt](../../../experiments/issue-1580/evidence/checks/link-transport-reset-20261123/manifest.json)
retains complete outputs, source identities and the frozen observation scope.
These tests qualify the current external recheck helper. Current lychee internal
I/O classification, actual malformed-lock rejection by Cargo audit and updater
engine compatibility remain unfinished. Historical npm-check-updates 23.1.1
metadata is not presented as a current publisher response.

All wider acceptance remains active in
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581), kept draft and
unmerged. Supplied test code receives zero independent authorship or reasoning
credit.
