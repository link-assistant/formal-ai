# First current-head CI round
Head: 0fa50f1db1c76d92e983fe7c468cf0d9a1776980. Both runs started 2026-11-17T19:41:37Z after that commit was pushed.
- https://github.com/link-assistant/formal-ai/actions/runs/44161313616
- https://github.com/link-assistant/formal-ai/actions/runs/44161313044
The full downloaded workflow logs, run metadata, check runs and annotations are compressed alongside this report.
Pipeline log lines 2093–2098 show curl 28, early EOF and a promisor-remote fetch failure during the fast job checkout. Line 5882 reports the native checkout's four-minute timeout. Neither checkout reached a source check or compiler.
Layered log lines 4972–4974 and 7723–7725 require the governed Cargo source witness to be refreshed. A complete object comparison found exactly four fixture SHA changes for rust/Cargo.toml; all 1185 inventory entries, assertions, unsupported cases and authoritative native source hashes otherwise remain identical. The review is archived.
Layered log lines 7830–7833 identify the redundant Spanish marker as the sole stale WASM input. It has been removed from both canonical and embedded seeds. The first focused rerun showed that ñ inside a word does not supply the standalone marker signal. Both source twins now use the matching seeded duration request prefix to select the response language. An additional Spanish sample without any accent verifies that choice. The compiled worker and its recorded manifest remain consistent without a local build.
The three reproducing regression files and all 39 feature checks pass after correction. All 138 local gates were repeated; only the already investigated unpatched picomatch advisory remains failing. No native build was run locally.
