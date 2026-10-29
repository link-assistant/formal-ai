---
bump: patch
---

### Fixed
- Treat unknown Markdown runtime inputs as code after documented paths are
  excluded, and test native change admission before CI uses its tier outputs.
- Cancel superseded read-only documentation checks on PRs while protecting main.
- Drop checkout push credentials and preserve native unsupported-work-item
  no-effect assertions with the structured gap protocol.
- Bind evidence reports to the caller's work-item URL and keep native regression
  tests in the integration-test tree, outside production source.
