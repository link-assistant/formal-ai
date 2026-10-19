//! Source-backed immutable GitHub Action release identities.
use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct TagTarget {
    sha: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct TagReference {
    #[serde(rename = "ref")]
    reference: String,
    object: TagTarget,
}

#[derive(Deserialize)]
struct AnnotatedTag {
    sha: String,
    object: TagTarget,
}

#[derive(Deserialize)]
struct LatestRelease {
    tag_name: String,
}

pub fn github_release_tag(body: &str) -> Result<String, String> {
    serde_json::from_str::<LatestRelease>(body)
        .map(|release| release.tag_name)
        .map_err(|error| format!("invalid latest release JSON: {error}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRelease {
    pub tag: String,
    pub commit: String,
    pub source_url: String,
    pub tag_source_url: String,
}

pub fn action_release_url(action: &str) -> String {
    format!(
        "https://api.github.com/repos/{}/releases/latest",
        action_repository(action)
    )
}

fn tag_reference_url(action: &str, tag: &str) -> String {
    let mut encoded = String::new();
    for byte in tag.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    format!(
        "https://api.github.com/repos/{}/git/ref/tags/{encoded}",
        action_repository(action)
    )
}

pub fn valid_action_release(action: &str, latest: &str, release: &ActionRelease) -> bool {
    release.tag == latest
        && is_commit_sha(&release.commit)
        && release.source_url == action_release_url(action)
        && release.tag_source_url == tag_reference_url(action, latest)
}

/// Only the publisher's exact tag ref and reconstructed same-repository tag
/// object URLs are fetched. Annotated tags may nest; cycles/depth fail closed.
pub fn resolve_action_release(
    action: &str,
    tag: &str,
    mut read: impl FnMut(&str) -> Result<String, String>,
) -> Result<ActionRelease, String> {
    if tag.is_empty() || tag.len() > 256 {
        return Err("invalid release tag".to_string());
    }
    let repository = action_repository(action);
    let parts: Vec<_> = repository.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|part| {
            part.is_empty()
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-._".contains(&byte))
        })
    {
        return Err("invalid action repository".to_string());
    }
    let tag_source_url = tag_reference_url(action, tag);
    let mut url = tag_source_url.clone();
    let mut seen = std::collections::BTreeSet::new();
    let mut expected_object: Option<String> = None;
    for _ in 0..8 {
        let body = read(&url)?;
        if body.len() > 1024 * 1024 {
            return Err("tag response exceeds budget".to_string());
        }
        // Typed deserialization rejects malformed/trailing JSON and duplicate
        // identity fields. Unknown official metadata remains harmless; nested
        // decoys cannot substitute for required root or object fields.
        let object = if let Some(expected) = &expected_object {
            let record: AnnotatedTag = serde_json::from_str(&body)
                .map_err(|error| format!("invalid annotated tag JSON: {error}"))?;
            if &record.sha != expected {
                return Err("tag object identity differs".to_string());
            }
            record.object
        } else {
            let record: TagReference = serde_json::from_str(&body)
                .map_err(|error| format!("invalid tag ref JSON: {error}"))?;
            if record.reference != format!("refs/tags/{tag}") {
                return Err("tag ref identity differs".to_string());
            }
            record.object
        };
        let sha = object.sha;
        if !is_commit_sha(&sha) {
            return Err("invalid tag object SHA".to_string());
        }
        match object.kind.as_str() {
            "commit" => {
                return Ok(ActionRelease {
                    tag: tag.to_string(),
                    commit: sha,
                    source_url: action_release_url(action),
                    tag_source_url,
                });
            }
            "tag" if seen.insert(sha.clone()) => {
                url = format!("https://api.github.com/repos/{repository}/git/tags/{sha}");
                expected_object = Some(sha);
            }
            _ => return Err("unsupported or cyclic tag object".to_string()),
        }
    }
    Err("annotated tag depth exhausted".to_string())
}

/// Missing/unresolved evidence is distinct from observed stale dependencies.
pub fn comparison_exit_code(findings: &[Finding], unverifiable: &[String]) -> i32 {
    if unverifiable.is_empty() {
        exit_code(findings)
    } else {
        EXIT_UNVERIFIABLE
    }
}

#[cfg(test)]
#[path = "check-dependencies-action-release-tests.rs"]
mod tests;
