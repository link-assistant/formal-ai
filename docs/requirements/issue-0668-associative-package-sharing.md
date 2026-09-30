## Issue #668 Shareable associative packages

| Requirement | Status | Evidence / remaining work |
|---|---|---|
| Scoped Links Notation manifest with identity, permissions, handlers, links and provenance | Implemented library | SharedPackage wraps the established AssociativePackage codec and preserves contained_links and provenance. |
| Review declared permissions; explicitly confirm agent tools | Implemented library | PermissionReview binds confirmation to the exact artifact; local catalog determines agent tags; unknown capabilities fail. |
| Reject unavailable handler references | Implemented library | All declared handlers and trigger references validated before installation. |
| Append package_imported event | Implemented library | Event contains the artifact and its provenance after successful installation. |
| Fresh-store round-trip and permission gates | Tests drafted, not run | issue_668_associative_package_sharing.rs covers transfer, missing handlers, agent consent and stale approval. |
| CLI export/import commands | Pending integration | Register module and connect the library to the CLI package command. |
| Web file picker and CLI/web e2e round-trip | Pending integration | Reuse identical artifact format; browser port and e2e fixture remain. |
| Document real example in main README | Pending integration | examples/packages/greeting.lino supplied; dedicated case study documents use. |

No execution or build verification was performed under the bulk-work directive.
