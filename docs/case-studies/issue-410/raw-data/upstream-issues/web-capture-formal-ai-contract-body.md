FormalAI issue link: https://github.com/link-assistant/formal-ai/issues/410

During the FormalAI issue 410 case study, I checked how `web-capture` can be used as the FormalAI web fetch/capture component.

Current state:

- `web-capture` is published as `@link-assistant/web-capture` on npm and `web-capture` on crates.io.
- It provides CLI and HTTP endpoints for HTML, text, Markdown, screenshots, archives, PDF, DOCX, fetch, stream, and structured search-provider capture.
- Structured search-provider capture currently supports `wikipedia`, `duckduckgo`, `google`, `bing`, and `brave`.
- The Rust crate has `rust-version = 1.88`; FormalAI currently declares Rust 1.70, so a direct Rust dependency would raise FormalAI's MSRV. CLI/HTTP integration avoids that coupling.

Why this matters for FormalAI:

FormalAI can use `web-capture` as an optional CLI/HTTP fetch/capture service sooner than it can use it as a direct Rust dependency. To make that integration stable, FormalAI needs a clear component contract for request/response schema, diagnostics, provider limits, and version compatibility.

Requested acceptance criteria:

- Document a stable FormalAI-compatible HTTP/CLI contract for `/fetch`, `/html`, `/txt`, `/markdown`, `/image`, `/archive`, `/stream`, and `/search`.
- Expose or document a provider catalog for `/search`, including the five current providers and any planned relationship to `web-search` for broader provider coverage.
- Keep normalized diagnostics for capture/search failures, including HTTP status, source URL, CAPTCHA/block detection, and errors.
- Document whether FormalAI should integrate via HTTP/CLI because of the Rust 1.88 MSRV, or lower/feature-gate the Rust crate MSRV if direct Rust library integration is a supported goal.
- Add smoke tests that assert the stable CLI and HTTP response shapes FormalAI should depend on.

Evidence captured in FormalAI PR 414 case-study raw data:

- `docs/case-studies/issue-410/raw-data/package-probes/npm-link-assistant-web-capture.json`
- `docs/case-studies/issue-410/raw-data/package-probes/cargo-info-web-capture.txt`
- `docs/case-studies/issue-410/raw-data/web-capture/README.md`
- `docs/case-studies/issue-410/raw-data/web-capture/search-provider-contract.txt`
