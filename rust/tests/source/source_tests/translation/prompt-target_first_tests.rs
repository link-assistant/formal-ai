use super::extract_unquoted_translation_surface;

#[test]
fn target_first_source_is_not_the_command_language() {
    assert_eq!(
        extract_unquoted_translation_surface("Translate to English: Программист"),
        Some("Программист".to_owned())
    );
    assert_eq!(
        crate::solver_helpers::infer_source_from_prompt("Translate to English: Программист"),
        "ru"
    );
}

#[test]
fn target_first_frames_accept_unseen_surfaces_from_seeded_targets() {
    assert_eq!(
        extract_unquoted_translation_surface("Translate to Spanish: quantum orchard"),
        Some("quantum orchard".to_owned())
    );
    assert_eq!(
        extract_unquoted_translation_surface("Переведи на английский: новая профессия"),
        Some("новая профессия".to_owned())
    );
}

#[test]
fn target_first_requires_both_action_and_exact_target() {
    for prompt in [
        "Translate to unknown: яблоко",
        "Explain to English: яблоко",
        "Translate to English: ",
    ] {
        assert_eq!(
            extract_unquoted_translation_surface(prompt),
            None,
            "{prompt}"
        );
    }
    assert_eq!(
        extract_unquoted_translation_surface("Translate apple to English: formal tone"),
        Some("apple".to_owned())
    );
}
