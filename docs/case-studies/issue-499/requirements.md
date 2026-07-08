# Issue 499 Requirements

| ID | Requirement | Acceptance |
| --- | --- | --- |
| R499-1 | Preserve the issue, PR, comments, related work, Google Trends source data, and online research under `docs/case-studies/issue-499`. | `tests/unit/docs_requirements_issue_499.rs` asserts the case-study files and raw data exist. |
| R499-2 | Provide an automated converter from a Google Trends RSS snapshot into structured Formal AI test-case data. | `formal-ai google-trends` reads a saved RSS file and writes `data/benchmarks/google-trends-top10-suite.lino`. |
| R499-3 | Convert the captured Google Trends top ten into Formal AI requests. | `data/benchmarks/google-trends-top10-suite.lino` records 10 trend topics and 40 prompt cases. |
| R499-4 | Generate supported languages for every trend topic. | English, Russian, Hindi, and Chinese prompts are generated for each of the top ten. |
| R499-5 | Keep the test path deterministic and offline. | CI tests use the committed RSS snapshot, not a live Google Trends request. |
| R499-6 | Make the conversion reachable through the in-repo Agent CLI recipe. | `TREND_PROMPT_CATALOG_TASK` writes and verifies the catalog through `write_file -> run_command -> final`. |
| R499-7 | Document prior art, existing libraries, and local components that informed the design. | `raw-data/online-research.md`, `solution-plan.md`, and this README summarize Google Trends, API alpha, pytrends, and prior benchmark/catalog PRs. |
| R499-8 | Add regression coverage so future changes cannot silently drop topics, languages, traceability, or generated-artifact reproducibility. | `tests/unit/issue_499_google_trends.rs` pins parsing, generation, prompt usability, and the Agent CLI session. |
