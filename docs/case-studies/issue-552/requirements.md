# Issue #552 Requirements

| ID | Requirement | Status |
| --- | --- | --- |
| R1 | Preserve issue, PR, comments, related work, captured pages, and search data under `docs/case-studies/issue-552`. | Done. Raw data and analysis live in this directory. |
| R2 | Convert the provided ChatGPT shared dialog to a demo/replay format. | Done. `shared-dialog convert` emits `demo_memory`; generated output is saved in `raw-data/chatgpt-share-6a3825b9.demo_memory.lino`. |
| R3 | Add automated tests for the converter. | Done. Unit tests cover ChatGPT HTML and Markdown transcript conversion; integration tests cover CLI output. |
| R4 | Preserve multi-line dialog content through memory export/import. | Done. Newline, carriage-return, and tab escaping now round-trip in quoted memory values. |
| R5 | Make "answer with single line" choose the shortest readable command, not a whitespace-compromised command. | Done for the captured shell-command dialog. Tests assert single-line output and readable spacing. |
| R6 | Solve the captured dialog by reasoning/generalizing rather than memorizing the transcript. | Done for the implemented slice. The handler extracts shell commands, wraps arbitrary command text, and uses conversation history for follow-up context. |
| R7 | Search for similar shared links and keep a shortest-link sample. | Done. The first 100 public GitHub code-search results for each family were fetched; unique links and shortest 10 are saved. |
| R8 | Convert the Google AI Mode shared dialog. | Blocked by dependency. Static capture returns an interstitial/challenge page, not transcript data. |
| R9 | Handle Markdown and other formats produced by `web-capture`. | Partially done. Markdown transcript conversion is implemented; provider-normalized web-capture output needs upstream schema work. |
| R10 | Report dependency issues to `web-capture` and `meta-language`. | Done. Reports filed as web-capture #141 and meta-language #168. |
| R11 | Keep the first stage focused on deep analysis, dependency reports, and a PR pause once reported. | Done for analysis; implementation also includes the first replayable ChatGPT slice because it was locally reproducible. |

## Non-Goals For This Slice

- Do not scrape or bypass provider access controls for Google AI Mode. The
  current converter reports unsupported captures when a transcript is absent.
- Do not train on or hard-code the provided transcript. Tests use exact prompts,
  but the solver code extracts command structure from the prompt/history.
- Do not define the full cross-repository shared-dialog schema inside
  formal-ai. That belongs in `link-foundation/meta-language` and should be
  consumed by `link-assistant/web-capture`.
