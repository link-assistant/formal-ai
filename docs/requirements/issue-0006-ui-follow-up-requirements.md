## Issue #6 UI Follow-Up Requirements

Issue [#6](https://github.com/link-assistant/formal-ai/issues/6) adds these UI requirements to the demo surface:

| ID | Requirement | Status |
| --- | --- | --- |
| R26 | Start the browser demo in interactive demo mode by default. | Implemented by initializing demo mode to `true`. |
| R27 | Update the next-dialog timer every second. | Implemented with `demoCountdown` state updated by a one-second interval. |
| R28 | Hide diagnostics unless diagnostics mode is enabled. | Implemented with a default-off diagnostics toggle gating trace, intent, evidence, worker status, and thinking steps. |
| R29 | Keep the default chat transcript focused on user-visible messages. | Implemented by hiding diagnostic chips and evidence in normal mode. |
| R30 | Preserve issue data and analysis under `docs/case-studies/issue-6`. | Implemented with raw GitHub data, the issue screenshot, online research, requirement extraction, and solution notes. |
