# rust-ai-driven-development-pipeline-template

A comprehensive template for AI-driven Rust development with full CI/CD pipeline support.

[![CI/CD Pipeline](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/actions?workflow=CI%2FCD+Pipeline)
[![Crates.io](https://img.shields.io/crates/v/example-sum-package-name?label=crates.io&style=flat)](https://crates.io/crates/example-sum-package-name)
[![Docs.rs](https://docs.rs/example-sum-package-name/badge.svg)](https://docs.rs/example-sum-package-name)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![Codecov](https://codecov.io/gh/link-foundation/rust-ai-driven-development-pipeline-template/branch/main/graph/badge.svg)](https://codecov.io/gh/link-foundation/rust-ai-driven-development-pipeline-template)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Features

- **Rust stable support**: Works with Rust stable version
- **Cross-platform testing**: CI runs on Ubuntu, macOS, and Windows
- **Comprehensive testing**: Unit tests, integration tests, and doc tests
- **Code quality**: rustfmt + Clippy with pedantic lints
- **Pre-commit hooks**: Automated code quality checks before commits
- **CI/CD pipeline**: GitHub Actions with multi-platform support
- **Changelog management**: Fragment-based changelog (like Changesets/Scriv)
- **Code coverage**: Automated coverage reports with cargo-llvm-cov and Codecov
- **Release automation**: Automatic GitHub releases, crates.io publishing, post-publish smoke tests, and optional Docker Hub image publishing
- **Template-safe defaults**: CI/CD skips publishing when package name is `example-sum-package-name`

## Quick Start

### Using This Template

1. Click "Use this template" on GitHub to create a new repository
2. Clone your new repository
3. Update `Cargo.toml`:
   - Change `name` from `example-sum-package-name` to your package name
   - Update `description`, `repository`, and `documentation` URLs
   - Update `[lib]` name and `[[bin]]` name
4. Keep `Cargo.lock` committed when the project has a binary target (`[[bin]]` or `src/main.rs`)
5. Update imports in `src/main.rs`, `tests/`, and `examples/`
6. Build and start developing!

### Development Setup

```bash
# Clone the repository
git clone https://github.com/link-foundation/rust-ai-driven-development-pipeline-template.git
cd rust-ai-driven-development-pipeline-template

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI binary
cargo run -- --a 3 --b 7

# Run an example
cargo run --example basic_usage
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test --verbose

# Run doc tests
cargo test --doc

# Run a specific test
cargo test test_sum_positive_numbers

# Run tests with output
cargo test -- --nocapture
```

CI caps each test-matrix job at 10 minutes. `cargo test` does not provide a portable global per-test timeout, so long-running network, IO, or async tests should use explicit test-level timeouts. Repositories that adopt `cargo nextest` can configure runner deadlines with options such as `--slow-timeout` and `--leak-timeout`.

### Code Quality Checks
