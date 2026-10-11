#!/usr/bin/env bash
# Build and execute a native image, then optionally observe its actual immutable registry manifest.
set -euo pipefail
: "${FORMAL_AI_CONTAINER_PROTOCOL_DIR:?}" "${CONTAINER_ARCH:?}" "${CONTAINER_VARIANT:?}"
: "${CONTAINER_PUBLISH:?}" "${CONTAINER_REPOSITORY:?}" "${CONTAINER_VERSION:?}"
case "$CONTAINER_ARCH" in amd64|arm64) ;; *) exit 64 ;; esac
case "$CONTAINER_VARIANT" in full) dockerfile=Dockerfile ;; slim) dockerfile=Dockerfile.slim ;; *) exit 64 ;; esac
case "$CONTAINER_PUBLISH" in true|false) ;; *) exit 64 ;; esac
node "$FORMAL_AI_CONTAINER_PROTOCOL_DIR/native-release-artifact.mjs" verify native-bin container-source "$CONTAINER_TARGET"
mkdir -p rust/target/release image-receipts
cp native-bin/formal-ai rust/target/release/formal-ai
binary_sha=$(node --input-type=module - <<'NODE'
import {readFileSync} from 'node:fs';
console.log(JSON.parse(readFileSync('native-bin/native-executable-receipt.json','utf8')).executable.sha256);
NODE
)
local_tag="formal-ai:${CONTAINER_ARCH}-${CONTAINER_VARIANT}"
docker buildx build --load --provenance=false --platform "linux/$CONTAINER_ARCH" \
  --file "$dockerfile" --build-arg BINARY_SOURCE=prebuilt --tag "$local_tag" \
  --label "org.opencontainers.image.revision=$CONTAINER_SOURCE_COMMIT" \
  --label "org.opencontainers.image.version=$CONTAINER_VERSION" \
  --label "io.link-assistant.formal-ai.executable.sha256=$binary_sha" \
  --metadata-file "image-receipts/build-${CONTAINER_VARIANT}-${CONTAINER_ARCH}.metadata" .
# Genuine startup, actual executable digest and runtime/size checks precede any registry write.
CONTAINER_PUBLISH=false node "$FORMAL_AI_CONTAINER_PROTOCOL_DIR/container-image-receipt.mjs" observe \
  container-source native-bin "$local_tag" "$CONTAINER_ARCH" "$CONTAINER_VARIANT" \
  "image-receipts/${CONTAINER_VARIANT}-${CONTAINER_ARCH}.json"
if [ "$CONTAINER_PUBLISH" = true ]; then
  : "${NATIVE_AUTHORIZED_RELEASE_COMMIT:?}"
  test "$NATIVE_AUTHORIZED_RELEASE_COMMIT" = "$CONTAINER_SOURCE_COMMIT"
  tag="$CONTAINER_REPOSITORY:run-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}-${CONTAINER_ARCH}-${CONTAINER_VARIANT}"
  docker tag "$local_tag" "$tag"
  docker push "$tag"
  docker pull "$tag"
  immutable=$(docker image inspect "$tag" --format '{{json .RepoDigests}}' | CONTAINER_REPOSITORY="$CONTAINER_REPOSITORY" node --input-type=module -e '
    import {readFileSync} from "node:fs";
    import assert from "node:assert/strict";
    const prefix=process.env.CONTAINER_REPOSITORY+"@sha256:";
    const names=[...new Set(JSON.parse(readFileSync(0,"utf8")).filter(value=>value.startsWith(prefix)))];
    assert.equal(names.length,1);assert.match(names[0],/@sha256:[a-f0-9]{64}$/u);console.log(names[0]);
  ')
  node "$FORMAL_AI_CONTAINER_PROTOCOL_DIR/container-image-receipt.mjs" observe \
    container-source native-bin "$immutable" "$CONTAINER_ARCH" "$CONTAINER_VARIANT" \
    "image-receipts/${CONTAINER_VARIANT}-${CONTAINER_ARCH}.json"
fi
