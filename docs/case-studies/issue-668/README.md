# Shareable package artifacts (#668)

Existing package codecs could serialize installed packages but offered no transfer-time handler availability or consent check. `associative_packages` adds a scoped artifact and one import boundary shared by future CLI and web adapters.

The artifact retains the package's handlers, triggers, dependencies and permission gates. `contained_links` retains additional knowledge as opaque Links Notation data in the imported memory event; it is not executed or injected into the active graph. Provenance travels with the artifact. A local handler catalog identifies available implementations and agent capabilities. A review confirms the exact artifact and explicitly approves agent capability grants. Rejections leave both stores unchanged.

`examples/packages/greeting.lino` is a concrete greeting pack. Register its `greeting-response` handler locally with kind `response`, capability `greeting`, and agent tag false. Parse with `SharedPackage::parse`, display `declared_permissions`, bind `PermissionReview.artifact` to the exact file, then call `import_package`. `SharedPackage::export` produces the transferable file. A fresh package store obtains the same package links and the greeting permission, without acquiring a shell permission.

Integration requires `pub mod associative_packages;` in `rust/src/lib.rs` and `mod issue_668_associative_package_sharing;` in the unit registry. CLI package export/import, browser file-picker integration, graph adoption of contained knowledge and web/CLI e2e remain pending. No tests/builds were run.
