use std::fs;
use std::path::Path;

use formal_ai::environment_records;

#[test]
fn dockerfile_defines_only_supported_dind_telegram_runtime() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dockerfile =
        fs::read_to_string(root.join("Dockerfile")).expect("Dockerfile should be readable");

    assert_contains_all(
        "Dockerfile",
        &dockerfile,
        &[
            "FROM rust:1.98-slim AS builder",
            "FROM konard/box-dind:2.1.1",
            "LABEL org.opencontainers.image.source=\"https://github.com/link-assistant/formal-ai\"",
            "FORMAL_AI_IMAGE_VARIANT=dind",
            "FORMAL_AI_START_ISOLATION=docker",
            "FORMAL_AI_START_RUNNER=\"$ --isolated docker --auto-remove-docker-container --\"",
            "DIND_STORAGE_DRIVER=\"vfs\"",
            "apt-get install -y --no-install-recommends nodejs",
            "node --version",
            "bun install -g start-command",
            "\"$\" --version",
            "agent --version",
            "start-agent --help",
            "COPY scripts/verify-docker-runtime.sh /usr/local/bin/verify-formal-ai-dind",
            "ENTRYPOINT [\"/usr/local/bin/dind-entrypoint.sh\"]",
            "CMD [\"formal-ai\", \"telegram\", \"--mode\", \"polling\"]",
        ],
    );

    assert!(
        !dockerfile.contains("FROM debian:"),
        "the supported runtime image must be Box Docker-in-Docker, not bare Debian"
    );
    assert!(
        !dockerfile.contains("CMD [\"serve\""),
        "the Docker image should start the Telegram bot by default, not the HTTP server"
    );
}

#[test]
fn docker_microservice_seed_declares_dind_start_command_contract() {
    let record = environment_records()
        .into_iter()
        .find(|record| record.id == "docker_microservice")
        .expect("docker_microservice environment should be declared");

    assert!(
        record.label.contains("Docker-in-Docker"),
        "docker_microservice label should describe the only supported image variant: {record:?}"
    );
    assert!(
        record.runtime.contains("konard/box-dind:2.1.1"),
        "docker_microservice runtime should pin the Box DinD image: {record:?}"
    );
    assert!(
        record.memory_export_command.contains("formal-ai telegram"),
        "docker_microservice should document the Telegram bot command: {record:?}"
    );

    for expected in ["telegram_polling", "start_command", "docker_isolation"] {
        assert!(
            record.tools.iter().any(|tool| tool == expected),
            "docker_microservice tools should include `{expected}`: {record:?}",
        );
    }
}

#[test]
fn compose_file_runs_prebuilt_telegram_image_with_minimum_configuration() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let compose = fs::read_to_string(root.join("compose.yaml"))
        .expect("compose.yaml should document the prebuilt Telegram bot image startup");

    assert_contains_all(
        "compose.yaml",
        &compose,
        &[
            "telegram-bot:",
            "${FORMAL_AI_DOCKER_IMAGE:-ghcr.io/link-assistant/formal-ai:latest}",
            "privileged: true",
            "TELEGRAM_BOT_TOKEN: ${TELEGRAM_BOT_TOKEN:?Set TELEGRAM_BOT_TOKEN to your Telegram bot token}",
            "FORMAL_AI_TELEGRAM_ALLOWED_UPDATES",
            "formal-ai-telegram-docker:/var/lib/docker",
        ],
    );
}

#[test]
#[allow(clippy::literal_string_with_formatting_args)]
fn compose_file_offers_optional_openai_compatible_server_profile() {
    // Issue #438 (follow-up): the same compose file must also start the
    // OpenAI-compatible API server (agentic mode) on a server, under an opt-in
    // profile so `docker compose up` keeps starting only the Telegram bot. The
    // container name matches the one the desktop app starts/stops with one click.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let compose = fs::read_to_string(root.join("compose.yaml"))
        .expect("compose.yaml should document the optional OpenAI-compatible server");

    assert_contains_all(
        "compose.yaml",
        &compose,
        &[
            "server:",
            "container_name: formal-ai-server",
            "profiles: [\"server\", \"all\"]",
            "[\"formal-ai\", \"serve\", \"--host\", \"0.0.0.0\", \"--port\", \"${FORMAL_AI_SERVER_PORT:-8080}\"]",
            "127.0.0.1:${FORMAL_AI_SERVER_PORT:-8080}:${FORMAL_AI_SERVER_PORT:-8080}",
            // each DinD service uses its own inner-Docker volume.
            "formal-ai-server-docker:/var/lib/docker",
        ],
    );
}

/// `COPY . .` before `cargo build` puts every file in the tree into the build
/// layer's cache key, so editing one `.rs` rebuilds all ~500 dependency crates.
/// The image build measured 24 minutes on run 32470623196, its slowest layers
/// at 428s, 419s and 355s, and it gates the pipeline's finish alone (#1029).
///
/// Dependencies therefore build from the manifests alone, before the sources
/// arrive. Measured locally: the manifest-only layer builds in 1m48s, and the
/// source layer then compiles `formal-ai` by itself, reusing every dependency.
#[test]
fn the_image_builds_dependencies_before_it_copies_the_sources() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dockerfile =
        fs::read_to_string(format!("{manifest_dir}/Dockerfile")).expect("Dockerfile is readable");

    let manifest_copy = dockerfile
        .find("COPY Cargo.toml Cargo.lock build.rs ./")
        .expect("the image must copy its manifests before its sources");
    let source_copy = dockerfile
        .find("\nCOPY . .")
        .expect("the image must still copy the sources");
    assert!(
        manifest_copy < source_copy,
        "the manifest copy must precede the source copy, or the dependency \
         layer is keyed on the whole tree again"
    );

    let dependency_build = dockerfile
        .find("cargo build --release --locked --lib --bins")
        .expect("the image must build dependencies from the manifests alone");
    assert!(
        dependency_build < source_copy,
        "dependencies must build before the sources arrive, or their layer is \
         invalidated by every source edit"
    );
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
