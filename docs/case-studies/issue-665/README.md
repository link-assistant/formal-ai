# Offline app and embeddable engine (#665)

Drafted: installable manifest, 192/512px maskable icons, root-scoped service
worker, content-hashed inventory including WASM and all canonical seed files,
and typed browser `solve()`/memory/export/import package. `build:web` prepares
both output trees and synchronizes package version with the crate.

Cache installation fails if any shell/seed resource is missing. Only declared
public assets are cached; API responses and user memory never enter the shell
cache. Assets with the deployment's `?v=` suffix match their unversioned cached
record. The manifest uses relative paths for GitHub Pages subdirectory hosting.
The existing deployment synchronizes the seed after build:web, so the inventory
reads canonical seed bytes while targeting deployed `seed/` URLs.

The dedicated npm release workflow runs for a published GitHub release, builds
web assets, produces a pack artifact and publishes with provenance through npm
trusted publishing (or configured NPM_TOKEN). Actual registry publication requires
package ownership/trusted-publisher configuration. No publication was performed.

Pending acceptance: offline reload/chat with trace-link browser e2e, installability
measurement, full Rust parity-dialog npm fixture, and confirmation that the same
release trigger publishes crate and npm artifact. Deployment stamping occurs after
the manifest content hash is calculated; hashing stamped HTML should be added to
the deployment tail before treating the hash as exact for every final byte.
No builds or tests were run for this draft.
