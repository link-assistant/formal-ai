---
bump: patch
---

Require successful registry evidence before release detection or version selection. Retry transient registry failures within bounded deadlines, reject malformed success responses, and replay both actual release CLIs against loopback registry failures in hosted native CI.
