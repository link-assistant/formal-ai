//! Physical manifest regressions for release identity and source-preserving writes.
use super::rust_paths::{
    get_package_manifest_path, package_info_from_str, read_package_info, read_package_name,
    update_package_version,
};
use std::fs;
use std::path::PathBuf;

struct ManifestDirectory(PathBuf);

impl Drop for ManifestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn manifest(content: &str) -> (ManifestDirectory, PathBuf) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("cargo-package-{}-{nanos}-{id}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let directory = ManifestDirectory(root);
    let path = directory.0.join("Cargo.toml");
    fs::write(&path, content).unwrap();
    (directory, path)
}

#[test]
fn literal_package_identity_does_not_follow_dependency_tables() {
    let content = "[dependencies.decoy]\nname = \"dependency-name\"\nversion = \"9.9.9\"\n\
                   [package] # actual release identity\nname = 'real-crate'\nversion = '1.2.3'\n";
    // The original whole-file regex demonstrably selected the dependency.
    let old = regex::Regex::new(r#"(?m)^version\s*=\s*"([^"]+)""#).unwrap();
    assert_eq!(&old.captures(content).unwrap()[1], "9.9.9");
    let (_directory, path) = manifest(content);
    let actual = read_package_info(&path).unwrap();
    assert_eq!(actual.name, "real-crate");
    assert_eq!(actual.version, "1.2.3");
    assert_eq!(get_package_manifest_path(&path).unwrap(), path);
}

#[test]
fn inherited_or_missing_identity_never_falls_back_to_dependencies() {
    for identity in [
        "name = 'actual'\nversion.workspace = true\n",
        "name = 'actual'\nversion = { workspace = true }\n",
        "name = 'actual'\n",
        "version = '1.2.3'\n",
    ] {
        let content = format!(
            "[dependencies.decoy]\nname = 'decoy'\nversion = '9.9.9'\n[package]\n{identity}"
        );
        let (_directory, path) = manifest(&content);
        let error = read_package_info(&path).unwrap_err();
        assert!(error.contains("[package]."), "{error}");
        assert!(update_package_version(&path, "2.0.0").is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), content);
    }
}

#[test]
fn unrelated_inheritance_does_not_hide_a_literal_package_version() {
    let info = package_info_from_str(
        "[dependencies.decoy]\nversion.workspace = true\n\
         [package]\nname = 'actual'\nversion = '1.2.3'\n\
         [package.metadata]\nversion = { workspace = true }\n",
    )
    .unwrap();
    assert_eq!(info.version, "1.2.3");
}

#[test]
fn version_write_changes_only_the_parsed_token_in_a_crlf_manifest() {
    let content = "# Unicode café\r\n[dependencies.decoy]\r\nversion = \"9.9.9\"\r\n\
                   [ 'package' ] # release\r\nname = 'actual'\r\n  version = '1.2.3' # keep\r\n\
                   [package.metadata]\r\nnote = '''version = \"5.5.5\"'''";
    let (_directory, path) = manifest(content);
    update_package_version(&path, "2.0.0").unwrap();
    let expected = content.replacen("'1.2.3'", "\"2.0.0\"", 1);
    assert_eq!(fs::read_to_string(&path).unwrap(), expected);
    assert_eq!(read_package_info(&path).unwrap().version, "2.0.0");
}

#[test]
fn missing_or_malformed_package_tables_are_rejected_before_writes() {
    for content in [
        "[dependencies.decoy]\nname = 'decoy'\nversion = '9.9.9'\n",
        "[package]\nname = 'actual'\nversion = '1.2.3'\nversion = '2.0.0'\n",
        "[package]\nname = 'actual'\nversion = true\n",
        "[package]\nname = 'actual'\nversion = ''\n",
    ] {
        let (_directory, path) = manifest(content);
        assert!(read_package_info(&path).is_err());
        assert!(update_package_version(&path, "2.0.0").is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), content);
    }
}

#[test]
fn an_empty_new_version_preserves_the_original_file() {
    let content = "[package]\nname = 'actual'\nversion = '1.2.3'\n";
    let (_directory, path) = manifest(content);
    assert!(update_package_version(&path, "").is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), content);
}

#[test]
fn workspace_selection_uses_actual_members_and_package_publish_configuration() {
    let content = "[workspace.metadata]\nmembers = ['decoy']\n[workspace]\n\
                   members = [\n  'private', # not published\n  'no-registries',\n\
                     'public,crate', # a comma inside a quoted member\n]\n";
    let (_directory, path) = manifest(content);
    for (member, publish) in [("private", "false # private"), ("no-registries", "[]")] {
        let directory = path.parent().unwrap().join(member);
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("Cargo.toml"), format!(
            "[package] # parsed header\nname = '{member}'\nversion = '0.1.0'\npublish = {publish}\n"
        )).unwrap();
    }
    let public = path.parent().unwrap().join("public,crate");
    fs::create_dir(&public).unwrap();
    fs::write(public.join("Cargo.toml"),
        "[package]\nname = 'public-crate'\nversion = '1.2.3'\n[package.metadata]\npublish = false\n"
    ).unwrap();
    let selected = get_package_manifest_path(&path).unwrap();
    assert_eq!(selected, public.join("Cargo.toml"));
    assert_eq!(read_package_info(&selected).unwrap().name, "public-crate");
}

#[test]
fn a_package_header_inside_a_multiline_string_is_not_a_package() {
    let content = "description = '''\n[package]\nname = \"decoy\"\nversion = \"9.9.9\"\n'''\n";
    let (_directory, path) = manifest(content);
    assert!(get_package_manifest_path(&path).is_err());
    assert!(read_package_info(&path).is_err());
}

#[test]
fn a_release_badge_can_read_a_name_without_a_literal_version() {
    for version in ["version.workspace = true\n", ""] {
        let content =
            format!("[dependencies.decoy]\nname = 'decoy'\n[package]\nname = 'actual'\n{version}");
        let (_directory, path) = manifest(&content);
        assert_eq!(read_package_name(&path).unwrap(), "actual");
        assert!(read_package_info(&path).is_err());
    }
}

#[test]
fn unreadable_workspace_members_do_not_redirect_release_identity() {
    for invalid_utf8 in [true, false] {
        let (_directory, path) = manifest("[workspace]\nmembers = ['broken', 'public']\n");
        let root = path.parent().unwrap();
        fs::create_dir(root.join("broken")).unwrap();
        let broken = root.join("broken/Cargo.toml");
        if invalid_utf8 {
            fs::write(&broken, [0xff]).unwrap();
        } else {
            fs::create_dir(&broken).unwrap();
        }
        fs::create_dir(root.join("public")).unwrap();
        let public = root.join("public/Cargo.toml");
        let content = "[package]\nname = 'public'\nversion = '1.2.3'\n";
        fs::write(&public, content).unwrap();

        let error = get_package_manifest_path(&path).unwrap_err();
        assert!(error.contains("Failed to read"), "{error}");
        assert!(error.contains(&broken.display().to_string()), "{error}");
        assert_eq!(fs::read_to_string(&public).unwrap(), content);
        if invalid_utf8 {
            assert_eq!(fs::read(&broken).unwrap(), [0xff]);
        } else {
            assert!(broken.is_dir());
        }
    }
}

#[test]
fn an_absent_workspace_member_preserves_existing_selection_of_the_next_member() {
    let (_directory, path) = manifest("[workspace]\nmembers = ['absent', 'public']\n");
    let root = path.parent().unwrap();
    fs::create_dir(root.join("public")).unwrap();
    let public = root.join("public/Cargo.toml");
    fs::write(&public, "[package]\nname = 'public'\nversion = '1.2.3'\n").unwrap();
    assert_eq!(get_package_manifest_path(&path).unwrap(), public);
    assert!(!root.join("absent").exists());
}
