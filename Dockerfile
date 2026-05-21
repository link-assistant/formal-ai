FROM rust:1.82-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release --locked

FROM konard/box-dind:2.1.1

ENV HOME=/home/box \
    FORMAL_AI_IMAGE_VARIANT=dind \
    FORMAL_AI_START_ISOLATION=docker \
    FORMAL_AI_START_RUNNER="$ --isolated docker --auto-remove-docker-container --" \
    DIND_STORAGE_DRIVER="vfs" \
    BUN_INSTALL=/home/box/.bun
ENV PATH="${BUN_INSTALL}/bin:${PATH}"

USER box
WORKDIR /home/box
RUN bun install -g start-command && "$" --version

USER root
COPY --from=builder /app/target/release/formal-ai /usr/local/bin/formal-ai
COPY scripts/verify-docker-runtime.sh /usr/local/bin/verify-formal-ai-dind
RUN chmod 0755 /usr/local/bin/formal-ai /usr/local/bin/verify-formal-ai-dind && \
    formal-ai --version

EXPOSE 8080
VOLUME ["/var/lib/docker"]
SHELL ["/bin/bash", "-c"]
ENTRYPOINT ["/usr/local/bin/dind-entrypoint.sh"]
CMD ["formal-ai", "telegram", "--mode", "polling"]
