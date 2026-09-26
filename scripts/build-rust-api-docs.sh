#!/usr/bin/env bash
# Build the rustdoc API reference published under /docs/api/ on GitHub Pages.
#
# `--lib --no-deps` documents just the formal_ai library and keeps the output
# small. RUSTDOCFLAGS=-D warnings is set by the caller so rustdoc stays
# fail-closed like rustc: a green Pages deployment must not be able to conceal
# broken intra-doc links or malformed markup.
#
# Issue #977: extracted from release.yml to keep that file under the 2000-line
# ceiling scripts/check-file-size.rs enforces.
set -euo pipefail

# Discover the crate manifest instead of assuming a fixed layout: the
# repository root (single-language) or rust/ (multi-language, plan 16 L1).
if [ -f ./Cargo.toml ]; then
  manifest=./Cargo.toml
elif [ -f ./rust/Cargo.toml ]; then
  manifest=rust/Cargo.toml
else
  echo "Error: could not find Cargo.toml in ./ or ./rust/" >&2
  exit 1
fi

# --target-dir pins the output to the repository-root ./target/doc that the
# Pages artifact assembly copies from, regardless of manifest location.
cargo doc --no-deps --lib \
  --manifest-path "$manifest" \
  --target-dir target

# rustdoc emits the crate docs under target/doc/formal_ai/; add a root redirect
# so /docs/api/ lands on the crate root instead of 404ing.
cat > target/doc/index.html <<'HTML'
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="refresh" content="0; url=formal_ai/index.html" />
    <link rel="canonical" href="formal_ai/index.html" />
    <title>formal-ai API reference</title>
  </head>
  <body>
    <p>Redirecting to the <a href="formal_ai/index.html">formal-ai API reference</a>&hellip;</p>
  </body>
</html>
HTML
