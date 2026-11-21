The Doxygen archive download in [`docs.yml` at 3769fa70](https://github.com/link-foundation/cpp-ai-driven-development-pipeline-template/blob/3769fa70a1bc30ce9516e01f78bcd7edea3b3f80/.github/workflows/docs.yml#L63) uses `curl --fail --location --retry 3`. A connection that closes after a successful HTTP status but before the declared response length is complete exits with curl error 18 without retrying. This leaves a transient transfer error fatal despite the apparent retry policy.

I found this while comparing the complete template source tree for https://github.com/link-assistant/formal-ai/issues/1580. This is a locally reproduced transport-handling defect; I have not observed a production Doxygen outage in this template.

Reproduction: serve a local HTTP endpoint that first returns status 200, `Content-Length: 100`, `Connection: close`, and a two-byte body, then serves a complete two-byte response. Run curl with `--fail --location --retry 3 --retry-delay 0 --max-time 2` against it. With curl 8.23.0, the command exits 18 after one request. Adding `--retry-all-errors` succeeds after two requests. The regression experiment uses Node's HTTP server, a 10-second process deadline, and a 15-second test deadline; it closes the server after testing both behaviors.

Suggested change: add `--retry-all-errors` to this existing retry policy and retain finite connection/transfer/job limits. Keep `--fail` so an unsuccessful download is never used as an archive. Add a small local HTTP transport regression check for the truncated-response case; no external network or native compilation is needed.

Related implementation and reproducible experiment: https://github.com/link-assistant/formal-ai/pull/1581 (`experiments/issue-1580/template-download-retry.test.mjs`, pending push).
