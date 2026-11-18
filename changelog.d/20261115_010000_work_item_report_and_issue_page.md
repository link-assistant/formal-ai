---
bump: patch
---

### Fixed
- A work-item session no longer answers with a raw JSON record (issue #1560). It answers with a localized report. The report says what stopped the session, which files it read, and which requirement a refusal concerns. The record stays in the trace.
- The read policy is evaluated over the user's own turn, not over fetched issue text. A Hive Mind session on issues 1486, 1489 and 1501 now reads repository files instead of refusing every read.
- A fetched GitHub issue page keeps only its title and body. A page without an issue body, such as the sign-in page, is rejected and the `gh issue view`, REST and `gh api` reads run instead.
- A Markdown link label that names a file is read at the path of the work item's own blob link, not at the repository root.
