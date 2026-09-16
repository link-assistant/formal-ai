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
use formal_ai::method_registry::{MethodRegistry, MethodSurface};
use formal_ai::seed;

/// Every ceiling this test enforces is read from the one ledger that holds them
/// (issue #1138 B9, plan 09 leaves 3-4).
///
/// `RECORDED_SPECIALIZED_HANDLER_FILES_MAX` (37) and
/// `RECORDED_TRY_DISPATCH_ENTRIES_MAX` (50) used to stand here as Rust
/// constants while `data/meta/debt-ratchet.lino` recorded 42 and
/// `data/meta/handler-migration-ledger.lino` cited a third, deleted file. Three
/// copies of one number is how 37/50 drifted from 36/39 unnoticed, so the
/// constants are gone and this test reads the ledger.
fn reviewed_ceiling(measure: &str) -> usize {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/meta/debt-ratchet.lino");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} readable: {error}", path.display()));
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("measure ") {
            current = Some(rest.trim().trim_matches('"').to_owned());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("value ")
            && current.take().as_deref() == Some(measure)
        {
            return rest
                .trim()
                .trim_matches('"')
                .parse()
                .unwrap_or_else(|error| panic!("{measure} value: {error}"));
        }
    }
    panic!(
        "data/meta/debt-ratchet.lino names no `{measure}` ceiling; plan 09 leaf 4 adds \
         try_dispatch_entries, promotion_predicates, dispatch_name_special_cases, \
         worker_sync_handler_literals and store_read_share through the strict checker"
    );
}

/// The one definition of a handler file, shared with
/// `scripts/check-minimal-core-boundary.rs::source_files` and
/// `scripts/check-debt-ratchet.rs` (plan 09 leaves 1 and 3).
///
/// Every `*.rs` under `src/solver_handlers/**` except the generated
/// `modules.rs` and the dispatch `mod.rs`, **plus** the four handler files that
/// live one directory up, so a migration cannot lower a count by moving a file.
fn handler_files() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<String> = walkdir::WalkDir::new(root.join("src/solver_handlers"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?.to_owned();
            // `mod.rs` holds the dispatch logic and `modules.rs` is the
            // generated `mod` list issue #991 split out of it; neither is a
            // handler, so neither may move a ratchet that counts handlers.
            let bookkeeping = matches!(name.as_str(), "mod.rs" | "modules.rs");
            (path.extension().is_some_and(|extension| extension == "rs") && !bookkeeping).then(|| {
                path.strip_prefix(root)
                    .unwrap_or(path)
                    .display()
                    .to_string()
            })
        })
        .collect();
    for outside in [
        "solver_handler_how.rs",
        "solver_handler_how_synthesis.rs",
        "solver_handler_units.rs",
        "solver_handler_oracle.rs",
    ] {
        if root.join("src").join(outside).is_file() {
            files.push(format!("src/{outside}"));
        }
    }
    files.sort();
    files
}

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
    let files = handler_files();
    let handler_files = files.len();
    let handler_ceiling = reviewed_ceiling("handler_files");
    assert!(
        handler_files <= handler_ceiling,
        "specialized handler files grew from {handler_ceiling} to {handler_files}; the \
         reviewed ceiling is data/meta/debt-ratchet.lino. Files: {files:?}",
    );
    assert!(
        handler_files >= handler_ceiling,
        "specialized handler files improved from {handler_ceiling} to {handler_files}; \
         lower the reviewed ceiling in data/meta/debt-ratchet.lino in this commit",
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
    let try_ceiling = reviewed_ceiling("try_dispatch_entries");
    assert!(
        try_entries <= try_ceiling,
        "try_* dispatch entries grew from {try_ceiling} to {try_entries}; the reviewed \
         ceiling is data/meta/debt-ratchet.lino",
    );
    assert!(
        try_entries >= try_ceiling,
        "try_* dispatch entries improved from {try_ceiling} to {try_entries}; lower the \
         reviewed ceiling in data/meta/debt-ratchet.lino in this commit",
    );
}

#[test]
fn migration_ledger_is_a_complete_live_registry_census() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let precedence = fs::read_to_string(root.join("data/seed/handler-precedence.lino"))
        .expect("handler precedence");
    // Issue #1138 B9, plan 09 leaf 5: the census is the live registry, which is
    // the specialized precedence table **plus** the five prelude methods that
    // run on every turn. Counting only the precedence table left five live
    // methods outside the ledger, so the pending count read 40 when it was 45.
    let registry = MethodRegistry::from_dispatch();
    let prelude: Vec<String> = registry
        .methods
        .iter()
        .filter(|method| method.surface == MethodSurface::Prelude)
        .map(|method| method.name.clone())
        .collect();
    let mut expected = precedence
        .lines()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    expected.extend(prelude.iter().cloned());
    let ledger = fs::read_to_string(root.join("data/meta/handler-migration-ledger.lino"))
        .expect("handler migration ledger");
    let actual = ledger
        .lines()
        .filter_map(|line| line.trim().strip_prefix("handler "))
        .map(str::to_owned)
        .collect::<Vec<_>>();

    assert_eq!(
        actual, expected,
        "the ledger must cover the live registry in order: the specialized precedence \
         table followed by the five prelude methods {prelude:?}"
    );
    assert_eq!(
        ledger.matches("status migrated").count(),
        16,
        "batches 1-3 migrated four methods; issue #1085 D1.3 migrated eleven handler names into \
         data/seed/handler-rules.lino, and issue #1095 a twelfth (agentic_continuation)",
    );
    assert_eq!(
        ledger.matches("status \"justified-native\"").count(),
        2,
        "the native set must stay explicit and small",
    );
    let pending = ledger.matches("status pending").count();
    assert_eq!(
        pending,
        expected.len() - 18,
        "every other current method must honestly remain pending",
    );
    assert_eq!(
        pending,
        reviewed_ceiling("handler_migration_pending"),
        "the pending count and its ceiling in data/meta/debt-ratchet.lino are one number, \
         recorded once. Plan 09 leaf 5 raises it 40 -> 45 with note \"corrected undercount\" \
         when the five prelude rows join the census, and it only falls afterwards",
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
