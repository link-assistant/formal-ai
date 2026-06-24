FormalAI issue link: https://github.com/link-assistant/formal-ai/issues/410

During the FormalAI issue 410 case study, I checked whether `web-search` can be consumed as the web-search component for FormalAI.

Current blocker:

- The repository contains a JavaScript package manifest for `@link-assistant/web-search` and a Rust crate manifest under `rust/`, but package lookup fails for both public registries:
  - `npm view @link-assistant/web-search ...` returns npm `E404`.
  - `cargo info web-search` reports that `web-search` cannot be found in crates.io.
- `web-capture` is already published on npm and crates.io, so FormalAI can reference it through normal package channels, but `web-search` currently requires a Git checkout/path dependency.

Why this blocks FormalAI:

FormalAI cannot safely swap its current in-repo web-search implementation to `web-search` until there is a stable install target for CI, releases, and downstream users. A Git checkout dependency is too brittle for the long-term component boundary requested in FormalAI issue 410.

Requested acceptance criteria:

- Publish `@link-assistant/web-search` to npm, or document the intended package name if it should be different.
- Publish the Rust crate to crates.io, or document that Rust consumption should use another supported distribution channel.
- Add release tags or GitHub releases that map to the published package versions.
- Document the supported library, CLI, and HTTP server entry points in the README with versioned install commands.
- Ensure CI exercises install-from-package smoke tests after publication.

Evidence captured in FormalAI PR 414 case-study raw data:

- `docs/case-studies/issue-410/raw-data/package-probes/npm-link-assistant-web-search.txt`
- `docs/case-studies/issue-410/raw-data/package-probes/cargo-info-web-search.txt`
- `docs/case-studies/issue-410/raw-data/web-search/package.json`
- `docs/case-studies/issue-410/raw-data/web-search/Cargo.toml`
