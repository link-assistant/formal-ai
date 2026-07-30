//! Issue #699 batch 1: migrate fixed number-riddle routing into seed data while
//! keeping only the language-neutral interval/proof primitive in Rust.

use std::fs;
use std::path::Path;

use formal_ai::seed;
use formal_ai::FormalAiEngine;

const RECORDED_SPECIALIZED_HANDLER_FILES_MAX: usize = 38;
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
fn handler_migration_ratchet() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let handler_files = fs::read_dir(root.join("src/solver_handlers"))
        .expect("solver_handlers directory")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "rs")
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
        1,
        "batch 1 migrates exactly one method",
    );
    assert_eq!(
        ledger.matches("status \"justified-native\"").count(),
        2,
        "the native set must stay explicit and small",
    );
    assert_eq!(
        ledger.matches("status pending").count(),
        expected.len() - 3,
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
