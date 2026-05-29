# lino-arguments

A unified configuration library combining environment variables and CLI arguments with a clear priority chain.

Available in both JavaScript and Rust.

[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Overview

`lino-arguments` provides a unified configuration system that automatically loads configuration from multiple sources with a clear priority chain:

1. **CLI arguments** - Highest priority (manually entered options)
2. **Environment variables** - With case-insensitive lookup
3. **Configuration file** - Dynamic config file path via CLI
4. **Default values** - Fallback values

## Languages

### JavaScript

[![npm version](https://img.shields.io/npm/v/lino-arguments.svg)](https://www.npmjs.com/package/lino-arguments)

The JavaScript version supports Node.js, Bun, and Deno runtimes.

```bash
npm install lino-arguments
```

```javascript
import { makeConfig } from 'lino-arguments';

const config = makeConfig({
  yargs: ({ yargs, getenv }) =>
    yargs.option('port', { default: getenv('PORT', 3000) }),
});
```

See [js/README.md](js/) for full documentation.

### Rust

[![Crates.io](https://img.shields.io/crates/v/lino-arguments.svg)](https://crates.io/crates/lino-arguments)
[![docs.rs](https://img.shields.io/docsrs/lino-arguments)](https://docs.rs/lino-arguments)

The Rust version uses clap for argument parsing.

```toml
[dependencies]
lino-arguments = "0.2"
```

```rust
use clap::Parser;
use lino_arguments::{getenv_int, getenv_bool};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value_t = getenv_int("PORT", 3000) as u16)]
    port: u16,
}
```

See [rust/README.md](rust/) for full documentation.

## Repository Structure

```
lino-arguments/
├── js/                    # JavaScript implementation
│   ├── src/              # Source code
│   ├── tests/            # Tests
│   ├── examples/         # Usage examples
│   ├── .changeset/       # Changesets for JS releases
│   └── package.json      # npm package config
├── rust/                  # Rust implementation
│   ├── src/              # Source code
│   ├── tests/            # Integration tests
│   ├── changelog.d/      # Changelog fragments for Rust releases
│   └── Cargo.toml        # Cargo package config
├── scripts/              # Shared release/CI scripts
├── .github/workflows/    # CI/CD workflows
│   ├── js.yml           # JavaScript CI/CD
│   └── rust.yml         # Rust CI/CD
└── docs/                 # Documentation
```

## Development

### JavaScript

```bash
cd js
npm install
npm test
npm run lint
```

### Rust

```bash
cd rust
cargo test
cargo clippy
cargo fmt
```

## Contributing

See the language-specific directories for contribution guidelines:
- [JavaScript](js/)
- [Rust](rust/)

Both languages use changelog fragments for versioning:
- JavaScript uses `.changeset/` with changesets
- Rust uses `changelog.d/` with markdown fragments

## License

This is free and unencumbered software released into the public domain. See the [LICENSE](LICENSE) file for details.
