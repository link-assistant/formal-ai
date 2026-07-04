//! Round-trip translation tests (issue #526).
//!
//! Issue #526 states the quality bar for translation: *"the best translation
//! is the translation that survives round-trip translation."* A faithful
//! translation `A --(source→target)--> B` must translate back
//! `B --(target→source)--> A` to the *same meaning* — and, for a well-seeded
//! vocabulary, the *same surface* it started from.
//!
//! These tests exercise the real
//! `source → formalize → semantic meta language → deformalize → target`
//! pipeline through [`translate_via_default_pipeline`], entirely offline
//! against the committed seed cache (no `FORMAL_AI_LIVE_API`). They assert the
//! two round-trip invariants that matter:
//!
//! 1. **Meaning survives** — the language-neutral [`MeaningId`] of the forward
//!    translation equals the meaning of the backward translation. Translation
//!    passes *through* the meta language, so the meta-language identity is the
//!    invariant the surfaces are grounded in.
//! 2. **Surface survives** — the backward surface equals the original surface
//!    (case-insensitively). This is the observable #526 criterion.
//!
//! The pipeline holds no hardcoded phrase table: every surface here is grounded
//! in `data/seed/meanings-*.lino` and the seeded Wiktionary/Wikidata cache, so
//! widening the covered vocabulary is a pure seed edit.

use formal_ai::translation::translate_via_default_pipeline;

#[derive(Clone, Copy)]
struct SurfaceCase {
    language: &'static str,
    surface: &'static str,
}

const APPLE_SURFACES: &[SurfaceCase] = &[
    SurfaceCase {
        language: "en",
        surface: "apple",
    },
    SurfaceCase {
        language: "ru",
        surface: "яблоко",
    },
    SurfaceCase {
        language: "hi",
        surface: "सेब",
    },
    SurfaceCase {
        language: "zh",
        surface: "苹果",
    },
];

/// Assert the full #526 round-trip for one surface across one language pair:
/// `source --A→B--> target --B→A--> source`, preserving meaning and surface.
fn assert_round_trip(surface: &str, source: &str, target: &str) {
    let forward = translate_via_default_pipeline(surface, source, target).unwrap_or_else(|error| {
        panic!("forward {surface:?} {source}->{target} should translate offline, got {error:?}")
    });
    let target_surface = forward.primary_surface().unwrap_or_else(|| {
        panic!("forward {surface:?} {source}->{target} should yield a target surface")
    });
    assert_ne!(
        target_surface.to_lowercase(),
        surface.to_lowercase(),
        "forward {surface:?} {source}->{target} should change the surface, got {target_surface:?}",
    );

    let backward =
        translate_via_default_pipeline(target_surface, target, source).unwrap_or_else(|error| {
            panic!(
                "backward {target_surface:?} {target}->{source} should translate offline, \
                 got {error:?}"
            )
        });
    let round_tripped = backward.primary_surface().unwrap_or_else(|| {
        panic!("backward {target_surface:?} {target}->{source} should yield a surface")
    });

    // 1. Meaning survives the round trip — the meta-language identity is stable.
    assert_eq!(
        forward.meaning.slug(),
        backward.meaning.slug(),
        "#526: meaning must survive the {source}->{target}->{source} round trip for {surface:?} \
         (forward {} vs backward {})",
        forward.meaning.slug(),
        backward.meaning.slug(),
    );

    // 2. Surface survives — the observable #526 criterion.
    assert_eq!(
        round_tripped.to_lowercase(),
        surface.to_lowercase(),
        "#526: {surface:?} should survive the {source}->{target}->{source} round trip, \
         got {surface:?} -> {target_surface:?} -> {round_tripped:?}",
    );
}

fn assert_surface_eq(left: &str, right: &str, message: &str) {
    assert_eq!(
        left.to_lowercase(),
        right.to_lowercase(),
        "{message}: expected {right:?}, got {left:?}",
    );
}

#[test]
fn english_russian_vocabulary_survives_round_trip() {
    for surface in ["hello", "apple", "thank you", "water", "bread"] {
        assert_round_trip(surface, "en", "ru");
    }
}

#[test]
fn english_hindi_vocabulary_survives_round_trip() {
    for surface in ["hello", "apple"] {
        assert_round_trip(surface, "en", "hi");
    }
}

#[test]
fn english_chinese_vocabulary_survives_round_trip() {
    for surface in ["hello", "apple"] {
        assert_round_trip(surface, "en", "zh");
    }
}

#[test]
fn supported_language_surfaces_survive_meta_language_round_trip() {
    for case in APPLE_SURFACES {
        let round_trip = translate_via_default_pipeline(case.surface, case.language, case.language)
            .unwrap_or_else(|error| {
                panic!(
                    "#526: {:?} ({}) should formalize to the meta language and deformalize \
                     back without data loss, got {error:?}",
                    case.surface, case.language
                )
            });
        let surface = round_trip.primary_surface().unwrap_or_else(|| {
            panic!(
                "#526: {:?} ({}) should deformalize from the meta language",
                case.surface, case.language
            )
        });
        assert_surface_eq(
            surface,
            case.surface,
            "#526: language-to-meta-to-same-language must be lossless",
        );
        assert_ne!(
            round_trip.meaning.slug(),
            "meaning:unknown",
            "#526: {:?} ({}) must carry a traceable meta-language meaning",
            case.surface,
            case.language,
        );
    }
}

#[test]
fn every_supported_language_pair_round_trips_via_meta_language() {
    for source in APPLE_SURFACES {
        for target in APPLE_SURFACES {
            if source.language == target.language {
                continue;
            }

            let forward =
                translate_via_default_pipeline(source.surface, source.language, target.language)
                    .unwrap_or_else(|error| {
                        panic!(
                            "#526: forward {:?} {}->{} should translate through the meta \
                             language, got {error:?}",
                            source.surface, source.language, target.language
                        )
                    });
            let target_surface = forward.primary_surface().unwrap_or_else(|| {
                panic!(
                    "#526: forward {:?} {}->{} should yield a target surface",
                    source.surface, source.language, target.language
                )
            });
            assert_surface_eq(
                target_surface,
                target.surface,
                "#526: forward translation should render the target-language surface",
            );

            let backward =
                translate_via_default_pipeline(target_surface, target.language, source.language)
                    .unwrap_or_else(|error| {
                        panic!(
                            "#526: backward {:?} {}->{} should translate through the meta \
                             language, got {error:?}",
                            target_surface, target.language, source.language
                        )
                    });
            let round_tripped = backward.primary_surface().unwrap_or_else(|| {
                panic!(
                    "#526: backward {:?} {}->{} should yield the original-language surface",
                    target_surface, target.language, source.language
                )
            });

            assert_eq!(
                forward.meaning.slug(),
                backward.meaning.slug(),
                "#526: meaning id must survive {}->{}->{} for {:?}",
                source.language,
                target.language,
                source.language,
                source.surface,
            );
            assert_surface_eq(
                round_tripped,
                source.surface,
                "#526: every supported language pair must round-trip through the meta language",
            );
        }
    }
}

/// The forward and backward legs must agree on the *same* meta-language
/// identity for every seeded target language — one meaning, many surfaces.
/// This is the property that lets a synonym in any language collapse to the
/// same node before the surface is chosen.
#[test]
fn one_meaning_backs_every_language_surface() {
    // Translating the same English noun into each seeded target must resolve to
    // one and the same meta-language meaning — the surfaces differ, the meaning
    // does not. This is the property that lets synonyms across languages
    // collapse to a single node before a surface is chosen.
    let mut shared_meaning: Option<String> = None;
    for target in ["ru", "hi", "zh"] {
        let forward = translate_via_default_pipeline("apple", "en", target)
            .unwrap_or_else(|error| panic!("apple en->{target} should translate, got {error:?}"));
        let meaning = forward.meaning.slug();
        match &shared_meaning {
            None => shared_meaning = Some(meaning),
            Some(expected) => assert_eq!(
                &meaning, expected,
                "apple should carry one meta-language meaning across every target language, \
                 got {meaning} for {target} vs {expected}",
            ),
        }
    }
}
