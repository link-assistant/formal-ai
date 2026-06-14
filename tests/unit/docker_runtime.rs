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
            "FROM rust:1.82-slim AS builder",
            "FROM konard/box-dind:2.1.1",
            "LABEL org.opencontainers.image.source=\"https://github.com/link-assistant/formal-ai\"",
            "FORMAL_AI_IMAGE_VARIANT=dind",
            "FORMAL_AI_START_ISOLATION=docker",
            "FORMAL_AI_START_RUNNER=\"$ --isolated docker --auto-remove-docker-container --\"",
            "DIND_STORAGE_DRIVER=\"vfs\"",
            "bun install -g start-command",
            "\"$\" --version",
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
            "formal-ai-docker:/var/lib/docker",
        ],
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
