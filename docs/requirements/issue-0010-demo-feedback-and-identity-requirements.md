## Issue #10 Demo Feedback and Identity Requirements

Issue [#10](https://github.com/link-assistant/formal-ai/issues/10) adds feedback, issue reporting, and identity-answer requirements to the demo.

| ID | Requirement | Status |
| --- | --- | --- |
| R48 | Remove the unused Preview button near Send. | Implemented by removing preview mode controls and rendering from the demo. |
| R49 | Include a prefilled GitHub issue link in unknown-intent responses. | Implemented with browser-native issue URL generation. |
| R50 | Include dialog history and environment metadata in generated issue reports. | Implemented for message-level and header-level report links. |
| R51 | Allow issue reporting from any dialog, not only unknown prompts. | Implemented by rendering report actions for assistant messages and the current transcript. |
| R52 | Answer "Who are you?" and close identity-question variations. | Implemented through the Rust engine, WebAssembly worker, JavaScript fallback, and tests. |
| R53 | Keep identity knowledge reviewable as Links Notation. | Implemented in `data/seed/identity.lino`. |
| R54 | Preserve issue #10 raw evidence and analysis under `docs/case-studies/issue-10`. | Implemented with issue, PR, screenshot, reference, and case-study data. |
