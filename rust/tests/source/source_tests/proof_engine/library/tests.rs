use super::*;

#[test]
fn pythagoras_matches_english() {
    assert!(PYTHAGOREAN.matches("can you prove the pythagorean theorem"));
}

#[test]
fn pythagoras_matches_russian() {
    assert!(PYTHAGOREAN.matches("докажите теорему пифагора"));
}

#[test]
fn fermat_little_matches_chinese() {
    assert!(FERMAT_LITTLE.matches("证明费马小定理"));
}

#[test]
fn euclid_primes_matches_english() {
    assert!(
        EUCLID_INFINITUDE_OF_PRIMES.matches("demonstrate that there are infinitely many primes")
    );
}

#[test]
fn euclid_primes_matches_compact_russian() {
    assert!(EUCLID_INFINITUDE_OF_PRIMES.matches("простых бесконечно"));
    assert!(EUCLID_INFINITUDE_OF_PRIMES.matches("простых чисел бесконечно много"));
}

#[test]
fn sqrt_two_matches_english() {
    assert!(SQRT_TWO_IRRATIONAL.matches("show that the square root of two is irrational"));
}

#[test]
fn godel_matches_english() {
    assert!(GODEL_FIRST_INCOMPLETENESS.matches("godel's incompleteness"));
}

#[test]
fn determinism_matches_english() {
    assert!(LAPLACIAN_DETERMINISM.matches("prove determinism"));
}

#[test]
fn build_proof_returns_localized_text() {
    let proof = PYTHAGOREAN.build_proof("ru");
    assert!(proof.statement.contains("прямоугольном"));
    assert!(proof.conclusion.contains("∎"));
    assert!(!proof.steps.is_empty());
}

#[test]
fn registry_lookup_is_first_match_wins() {
    // The Gödel entry should match when both Gödel and determinism appear, ensuring the
    // engine handles the issue-185 prompt deterministically.
    let prompt =
        "prove determinism the way logic can handle paradoxes like godel's math incompleteness";
    let mut godel_index = None;
    let mut determinism_index = None;
    for (index, entry) in REGISTRY.iter().enumerate() {
        if entry.matches(prompt) {
            if entry.id == "godel_first_incompleteness" && godel_index.is_none() {
                godel_index = Some(index);
            }
            if entry.id == "laplacian_determinism" && determinism_index.is_none() {
                determinism_index = Some(index);
            }
        }
    }
    assert!(
        godel_index.is_some(),
        "godel entry should match the issue prompt"
    );
    assert!(
        determinism_index.is_some(),
        "determinism entry should match the issue prompt"
    );
    assert!(
        godel_index < determinism_index,
        "godel entry must precede determinism in REGISTRY"
    );
}
