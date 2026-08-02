//! Issue #776: a source-first translation request must route through the
//! language-neutral meaning pipeline even when its command follows the text.

use formal_ai::{
    translation::{extract_unquoted_translation_surface, translate_via_default_pipeline},
    UniversalSolver,
};

const RUSSIAN: &str = "любая формальная система либо неполна, либо противоречива";
const ENGLISH: &str = "any formal system is either incomplete or inconsistent";
const HINDI: &str = "कोई भी औपचारिक प्रणाली या तो अपूर्ण होती है या असंगत";
const CHINESE: &str = "任何形式系统要么是不完备的，要么是不一致的";

#[test]
fn reported_source_first_prompt_translates_to_english() {
    let prompt = format!("{RUSSIAN} - translate to english");
    assert_eq!(
        extract_unquoted_translation_surface(&prompt).as_deref(),
        Some(RUSSIAN),
    );
    let response = UniversalSolver::default().solve(&prompt);

    assert_eq!(response.intent, "translate_ru_to_en", "{}", response.answer);
    assert_eq!(response.answer, format!("\"{ENGLISH}\""));
    assert!(response.links_notation.contains("language_from ru"));
    assert!(response.links_notation.contains("language_to en"));
    assert!(!response.links_notation.contains("translation_gap"));
}

#[test]
fn proposition_survives_every_supported_language_round_trip() {
    let surfaces = [
        ("en", ENGLISH),
        ("ru", RUSSIAN),
        ("hi", HINDI),
        ("zh", CHINESE),
    ];

    let mut shared_meaning = None;
    for (source_language, source_surface) in surfaces {
        for (target_language, target_surface) in surfaces {
            if source_language == target_language {
                continue;
            }
            let forward = translate_via_default_pipeline(
                source_surface,
                source_language,
                target_language,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "forward {source_language}->{target_language} should formalize through one meaning: {error:?}"
                )
            });
            assert_eq!(forward.primary_surface(), Some(target_surface));

            let backward = translate_via_default_pipeline(
                target_surface,
                target_language,
                source_language,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "backward {target_language}->{source_language} should deformalize the same meaning: {error:?}"
                )
            });
            assert_eq!(backward.primary_surface(), Some(source_surface));
            assert_eq!(forward.meaning.slug(), backward.meaning.slug());
            match &shared_meaning {
                None => shared_meaning = Some(forward.meaning.slug()),
                Some(expected) => assert_eq!(
                    &forward.meaning.slug(),
                    expected,
                    "every language pair must use one proposition meaning",
                ),
            }
        }
    }
}
