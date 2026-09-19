//! Idempotent detection and repair of an already prepared release.

use std::fs;
use std::path::Path;
use std::process::Command;

use super::git;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PreparedRelease {
    Tagged { version: String },
    NeedsTag { version: String },
}

fn git_check(repo: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// True when `changelog_dir` contains a release fragment rather than only its
/// instructions file. A newly merged fragment always wins over recovery: it
/// represents a new desired release and must not be silently consumed by an
/// older prepared version.
fn has_changelog_fragments(changelog_dir: &Path) -> Result<bool, String> {
    if !changelog_dir.exists() {
        return Ok(false);
    }

    let entries = fs::read_dir(changelog_dir)
        .map_err(|e| format!("Failed to read {}: {}", changelog_dir.display(), e))?;
    for entry in entries {
        let path = entry
            .map_err(|e| format!("Failed to read {} entry: {}", changelog_dir.display(), e))?
            .path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md")
            && path.file_name().and_then(|name| name.to_str()) != Some("README.md")
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Recognize a release that an earlier attempt already prepared on the
/// synchronized branch. Recovery is intentionally evidence-based: either the
/// current version has a tag reachable from HEAD, or HEAD is the exact release
/// commit whose tag was not pushed yet. An ordinary untagged commit is never
/// treated as a prepared release.
pub(super) fn prepared_release(
    repo: &Path,
    version: &str,
    tag_prefix: &str,
    changelog_dir: &Path,
) -> Result<Option<PreparedRelease>, String> {
    if has_changelog_fragments(changelog_dir)? {
        return Ok(None);
    }

    let tag = format!("{}{}", tag_prefix, version);
    let tag_ref = format!("refs/tags/{}^{{commit}}", tag);
    if let Ok(tag_commit) = git(repo, &["rev-parse", "--verify", &tag_ref]) {
        if !git_check(repo, &["merge-base", "--is-ancestor", &tag_commit, "HEAD"]) {
            return Err(format!(
                "Prepared release tag {} exists at {}, but it is not an ancestor of HEAD",
                tag, tag_commit
            ));
        }
        return Ok(Some(PreparedRelease::Tagged {
            version: version.to_string(),
        }));
    }

    let subject = git(repo, &["log", "-1", "--format=%s"])?;
    let expected = format!("chore: release {}{}", tag_prefix, version);
    let labelled_prefix = format!("{} (", expected);
    let is_release_subject = subject == expected
        || subject
            .strip_prefix(&labelled_prefix)
            .map(|label| label.ends_with(')') && label.len() > 1)
            .unwrap_or(false);
    if is_release_subject {
        Ok(Some(PreparedRelease::NeedsTag {
            version: version.to_string(),
        }))
    } else {
        Ok(None)
    }
}

pub(super) fn ensure_prepared_release_tag(
    repo: &Path,
    prepared: &PreparedRelease,
    tag_prefix: &str,
) -> Result<(), String> {
    let PreparedRelease::NeedsTag { version } = prepared else {
        return Ok(());
    };
    let tag = format!("{}{}", tag_prefix, version);
    let message = format!("Release {}", tag);
    git(repo, &["tag", "-a", &tag, "-m", &message])?;
    git(repo, &["push", "origin", &format!("refs/tags/{}", tag)])?;
    println!("Created and pushed missing prepared release tag {}", tag);
    Ok(())
}
