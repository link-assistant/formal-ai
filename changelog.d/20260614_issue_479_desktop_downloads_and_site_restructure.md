---
bump: minor
---

### Fixed
- Desktop app downloads are available again on `/download` for every platform. The desktop-release workflow resolved the parent commit SHA, but the auto-release tag is created on the child "chore: release" commit, so the exact-SHA match never succeeded and v0.187.0–v0.201.0 shipped zero desktop assets. Resolution is now two-tier (exact SHA, then the latest published release / auto-release child commit), with verbose logging for future diagnosis (issue #479).
- Refreshed the obsolete `/download` desktop app-preview and page screenshots.
- Replaced a manual `(len + 1) / 2` ceiling in the lexicon matcher with `div_ceil`, satisfying clippy's `manual_div_ceil` lint under newer stable toolchains.

### Added
- macOS Gatekeeper install screenshots on the `/download` page, mirroring the vk-bot-desktop walkthrough.
- A landing-page chooser at `/` that links to the web app (`/app/`), the documentation hub (`/docs/`), and the desktop download (`/download/`), wired to the shared theme + UI-language preference store and localized into en/ru/zh/hi.
- A documentation hub at `/docs/` and a generated Rust API reference at `/docs/api/`, built with `cargo doc` during the GitHub Pages deploy.
- End-to-end coverage for the new landing and documentation pages, plus CI guards for the new static and deploy invariants.

### Changed
- The interactive web app moved from `/` to `/app/`, served with `<base href="../">` so its shared site-root assets still resolve under both the GitHub Pages path prefix and the desktop static server; the desktop wrapper and in-site back-links now target `/app/`.
- The release pipeline's concurrency is now main-safe, so a release run on `main` is never cancelled mid-flight.
