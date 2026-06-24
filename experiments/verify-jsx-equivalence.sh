#!/bin/sh
# Issue #550: prove an h()->JSX conversion of the front-end is behaviour-
# preserving. Because tsconfig pins bun's JSX transform to the classic runtime
# (jsxFactory: h), JSX compiles back to the same h() calls. So if we compile a
# file before and after the conversion with `bun build --packages external`
# (unminified, app code only) and normalize, the output must be byte-identical
# except the leading `// <path>` banner bun prepends.
#
# Usage: verify-jsx-equivalence.sh <before.jsx> <after.jsx>
set -e
BEFORE="$1"
AFTER="$2"
norm() {
  bun build "$1" --target browser --format esm --packages external 2>/dev/null \
    | perl -pe 's{/\* \@__PURE__ \*/}{}g' \
    | grep -v '^// ' \
    | tr -s ' \n\t' ' '
}
norm "$BEFORE" > /tmp/eq_before.txt
norm "$AFTER"  > /tmp/eq_after.txt
if diff -q /tmp/eq_before.txt /tmp/eq_after.txt >/dev/null; then
  echo "EQUIVALENT: normalized compiled output is byte-identical."
  exit 0
else
  echo "DIFFERS: conversion is NOT equivalent. First differences:"
  diff /tmp/eq_before.txt /tmp/eq_after.txt | head -40
  exit 1
fi
