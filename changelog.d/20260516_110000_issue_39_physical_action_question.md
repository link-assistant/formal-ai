---
bump: patch
---

### Fixed
- Issue #39: Queries asking whether formal-ai performed a physical action (e.g. Russian «Сосал?») are now answered factually ("No. I have no physical body.") via a new `try_physical_action_question` handler instead of being refused as inappropriate content
- Removed «сосал»/«сосёшь»/«соси»/«сосать» from the vulgar-content word list since these words describe physical actions and deserve a factual response, not a policy refusal
