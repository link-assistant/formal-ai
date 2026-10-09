
#[test]
fn external_registration_proofs_preserve_the_original_physical_fixtures() {
    use sha2::{Digest, Sha256};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (source, fixture) in [
        ("src/agentic_coding/function_expectation.rs", "tests/fixtures/test-target-ownership.rs"),
        ("src/agentic_coding/driver.rs", "tests/fixtures/algorithm-operation-receipts.rs"),
    ] {
        let source_path = root.join(source);
        let content = fs::read_to_string(&source_path).expect("physical source");
        let registrations = external_test_registrations(root, &source_path, &content);
        let fixture_path = root.join(fixture).canonicalize().expect("physical fixture");
        let bytes = fs::read(&fixture_path).expect("complete fixture bytes");
        let expected = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(registrations.len(), 1, "one complete external declaration");
        assert_eq!(registrations[0].2, fixture_path);
        assert_eq!(registrations[0].3, expected, "whole fixture byte binding");
    }
}

#[test]
fn external_registration_is_structural_and_refuses_unproven_paths_or_bodies() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!("external-test-registration-{}-{}",
        std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir(&root).expect("fresh isolated root");
    fs::create_dir_all(root.join("src/nested")).expect("source directory");
    fs::create_dir(root.join("tests")).expect("test directory");
    let source_path = root.join("src/nested/module.rs");
    let fixture = "#[test]\nfn physical_assertion() { assert_eq!(2 + 2, 4); }\n";
    for name in ["renamed-alpha.rs", "renamed-beta.rs"] {
        fs::write(root.join("tests").join(name), fixture).expect("complete test fixture");
        let declaration = format!("#[cfg(test)]\n#[path = \"../../tests/{name}\"]\nmod arbitrary_name;");
        let registrations = external_test_registrations(&root, &source_path, &declaration);
        assert_eq!(registrations.len(), 1, "renamed physical registration");
        assert_eq!(fs::read_to_string(&registrations[0].2).unwrap(), fixture);
        assert!(inline_test_violations(&root, &source_path, &declaration).is_empty());
        let contextual = format!("const TEXT: &str = r###\"{{ #[cfg(test)] }}\"###;\n/* {{ nested /* }} */ }} */\n{declaration}");
        assert_eq!(external_test_registrations(&root, &source_path, &contextual).len(), 1);
    }
    for source in [
        "#[cfg(test)]\nfn inline() {}",
        "#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod inline {}",
        "#[unknown]\n#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod hidden;",
        "#[cfg(test)]\n#[unknown]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod hidden;",
        "mod nested {\n#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod hidden;\n}",
        "#[cfg(test)]\n#[path = \"../../tests/missing.rs\"]\nmod absent;",
        "#[cfg(test)]\n#[path = \"../../../escaped.rs\"]\nmod escaped;",
        "#[cfg(test)]\n#[path = \"../../tests/../tests/renamed-alpha.rs\"]\nmod redundant;",
        "#[cfg(test)]\n#[path = \"../../tests/./renamed-alpha.rs\"]\nmod redundant;",
        "#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod incomplete",
        "#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod hidden;\n}",
        "#[test]\nfn actual_inline_test() {}",
        "#[tokio::test]\nasync fn actual_inline_test() {}",
        "mod tests { fn body() {} }",
    ] {
        assert!(external_test_registrations(&root, &source_path, source).is_empty(), "must refuse {source}");
        assert!(!inline_test_violations(&root, &source_path, source).is_empty(), "original detector must reject {source}");
    }
    fs::remove_dir_all(root).expect("remove isolated fixtures");
}

#[cfg(unix)]
#[test]
fn external_registration_refuses_a_symlink_fixture() {
    let root = std::env::temp_dir().join(format!("external-test-symlink-{}", std::process::id()));
    fs::create_dir(&root).expect("fresh isolated root");
    fs::create_dir_all(root.join("src/nested")).expect("source directory");
    fs::create_dir(root.join("tests")).expect("test directory");
    fs::write(root.join("tests/actual.rs"), "#[test] fn physical() {}\n").unwrap();
    std::os::unix::fs::symlink(root.join("tests/actual.rs"), root.join("tests/alias.rs")).unwrap();
    let source = "#[cfg(test)]\n#[path = \"../../tests/alias.rs\"]\nmod alias;";
    assert!(external_test_registrations(&root, &root.join("src/nested/module.rs"), source).is_empty());
    assert!(!inline_test_violations(&root, &root.join("src/nested/module.rs"), source).is_empty());
    fs::remove_dir_all(root).unwrap();
}
