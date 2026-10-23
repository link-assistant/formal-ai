# Issue #1051: two ways to get the binary, selected by `--build-arg BINARY_SOURCE`.
#
#   compile   (default) build from source in this image. What `main` uses, so
#             the Dockerfile is always proven able to build the project alone.
#   prebuilt  copy the binary the pipeline already built. A pull request
#             compiles `formal-ai` in four separate jobs; this stage was the
#             slowest at 33 minutes -- 510 crates with no sccache, since the
#             runner's cache cannot reach inside BuildKit -- and it gated the
#             whole run on its own.
#
# The copied binary runs because both sides are Ubuntu 24.04 on glibc 2.39:
# `ubuntu-latest` builds it and `konard/box-dind:2.10.2` runs it.
ARG BINARY_SOURCE=compile

FROM rust:1.99-slim AS builder

WORKDIR /app
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Dependencies are their own layer, keyed on the manifests alone.
#
# `COPY . .` before `cargo build` made every file in the tree part of the
# build layer's cache key, so editing one `.rs` rebuilt all ~500 dependency
# crates. The image build measured 24 minutes with its slowest layers at 428s,
# 419s and 355s, and it gates the pipeline's finish on its own.
#
# The stand-in sources exist because `cargo build` needs the targets its
# manifest declares. `build.rs` is copied for real -- it reads
# `data/seed/api-cache/` and emits an empty registry when absent, which is the
# case in this build, so it does not pull the data tree into this layer.
# Plan 16 L1 moved the crate to `rust/`, so the manifests and the stand-in
# sources are staged under /app/rust and cargo runs against that manifest.
COPY rust/Cargo.toml rust/Cargo.lock rust/build.rs rust/
RUN mkdir -p rust/src rust/tests/unit rust/tests/integration && \
    echo 'fn main() {}' > rust/src/main.rs && \
    echo '' > rust/src/lib.rs && \
    echo '' > rust/tests/unit/mod.rs && \
    echo '' > rust/tests/integration/mod.rs && \
    cargo build --release --locked --lib --bins --manifest-path rust/Cargo.toml && \
    rm -rf rust/src rust/tests

# Only this layer is invalidated by a source edit.
COPY . .
RUN cargo build --release --locked --bins --manifest-path rust/Cargo.toml

# `builder` under the name `BINARY_SOURCE` selects, so the historical stage name
# stays what it has always been -- `docker_runtime` pins it, and renaming a
# stage to satisfy a build argument would be the tail wagging the dog.
#
# Contract, shared by both source stages: the binary at
# /app/target/release/formal-ai, which `selected-binary` copies from. The
# build above runs `cargo build --manifest-path rust/Cargo.toml`, and cargo
# puts the target directory next to the top-level manifest -- so after the
# Plan 16 L1 move the binary lands in rust/target/release. The prebuilt stage
# below stages its copy at the same path for the same reason (issue #1151:
# the compile path never did, and no pull request exercises it -- the
# docker-build check pins BINARY_SOURCE=prebuilt -- so the first main release
# to reach the GHCR push found nothing at /app/target/release/formal-ai).
FROM builder AS compile-binary
RUN mkdir -p target/release && \
    cp rust/target/release/formal-ai target/release/formal-ai

# The prebuilt path: no toolchain, no compilation, just the artifact the
# pipeline already produced and tested.
FROM scratch AS prebuilt-binary
COPY rust/target/release/formal-ai /app/target/release/formal-ai

# Resolves to whichever stage `BINARY_SOURCE` names.
FROM ${BINARY_SOURCE}-binary AS selected-binary

# Official multiarch Node binary: target architecture follows the image build.
FROM node:26.12.0-bookworm-slim AS node-runtime

FROM konard/box-dind:2.10.2

LABEL org.opencontainers.image.source="https://github.com/link-assistant/formal-ai"

ENV HOME=/home/box \
    FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino \
    FORMAL_AI_IMAGE_VARIANT=dind \
    FORMAL_AI_START_ISOLATION=docker \
    FORMAL_AI_START_RUNNER="$ --isolated docker --auto-remove-docker-container --" \
    DIND_STORAGE_DRIVER="vfs" \
    BUN_INSTALL=/home/box/.bun
ENV PATH="${BUN_INSTALL}/bin:/usr/local/bin:${PATH}"

# box-dind 2.10.2 ends its own build as `USER box` (2.1.1 ended as root), so
# the system package install switches back to root explicitly; apt cannot
# write /var/lib/apt/lists as box. The bun installs below run as box again.
USER root
# Reuse the bounded mirror retry; every attempt retains apt's integrity checks.
COPY --chmod=0755 scripts/apt-install-with-retry.sh scripts/run-with-deadline.sh scripts/prepare-ubuntu-apt-mirror.sh /usr/local/lib/formal-ai-build/
RUN FORMAL_AI_APT_PRIVILEGE= bash /usr/local/lib/formal-ai-build/apt-install-with-retry.sh \
      --no-install-recommends libstdc++6 libatomic1 && \
    rm -rf /var/lib/apt/lists/* /usr/local/lib/formal-ai-build

# Ubuntu 24.04 supplies the shared libraries required by official Node 26.
# Its distro Node 18 cannot parse the /v regex used by agent dependencies.
COPY --from=node-runtime /usr/local/bin/node /usr/local/bin/node
# Node's CLI alone omits npm: derived Hive images also run npm ci as box.
# Keep the package manager and both launchers from the same pinned donor.
COPY --from=node-runtime /usr/local/lib/node_modules/npm/ /usr/local/lib/node_modules/npm/
RUN ln -sf ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm && \
    ln -sf ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx && \
    node --version && \
    node --eval "if (process.versions.node.split('.')[0] !== '26') process.exit(1); new RegExp('', 'v')"

USER box
WORKDIR /home/box
ENV PATH="/home/box/.formal-ai-runtime-tools/node_modules/.bin:${PATH}"
RUN mkdir -p /home/box/.formal-ai-runtime-tools
COPY --chown=box:box packages/container-runtime-tools/package.json /home/box/.formal-ai-runtime-tools/package.json
COPY --chmod=0755 scripts/install-container-runtime-tools.sh scripts/verify-container-runtime-tools.mjs \
    scripts/install-client-registry-recovery.mjs /usr/local/lib/formal-ai-runtime-tools/scripts/
COPY js/server/bounded-command.mjs js/server/structured-log.mjs js/server/json.mjs /usr/local/lib/formal-ai-runtime-tools/js/server/
COPY js/logging/diagnostic-metadata.mjs js/logging/typed-codec.mjs js/logging/text-lines.mjs \
    js/logging/json-numbers.mjs js/logging/lino-codec.mjs /usr/local/lib/formal-ai-runtime-tools/js/logging/
RUN /usr/local/lib/formal-ai-runtime-tools/scripts/install-container-runtime-tools.sh /home/box/.formal-ai-runtime-tools && \
    npm --version && \
    npx --version && \
    "$" --version && \
    agent --version && \
    start-agent --help >/dev/null

USER root
COPY --from=selected-binary /app/target/release/formal-ai /usr/local/bin/formal-ai
COPY scripts/verify-docker-runtime.sh /usr/local/bin/verify-formal-ai-dind
RUN chmod 0755 /usr/local/bin/formal-ai /usr/local/bin/verify-formal-ai-dind && \
    formal-ai --version

EXPOSE 8080
VOLUME ["/var/lib/docker", "/root/.formal-ai"]
SHELL ["/bin/bash", "-c"]
ENTRYPOINT ["/usr/local/bin/dind-entrypoint.sh"]
CMD ["formal-ai", "telegram", "--mode", "polling"]
