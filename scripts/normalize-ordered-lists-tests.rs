use super::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("normalize-ordered-lists-{name}-{nanos}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn registry() -> Vec<ListFile> {
    parse_registry(
        "merge_conflict_policy\n  \
         artifact rust_declaration_lists\n    \
         list_kind rust_declarations\n    \
         file\n      \
         path \"src/lib.rs\"\n    \
         file\n      \
         path \"tests/unit/mod.rs\"\n      \
         declares_directory \"tests/unit\"\n  \
         artifact branch_placeholder\n    \
         merge union\n    \
         file\n      \
         path \".gitkeep\"\n",
    )
}

#[test]
fn the_registry_only_yields_files_that_declare_a_list_kind() {
    let files = registry();
    assert_eq!(files.len(), 2, "the placeholder artifact has no list_kind");
    assert_eq!(files[0].path, "src/lib.rs");
    assert_eq!(files[0].list_kind, "rust_declarations");
    assert_eq!(files[1].declares_directory.as_deref(), Some("tests/unit"));
}

#[test]
fn a_union_merge_of_two_branches_is_restored_to_one_canonical_list() {
    // Exactly what `merge=union` produces when two branches each append a
    // module: both sides kept, in the order git happened to see them.
    let unioned = "mod alpha;\nmod zulu;\nmod beta;\nmod zulu;\n";
    assert_eq!(
        normalize_rust_declarations(unioned).unwrap(),
        "mod alpha;\nmod beta;\nmod zulu;\n",
        "the union is sorted and the repeated entry is dropped"
    );
}

#[test]
fn code_around_a_declaration_run_is_left_untouched() {
    let source = "use std::sync::Mutex;\n\nmod zulu;\nmod alpha;\n\nfn helper() {}\n";
    assert_eq!(
        normalize_rust_declarations(source).unwrap(),
        "use std::sync::Mutex;\n\nmod alpha;\nmod zulu;\n\nfn helper() {}\n"
    );
}

#[test]
fn visibility_is_preserved_and_a_genuine_disagreement_is_reported() {
    assert_eq!(
        normalize_rust_declarations("pub mod zulu;\nmod alpha;\n").unwrap(),
        "mod alpha;\npub mod zulu;\n"
    );
    let error = normalize_rust_declarations("pub mod alpha;\nmod alpha;\n").unwrap_err();
    assert!(error.contains("different visibility"), "{error}");
}

#[test]
fn a_restricted_reexport_sorts_with_the_public_ones() {
    // `src/agentic_coding/modules.rs` interleaves `pub(crate) use` with
    // `pub use`. If the parser skipped the restricted lines they would end
    // the run, and a union merge would sort each fragment on its own.
    let unioned = "pub use zulu::Later;\npub(crate) use alpha::beta;\n";
    assert_eq!(
        normalize_rust_declarations(unioned).unwrap(),
        "pub(crate) use alpha::beta;\npub use zulu::Later;\n"
    );
    let folded = "pub(crate) use alpha::{beta};\npub(crate) use alpha::{gamma};\n";
    assert_eq!(
        normalize_rust_declarations(folded).unwrap(),
        "pub(crate) use alpha::{beta, gamma};\n",
        "a fold keeps the visibility the branches agreed on"
    );
    let error =
        normalize_rust_declarations("pub use alpha::{beta};\npub(crate) use alpha::{gamma};\n")
            .unwrap_err();
    assert!(error.contains("more than one visibility"), "{error}");
}

#[test]
fn repeated_reexports_of_one_module_are_folded_into_a_single_block() {
    let unioned = "pub use zulu::Later;\npub use alpha::{beta};\npub use alpha::{gamma};\n";
    assert_eq!(
        normalize_rust_declarations(unioned).unwrap(),
        "pub use alpha::{beta, gamma};\npub use zulu::Later;\n",
        "two branches extending the same re-export merge into one sorted block"
    );
}

#[test]
fn a_nested_reexport_path_keeps_its_middle_segment() {
    // The bug this test exists for: keyed on the first segment alone,
    // `learning_report::self_hosting_learning::{..}` folded into
    // `learning_report::{..}` and the middle segment vanished, so the file
    // stopped compiling. rustfmt sorts the brace group last, and so does the
    // key, because `{` is above every letter in ASCII.
    let source = "pub use learning_report::self_hosting_learning::{is_task, PATH};\n\
                  pub use learning_report::{LearningReport, REPORTS};\n";
    assert_eq!(normalize_rust_declarations(source).unwrap(), source);
}

#[test]
fn a_reexport_that_appears_once_keeps_the_bytes_rustfmt_gave_it() {
    // rustfmt owns the wrapping of a `use` group; re-deriving it here would
    // fight `cargo fmt --check` over lines this script has no reason to touch.
    let formatted =
        "pub use zulu::{\n    a_value, ZuluType, ZULU_CONSTANT,\n};\npub use alpha::beta;\n";
    assert_eq!(
        normalize_rust_declarations(formatted).unwrap(),
        "pub use alpha::beta;\npub use zulu::{\n    a_value, ZuluType, ZULU_CONSTANT,\n};\n",
        "blocks are reordered, never reflowed"
    );
}

#[test]
fn merged_items_use_the_rustfmt_group_order() {
    let unioned = "pub use alpha::{ZULU_CONST, TypeB};\npub use alpha::{a_function, TypeA};\n";
    assert_eq!(
        normalize_rust_declarations(unioned).unwrap(),
        "pub use alpha::{a_function, TypeA, TypeB, ZULU_CONST};\n",
        "values, then types, then screaming-case constants"
    );
}

#[test]
fn a_long_reexport_wraps_the_way_rustfmt_wraps_it() {
    let items: Vec<String> = (0..12)
        .map(|index| format!("SomeLongTypeName{index:02}"))
        .collect();
    let lines = render_reexport("pub", "module", &items);
    assert_eq!(lines[0], "pub use module::{");
    assert_eq!(lines[lines.len() - 1], "};");
    assert!(
        lines.iter().all(|line| line.len() <= 100),
        "rustfmt keeps every line inside the 100 column limit"
    );
    assert!(lines[1].starts_with("    SomeLongTypeName00,"));
}

#[test]
fn rustfmt_orders_values_before_types() {
    let mut items = vec![
        "TypeB".to_string(),
        "value_a".to_string(),
        "TypeA".to_string(),
        "value_b".to_string(),
    ];
    items.sort_by(|left, right| rustfmt_item_key(left).cmp(&rustfmt_item_key(right)));
    assert_eq!(items, ["value_a", "value_b", "TypeA", "TypeB"]);
}

#[test]
fn a_path_attribute_names_the_directory_it_points_at() {
    let source = "mod alpha;\n\n#[path = \"ci-cd/mod.rs\"]\nmod ci_cd;\n";
    assert_eq!(declared_modules(source), ["alpha", "ci-cd"]);
}

#[test]
fn a_stale_list_fails_the_check_and_is_repaired_by_write() {
    let root = temp_dir("stale");
    write(&root, "src/lib.rs", "mod zulu;\nmod alpha;\n");
    let file = ListFile {
        path: "src/lib.rs".to_string(),
        list_kind: "rust_declarations".to_string(),
        declares_directory: None,
        inert: Vec::new(),
    };

    assert_eq!(process(&root, &file, false), Outcome::Stale);
    assert_eq!(process(&root, &file, true), Outcome::Rewritten);
    assert_eq!(
        fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        "mod alpha;\nmod zulu;\n"
    );
    assert_eq!(process(&root, &file, false), Outcome::Unchanged);
}

#[test]
fn an_undeclared_module_file_fails_the_directory_mirror_check() {
    let root = temp_dir("mirror");
    write(&root, "tests/unit/alpha.rs", "// alpha\n");
    write(&root, "tests/unit/beta.rs", "// beta\n");
    write(&root, "tests/unit/specification/mod.rs", "// nested\n");
    write(&root, "tests/unit/mod.rs", "mod alpha;\n");
    let file = ListFile {
        path: "tests/unit/mod.rs".to_string(),
        list_kind: "rust_declarations".to_string(),
        declares_directory: Some("tests/unit".to_string()),
        inert: Vec::new(),
    };

    assert_eq!(
        module_sources(&root.join("tests/unit")),
        ["alpha", "beta", "specification"]
    );
    let Outcome::Failed(error) = process(&root, &file, false) else {
        panic!("an undeclared module must fail the check");
    };
    assert!(error.contains("beta"), "{error}");
    assert!(error.contains("specification"), "{error}");
}

#[test]
fn a_module_the_registry_marks_inert_is_a_draft_not_drift() {
    // Plan 16 leaf L1 is drafted as tests before the layout exists, so its
    // file sits in tests/unit/ unregistered. The mirror must stay exact for
    // every compiled module without forcing the draft to be registered —
    // registering it early would only land a red test in CI.
    let root = temp_dir("inert");
    write(&root, "tests/unit/alpha.rs", "// alpha\n");
    write(
        &root,
        "tests/unit/issue_1138_three_source_roots.rs",
        "// inert draft\n",
    );
    write(&root, "tests/unit/mod.rs", "mod alpha;\n");
    let file = ListFile {
        path: "tests/unit/mod.rs".to_string(),
        list_kind: "rust_declarations".to_string(),
        declares_directory: Some("tests/unit".to_string()),
        inert: vec!["issue_1138_three_source_roots".to_string()],
    };

    assert_eq!(process(&root, &file, false), Outcome::Unchanged);

    // The exemption is per module, not per file: a second unregistered
    // module is still drift.
    write(&root, "tests/unit/beta.rs", "// beta\n");
    let Outcome::Failed(error) = process(&root, &file, false) else {
        panic!("an undeclared module outside the inert list must fail");
    };
    assert!(error.contains("beta"), "{error}");
    assert!(!error.contains("three_source_roots"), "{error}");
}

#[test]
fn the_worker_list_is_regenerated_from_the_directory_it_mirrors() {
    let root = temp_dir("worker");
    write(&root, "js/worker/formal_ai_worker.js", "// entry\n");
    write(
        &root,
        "js/worker/formal_ai_worker_seed_responses_and_language.js",
        "// zero\n",
    );
    write(&root, "js/worker/how_to_guide.js", "// guide\n");
    write(&root, "js/worker/notes.md", "not a module\n");
    write(
        &root,
        "js/worker-modules.js",
        "self.FORMAL_AI_WORKER_MODULES = [];\n",
    );
    let file = ListFile {
        path: "js/worker-modules.js".to_string(),
        list_kind: "js_module_list".to_string(),
        declares_directory: Some("js/worker".to_string()),
        inert: Vec::new(),
    };

    assert_eq!(
        js_modules_in(&root.join("js/worker")),
        [
            "formal_ai_worker_seed_responses_and_language.js",
            "how_to_guide.js"
        ],
        "only JavaScript modules are listed, and the entry point that loads the list is not a module in it"
    );
    assert_eq!(process(&root, &file, true), Outcome::Rewritten);
    let rendered = fs::read_to_string(root.join("js/worker-modules.js")).unwrap();
    assert!(rendered.contains("\"worker/formal_ai_worker_seed_responses_and_language.js\""));
    assert!(rendered.contains("\"worker/how_to_guide.js\""));
    assert!(!rendered.contains("notes.md"));
    assert_eq!(process(&root, &file, false), Outcome::Unchanged);
}

#[test]
fn the_worker_list_is_rendered_from_the_directory_contents() {
    let rendered = render_js_module_list(
        "js/worker",
        &[
            "formal_ai_worker_seed_responses_and_language.js".to_string(),
            "how_to_guide.js".to_string(),
        ],
    );
    assert!(rendered.contains("self.FORMAL_AI_WORKER_MODULES = Object.freeze(["));
    assert!(rendered.contains("  \"worker/formal_ai_worker_seed_responses_and_language.js\",\n"));
    assert!(rendered.contains("  \"worker/how_to_guide.js\",\n"));
    assert!(rendered.ends_with("]);\n"));
}
