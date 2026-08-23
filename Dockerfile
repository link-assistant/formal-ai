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
# `ubuntu-latest` builds it and `konard/box-dind:2.1.1` runs it.
ARG BINARY_SOURCE=compile

FROM rust:1.96-slim AS builder

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
COPY Cargo.toml Cargo.lock build.rs ./
RUN mkdir -p src tests/unit tests/integration && \
    echo 'fn main() {}' > src/main.rs && \
    echo '' > src/lib.rs && \
    echo '' > tests/unit/mod.rs && \
    echo '' > tests/integration/mod.rs && \
    cargo build --release --locked --lib --bins && \
    rm -rf src tests

# Only this layer is invalidated by a source edit.
COPY . .
RUN cargo build --release --locked --bins

# `builder` under the name `BINARY_SOURCE` selects, so the historical stage name
# stays what it has always been -- `docker_runtime` pins it, and renaming a
# stage to satisfy a build argument would be the tail wagging the dog.
FROM builder AS compile-binary

# The prebuilt path: no toolchain, no compilation, just the artifact the
# pipeline already produced and tested.
FROM scratch AS prebuilt-binary
COPY target/release/formal-ai /app/target/release/formal-ai

# Resolves to whichever stage `BINARY_SOURCE` names.
FROM ${BINARY_SOURCE}-binary AS selected-binary

FROM konard/box-dind:2.1.1

LABEL org.opencontainers.image.source="https://github.com/link-assistant/formal-ai"

ENV HOME=/home/box \
    FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino \
    FORMAL_AI_IMAGE_VARIANT=dind \
    FORMAL_AI_START_ISOLATION=docker \
    FORMAL_AI_START_RUNNER="$ --isolated docker --auto-remove-docker-container --" \
    DIND_STORAGE_DRIVER="vfs" \
    BUN_INSTALL=/home/box/.bun
ENV PATH="${BUN_INSTALL}/bin:${PATH}"

RUN apt-get update && \
    apt-get install -y --no-install-recommends nodejs && \
    rm -rf /var/lib/apt/lists/* && \
    node --version

USER box
WORKDIR /home/box
RUN bun install -g start-command @link-assistant/agent agent-commander && \
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
