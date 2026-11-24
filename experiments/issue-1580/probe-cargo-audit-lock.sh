#!/usr/bin/env bash
# Run the published cargo-audit against a finite offline dependency graph.
# Pass the path to the unpacked cargo-audit executable.
set -euo pipefail

audit_binary="$(realpath -- "${1:?provide a cargo-audit executable}")"
fixture="$(mktemp -d)"
trap 'rm -rf -- "$fixture"' EXIT
cd "$fixture"

mkdir -p advisory-db/crates missing/src src
git -C advisory-db init --quiet --initial-branch main
git -C advisory-db -c user.name=Fixture -c user.email=fixture@example.invalid \
  -c commit.gpgsign=false commit --quiet --allow-empty -m fixture
cat > Cargo.toml <<'EOF'
[package]
name = "audit-panic-repro"
version = "0.1.0"
edition = "2024"
[dependencies]
missing = { path = "missing" }
EOF
cat > missing/Cargo.toml <<'EOF'
[package]
name = "missing"
version = "1.0.0"
edition = "2024"
EOF
touch src/lib.rs missing/src/lib.rs

cargo generate-lockfile --offline
"$audit_binary" audit --version
cargo metadata --locked --offline --format-version 1 > /dev/null
"$audit_binary" audit --file Cargo.lock --db "$fixture/advisory-db" --no-fetch --no-yanked --deny warnings

# Remove the named dependency's record while retaining the incoming reference.
sed -i '/^name = "missing"$/,$d' Cargo.lock
sed -i '${/^\[\[package\]\]$/d;}' Cargo.lock

metadata_status=0
cargo metadata --locked --offline --format-version 1 > /dev/null 2> metadata.log || metadata_status=$?
cat metadata.log
test "$metadata_status" -eq 101
grep -F 'cannot update the lock file' metadata.log

audit_status=0
"$audit_binary" audit --file Cargo.lock --db "$fixture/advisory-db" --no-fetch --no-yanked --deny warnings \
  > audit.log 2>&1 || audit_status=$?
cat audit.log
test "$audit_status" -ne 0
grep -F 'failed to find dependency: missing' audit.log
printf 'Malformed lock refused: cargo metadata status %s, cargo-audit status %s\n' "$metadata_status" "$audit_status"
