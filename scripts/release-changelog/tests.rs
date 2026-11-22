use super::*;

#[test]
fn fenced_examples_cannot_select_or_terminate_release_notes() {
    for marker in ["```", "~~~", "````", "   ~~~"] {
        let text = format!(
            "# Changelog\n\n{marker}markdown\n## [1.2.3]\nFake notes.\n{marker}\n\n## [1.2.3] - 2026-11-22\n\nActual notes.\n{marker}markdown\n## [0.0.0]\nExample heading.\n{marker}\nAfter example.\n\n## [1.2.2]\nOld notes.\n"
        );
        let notes = section(&text, "1.2.3").unwrap();
        assert!(notes.starts_with("Actual notes."), "{notes}");
        assert!(notes.contains("## [0.0.0]"));
        assert!(notes.ends_with("After example."));
        assert!(!notes.contains("Fake notes.") && !notes.contains("Old notes."));
    }
}

#[test]
fn fence_delimiter_lengths_and_exact_release_names_are_preserved() {
    let text = "## [1.2.3]suffix\nWrong identity.\n\n## [1.2.3]\nNotes.\n````text\n```\n## [0.0.0]\n~~~~\n```` extra\nStill an example.\n````\nFinal paragraph.\n## [1.2.2]\nOlder.\n";
    let notes = section(text, "1.2.3").unwrap();
    assert!(notes.starts_with("Notes."));
    assert!(notes.contains("## [0.0.0]"));
    assert!(notes.ends_with("Final paragraph."));
    assert!(!notes.contains("Wrong identity.") && !notes.contains("Older."));
    assert_eq!(
        section("## [1.2.3]\r\n\r\n## [1.2.2]\r\n", "1.2.3"),
        Some("Release v1.2.3".to_owned())
    );
    assert_eq!(section("```\n## [1.2.3]\nNever closes.\n", "1.2.3"), None);
    assert_eq!(section("    ## [1.2.3]\nIndented code.\n", "1.2.3"), None);
}

#[test]
fn unicode_whitespace_does_not_close_a_commonmark_fence() {
    let text = "## [1.2.3]\nNotes.\n```md\n```\u{a0}\n## [0.0.0]\nStill an example.\n```\nFinal paragraph.\n## [1.2.2]\nOlder.\n";
    let notes = section(text, "1.2.3").unwrap();
    assert!(notes.contains("## [0.0.0]"));
    assert!(notes.ends_with("Final paragraph."));
    assert!(!notes.contains("Older."));
}

#[test]
fn physical_release_sources_distinguish_missing_from_unreadable_observations() {
    let root = std::env::temp_dir().join(format!(
        "release-notes-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(root.clone());
    let main = root.join("CHANGELOG.md");
    let directory = root.join("docs/changelog");
    assert_eq!(for_version(&root, "1.2.3").unwrap(), "Release v1.2.3");
    fs::write(&main, [0xff]).unwrap();
    assert!(
        for_version(&root, "1.2.3")
            .unwrap_err()
            .to_string()
            .contains("CHANGELOG.md")
    );
    fs::remove_file(&main).unwrap();
    fs::create_dir(&main).unwrap();
    assert!(for_version(&root, "1.2.3").is_err());
    fs::remove_dir(&main).unwrap();
    fs::create_dir(root.join("docs")).unwrap();
    fs::write(&directory, "not a directory").unwrap();
    assert!(
        for_version(&root, "1.2.3")
            .unwrap_err()
            .to_string()
            .contains("docs/changelog")
    );
    fs::write(&main, "## [1.2.3]\nCurrent notes.\n").unwrap();
    assert_eq!(for_version(&root, "1.2.3").unwrap(), "Current notes.");
    fs::remove_file(&main).unwrap();
    fs::remove_file(&directory).unwrap();
    fs::create_dir(&directory).unwrap();
    let old = directory.join("releases-from-0.1.0.md");
    let new = directory.join("releases-from-1.0.0.md");
    fs::write(&old, "## [1.2.3]\nOlder archive copy.\n").unwrap();
    fs::write(&new, "## [1.2.3]\nNewest archive copy.\n").unwrap();
    assert_eq!(for_version(&root, "1.2.3").unwrap(), "Newest archive copy.");
    fs::write(&new, [0xff]).unwrap();
    assert!(
        for_version(&root, "1.2.3")
            .unwrap_err()
            .to_string()
            .contains("releases-from-1.0.0.md")
    );
    fs::remove_file(&new).unwrap();
    fs::create_dir(&new).unwrap();
    assert!(for_version(&root, "1.2.3").is_err());
}
