#!/usr/bin/env sh
set -eu

rustc \
  --target wasm32-unknown-unknown \
  --crate-type cdylib \
  -C opt-level=z \
  -C panic=abort \
  -C lto=fat \
  -C codegen-units=1 \
  -C strip=symbols \
  -C link-arg=-s \
  "$(dirname "$0")/src/lib.rs" \
  -o "$(dirname "$0")/../formal_ai_worker.wasm"
