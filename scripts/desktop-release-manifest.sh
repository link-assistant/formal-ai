#!/usr/bin/env bash
# Workflow-selected desktop checksum and complete/partial provenance reporting.
set -euo pipefail
shopt -s nullglob
cp release-evidence/SHA256SUMS-evidence.partial fragments/
parts=(fragments/*.partial)
if [ ${#parts[@]} -eq 0 ]; then
  echo "No checksum fragments were produced; nothing to finalize." >&2
  exit 1
fi
# Reconcile shared updater metadata against actual published bytes.
node "$FORMAL_AI_NATIVE_PROTOCOL_DIR/release-checksums.mjs" "${parts[@]}" > SHA256SUMS.txt
echo "=== SHA256SUMS.txt ==="
cat SHA256SUMS.txt
commit="$(gh release view "$TAG" --repo "$REPO" --json targetCommitish --jq .targetCommitish || echo unknown)"

# Report the builders that actually produced a fragment. Hardcoding the
# full matrix here claimed builds that never happened whenever an arch
# failed under fail-fast: false.
declare -A builder_of=(
  [linux-x64]="ubuntu-24.04"
  [linux-arm64]="ubuntu-24.04-arm"
  [macos-x64]="macos-15-intel"
  [macos-arm64]="macos-15"
  [windows-x64]="windows-2025"
  [windows-arm64]="windows-11-arm"
  [cli-linux-x64]="ubuntu-24.04"
  [cli-linux-arm64]="ubuntu-24.04-arm"
  [cli-macos-x64]="macos-15-intel"
  [cli-macos-arm64]="macos-15"
  [cli-windows-x64]="windows-2025"
  [vscode]="ubuntu-24.04"
)
built=()
missing=()
for label in linux-x64 linux-arm64 macos-x64 macos-arm64 windows-x64 windows-arm64 cli-linux-x64 cli-linux-arm64 cli-macos-x64 cli-macos-arm64 cli-windows-x64 vscode; do
  if [ -f "fragments/SHA256SUMS-${label}.partial" ]; then
    built+=("${builder_of[$label]} (${label})")
  else
    missing+=("$label")
  fi
done

if [ -n "$EVIDENCE_INCOMPLETE" ]; then missing+=("native-evidence:$EVIDENCE_INCOMPLETE"); fi

{
  echo "formal-ai Desktop build provenance"
  echo "=================================="
  echo "Repository : $REPO"
  echo "Release tag: $TAG"
  echo "Commitish  : $commit"
  echo "Source SHA : $NATIVE_SOURCE_COMMIT"
  echo "Source tree: $NATIVE_SOURCE_TREE"
  echo "Source receipt SHA256: $NATIVE_SELECTION_SHA256"
  echo "Rust compiler: $NATIVE_COMPILER_RELEASE ($NATIVE_COMPILER_COMMIT)"
  echo "Workflow   : Desktop Release"
  echo "Run        : $RUN_URL"
  printf 'Builders   : %s\n' "${built[0]}"
  printf '             %s\n' "${built[@]:1}"
  if [ ${#missing[@]} -gt 0 ]; then
    echo "INCOMPLETE : no artifacts from: ${missing[*]}"
    echo "             This release is missing those targets; the checksums"
    echo "             below cover only the builders listed above."
  fi
  echo "Signing    : validated actual package observations (per macOS target):"
  cat release-evidence/signing-summary.txt
  echo "Attestation: SLSA build provenance attached via actions/attest."
  echo "             Verify with: gh attestation verify <file> --repo $REPO"
  echo ""
  # Deliberately not "authoritative": when `missing` is non-empty this
  # manifest covers only part of the release, and calling it
  # authoritative is exactly the false positive we are fixing.
  echo "SHA-256 checksums of the artifacts listed above (SHA256SUMS.txt is the machine-readable copy):"
  echo ""
  cat SHA256SUMS.txt
} > BUILD-PROVENANCE.txt
echo "=== BUILD-PROVENANCE.txt ==="
cat BUILD-PROVENANCE.txt

# Still upload what we have (a partial manifest beats none), but never
# report success for it: a green run here means "all desktop assets
# shipped", and callers rely on that.
echo "incomplete=${missing[*]}" >> "$GITHUB_OUTPUT"
node "$FORMAL_AI_NATIVE_PROTOCOL_DIR/release-producer-output-receipt.mjs" "$GITHUB_OUTPUT" "$FORMAL_AI_NATIVE_PROTOCOL_DIR/producer-output-workflow.yml" "$FORMAL_AI_NATIVE_PROTOCOL_DIR/protocol-selection.json" producer-output-receipt.json
