use super::canonical_creation;

#[test]
fn canonical_add_file_retains_exact_literal_content() {
    for (path, content) in [
        ("candidate.json", "{\"enabled\":false}\n"),
        ("renamed.txt", "\n  ключ? Error: failed  \n\n"),
        ("linebreaks.txt", "alpha\u{2028}+literal\u{2029}+another\n"),
    ] {
        let arguments = serde_json::json!({"path":path,"content":content}).to_string();
        let raw = crate::protocol_responses::apply_patch_input(&arguments).unwrap();
        let decoded = canonical_creation(&raw).expect("Canonical single AddFile declaration");
        assert_eq!(decoded.path, path);
        assert_eq!(decoded.content, content);
        assert_eq!(super::super::canonical_creation_path(&raw), Some(path));
    }
}

#[test]
fn noncanonical_or_mixed_patches_grant_no_creation_declaration() {
    for raw in [
        "*** Begin Patch\n*** Add File: x.txt\n+hello\n*** End Patch",
        "*** Begin Patch\n*** Add File: \n+hello\n*** End Patch\n",
        "*** Begin Patch\n*** Add File: x.txt\nhello\n*** End Patch\n",
        "*** Begin Patch\n*** Add File: x.txt\n+hello\n*** Add File: other.txt\n+foreign\n*** End Patch\n",
        "*** Begin Patch\n*** Update File: x.txt\n+hello\n*** End Patch\n",
    ] {
        assert!(canonical_creation(raw).is_none(), "{raw}");
        assert!(super::super::canonical_creation_path(raw).is_none());
    }
}
