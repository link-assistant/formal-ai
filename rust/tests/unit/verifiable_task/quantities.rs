//! Issue #1138 B8 (plan 08, L4): quantities and multiplicity, five languages.
//!
//! A spelled-out numeral is a quantity in every language the doctrine covers, not
//! only where an ASCII-digit scan happens to work. Multiplicity attaches to the
//! entity it modifies — "two oboes, a stethoscope, three scalpels" is
//! `oboe×2, stethoscope×1, scalpel×3`, which is precisely what an object-count
//! table cannot express.

use formal_ai::verifiable_task::quantities::{extract_entities, extract_quantities};

use super::{LANGUAGES, case};

/// The same numeral, spelled out in each language, must extract as `2` with the
/// offset it was found at.
#[test]
fn spelled_out_numerals_extract_in_five_languages() {
    let spelled: &[(&str, &str)] = &[
        ("en", "I brought two spare batteries to the workshop."),
        ("ru", "Я принёс два запасных аккумулятора в мастерскую."),
        ("hi", "मैं कार्यशाला में दो अतिरिक्त बैटरियाँ लाया।"),
        ("zh", "我带了两块备用电池到工作室。"),
        ("es", "Llevé dos baterías de repuesto al taller."),
    ];

    for (language, prompt) in spelled {
        let quantities = extract_quantities(prompt, language);
        assert!(
            quantities.iter().any(|quantity| quantity.value == "2"),
            "{language}: the spelled-out numeral must normalize to 2, got {quantities:?}"
        );
        let two = quantities
            .iter()
            .find(|quantity| quantity.value == "2")
            .expect("the quantity was just asserted present");
        assert!(
            two.offset < prompt.len(),
            "{language}: the quantity carries the byte offset it was found at"
        );
    }
    assert_eq!(
        spelled.len(),
        LANGUAGES.len(),
        "every supported language must be covered"
    );
}

/// Each stated multiplicity belongs to its own entity, and an entity with no
/// stated multiplicity is one.
#[test]
fn multiplicity_is_attached_to_its_entity() {
    for language in LANGUAGES {
        let paraphrase = case("counted_category", language);
        let entities = extract_entities(&paraphrase.prompt, language);
        assert_eq!(
            entities.len(),
            5,
            "{language}: the prompt lists five entities, got {entities:?}"
        );

        let multiplicities: Vec<&str> = entities
            .iter()
            .map(|entity| entity.multiplicity.as_str())
            .collect();
        assert_eq!(
            multiplicities.iter().filter(|value| **value == "2").count(),
            1,
            "{language}: exactly one entity carries a multiplicity of two, got {multiplicities:?}"
        );
        assert_eq!(
            multiplicities.iter().filter(|value| **value == "3").count(),
            1,
            "{language}: exactly one entity carries a multiplicity of three, got {multiplicities:?}"
        );
        assert_eq!(
            multiplicities.iter().filter(|value| **value == "1").count(),
            3,
            "{language}: the remaining entities are one each, got {multiplicities:?}"
        );
    }
}
