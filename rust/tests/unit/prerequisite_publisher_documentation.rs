use super::seed_publishers;

#[test]
fn all_declared_publisher_document_paths_retain_exact_authority() {
    for publisher in seed_publishers() {
        assert_eq!(publisher.documentation_url(), publisher.documentation);
        assert!(publisher.authorizes_document_url(&publisher.documentation));
        assert!(!publisher.authorizes_document_url("https://unrelated.invalid/"));
    }
}
