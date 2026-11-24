//! Changelog collection used by the version-and-commit release command.
//!
//! Keep the real filesystem operation separate from registry and Git operations
//! so the focused native CI target can exercise it without a release or network.

use regex::Regex;
use std::fs;
use std::path::Path;

#[path = "changelog-files.rs"]
mod changelog_files;

#[allow(dead_code)]
#[path = "release-changelog.rs"]
mod release_changelog;

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

    // Keep one blank line between the inserted release and the previous one.
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
    if let Some(offset) = release_changelog::first_heading_offset(&content) {
        content.insert_str(offset, &format!("{new_entry}\n"));
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
    fn collection_ignores_fenced_examples_and_preserves_existing_bytes() {
        let directory = std::env::temp_dir().join(format!(
            "version-changelog-fenced-{}-{}",
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
        fs::create_dir(&fragments).unwrap();
        let fragment = fragments.join("new.md");
        let changelog = directory.join("CHANGELOG.md");
        for newline in ["\n", "\r\n"] {
            for delimiter in ["```", "~~~", "   ````"] {
                let prefix = format!(
                    "# Changelog{newline}{newline}{delimiter}md{newline}## [9.9.9]{newline}{delimiter}{newline}{newline}"
                );
                let previous = format!("## [1.0.0]{newline}  Preserve these bytes.{newline}");
                fs::write(&changelog, format!("{prefix}{previous}")).unwrap();
                fs::write(&fragment, "### Fixed\n- New notes").unwrap();
                assert!(
                    collect_changelog_with_date(
                        fragments.to_str().unwrap(),
                        changelog.to_str().unwrap(),
                        "1.1.0",
                        "2026-11-22",
                    )
                    .unwrap()
                );
                let content = fs::read_to_string(&changelog).unwrap();
                assert!(content.starts_with(&prefix), "{content:?}");
                assert!(content.ends_with(&previous), "{content:?}");
                assert_eq!(
                    release_changelog::section(&content, "1.1.0").unwrap(),
                    "### Fixed\n- New notes"
                );
                assert!(!fragment.exists());
            }
        }
    }

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
