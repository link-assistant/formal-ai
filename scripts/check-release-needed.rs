#!/usr/bin/env rust-script
//! Check if a release is needed based on changelog fragments and version state
//!
//! This script checks:
//! 1. If there are changelog fragments to process
//! 2. If the current version has already been published to crates.io
//! 3. If the matching GitHub release and configured Docker Hub image tag exist
//!
//! IMPORTANT: This script checks external release artifacts, NOT git tags.
//! This is critical because:
//! - Git tags can exist without the package being published
//! - GitHub releases create tags but do not publish to crates.io or Docker Hub
//! - A crates.io publish can succeed while later Docker/GitHub release steps fail
//!
//! Supports both single-language and multi-language repository structures:
//! - Single-language: Cargo.toml in repository root
//! - Multi-language: Cargo.toml in rust/ subfolder
//!
//! Usage: rust-script scripts/check-release-needed.rs [--rust-root <path>]
//!
//! Environment variables:
//!   - HAS_FRAGMENTS: 'true' if changelog fragments exist (from get-bump-type.rs)
//!   - DOCKERHUB_IMAGE: Optional Docker Hub image name to verify (namespace/repository)
//!   - GITHUB_REPOSITORY: GitHub repository to verify (owner/repository)
//!
//! Outputs (written to GITHUB_OUTPUT):
//!   - should_release: 'true' if a release should be created
//!   - skip_bump: 'true' if version bump should be skipped while missing artifacts are recreated
//!   - crate_published: 'true' if the current version already exists on crates.io
//!   - dockerhub_required: 'true' if Docker Hub publishing is configured and a Dockerfile exists
//!   - dockerhub_published: 'true' if the configured Docker Hub tag exists
//!   - github_release_published: 'true' if the matching GitHub release exists
//!   - max_published_version: the highest non-yanked version on crates.io (for downstream use)
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! toml_edit = "0.25.15"
//! regex = "1"
//! ureq = "3.5.2"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! semver = "1.0.28"
//! ```

use std::env;
use std::path::Path;
use std::process::exit;

#[path = "github-output.rs"]
mod github_output;
use github_output::set_output;

#[path = "registry.rs"]
mod registry;
#[path = "rust-paths.rs"]
mod rust_paths;

fn get_arg(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let flag = format!("--{}", name);

    if let Some(idx) = args.iter().position(|a| a == &flag) {
        return args.get(idx + 1).cloned();
    }

    let env_name = name.to_uppercase().replace('-', "_");
    env::var(&env_name).ok().filter(|s| !s.is_empty())
}

fn check_version_on_crates_io(crate_name: &str, version: &str) -> bool {
    registry::or_exit(
        "crates.io version",
        registry::version_exists(crate_name, version),
    )
}

fn split_docker_image(image: &str) -> Option<(&str, &str)> {
    let mut parts = image.split('/');
    let namespace = parts.next()?;
    let repository = parts.next()?;

    if parts.next().is_some() || namespace.is_empty() || repository.is_empty() {
        None
    } else {
        Some((namespace, repository))
    }
}

fn check_docker_hub_tag(image: &str, version: &str) -> bool {
    let Some((namespace, repository)) = split_docker_image(image) else {
        registry::or_exit::<()>(
            "Docker Hub image",
            Err(format!(
                "Invalid image '{image}'; expected namespace/repository"
            )),
        );
        unreachable!();
    };
    let url =
        format!("https://hub.docker.com/v2/repositories/{namespace}/{repository}/tags/{version}");
    registry::or_exit(
        "Docker Hub tag",
        registry::artifact_exists(&url, &[], "name", version),
    )
}

fn check_github_release(repository: &str, tag_prefix: &str, version: &str) -> bool {
    let tag = format!("{tag_prefix}{version}");
    let url = format!("https://api.github.com/repos/{repository}/releases/tags/{tag}");
    let authorization = env::var("GITHUB_TOKEN")
        .ok()
        .filter(|token| !token.is_empty())
        .map(|token| format!("Bearer {token}"));
    let mut headers = vec![("Accept", "application/vnd.github+json")];
    if let Some(ref authorization) = authorization {
        headers.push(("Authorization", authorization));
    }
    registry::or_exit(
        "GitHub release",
        registry::artifact_exists(&url, &headers, "tag_name", &tag),
    )
}

fn docker_hub_image_to_check() -> Option<String> {
    get_arg("dockerhub-image")
        .or_else(|| get_arg("docker-hub-image"))
        .or_else(|| get_arg("dockerhub_image"))
        .filter(|image| Path::new("Dockerfile").exists() && !image.trim().is_empty())
}

fn release_is_complete(
    crate_published: bool,
    dockerhub_required: bool,
    dockerhub_published: bool,
    github_release_published: bool,
) -> bool {
    crate_published && (!dockerhub_required || dockerhub_published) && github_release_published
}

fn get_max_published_version(crate_name: &str) -> Option<String> {
    registry::or_exit(
        "crates.io published versions",
        registry::max_published_name(crate_name),
    )
}

// Bound the observer independently of its own API and source-check deadlines.
fn bounded_recovery_observer(
    command: &mut std::process::Command,
) -> Option<(std::process::ExitStatus, Vec<u8>)> {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let stdout = child.stdout.take()?;
    let stderr = child.stderr.take()?;
    let (sender, receiver) = std::sync::mpsc::channel::<(bool, Option<Vec<u8>>)>();
    let error_sender = sender.clone();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout
            .take(65537)
            .read_to_end(&mut bytes)
            .ok()
            .map(|_| bytes);
        let _ = sender.send((true, result));
    });
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stderr
            .take(16385)
            .read_to_end(&mut bytes)
            .ok()
            .map(|_| bytes);
        let _ = error_sender.send((false, result));
    });
    let deadline = Instant::now() + Duration::from_secs(150);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(25)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };
    let status = status?;
    let mut output = None;
    let mut errors = None;
    for _ in 0..2 {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let (is_output, bytes) = receiver.recv_timeout(remaining).ok()?;
        if is_output {
            output = bytes;
        } else {
            errors = bytes;
        }
    }
    let output = output?;
    let errors = errors?;
    if output.len() > 65536 || errors.len() > 16384 {
        return None;
    }
    Some((status, output))
}

// Explicit source-qualified recovery mode never promotes metadata presence to delivery.
// Fragment/version handling stays intact; ordinary no-fragment checks can propose recovery.
fn qualified_recovery_requested(current_version: &str) -> bool {
    let automatic_root = env::var("RELEASE_RECOVERY_PROTOCOL_ROOT").ok();
    let protocol_directory = get_arg("all-target-recovery-protocol");
    if automatic_root.is_none() && protocol_directory.is_none() {
        return false;
    }
    set_output("should_release", "false");
    set_output("skip_bump", "true");
    set_output("release_complete", "false");
    set_output("recovery_state", "UnavailableFullTargetEvidence");
    set_output("recovery_missing_assets", "[]");
    let observed = if let Some(root) = automatic_root {
        if root != "_protocol" {
            eprintln!("Unexpected immutable recovery root; deferring");
            return true;
        }
        let helper = Path::new(&root).join("scripts/release-recovery-decision.mjs");
        bounded_recovery_observer(
            std::process::Command::new("node")
                .arg(helper)
                .args(["automatic", current_version]),
        )
    } else {
        let required = [
            "all-target-recovery-root",
            "all-target-recovery-protocol-commit",
            "all-target-recovery-protocol-sha256",
            "all-target-recovery-source-commit",
        ];
        let values: Option<Vec<String>> = required.iter().map(|name| get_arg(name)).collect();
        let Some(values) = values else {
            eprintln!(
                "Source-qualified recovery requires complete source/protocol identity; deferring"
            );
            return true;
        };
        let helper = Path::new(&values[0]).join("scripts/release-recovery-decision.mjs");
        let observed =
            bounded_recovery_observer(std::process::Command::new("node").arg(helper).args([
                "observe",
                values[0].as_str(),
                protocol_directory.as_ref().unwrap().as_str(),
                values[1].as_str(),
                values[2].as_str(),
                values[3].as_str(),
                current_version,
            ]));
        observed
    };
    let Some((status, output)) = observed else {
        eprintln!("Source-qualified recovery observer unavailable; deferring");
        return true;
    };
    if !status.success() || output.len() > 65536 {
        eprintln!(
            "Source-qualified recovery observer failed or exceeded decision bounds; deferring"
        );
        return true;
    }
    let Ok(decision) = serde_json::from_slice::<serde_json::Value>(&output) else {
        eprintln!("Malformed source-qualified recovery decision; deferring");
        return true;
    };
    if decision["schema"].as_str() != Some("release-recovery-decision/v1")
        || decision["version"].as_str() != Some(current_version)
        || decision["publicationComplete"].as_bool() != Some(false)
        || decision["shouldRelease"].as_bool() != Some(false)
        || decision["automaticRepairAuthorized"].as_bool() != Some(false)
        || decision["state"].as_str() != Some("UnavailableFullTargetEvidence")
    {
        eprintln!("Unqualified recovery authorization refused; deferring");
        return true;
    }
    if let Some(missing) = decision["sameVersionRepairProposal"].as_array() {
        if missing.len() <= 512 && missing.iter().all(|name| name.as_str().is_some()) {
            set_output(
                "recovery_missing_assets",
                &serde_json::to_string(missing).unwrap_or_default(),
            );
        }
    }
    println!(
        "All-target recovery remains deferred; selective same-version proposals are not writer authority"
    );
    true
}

fn main() {
    let rust_root = match rust_paths::get_rust_root(None, true) {
        Ok(root) => root,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };
    let cargo_toml = rust_paths::get_cargo_toml_path(&rust_root);
    let package_manifest = match rust_paths::get_package_manifest_path(&cargo_toml) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    let has_fragments = env::var("HAS_FRAGMENTS")
        .map(|v| v == "true")
        .unwrap_or(false);

    let package_info = match rust_paths::read_package_info(&package_manifest) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };
    let crate_name = package_info.name;
    let current_version = package_info.version;

    if !has_fragments && qualified_recovery_requested(&current_version) {
        return;
    }

    let max_published = get_max_published_version(&crate_name);
    if let Some(ref max_ver) = max_published {
        println!("Max published version on crates.io: {}", max_ver);
        set_output("max_published_version", max_ver);
    } else {
        println!("No versions published on crates.io yet (or crate not found)");
        set_output("max_published_version", "");
    }

    if !has_fragments {
        let crate_published = check_version_on_crates_io(&crate_name, &current_version);
        let tag_prefix = get_arg("tag-prefix").unwrap_or_else(|| "v".to_string());
        let dockerhub_image = docker_hub_image_to_check();
        let dockerhub_required = dockerhub_image.is_some();
        let dockerhub_published = dockerhub_image
            .as_deref()
            .map(|image| {
                check_docker_hub_tag(image, &current_version)
                    && check_docker_hub_tag(image, "latest")
            })
            .unwrap_or(false);
        let github_release_published = get_arg("repository")
            .or_else(|| env::var("GITHUB_REPOSITORY").ok().filter(|s| !s.is_empty()))
            .map(|repository| check_github_release(&repository, &tag_prefix, &current_version))
            .unwrap_or_else(|| {
                registry::or_exit::<bool>(
                    "GitHub release",
                    Err("GITHUB_REPOSITORY or --repository is required".into()),
                )
            });

        set_output(
            "crate_published",
            if crate_published { "true" } else { "false" },
        );
        set_output(
            "dockerhub_required",
            if dockerhub_required { "true" } else { "false" },
        );
        set_output(
            "dockerhub_published",
            if dockerhub_published { "true" } else { "false" },
        );
        set_output(
            "github_release_published",
            if github_release_published {
                "true"
            } else {
                "false"
            },
        );

        println!(
            "Crate: {}, Version: {}, Published on crates.io: {}",
            crate_name, current_version, crate_published
        );
        if let Some(image) = dockerhub_image {
            println!(
                "Docker image: {}, version/latest tags published on Docker Hub: {}",
                image, dockerhub_published
            );
        } else {
            println!(
                "Docker Hub artifact check skipped: DOCKERHUB_IMAGE or Dockerfile is not configured"
            );
        }
        println!(
            "GitHub release {}{} published: {}",
            tag_prefix, current_version, github_release_published
        );

        if release_is_complete(
            crate_published,
            dockerhub_required,
            dockerhub_published,
            github_release_published,
        ) {
            println!(
                "No changelog fragments and v{} is fully published",
                current_version
            );
            set_output("should_release", "false");
        } else {
            println!(
                "No changelog fragments but v{} is missing at least one release artifact",
                current_version
            );
            set_output("should_release", "true");
            set_output("skip_bump", "true");
        }
    } else {
        println!("Found changelog fragments, proceeding with release");
        set_output("should_release", "true");
        set_output("skip_bump", "false");
    }
}
