//! Plan 16 L2's translation tool, first rung (issue #1138): the any-direction
//! dispatcher through the meta pivot, and its honest-gap contract — a leg that
//! is not materialized names the plan-16 leaf that owes it instead of
//! silently producing nothing.

use formal_ai::meta_translate::{self, SourceRoot, TranslationOutcome};

#[test]
fn every_translate_cli_text_lives_in_the_seed() {
    for intent in [
        "translate_needs_from_to",
        "translate_unknown_from",
        "translate_unknown_to",
        "translate_same_roots",
        "translate_pending",
        "translate_pending_docs",
        "translate_needs_input",
        "translate_leg_live",
        "translate_leg_pending",
        "translate_refused",
        // Plan 16 L2g: the write-mode report texts.
        "translate_wrote_file",
        "translate_wrote",
        "translate_write_wrong_root",
        "translate_write_unsupported",
        "translate_write_refused",
        "translate_source_missing",
        "translate_source_invalid",
        "translate_write_empty",
        // The per-file refusal item and the agent tool's schema row.
        "translate_write_refused_item",
        "translate_tool_schema",
    ] {
        assert!(
            formal_ai::response_for(intent, "en").is_some(),
            "intent {intent} must live in data/seed/multilingual-responses-translate lino"
        );
    }
    assert_eq!(
        formal_ai::render_response(
            "translate_leg_live",
            "en",
            &[("from", "rust"), ("to", "meta")]
        ),
        Some("rust → meta  live".to_owned())
    );
    assert_eq!(
        formal_ai::render_response(
            "translate_leg_pending",
            "en",
            &[("from", "rust"), ("to", "js"), ("leaf", "L8")]
        ),
        Some("rust → js  pending (plan 16 L8)".to_owned())
    );
    assert_eq!(
        formal_ai::render_response(
            "translate_write_refused_item",
            "en",
            &[
                ("source", "ts/typed.ts"),
                ("constructs", "interface_declaration")
            ]
        ),
        Some("ts/typed.ts: interface_declaration".to_owned())
    );
}

/// Plan 16 L2g: the sibling-root write mapping is a total contract on the
/// two ES roots and refuses everything else by name.
#[test]
fn write_target_maps_the_sibling_roots_and_refuses_the_rest() {
    use formal_ai::meta_translate::{SourceRoot, WriteTargetError, write_target};
    assert_eq!(
        write_target(
            SourceRoot::JavaScript,
            SourceRoot::TypeScript,
            "js/app/foo.js"
        ),
        Ok("ts/app/foo.ts".to_owned())
    );
    assert_eq!(
        write_target(SourceRoot::TypeScript, SourceRoot::JavaScript, "ts/one.ts"),
        Ok("js/one.js".to_owned())
    );
    for wrong in [
        "src/app.js",
        "js/../secret.js",
        "js/catalog.lino",
        "app.js",
        "/js/a.js",
    ] {
        assert_eq!(
            write_target(SourceRoot::JavaScript, SourceRoot::TypeScript, wrong),
            Err(WriteTargetError::WrongRoot {
                path: wrong.to_owned()
            }),
            "{wrong} must be a wrong root, not a mapped write"
        );
    }
    assert_eq!(
        write_target(SourceRoot::JavaScript, SourceRoot::Meta, "js/app.js"),
        Err(WriteTargetError::UnsupportedLeg {
            from: SourceRoot::JavaScript,
            to: SourceRoot::Meta,
        }),
        "the pivot document is not a committed tree to write"
    );
    assert_eq!(
        write_target(SourceRoot::Rust, SourceRoot::Meta, "rust/src/lib.rs"),
        Err(WriteTargetError::UnsupportedLeg {
            from: SourceRoot::Rust,
            to: SourceRoot::Meta,
        })
    );
}

/// Plan 16 L2g: the recognizer reads only structural vocabulary — a path
/// token under a source root and the target's canonical spelling.
#[test]
fn source_tree_requests_recognize_paths_and_targets() {
    use formal_ai::meta_translate::{SourceRoot, source_tree_request};
    assert_eq!(
        source_tree_request("Translate js/app/foo.js to TypeScript and write it"),
        Some(formal_ai::meta_translate::SourceTreeRequest {
            from: SourceRoot::JavaScript,
            to: SourceRoot::TypeScript,
            path: "js/app/foo.js".to_owned(),
        })
    );
    assert_eq!(
        source_tree_request("translate ts/one.ts to javascript"),
        Some(formal_ai::meta_translate::SourceTreeRequest {
            from: SourceRoot::TypeScript,
            to: SourceRoot::JavaScript,
            path: "ts/one.ts".to_owned(),
        })
    );
    // No path token, no target window, or the same root twice: not ours. A
    // verb like "summarize" is deliberately NOT screened here — the meaning
    // gate belongs to the solver arm that calls this recognizer.
    assert_eq!(source_tree_request("translate \"apple\" to russian"), None);
    assert_eq!(source_tree_request("translate js/a.js to javascript"), None);
    assert_eq!(
        source_tree_request("summarize js/app.js for the typescript review"),
        None
    );
}

/// Plan 16 L2g: one mapped file writes its sibling target; the report is
/// seed-rendered and refuses wrong roots and unsupported legs without
/// touching the tree.
#[test]
fn write_one_writes_the_mapped_sibling_file() {
    use formal_ai::meta_translate::SourceRoot;
    use formal_ai::translate_write::{self, WriteReport};
    let root = temp_source_tree(&[
        ("js/greeting.js", "export const greeting = \"hi\";\n"),
        ("js/notes/readme.js", "export const note = 1;\n"),
    ]);
    let report = translate_write::write_one(
        SourceRoot::JavaScript,
        SourceRoot::TypeScript,
        &root,
        "js/greeting.js",
    );
    let translate_write::WriteReport::WroteOne(file) = &report else {
        panic!("the mapped file must write, got {report:?}");
    };
    assert_eq!(file.source, "js/greeting.js");
    assert_eq!(file.target, "ts/greeting.ts");
    assert!(
        file.carried >= 5,
        "the whole token tree must cross the pivot: {carried}",
        carried = file.carried
    );
    let written = std::fs::read_to_string(root.join("ts/greeting.ts")).expect("the mapped file");
    assert!(written.contains("export const greeting"));
    assert!(
        !root.join("ts/notes").exists(),
        "an untouched file must not create its target directories"
    );
    assert_eq!(
        translate_write::write_one(
            SourceRoot::JavaScript,
            SourceRoot::TypeScript,
            &root,
            "js/../escape.js"
        ),
        WriteReport::WrongRoot {
            path: "js/../escape.js".to_owned()
        }
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Plan 16 L2g: a tree write is transactional — one refused file writes
/// nothing, because a half-generated sibling tree is the corrupt state the
/// L3 mismatch check exists to refuse.
#[test]
fn write_tree_is_transactional_on_refusal() {
    use formal_ai::meta_translate::SourceRoot;
    use formal_ai::translate_write::{self, WriteReport};
    // `ts → js` is the direction with a real refusal vocabulary: the
    // projection seed refuses TypeScript-only constructs when the target
    // is plain JavaScript.
    let root = temp_source_tree(&[
        ("ts/plain.ts", "export const plain = 1;\n"),
        (
            "ts/typed.ts",
            "interface Shape { sides: number }\nexport const shape: Shape = { sides: 3 };\n",
        ),
    ]);
    let report = translate_write::write_tree(SourceRoot::TypeScript, SourceRoot::JavaScript, &root);
    let WriteReport::Refused { items } = &report else {
        panic!("the typed file must be refused by name, got {report:?}");
    };
    assert!(
        items.iter().any(|item| item.starts_with("ts/typed.ts")),
        "the refused item names its file: {items:?}"
    );
    assert!(
        !root.join("js").exists(),
        "a refused tree writes nothing at all"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Plan 16 L2g: a clean tree writes every owned file and skips the assets
/// the translator does not own.
#[test]
fn write_tree_writes_every_owned_file() {
    use formal_ai::meta_translate::SourceRoot;
    use formal_ai::translate_write::{self, WriteReport};
    let root = temp_source_tree(&[
        ("js/one.js", "export const one = 1;\n"),
        ("js/nested/two.js", "export const two = 2;\n"),
        ("js/landing.css", "body { margin: 0 }\n"),
    ]);
    let report = translate_write::write_tree(SourceRoot::JavaScript, SourceRoot::TypeScript, &root);
    let WriteReport::Wrote { files, to } = &report else {
        panic!("the clean tree must write, got {report:?}");
    };
    assert_eq!(*to, SourceRoot::TypeScript);
    let mut targets = files
        .iter()
        .map(|file| file.target.as_str())
        .collect::<Vec<_>>();
    targets.sort_unstable();
    assert_eq!(targets, ["ts/nested/two.ts", "ts/one.ts"]);
    assert!(root.join("ts/nested/two.ts").is_file());
    assert!(
        !root.join("ts/landing.css").exists(),
        "assets are not the translator's"
    );
    let _ = std::fs::remove_dir_all(&root);
}

fn temp_source_tree(files: &[(&str, &str)]) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "formal-ai-l2g-write-{}-{:p}",
        std::process::id(),
        std::ptr::from_ref(files)
    ));
    let _ = std::fs::remove_dir_all(&root);
    for (path, content) in files {
        let full = root.join(path);
        std::fs::create_dir_all(full.parent().expect("a parent")).expect("create the directory");
        std::fs::write(full, content).expect("seed the source file");
    }
    root
}

#[test]
fn every_distinct_pair_is_listed_exactly_once() {
    let directions = meta_translate::directions();
    assert_eq!(
        directions.len(),
        12,
        "four roots give twelve directed pairs, and every pair is a real direction"
    );
    let mut seen = std::collections::HashSet::new();
    for (from, to, pending) in &directions {
        assert_ne!(from, to);
        assert!(
            seen.insert((*from, *to)),
            "duplicate direction {from:?} → {to:?}"
        );
        assert_ne!(
            *pending,
            Some("L0"),
            "a listed direction is never a same-root non-direction"
        );
    }
}

#[test]
fn rust_to_meta_is_the_live_signature_leg() {
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::Rust, SourceRoot::Meta),
        None
    );
    let source = "fn answer() -> u32 {\n    41 + 1\n}\n";
    match meta_translate::translate(SourceRoot::Rust, SourceRoot::Meta, "probe.rs", source) {
        TranslationOutcome::Rendered { target, .. } => {
            assert!(target.contains("self_ast"));
            assert!(target.contains("target probe.rs"));
            assert!(target.contains("language rust"));
            assert!(target.contains("engine meta_language"));
            assert!(target.contains("named_node_count"));
        }
        other @ (TranslationOutcome::Pending { .. }
        | TranslationOutcome::Refused { .. }
        | TranslationOutcome::Invalid { .. }) => {
            panic!("rust → meta must render the self-AST document; got {other:?}")
        }
    }
}

#[test]
fn pending_legs_name_their_plan_leaf() {
    for (from, to, pending) in meta_translate::directions() {
        let Some(leaf) = pending else { continue };
        assert!(
            leaf.starts_with('L') && leaf.len() >= 2 && leaf[1..].chars().all(char::is_numeric),
            "leaf {leaf:?} for {from:?} → {to:?} must name a plan 16 leaf"
        );
    }
    // Plan 16 L2 opened the ES quadrant: js/ts ↔ meta and js ↔ ts carry
    // through the token-tree pivot, L7 added meta → rust through the
    // lossless network serialization, and L8's grammar projection rules
    // carry js/ts → rust (every corpus kind is ruled, spliced or declared
    // no-form, so the walk renders or refuses by name). The rust → js/ts
    // renders are still owed by L8, whose rule table leaves a
    // type-position tail.
    for (from, to) in [
        (SourceRoot::JavaScript, SourceRoot::Meta),
        (SourceRoot::Meta, SourceRoot::JavaScript),
        (SourceRoot::TypeScript, SourceRoot::Meta),
        (SourceRoot::Meta, SourceRoot::TypeScript),
        (SourceRoot::JavaScript, SourceRoot::TypeScript),
        (SourceRoot::TypeScript, SourceRoot::JavaScript),
        (SourceRoot::JavaScript, SourceRoot::Rust),
        (SourceRoot::TypeScript, SourceRoot::Rust),
    ] {
        assert_eq!(
            meta_translate::pending_leg(from, to),
            None,
            "{from:?} → {to:?} is a delivered plan 16 leg and must be live"
        );
    }
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::Rust, SourceRoot::JavaScript),
        Some("L8")
    );
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::Meta, SourceRoot::Rust),
        None
    );
    // A same-root call is a non-direction, not a leaf.
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::Rust, SourceRoot::Rust),
        Some("L0")
    );
}

#[test]
fn js_to_rust_renders_the_plain_subset_and_refuses_objects_by_name() {
    // The plain subset — declarations, calls, arithmetic — projects into
    // rust through the L8 grammar rules.
    let source = "function double(x) {\nreturn x * 2;\n}\n";
    match meta_translate::translate(
        SourceRoot::JavaScript,
        SourceRoot::Rust,
        "double.js",
        source,
    ) {
        TranslationOutcome::Rendered { target, .. } => {
            assert!(
                target.contains("fn double(x)"),
                "the function declaration projects: {target}"
            );
            assert!(
                target.contains("return x * 2;"),
                "the return carries its expression: {target}"
            );
        }
        other @ (TranslationOutcome::Pending { .. }
        | TranslationOutcome::Refused { .. }
        | TranslationOutcome::Invalid { .. }) => {
            panic!("js → rust must project the plain subset; got {other:?}")
        }
    }
    // An object literal has no rust spelling: the leg must refuse it by
    // name, not guess a struct the construct does not carry.
    let refused = "const point = { x: 1, y: 2 };\n";
    match meta_translate::translate(
        SourceRoot::JavaScript,
        SourceRoot::Rust,
        "point.js",
        refused,
    ) {
        TranslationOutcome::Refused { refusals } => {
            assert!(
                refusals.iter().any(|refusal| refusal.construct == "object"),
                "the object literal must be refused by name: {refusals:?}"
            );
        }
        other @ (TranslationOutcome::Rendered { .. }
        | TranslationOutcome::Pending { .. }
        | TranslationOutcome::Invalid { .. }) => {
            panic!("js → rust must refuse the object literal; got {other:?}")
        }
    }
}

#[test]
fn ts_to_rust_renders_and_refuses_by_name() {
    let source = "function twice(n: number) {\nreturn n + n;\n}\n";
    match meta_translate::translate(SourceRoot::TypeScript, SourceRoot::Rust, "twice.ts", source) {
        TranslationOutcome::Rendered { target, .. } => {
            // The annotation carries verbatim: `n: number` is grammatical
            // rust (a path type), and carrying it loses nothing.
            assert!(
                target.contains("fn twice(n: number)"),
                "the typed declaration projects: {target}"
            );
        }
        other @ (TranslationOutcome::Pending { .. }
        | TranslationOutcome::Refused { .. }
        | TranslationOutcome::Invalid { .. }) => {
            panic!("ts → rust must project the plain subset; got {other:?}")
        }
    }
    let refused = "try {\nstep();\n} catch (e) {\n}\n";
    match meta_translate::translate(SourceRoot::TypeScript, SourceRoot::Rust, "try.ts", refused) {
        TranslationOutcome::Refused { refusals } => {
            assert!(
                refusals
                    .iter()
                    .any(|refusal| refusal.construct == "try_statement"),
                "the try statement must be refused by name: {refusals:?}"
            );
        }
        other @ (TranslationOutcome::Rendered { .. }
        | TranslationOutcome::Pending { .. }
        | TranslationOutcome::Invalid { .. }) => {
            panic!("ts → rust must refuse the try statement; got {other:?}")
        }
    }
}

#[test]
fn js_to_ts_carries_through_the_pivot() {
    match meta_translate::translate(
        SourceRoot::JavaScript,
        SourceRoot::TypeScript,
        "app.js",
        "export const x = 1;\n",
    ) {
        TranslationOutcome::Rendered { target, carried } => {
            assert!(carried >= 5, "five tokens must cross: {carried}");
            assert!(
                target.contains("export const x = 1"),
                "the token-faithful carry renders the tokens: {target}"
            );
        }
        other @ (TranslationOutcome::Pending { .. }
        | TranslationOutcome::Refused { .. }
        | TranslationOutcome::Invalid { .. }) => {
            panic!("js → ts must carry through the pivot; got {other:?}")
        }
    }
}

#[test]
fn source_root_names_round_trip() {
    for root in [
        SourceRoot::Rust,
        SourceRoot::JavaScript,
        SourceRoot::TypeScript,
        SourceRoot::Meta,
    ] {
        assert_eq!(SourceRoot::parse(root.name()), Some(root));
    }
    assert_eq!(SourceRoot::parse("rs"), Some(SourceRoot::Rust));
    assert_eq!(
        SourceRoot::parse("javascript"),
        Some(SourceRoot::JavaScript)
    );
    assert_eq!(
        SourceRoot::parse("typescript"),
        Some(SourceRoot::TypeScript)
    );
    assert_eq!(SourceRoot::parse("lino"), Some(SourceRoot::Meta));
    assert_eq!(SourceRoot::parse(""), None);
    assert_eq!(SourceRoot::parse("python"), None);
}
