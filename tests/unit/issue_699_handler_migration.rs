//! Issue #699 handler migration batches.
//!
//! Batch 1 migrated fixed number-riddle routing into seed data, keeping only
//! the language-neutral interval/proof primitive in Rust. Batch 2 migrated the
//! `who_is` and `definition_merge` methods: the fixed misspelling table became
//! approximate matching over remembered names, and the definition merger's
//! host-to-language mapping and rendered labels became seed data. Batch 3
//! migrated the `program_synthesis` dead end: an underivable request now fails
//! with a named, seed-driven skill gap instead of reciting the catalogue.

use std::fs;
use std::path::Path;

use formal_ai::FormalAiEngine;
use formal_ai::seed;

/// Handler modules only: `mod.rs` and the generated `modules.rs` are excluded
/// from the count, so the ceiling dropped by one when they stopped counting.
const RECORDED_SPECIALIZED_HANDLER_FILES_MAX: usize = 37;
const RECORDED_TRY_DISPATCH_ENTRIES_MAX: usize = 50;

#[test]
fn held_out_number_constraint_paraphrases_are_data_driven() {
    for (language, prompt) in [
        // English held-out paraphrase: neither relation appeared in the old recognizer.
        (
            "en",
            "Find the integer I am thinking of: it exceeds 4 and is below 6.",
        ),
        (
            "ru",
            "Найди задуманное целое: оно превышает 4 и не достигает 6.",
        ),
        ("hi", "वह पूर्णांक बताइए जो 4 से अधिक और 6 से कम है।"),
        ("zh", "请找出大于4且小于6的那个整数。"),
    ] {
        let normalized = prompt.to_lowercase().replace([':', '।', '。'], " ");
        for role in [
            seed::ROLE_NUMBER_CONSTRAINT_ENTITY,
            seed::ROLE_NUMBER_CONSTRAINT_QUERY,
            seed::ROLE_NUMBER_CONSTRAINT_LOWER,
            seed::ROLE_NUMBER_CONSTRAINT_UPPER,
        ] {
            assert!(
                seed::lexicon().mentions_role(role, &normalized),
                "{language} prompt does not ground role {role} from seed: {normalized}",
            );
        }
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "number_constraint_reasoning",
            "{language} held-out paraphrase was not routed through the migrated method: {}",
            response.answer,
        );
        assert!(
            response.answer.contains('5'),
            "{language} answer did not preserve the solved interval: {}",
            response.answer,
        );
    }
}

#[test]
fn held_out_entity_typos_resolve_from_memory() {
    // Batch 2: the retired `suggest_correction` table listed eight people and
    // three hand-written misspellings each. None of these names or spellings
    // appeared in it, and no misspelling is stored anywhere: every suggestion
    // below comes from approximate matching against remembered correct names.
    for (language, prompt, expected) in [
        ("en", "who is ada lovlace", "Ada Lovelace"),
        ("en", "who was alan turring", "Alan Turing"),
        ("ru", "кто такой альберт эйнштеин", "Альберт Эйнштейн"),
        ("hi", "निकोला टेस्ल कौन है", "निकोला टेस्ला"),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "who_is_question",
            "{language} held-out prompt left the migrated method: {}",
            response.answer,
        );
        assert!(
            response.answer.contains(expected),
            "{language} held-out typo did not resolve to {expected}: {}",
            response.answer,
        );
    }
}

#[test]
fn entity_suggestions_never_come_from_stored_misspellings() {
    // Anti-memoization guard: the seed registry stores canonical spellings
    // only. If a future edit smuggles a typo back into data, the suggestion
    // path would stop proving generality, so assert the registry itself is
    // clean of the historical hardcoded variants.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let registry =
        fs::read_to_string(root.join("data/seed/entity-names.lino")).expect("entity names seed");
    for misspelling in [
        "mask", "tramp", "tromp", "bidan", "bidon", "einstien", "enstien", "issac", "isaak",
        "vladmir", "puting", "barrack",
    ] {
        assert!(
            !registry.to_lowercase().contains(misspelling),
            "entity-names.lino must not store the misspelling {misspelling:?}",
        );
    }
    // Correct spellings resolve to themselves, i.e. produce no correction.
    assert_eq!(
        formal_ai::entity_resolution::suggest_known_name("Ada Lovelace"),
        None
    );
}

#[test]
fn unsupported_write_program_fails_with_a_named_skill_gap() {
    // Issue #699 requirement 3: the `write_program` meta-builder must either
    // synthesize outside the curated catalogue — it already does, via the
    // blueprint recipes, the coding oracle and the seed idiom composer — or
    // fail with a *named* skill gap. Reciting the curated catalogue back at the
    // requester ("Supported tasks: hello_world, …") is neither.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (language, prompt) in [
        ("en", "write a rust program that reverses a linked list"),
        (
            "ru",
            "Напиши программу на Rust, которая разворачивает связный список",
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "write_program_skill_gap",
            "{language} underivable program request must name a skill gap: {}",
            response.answer,
        );
        assert!(
            response.answer.contains("seed_idiom_composer"),
            "{language} skill gap must name the synthesis routes that missed: {}",
            response.answer,
        );
        // The named gap is a stable English identity, whatever the request
        // language, so the event log and the ledger can quote it.
        let gap = formal_ai::program_skill_gap::gap_name(None, Some("rust"));
        assert!(
            gap.contains("rust") && !gap.is_empty(),
            "{language} gap name must identify the program language: {gap}",
        );
    }

    // Anti-recitation guard: neither engine may answer with the catalogue.
    let recitation = ["Supported", "tasks:"].join(" ");
    for source in [
        "src/engine.rs",
        "src/web/worker/formal_ai_worker_14.js",
        "src/web/worker/formal_ai_worker_16.js",
    ] {
        let text = fs::read_to_string(root.join(source)).expect("engine source");
        assert!(
            !text.contains(&recitation),
            "{source} still recites the template catalogue as an answer",
        );
    }

    // The gap wording is seed data in every supported response language (R379).
    for language in ["en", "ru", "hi", "zh"] {
        for intent in ["write_program_skill_gap", "write_program_skill_gap_name"] {
            assert!(
                seed::response_for(intent, language).is_some(),
                "{intent} must be seeded for {language}",
            );
        }
    }
}

#[test]
fn handler_migration_ratchet() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let handler_files = fs::read_dir(root.join("src/solver_handlers"))
        .expect("solver_handlers directory")
        .filter_map(Result::ok)
        .filter(|entry| {
            let path = entry.path();
            // `mod.rs` holds the dispatch logic and `modules.rs` is the generated
            // `mod` list issue #991 split out of it; neither is a handler, so
            // neither may move a ratchet that counts specialized handlers.
            let bookkeeping = matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("mod.rs" | "modules.rs")
            );
            path.extension().is_some_and(|extension| extension == "rs") && !bookkeeping
        })
        .count();
    assert!(
        handler_files <= RECORDED_SPECIALIZED_HANDLER_FILES_MAX,
        "specialized handler files grew to {handler_files}; recorded max is \
         {RECORDED_SPECIALIZED_HANDLER_FILES_MAX}",
    );

    let dispatch =
        fs::read_to_string(root.join("src/solver_dispatch.rs")).expect("solver dispatch source");
    let table = dispatch
        .split_once("const HANDLER_FUNCTIONS")
        .and_then(|(_, tail)| tail.split_once("];"))
        .map(|(table, _)| table)
        .expect("HANDLER_FUNCTIONS table");
    let try_entries = table
        .lines()
        .filter(|line| {
            line.split_once(',')
                .is_some_and(|(_, function)| function.trim_start().starts_with("try_"))
        })
        .count();
    assert!(
        try_entries <= RECORDED_TRY_DISPATCH_ENTRIES_MAX,
        "try_* dispatch entries grew to {try_entries}; recorded max is \
         {RECORDED_TRY_DISPATCH_ENTRIES_MAX}",
    );
}

#[test]
fn migration_ledger_is_a_complete_live_registry_census() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let precedence = fs::read_to_string(root.join("data/seed/handler-precedence.lino"))
        .expect("handler precedence");
    let expected = precedence
        .lines()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .collect::<Vec<_>>();
    let ledger = fs::read_to_string(root.join("data/meta/handler-migration-ledger.lino"))
        .expect("handler migration ledger");
    let actual = ledger
        .lines()
        .filter_map(|line| line.trim().strip_prefix("handler "))
        .collect::<Vec<_>>();

    assert_eq!(
        actual, expected,
        "ledger must cover the live registry in order"
    );
    assert_eq!(
        ledger.matches("status migrated").count(),
        4,
        "batches 1-3 migrate four methods in total",
    );
    assert_eq!(
        ledger.matches("status \"justified-native\"").count(),
        2,
        "the native set must stay explicit and small",
    );
    assert_eq!(
        ledger.matches("status pending").count(),
        expected.len() - 6,
        "every other current method must honestly remain pending",
    );
}

#[test]
fn committed_agent_cli_batch_record_is_byte_reproducible() {
    const EXPECTED: &str = concat!(
        "handler_migration_batch:number_constraints status \"migrated\" ",
        "recognition \"seed_roles\" native_primitive \"interval_reasoning\" ",
        "held_out_languages \"en,ru,hi,zh\".",
    );
    const COMMITTED: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/case-studies/issue-699/agent-cli-evidence/",
        "handler-migration-batch-report.lino",
    ));

    assert_eq!(COMMITTED, EXPECTED);
}
