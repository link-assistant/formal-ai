#!/usr/bin/env bash
#
# Reproduces the docs.rs build failure that makes formal-ai's documentation show
# as "All builds failed" (issue #736, docs.rs build 3868612).
#
# The failing crate is lindera-jieba, pulled in non-optionally via meta-language.
# The defect is in lindera-dictionary/src/assets.rs, inside the branch that
# exists specifically to support docs.rs. Two distinct bugs, both shown here:
#
#   Build #1 (fresh OUT_DIR): the dummy dictionary is scaffolded into input_dir/
#   but the builder reads input_dir/<src_subdir>/, which lindera-jieba sets to
#   "dict-src". Fails: "Failed to open file: .../out/mecab-jieba-0.1.1/dict-src/char.def"
#
#   Build #2 (same OUT_DIR): fs::create_dir is not idempotent, so the re-run
#   trips over the directory build #1 created. Fails with the exact error seen on
#   docs.rs: "Failed to create dummy input directory ... File exists (os error 17)"
#
# Verified against lindera-jieba 4.0.0 (the latest release), so upgrading does
# not help.
set -uo pipefail
cd "$(dirname "$0")"

echo "=== Build #1: fresh OUT_DIR, DOCS_RS=1 (as docs.rs sets it) ==="
DOCS_RS=1 cargo build 2>&1 | tail -6

echo
echo "=== Build #2: same OUT_DIR, DOCS_RS=1 (a docs.rs build-script re-run) ==="
DOCS_RS=1 cargo build 2>&1 | tail -6
