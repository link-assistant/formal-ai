#!/usr/bin/env bash
# Workflow-selected durable desktop worker and receipt evidence.
set -euo pipefail
mkdir -p native-receipts native-signing
node "$FORMAL_AI_NATIVE_PROTOCOL_DIR/native-release-evidence.mjs" collect native-source native-receipts native-signing release-evidence
node "$FORMAL_AI_NATIVE_PROTOCOL_DIR/release-advertisement.mjs" verify "$GITHUB_WORKSPACE" native-source/advertisement "$NATIVE_PROTOCOL_COMMIT" "$ADVERTISEMENT_PROTOCOL_SHA256" "release-evidence/formal-ai-advertisement-$NATIVE_PACKAGE_VERSION.json"
(cd release-evidence && sha256sum "formal-ai-advertisement-$NATIVE_PACKAGE_VERSION.json") >> release-evidence/SHA256SUMS-evidence.partial
node --input-type=module - <<'NODE'
import {readFileSync,appendFileSync} from 'node:fs';
const result=JSON.parse(readFileSync('release-evidence/collection-result.json'));
appendFileSync(process.env.GITHUB_OUTPUT,'incomplete='+result.failures.map(failure=>failure.kind+':'+failure.target).join(',')+'\n');
NODE
