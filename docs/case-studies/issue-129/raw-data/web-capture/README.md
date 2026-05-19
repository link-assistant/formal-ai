# web-capture

**JavaScript:** [![npm version](https://img.shields.io/npm/v/@link-assistant/web-capture?label=npm&color=blue)](https://www.npmjs.com/package/@link-assistant/web-capture) [![npm downloads](https://img.shields.io/npm/dm/@link-assistant/web-capture?label=downloads&color=blue)](https://www.npmjs.com/package/@link-assistant/web-capture) [![CI - JavaScript](https://img.shields.io/github/actions/workflow/status/link-assistant/web-capture/js.yml?branch=main&label=JS%20CI)](https://github.com/link-assistant/web-capture/actions/workflows/js.yml)

**Rust:** [![crates.io version](https://img.shields.io/crates/v/web-capture?label=crates.io&color=orange)](https://crates.io/crates/web-capture) [![crates.io downloads](https://img.shields.io/crates/d/web-capture?label=downloads&color=orange)](https://crates.io/crates/web-capture) [![docs.rs](https://img.shields.io/docsrs/web-capture?label=docs.rs)](https://docs.rs/web-capture) [![CI - Rust](https://img.shields.io/github/actions/workflow/status/link-assistant/web-capture/rust.yml?branch=main&label=Rust%20CI)](https://github.com/link-assistant/web-capture/actions/workflows/rust.yml)

**Release:** [![GitHub Release](https://img.shields.io/github/v/release/link-assistant/web-capture?label=GitHub%20Release)](https://github.com/link-assistant/web-capture/releases) [![License: Unlicense](https://img.shields.io/badge/license-Unlicense-green)](https://unlicense.org)

A CLI and microservice to fetch URLs and render them as:

- **Markdown**: Clean HTML-to-Markdown conversion with image extraction
- **HTML**: Rendered page content
- **PNG/JPEG screenshot**: Viewport or full-page capture
- **ZIP archive**: Markdown/HTML + locally downloaded images
- **PDF**: Print-quality document
- **DOCX**: Word document

## Language Implementations

This repository contains two implementations with compatible APIs:

| Implementation         | Directory          | Package                                                                                  | Status     |
| ---------------------- | ------------------ | ---------------------------------------------------------------------------------------- | ---------- |
| **JavaScript/Node.js** | [`./js`](./js)     | [@link-assistant/web-capture](https://www.npmjs.com/package/@link-assistant/web-capture) | Production |
| **Rust**               | [`./rust`](./rust) | [web-capture](https://crates.io/crates/web-capture)                                      | Production |

Both implementations provide the same CLI interface and HTTP API endpoints, allowing you to choose based on your deployment preferences.

## Quick Start

### JavaScript

```bash
cd js
npm install
npm run dev
```

### Rust

```bash
cd rust
cargo run -- --serve
```

## CLI Usage

```bash
# Capture a URL as Markdown (default format, writes to ./data/web-capture/<host>/<path>/)
web-capture https://example.com

# Capture as Markdown and save to specific file
web-capture https://example.com -o page.md

# Write to stdout explicitly
web-capture https://example.com -o -

# Capture as HTML
web-capture https://example.com --format html

# Take a screenshot
web-capture https://example.com --format png -o screenshot.png

# Create a ZIP archive
web-capture https://example.com --archive

# Start as API server
web-capture --serve

# Start server on custom port
web-capture --serve --port 8080
```

## CLI Options

| Option                   | Short | Description                                                                             | Default               |
| ------------------------ | ----- | --------------------------------------------------------------------------------------- | --------------------- |
| `--serve`                | `-s`  | Start as HTTP API server                                                                | -                     |
| `--port`                 | `-p`  | Port to listen on                                                                       | 3000                  |
| `--format`               | `-f`  | Output format: `markdown`/`md`, `html`, `image`/`png`, `jpeg`, `pdf`, `docx`, `archive` | `markdown`            |
| `--output`               | `-o`  | Output file path. Use `-o -` for stdout                                                 | auto-derived from URL |
| `--data-dir`             |       | Base directory for auto-derived output paths                                            | `./data/web-capture`  |
| `--engine`               | `-e`  | Browser engine (JS only): `puppeteer`, `playwright`                                     | `puppeteer`           |
| `--embed-images`         |       | Keep images as inline base64 data URIs                                                  | `false`               |
| `--no-extract-images`    |       | Alias for `--embed-images`                                                              | `false`               |
| `--keep-original-links`  |       | Keep original remote image URLs, strip base64                                           | `false`               |
| `--images-dir`           |       | Subdirectory name for extracted images                                                  | `images`              |
| `--archive`              |       | Create archive: `zip` (default), `7z`, `tar.gz`, `tar`                                  | -                     |
| `--extract-latex`        |       | Extract LaTeX formulas                                                                  | `true`                |
| `--extract-metadata`     |       | Extract article metadata                                                                | `true`                |
| `--post-process`         |       | Apply post-processing                                                                   | `true`                |
| `--detect-code-language` |       | Detect code block languages                                                             | `true`                |

## Environment Variables

All flags can be controlled via environment variables:

| Variable                           | Description                         | Default              |
| ---------------------------------- | ----------------------------------- | -------------------- |
| `WEB_CAPTURE_DATA_DIR`             | Base directory for output           | `./data/web-capture` |
| `WEB_CAPTURE_EMBED_IMAGES`         | `0`/`1` — keep images inline        | `0`                  |
| `WEB_CAPTURE_KEEP_ORIGINAL_LINKS`  | `0`/`1` — keep original remote URLs | `0`                  |
| `WEB_CAPTURE_IMAGES_DIR`           | Subdirectory for extracted images   | `images`             |
| `WEB_CAPTURE_EXTRACT_LATEX`        | `0`/`1` — extract LaTeX             | `1`                  |
| `WEB_CAPTURE_EXTRACT_METADATA`     | `0`/`1` — extract metadata          | `1`                  |
| `WEB_CAPTURE_POST_PROCESS`         | `0`/`1` — post-processing           | `1`                  |
| `WEB_CAPTURE_DETECT_CODE_LANGUAGE` | `0`/`1` — detect code langs         | `1`                  |

## API Endpoints

Both implementations expose the same API:

| Endpoint                                          | Description                                               |
| ------------------------------------------------- | --------------------------------------------------------- |
| `GET /html?url=<URL>`                             | Get rendered HTML content                                 |
| `GET /markdown?url=<URL>`                         | Get Markdown (original links kept, base64 stripped)       |
| `GET /markdown?url=<URL>&embedImages=true`        | Get Markdown with base64 images inline                    |
| `GET /markdown?url=<URL>&keepOriginalLinks=false` | Get Markdown with all images stripped                     |
| `GET /image?url=<URL>`                            | Get PNG screenshot                                        |
| `GET /archive?url=<URL>`                          | ZIP archive with markdown + images extracted to `images/` |
| `GET /archive?url=<URL>&keepOriginalLinks=true`   | ZIP archive keeping original remote image URLs            |
| `GET /archive?url=<URL>&embedImages=true`         | ZIP archive with base64 images inline                     |
| `GET /pdf?url=<URL>`                              | PDF with embedded images                                  |
| `GET /docx?url=<URL>`                             | DOCX with embedded images                                 |
| `GET /fetch?url=<URL>`                            | Proxy fetch content                                       |
| `GET /stream?url=<URL>`                           | Stream content                                            |

## Docker

### JavaScript

```bash
cd js
docker build -t web-capture-js .
docker run -p 3000:3000 web-capture-js
```

### Rust

```bash
cd rust
docker build -t web-capture-rust .
docker run -p 3000:3000 web-capture-rust
```

## Project Structure

```
web-capture/
├── js/                          # JavaScript/Node.js implementation
│   ├── src/                     # Source code
│   ├── bin/                     # CLI entry point
│   ├── tests/                   # Test files
│   ├── examples/                # Usage examples
│   ├── package.json             # npm package manifest
│   ├── Dockerfile               # Docker build file
│   └── README.md                # JavaScript-specific docs
│
├── rust/                        # Rust implementation
│   ├── src/                     # Source code
│   │   ├── lib.rs               # Library exports
│   │   ├── main.rs              # CLI/server entry point
│   │   ├── browser.rs           # Browser automation
│   │   ├── html.rs              # HTML processing
│   │   └── markdown.rs          # Markdown conversion
│   ├── tests/                   # Test files
│   ├── examples/                # Usage examples
│   ├── Cargo.toml               # Cargo package manifest
│   ├── Dockerfile               # Docker build file
│   └── README.md                # Rust-specific docs
│
├── scripts/                     # Shared build/release scripts
│   ├── *.mjs                    # JavaScript-specific scripts
│   └── rust-*.mjs               # Rust-specific scripts
│
├── .github/workflows/
│   ├── js.yml                   # JavaScript CI/CD
│   └── rust.yml                 # Rust CI/CD
│
└── README.md                    # This file
```

## Development

### JavaScript

```bash
cd js
npm install
npm run dev          # Start dev server
npm test             # Run tests
npm run lint         # Run linter
```

### Rust

```bash
cd rust
cargo build          # Build
cargo test           # Run tests
cargo clippy         # Run linter
cargo fmt            # Format code
```

## Features

- **Markdown Conversion**: Clean HTML-to-Markdown with LaTeX extraction, metadata, and code language detection
- **Image Extraction**: Base64 data URI images extracted to files with content-hash filenames
- **HTML Rendering**: Fetch and render HTML with JavaScript support via headless browsers
- **Screenshots**: Capture PNG/JPEG screenshots with theme and viewport control
- **Archives**: ZIP archives with markdown/HTML + locally downloaded images
- **Google Docs**: Public export, Google Docs REST API, and editor-model capture
- **URL Normalization**: Convert relative URLs to absolute
- **Encoding Detection**: Automatic charset detection and UTF-8 conversion

## Browser Engines

### JavaScript Version

- **Puppeteer** (default): Mature, well-tested Chrome automation
- **Playwright**: Cross-browser automation with similar capabilities

### Rust Version

- **browser-commander**: A Rust crate for browser automation using chromiumoxide

## License

[Unlicense](LICENSE) — This is free and unencumbered software released into the public domain. You are free to copy, modify, publish, use, compile, sell, or distribute this software for any purpose, commercial or non-commercial, and by any means. See [https://unlicense.org](https://unlicense.org) for details.

## Related Projects

- [browser-commander](https://github.com/link-foundation/browser-commander) - Browser automation library used in Rust implementation
- [turndown](https://github.com/mixmark-io/turndown) - HTML to Markdown converter used in JS implementation
- [html2md](https://github.com/nickyc975/html2md-rs) - HTML to Markdown converter used in Rust implementation
