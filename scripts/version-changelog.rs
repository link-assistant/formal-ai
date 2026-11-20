//! Changelog collection used by the version-and-commit release command.
//!
//! Keep the real filesystem operation separate from registry and Git operations
//! so the focused native CI target can exercise it without a release or network.

use regex::Regex;
use std::fs;
use std::path::Path;

#[path = "changelog-files.rs"]
mod changelog_files;

fn strip_frontmatter(content: &str) -> String {
    let re = Regex::new(r"(?s)^---\s*\n.*?\n---\s*\n(.*)$").unwrap();
    if let Some(caps) = re.captures(content) {
        caps.get(1).unwrap().as_str().trim().to_string()
    } else {
        content.trim().to_string()
    }
}

pub fn collect_changelog_with_date(
    changelog_dir: &str,
    changelog_file: &str,
    version: &str,
    date_str: &str,
) -> std::io::Result<bool> {
    let files = changelog_files::read_fragments(Path::new(changelog_dir))?;

    let fragments: Vec<String> = files
        .iter()
        .map(|file| strip_frontmatter(&file.content))
        .filter(|c| !c.is_empty())
        .collect();

    if fragments.is_empty() {
        return Ok(false);
    }

    // No leading newline: `lines[..idx]` already ends with the blank line that
    // follows the insert marker, so opening with one produced a second blank
    // line that `issue_711_rebuild_changelog.mjs --check` rejects.
    let new_entry = format!(
        "## [{}] - {}\n\n{}\n",
        version,
        date_str,
        fragments.join("\n\n")
    );

    let mut content = match changelog_files::read_file(Path::new(changelog_file)) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let lines: Vec<&str> = content.lines().collect();
    let mut insert_index = None;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("## [") {
            insert_index = Some(i);
            break;
        }
    }

    if let Some(idx) = insert_index {
        let mut new_lines: Vec<String> = lines[..idx].iter().map(|s| s.to_string()).collect();
        new_lines.push(new_entry.clone());
        new_lines.extend(lines[idx..].iter().map(|s| s.to_string()));
        // `lines()` drops the trailing newline and `join` never restores it, so
        // every release used to strip the file's final newline.
        content = format!("{}\n", new_lines.join("\n"));
    } else {
        content.push_str(&new_entry);
    }

    fs::write(changelog_file, content).map_err(|error| {
        changelog_files::path_error("write changelog", Path::new(changelog_file), error)
    })?;
    changelog_files::remove_fragments(&files)?;

    println!("Collected {} changelog fragment(s)", files.len());
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_collection_refuses_io_errors_and_preserves_canonical_output() {
        let directory = std::env::temp_dir().join(format!(
            "version-changelog-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let fragments = directory.join("changelog.d");
        let changelog = directory.join("CHANGELOG.md");
        let collect = || {
            collect_changelog_with_date(
                fragments.to_str().unwrap(),
                changelog.to_str().unwrap(),
                "0.2.0",
                "2026-11-20",
            )
        };
        assert!(!collect().unwrap());
        fs::write(&fragments, "not a directory").unwrap();
        assert!(collect().unwrap_err().to_string().contains("changelog.d"));
        fs::remove_file(&fragments).unwrap();
        fs::create_dir(&fragments).unwrap();
        let good = fragments.join("a.md");
        let bad = fragments.join("b.md");
        fs::write(&good, "---\nbump: patch\n---\n\nA release note.\n").unwrap();
        fs::write(&bad, [0xff]).unwrap();
        let original = "# Changelog\n\n<!-- changelog-insert-here -->\n\n## [0.1.0] - 2026-01-01\n\nInitial release.\n";
        fs::write(&changelog, original).unwrap();
        assert!(collect().unwrap_err().to_string().contains("b.md"));
        assert_eq!(fs::read_to_string(&changelog).unwrap(), original);
        assert!(good.exists());
        assert_eq!(fs::read(&bad).unwrap(), [0xff]);
        fs::write(&bad, "Second note.\n").unwrap();
        fs::write(&changelog, [0xff]).unwrap();
        assert!(collect().unwrap_err().to_string().contains("CHANGELOG.md"));
        assert_eq!(fs::read(&changelog).unwrap(), [0xff]);
        assert!(good.exists() && bad.exists());
        fs::remove_file(&changelog).unwrap();
        fs::create_dir(&changelog).unwrap();
        assert!(collect().unwrap_err().to_string().contains("CHANGELOG.md"));
        assert!(good.exists() && bad.exists());
        fs::remove_dir(&changelog).unwrap();
        fs::write(&changelog, original).unwrap();
        fs::write(fragments.join("README.md"), "instructions").unwrap();
        assert!(collect().unwrap());
        assert_eq!(
            fs::read_to_string(&changelog).unwrap(),
            "# Changelog\n\n<!-- changelog-insert-here -->\n\n## [0.2.0] - 2026-11-20\n\nA release note.\n\nSecond note.\n\n## [0.1.0] - 2026-01-01\n\nInitial release.\n"
        );
        assert!(!good.exists() && !bad.exists());
        assert!(fragments.join("README.md").exists());
        let after = fs::read(&changelog).unwrap();
        assert!(!collect().unwrap());
        assert_eq!(fs::read(&changelog).unwrap(), after);
    }
}
