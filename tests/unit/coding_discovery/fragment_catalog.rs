//! Issue #1138, plan 02 L4 and L16–L18 — the seed becomes a deletable bootstrap.
//!
//! Two `include_str!` calls make the idiom seed a compile-time dependency today,
//! so "delete the seed and see what the system can still do" cannot even be
//! expressed. After plan 02 the catalog reads the seed at runtime, an absent
//! seed yields an empty catalog rather than a panic, a forgotten fragment is
//! rediscovered to the same content id, and no rediscovery query names a
//! benchmark identifier.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::fragment_catalog::{Fragment, FragmentCatalog, FragmentLedger, FragmentOrigin};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

/// The seed files the bootstrap catalog reads.
const BOOTSTRAP_SEED: [&str; 4] = [
    "data/seed/meanings-coding-structure.lino",
    "data/seed/meanings-coding-structure-2.lino",
    "data/seed/coding-composition-fragments.lino",
    "data/seed/coding-discovery-runtime.lino",
];

fn temp_dir(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-1138-fragments-{label}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("temporary directory");
    path
}

#[test]
fn bootstrap_catalog_is_absent_without_the_seed_and_never_panics() {
    // A seed directory that exists but holds none of the bootstrap files is
    // exactly the state the deletability job runs the suite in.
    let empty_seed = temp_dir("no-seed");
    let catalog = FragmentCatalog::bootstrap_from(&empty_seed);

    assert!(
        catalog.fragments().is_empty(),
        "a missing seed yields an empty catalog, not a panic and not a guess"
    );
    assert_eq!(
        catalog.get("extend_run"),
        None,
        "a missing fragment id is an absence, never a panic"
    );
    assert_eq!(
        catalog.render_named("python_return", "python", &[("expression", "value")]),
        None,
        "a named lookup against an empty catalog remains an explicit absence"
    );
    assert_eq!(
        formal_ai::python_render::runtime_template_from(
            &empty_seed,
            "python_return",
            &[("expression", "value")],
        ),
        None,
        "the runtime renderer must not retain a compile-time seed copy"
    );

    assert_eq!(
        FragmentCatalog::absent_seed_files(&empty_seed).len(),
        BOOTSTRAP_SEED.len(),
        "an absent seed is visible as the named files the bootstrap could not read"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in BOOTSTRAP_SEED {
        assert!(
            root.join(relative).is_file(),
            "{relative} is the bootstrap the catalog reads at runtime"
        );
    }
    assert!(
        FragmentCatalog::absent_seed_files(root).is_empty(),
        "the shipped tree is not in the absent-seed state"
    );
    assert!(
        !FragmentCatalog::bootstrap().fragments().is_empty(),
        "with the seed present the same code path yields the bootstrap fragments"
    );
    assert_eq!(
        FragmentCatalog::bootstrap().render_named("python_return", "python", &[]),
        None,
        "a missing required placeholder is not rendered as malformed source"
    );
}

#[test]
fn structural_algorithms_are_composed_from_primitives_not_catalogued_as_answers() {
    let catalog = FragmentCatalog::bootstrap();
    for answer_id in [
        "balanced_delimiter_groups",
        "group_max_nesting",
        "grid_minimum_cost_path",
        "one_bit_difference",
        "remove_boundary_occurrences",
        "rotation_period",
    ] {
        assert_eq!(
            catalog.get(answer_id),
            None,
            "{answer_id} is a requested meaning, not a reusable implementation fragment"
        );
    }
    for primitive_id in [
        "bitwise_xor",
        "delimiter_depth_delta",
        "drop_whitespace",
        "first_truthy_position",
        "positive_offsets",
        "remove_first_occurrence",
        "reverse_text",
        "rotate_left_text",
        "single_set_bit",
        "slice_at_end_offsets",
        "values_equal",
        "zero_depth_end_offsets",
    ] {
        assert!(
            catalog.get(primitive_id).is_some(),
            "the reusable primitive {primitive_id} must remain discoverable"
        );
    }
}

#[test]
fn named_rendering_preserves_target_language_braces_around_a_slot() {
    // The brace-preservation rule is generic rendering behavior, so the probe
    // fragment lives in a temporary seed rather than the shipped one: no
    // shipped fragment exists to carry a regex-shaped template any more.
    let seed = temp_dir("brace-rendering");
    fs::write(
        seed.join("coding-discovery-runtime.lino"),
        "coding_discovery_runtime\n  template brace_probe\n    text \"findall(r'\\\\b\\\\w{{least},}\\\\b', {text})\"\n    grounding \"https://docs.python.org/3.12/library/re.html#re.findall\"\n    license \"PSF-2.0\"\n    rediscovery_query \"find words of at least a given length\"\n    fragment_signature (integer text)\n    fragment_result \"sequence<text>\"\n",
    )
    .expect("write brace rendering seed");
    let rendered = FragmentCatalog::bootstrap_from(&seed)
        .render_named(
            "brace_probe",
            "python",
            &[("least", "5"), ("text", "sentence")],
        )
        .expect("the brace probe fragment renders");

    assert_eq!(rendered, "findall(r'\\b\\w{5,}\\b', sentence)");
}

#[test]
fn runtime_lowering_template_changes_when_seed_data_changes() {
    let seed = temp_dir("runtime-template-mutation");
    fs::write(
        seed.join("coding-discovery-runtime.lino"),
        "coding_discovery_runtime\n  template python_ir_emit\n    text \"emit({value})\"\n",
    )
    .expect("write runtime template seed");
    let first = formal_ai::python_render::runtime_template_from(
        &seed,
        "python_ir_emit",
        &[("value", "answer")],
    );

    fs::write(
        seed.join("coding-discovery-runtime.lino"),
        "coding_discovery_runtime\n  template python_ir_emit\n    text \"observe({value})\"\n",
    )
    .expect("mutate runtime template seed");
    let second = formal_ai::python_render::runtime_template_from(
        &seed,
        "python_ir_emit",
        &[("value", "answer")],
    );

    assert_eq!(first.as_deref(), Some("emit(answer)"));
    assert_eq!(second.as_deref(), Some("observe(answer)"));
    assert_ne!(
        first, second,
        "runtime lowering must remain controlled by data"
    );
    assert_eq!(
        FragmentCatalog::bootstrap().get("ir_type_mismatch"),
        None,
        "diagnostic/rendering templates are data, not searchable solution fragments"
    );
}

#[test]
fn forgotten_fragments_are_rediscovered_to_the_same_content_id() {
    let before = FragmentCatalog::bootstrap();
    let expected = before.content_id();
    assert!(
        !expected.is_empty(),
        "the catalog is content-addressed over its fragments"
    );

    // Forget: rebuild with the seed moved aside. Rediscover: replay every
    // fragment through the ledger the offline rediscovery writes.
    let ledger_dir = temp_dir("rediscover");
    let ledger = FragmentLedger::new(&ledger_dir);
    for fragment in before.fragments() {
        let rediscovered = Fragment {
            origin: FragmentOrigin::Rediscovered,
            ..fragment.clone()
        };
        assert!(
            !rediscovered.rediscovery_query.is_empty(),
            "{} has no query that could rediscover it",
            rediscovered.id
        );
        ledger.remember(&rediscovered).expect("remember a fragment");
    }

    let after = FragmentCatalog::bootstrap_from(temp_dir("forgotten")).with_rediscovered(&ledger);
    assert_eq!(
        after.content_id(),
        expected,
        "forget → rediscover → the same content id, or the seed was the capability"
    );
    assert!(
        after
            .fragments()
            .iter()
            .all(|fragment| fragment.origin == FragmentOrigin::Rediscovered),
        "every fragment in the rebuilt catalog came back from a source"
    );
    assert_eq!(
        ledger.forget_all().expect("forget rediscovered cache"),
        before.fragments().len()
    );
    assert!(
        FragmentCatalog::default()
            .with_rediscovered(&ledger)
            .fragments()
            .is_empty(),
        "the command's forget operation removes only the rediscovery ledger"
    );
}

#[test]
fn rediscovery_queries_name_no_benchmark_identifier() {
    for fragment in FragmentCatalog::bootstrap().fragments() {
        let query = &fragment.rediscovery_query;
        assert!(
            !query.contains("http"),
            "{} rediscovers itself by a phrase, not a URL: {query}",
            fragment.id
        );
        assert!(
            query.split_whitespace().count() >= 2,
            "{} rediscovers itself by a natural-language phrase: {query}",
            fragment.id
        );
        for forbidden in [
            "humaneval",
            "mbpp",
            "run_length",
            "find_position",
            "edit_steps",
        ] {
            assert!(
                !query.to_lowercase().contains(forbidden),
                "{} names a benchmark identifier in its query: {query}",
                fragment.id
            );
        }
        assert!(
            !fragment.grounding.is_empty() && !fragment.license.is_empty(),
            "{} must carry its grounding and license",
            fragment.id
        );
    }
}
