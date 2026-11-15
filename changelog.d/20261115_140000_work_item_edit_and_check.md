---
bump: minor
---

### Fixed
- A work-item session no longer stops after reading evidence (#1483). With a Run tool, a boundary repair or regression test is checked after its edit by running samples derived from the requested bound against the original signature, never against the candidate. A pass verifies the requirement; a source that contradicts the bound fails the check (`IndependentCheckFailed`). Running the same task again is a verified no-op.
- A work item that names no repository file collects its leads: attachments become explicit unresolved evidence and are never described, linked documents are fetched as evidence only, and code identifiers in backticks are searched and the best match is read. A work item with no file and no lead is still left to the general planner.
- The localized report states which files changed, every check outcome and every unresolved lead. A verified record that leaves requirements unverified is reported as partly done, never as done.
