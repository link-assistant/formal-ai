use super::source_cache_directory;

#[test]
fn procedure_uses_the_public_source_cache_override() {
    let override_path = "isolated/procedural-captures";
    temp_env::with_var("FORMAL_AI_SOURCE_CACHE_DIR", Some(override_path), || {
        assert_eq!(source_cache_directory(), override_path);
    });
    temp_env::with_var("FORMAL_AI_SOURCE_CACHE_DIR", None::<&str>, || {
        assert_eq!(source_cache_directory(), "data");
    });
}
