use super::*;

pub(super) fn the_relation_seed_declares_how_a_gloss_is_read_and_never_what_a_word_means() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let text = fs::read_to_string(root.join(RELATION_SEED))
        .unwrap_or_else(|error| panic!("{RELATION_SEED}: {error}"));

    let relations = [
        "is_a",
        "has_property",
        "part_of",
        "requires",
        "produces",
        "precedes",
        "excludes",
        "measured_in",
    ];
    let schema = [
        "relations",
        "determiners",
        "kind",
        "inverse",
        "grounding",
        "lexeme",
        "surface",
        "text",
    ];

    let mut declared: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let head = trimmed.split_whitespace().next().unwrap_or_default();
        if schema.contains(&head) {
            continue;
        }
        assert!(
            relations.contains(&head),
            "{RELATION_SEED} declares `{head}`, which is neither the relation schema \
             nor one of the eight declared relations: a subject matter may not enter \
             this file under any record kind"
        );
        declared.push(head.to_owned());
    }
    declared.sort();
    let mut expected: Vec<String> = relations.iter().map(|name| (*name).to_owned()).collect();
    expected.sort();
    assert_eq!(
        declared, expected,
        "the relation vocabulary is the eight the plan declares, no more and no fewer"
    );

    for line in text.lines() {
        let trimmed = line.trim();
        let Some(surface) = trimmed.strip_prefix("text ") else {
            continue;
        };
        let surface = surface.trim().trim_matches('"');
        assert!(
            surface.split_whitespace().count() <= 4,
            "a cue is a fragment, not a definition: {surface:?} in {RELATION_SEED}"
        );
    }
}
