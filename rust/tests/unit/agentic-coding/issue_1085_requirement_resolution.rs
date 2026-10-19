//! Issue #1085 (D2.2, D2.3, D4): a requirement that names behaviour resolves to
//! its file through the self-AST census, every ladder leaf is a committed
//! link-edit rule, and the ladder compiles, tests and merges what it verifies.

use std::fs;
use std::path::Path;

use formal_ai::agentic_coding::resolve_requirement_target;
#[cfg(feature = "meta-language")]
use formal_ai::agentic_coding::{LinkEditRule, apply_link_edit, parse_rule_document};

const LADDER: &str = "experiments/issue_1028_agent_cli_ladder";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
}

fn leaves() -> Vec<Vec<String>> {
    fs::read_to_string(root().join(LADDER).join("leaves.tsv"))
        .expect("the ladder leaf table is committed")
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect()
}

#[test]
fn a_requirement_without_a_file_name_resolves_through_the_census() {
    let target = resolve_requirement_target("Add wikiquote as a web search provider.")
        .expect("the census declares one web search provider list");
    assert_eq!(target.module_path, "src/web_search_core.rs");
    assert_eq!(target.symbol, "WEB_SEARCH_PROVIDERS");
    assert_eq!(target.kind, "const");

    let named = resolve_requirement_target("Rename the constant ERROR_JOIN to ERROR_LIST_JOIN.")
        .expect("an identifier named verbatim resolves to its declaring module");
    assert_eq!(named.module_path, "src/cli_context.rs");
    assert_eq!(named.symbol, "ERROR_JOIN");

    assert_eq!(
        resolve_requirement_target("Improve the code."),
        None,
        "a requirement that names nothing resolvable is not guessed"
    );
}

#[test]
fn original_ladder_scope_is_preserved_when_actual_ownership_moves() {
    let rows = leaves();
    assert_eq!(rows.len(), 32);
    for row in &rows {
        let (leaf, path, requirement) = (&row[0], &row[2], &row[5]);
        let scoped_request = if path.starts_with("rust/") {
            &row[1]
        } else {
            requirement
        };
        if leaf == "L09" {
            // The historical request permits only a Rust file; vocabulary now belongs to data.
            // Keep it as a conflict, never as a fulfilled original ladder task.
            assert_eq!(resolve_requirement_target(scoped_request), None);
            // The unscoped native positive remains isolated below as explicit parity debt.
            continue;
        }
        // Preserve the original explicit file operand instead of guessing an ambiguous declaration.
        let target = resolve_requirement_target(scoped_request)
            .unwrap_or_else(|| panic!("{leaf}: {requirement:?} must resolve"));
        if path.starts_with("data/seed/") {
            assert_eq!(
                &target.module_path, "src/seed/embedded_registry.rs",
                "{leaf}: a data-owned requirement resolves through its generated seed declaration"
            );
            // The census records crate-relative module paths; the crate sits
            // under rust/ in the working tree.
            let registry = fs::read_to_string(root().join("rust").join(&target.module_path))
                .unwrap_or_else(|error| panic!("{leaf}: {}: {error}", target.module_path));
            let declaration = format!("pub const {}", target.symbol);
            let offset = registry.find(&declaration).unwrap_or_else(|| {
                panic!(
                    "{leaf}: generated registry does not declare {}",
                    target.symbol
                )
            });
            assert!(
                registry[offset..]
                    .lines()
                    .take(3)
                    .any(|line| line.contains(&format!("../../embedded/{path}"))),
                "{leaf}: {} does not embed {path}",
                target.symbol
            );
        } else {
            // leaves.tsv names tree paths (rust/src/...); the census namespace
            // stays crate-relative (src/...).
            let census_path = path.strip_prefix("rust/").unwrap_or(path);
            assert_eq!(&target.module_path, census_path, "{leaf}: {requirement:?}");
        }
    }
}

#[cfg(feature = "meta-language")]
#[test]
fn committed_leaf_rules_apply_or_refuse_their_original_conflicting_scope() {
    let rows = leaves();
    for row in &rows {
        let leaf = &row[0];
        let text = fs::read_to_string(root().join(LADDER).join(format!("rules/{leaf}.lino")))
            .unwrap_or_else(|error| panic!("{leaf}: rule file must exist: {error}"));
        let document = parse_rule_document(&text).unwrap_or_else(|error| panic!("{leaf}: {error}"));
        assert_eq!(&document.leaf, leaf);
        assert_eq!(
            &document.path, &row[2],
            "{leaf}: the rule targets the leaf file"
        );
        let source = fs::read_to_string(root().join(&document.path))
            .unwrap_or_else(|error| panic!("{leaf}: {}: {error}", document.path));
        if leaf == "L09" {
            let reason = match apply_link_edit(&source, &document.language, &document.rule) {
                Err(error) => error.to_string(),
                Ok(_) => panic!("the original Rust-only rule must not redirect to data"),
            };
            assert_eq!(
                reason,
                "link_edit:no_matching_link:insert_member:PROSE_WORDS"
            );
            assert_eq!(
                fs::read_to_string(root().join(&document.path)).unwrap(),
                source
            );
            continue;
        }
        let (edited, report) = apply_link_edit(&source, &document.language, &document.rule)
            .unwrap_or_else(|error| panic!("{leaf}: {error}"));
        assert_ne!(edited, source, "{leaf}: the rule must change the source");
        assert!(report.clean_after, "{leaf}: the edited network must verify");
        for expected in &document.expect {
            assert!(
                edited.contains(expected.as_str()),
                "{leaf}: edited source must contain {expected:?}"
            );
        }
        let task = &row[1];
        match &document.rule {
            LinkEditRule::InsertMember { .. } => {
                assert!(task.contains(" list."), "{leaf}: {task:?}");
            }
            LinkEditRule::ReplaceLiteral { .. } => {
                assert!(task.contains(", replace "), "{leaf}: {task:?}");
            }
            LinkEditRule::RenameIdentifier { .. } => {
                assert!(task.contains(", rename the constant "), "{leaf}: {task:?}");
            }
        }
    }
}

#[test]
fn the_ladder_compiles_tests_merges_and_verifies_requirement_levels() {
    let verifier =
        fs::read_to_string(root().join(LADDER).join("verify-node.sh")).expect("verifier");
    for needle in [
        "cargo check --lib",
        "cargo test --test unit",
        "unmergeable_child_diffs",
        "uncompilable_composite_change",
        "node_kind=requirement",
        "missing_requirement_change",
    ] {
        assert!(
            verifier.contains(needle),
            "verify-node.sh must contain {needle:?}"
        );
    }
    let runner = fs::read_to_string(root().join(LADDER).join("run.sh")).expect("runner");
    for needle in [
        "leaves.tsv",
        "ladder-result.lino",
        "deepest_passing_level",
        "leaf_nodes_passing",
        "change.diff",
    ] {
        assert!(runner.contains(needle), "run.sh must contain {needle:?}");
    }
    let workflow = fs::read_to_string(root().join(".github/workflows/issue-1028-agent-ladder.yml"))
        .expect("workflow");
    for needle in [
        "pull_request:",
        "schedule:",
        "data/meta/ladder-ratchet.lino",
    ] {
        assert!(
            workflow.contains(needle),
            "the ladder workflow must contain {needle:?}"
        );
    }
    let ratchet =
        fs::read_to_string(root().join("data/meta/ladder-ratchet.lino")).expect("ratchet");
    // What is ratcheted is how many of the 32 leaves Formal AI actually
    // changed. The first run under the compile-and-test criteria passed 15;
    // the record may only rise (issue #1085 D4).
    assert!(ratchet.contains("  leaf_nodes_selected 32\n"));
    let passing = ratchet
        .lines()
        .find_map(|line| line.trim().strip_prefix("leaf_nodes_passing "))
        .and_then(|value| value.parse::<u32>().ok())
        .expect("the ratchet records how many leaves pass");
    assert!(
        (15..=32).contains(&passing),
        "leaf_nodes_passing is {passing}, outside the 15 the first measured run passed and the 32 leaves there are"
    );
}

#[test]
fn the_ladder_can_measure_without_per_leaf_authored_answers() {
    let runner = fs::read_to_string(root().join(LADDER).join("run.sh")).expect("runner");
    for needle in [
        "--no-authored-rules",
        "!/experiments/issue_1028_agent_cli_ladder/rules/",
        "!/experiments/issue_1028_agent_cli_ladder/leaves.tsv",
        "authored_rules_enabled",
        "leaf_nodes_passing_without_authored_rules",
    ] {
        assert!(runner.contains(needle), "run.sh must contain {needle:?}");
    }

    let ratchet =
        fs::read_to_string(root().join("data/meta/ladder-ratchet.lino")).expect("ratchet");
    assert!(ratchet.contains("leaf_nodes_passing_without_authored_rules"));

    let debt = fs::read_to_string(root().join("data/meta/debt-ratchet.lino")).expect("debt");
    assert!(debt.contains("measure authored-ladder-rules\n    value 32"));
}

#[test]
fn canonical_seed_goal_binding_preserves_ambiguity_and_declared_ownership() {
    use formal_ai::agentic_coding::requirement_resolution::{
        RequirementTarget, resolve_seed_target,
    };
    use formal_ai::self_ast_census::WorkspaceCensus;
    let census = WorkspaceCensus::compile(&[
        ("src/a.rs", "pub const ALPHA_LINO: &str = \"source\";"),
        ("src/b.rs", "pub const BETA_LINO: &str = \"source\";"),
    ]);
    let source = "meanings\n  arbitrary\n    role request_signal\n    lexeme en \"azure dawn\"\n";
    let sources = [("data/seed/alpha.lino", source)];
    let expected = Some(RequirementTarget {
        module_path: "src/a.rs".to_owned(),
        symbol: "ALPHA_LINO".to_owned(),
        kind: "const".to_owned(),
    });
    assert_eq!(
        resolve_seed_target(
            &census,
            "Change signal surface from azure dawn to cobalt sun",
            &sources
        ),
        expected
    );
    for request in [
        "Improve the code",
        "Change signal surface from azure dawnish",
        "Change unrelated surface from azure dawn",
    ] {
        assert_eq!(resolve_seed_target(&census, request, &sources), None);
    }
    assert_eq!(
        resolve_seed_target(
            &census,
            "Change signal surface from azure dawn",
            &[("data/seed/missing.lino", source)]
        ),
        None
    );
    assert_eq!(
        resolve_seed_target(
            &census,
            "Change signal surface from azure dawn",
            &[
                ("data/seed/alpha.lino", source),
                ("data/seed/beta.lino", source)
            ]
        ),
        None
    );
    let longer = source.replace("azure dawn", "azure dawn bright");
    assert_eq!(
        resolve_seed_target(
            &census,
            "Change signal surface from azure dawn bright",
            &[
                ("data/seed/alpha.lino", source),
                ("data/seed/beta.lino", &longer)
            ]
        ),
        Some(RequirementTarget {
            module_path: "src/b.rs".to_owned(),
            symbol: "BETA_LINO".to_owned(),
            kind: "const".to_owned()
        })
    );
    assert_eq!(
        resolve_seed_target(
            &census,
            "Change signal surface from azure dawn",
            &[("invalid.txt", source), ("data/seed/alpha.lino", source)]
        ),
        expected
    );
}

#[test]
fn explicit_module_scope_preserves_actual_l17_and_ambiguous_refusal() {
    use formal_ai::agentic_coding::{RequirementTarget, resolve_in};
    use formal_ai::self_ast_census::WorkspaceCensus;
    let census = WorkspaceCensus::compile(&[
        (
            "src/目录/alpha-beta.rs",
            "pub const SHARED: &str = \"same\";",
        ),
        ("src/other_alpha.rs", "pub const SHARED: &str = \"same\";"),
    ]);
    for path in ["src/目录/alpha-beta.rs", "src/other_alpha.rs"] {
        for reference in [
            path.to_owned(),
            format!("rust/{path}"),
            format!("./rust/{path}"),
        ] {
            let expected = Some(RequirementTarget {
                module_path: path.to_owned(),
                symbol: "SHARED".to_owned(),
                kind: "const".to_owned(),
            });
            assert_eq!(
                resolve_in(&census, &format!("In {reference}, rename SHARED.")),
                expected
            );
            assert_eq!(
                resolve_in(&census, &format!("In «{reference}», rename SHARED.")),
                expected
            );
        }
    }
    for request in [
        "Rename SHARED.",
        "In src/missing.rs rename SHARED.",
        "In src/other_alpha.rs and src/目录/alpha-beta.rs rename SHARED.",
        "In ../src/other_alpha.rs rename SHARED.",
        "In prefixsrc/other_alpha.rs rename SHARED.",
        "In src/other_alpha.rs.backup rename SHARED.",
        "In src/other_alpha.rs rename MISSING.",
    ] {
        assert_eq!(resolve_in(&census, request), None, "{request}");
    }
    let row = leaves()
        .into_iter()
        .find(|row| row[0] == "L17")
        .expect("original leaf");
    assert_eq!(
        resolve_requirement_target(&row[1]),
        Some(RequirementTarget {
            module_path: "src/web_search_fusion_core.rs".to_owned(),
            symbol: "NEGATION_ROLE".to_owned(),
            kind: "const".to_owned(),
        })
    );
    assert_eq!(resolve_requirement_target(&row[5]), None);
}

#[test]
fn scoped_literal_owner_is_unique_and_requires_complete_scalar_initializer() {
    use formal_ai::agentic_coding::{RequirementTarget, resolve_in};
    use formal_ai::self_ast_census::WorkspaceCensus;
    let request =
        "In the file rust/src/literal_owner.rs, replace \"azure dawn\" with \"cobalt sun\".";
    let source = "pub const VALUE: &str = \"azure dawn\";\n";
    let census = WorkspaceCensus::compile(&[("src/literal_owner.rs", source)]);
    assert_eq!(census.modules[0].source(), source);
    assert_eq!(
        resolve_in(&census, request),
        Some(RequirementTarget {
            module_path: "src/literal_owner.rs".to_owned(),
            symbol: "VALUE".to_owned(),
            kind: "const".to_owned(),
        })
    );
    for source in [
        "pub const VALUE: &str = \"other words\";",
        "pub const VALUE: &str = \"azure dawn\";\npub const SECOND: &str = \"azure dawn\";",
        "pub const VALUE: &str = /* \"azure dawn\" */ \"other\";",
        "pub const VALUE: &str = \"azure dawn\".trim();",
        "pub const VALUE: &[&str] = &[\"azure dawn\"];",
        "pub const VALUE: &str = \"azure dawn bright\";",
    ] {
        let census = WorkspaceCensus::compile(&[("src/literal_owner.rs", source)]);
        assert_eq!(resolve_in(&census, request), None, "{source}");
    }
    assert_eq!(
        resolve_in(
            &census,
            "Inspect rust/src/literal_owner.rs and the words \"azure dawn\"."
        ),
        None
    );
}

#[test]
fn canonical_surface_requires_role_evidence_outside_its_own_span() {
    use formal_ai::agentic_coding::requirement_resolution::{
        RequirementTarget, resolve_seed_target,
    };
    use formal_ai::self_ast_census::WorkspaceCensus;
    let census =
        WorkspaceCensus::compile(&[("src/a.rs", "pub const ALPHA_LINO: &str = \"source\";")]);
    let expected = Some(RequirementTarget {
        module_path: "src/a.rs".to_owned(),
        symbol: "ALPHA_LINO".to_owned(),
        kind: "const".to_owned(),
    });
    for (surface, role, negative, positive) in [
        (
            "signal",
            "request_signal",
            "Improve signal.",
            "Change request surface from signal.",
        ),
        (
            "code",
            "script_or_code_artifact",
            "Improve the code.",
            "Change artifact surface from code.",
        ),
        (
            "request signal",
            "request_signal",
            "Improve request signal.",
            "Change signal surface from request signal.",
        ),
        (
            "返回结果",
            "request_signal",
            "Improve 返回结果.",
            "Change request surface from 返回结果.",
        ),
    ] {
        let seed = format!("meanings\n  arbitrary\n    role {role}\n    lexeme en \"{surface}\"\n");
        let sources = [("data/seed/alpha.lino", seed.as_str())];
        assert_eq!(
            resolve_seed_target(&census, negative, &sources),
            None,
            "{negative}"
        );
        assert_eq!(
            resolve_seed_target(&census, positive, &sources),
            expected,
            "{positive}"
        );
    }
}

/// Retain the exact unresolved native positive independently of the original scoped refusals.
#[test]
fn native_unscoped_l09_resolves_actual_terminal_commands_owner() {
    let rows = leaves();
    assert_eq!(rows.len(), 32);
    let row = rows
        .iter()
        .find(|row| row[0] == "L09")
        .expect("original L09");
    let requirement = &row[5];
    let current = resolve_requirement_target(requirement)
        .expect("the unscoped semantic request identifies actual shared seed ownership");
    assert_eq!(current.module_path, "src/seed/embedded_registry.rs");
    assert_eq!(current.symbol, "TERMINAL_COMMANDS_LINO");
    assert_eq!(current.kind, "const");
}

#[test]
fn explicit_seed_fields_constrain_weak_lexical_surfaces_without_guessing() {
    use formal_ai::agentic_coding::requirement_resolution::{
        RequirementTarget, resolve_seed_target, structured_seed_owner,
    };
    use formal_ai::self_ast_census::WorkspaceCensus;
    let census = WorkspaceCensus::compile(&[
        (
            "src/config.rs",
            "pub const COBALT_CONFIG_LINO: &str = \"\";",
        ),
        ("src/words.rs", "pub const AUXILIARY_LINO: &str = \"\";"),
        (
            "src/archive.rs",
            "pub const COBALT_ARCHIVE_LINO: &str = \"\";",
        ),
    ]);
    let wanted = Some(RequirementTarget {
        module_path: "src/config.rs".to_owned(),
        symbol: "COBALT_CONFIG_LINO".to_owned(),
        kind: "const".to_owned(),
    });
    let settings = (
        "data/seed/cobalt-config.lino",
        "settings\n  data-policy\n    value preserve\n",
    );
    let weak = (
        "data/seed/auxiliary.lino",
        "meanings\n  connector\n    role request_function_word\n    lexeme en the\n",
    );
    for request in [
        "Update cobalt data policy with the new word.",
        "Обнови data-policy для cobalt и слово the.",
        "cobalt का data-policy बदलें और the शब्द रखें।",
        "更新 cobalt 的 data-policy，并保留单词 the。",
        "Actualiza data-policy para cobalt y conserva la palabra the.",
    ] {
        assert_eq!(
            resolve_seed_target(&census, request, &[settings, weak]),
            wanted,
            "{request}"
        );
    }
    assert_eq!(
        structured_seed_owner(&census, "Update cobalt data-policy.", &[settings, weak]),
        wanted
    );
    let request = "Update cobalt data policy with the new word.";
    assert_eq!(
        resolve_seed_target(
            &census,
            request,
            &[
                settings,
                weak,
                (
                    "data/seed/cobalt-unsupported.lino",
                    "settings\n  data-policy-new-word\n"
                ),
            ]
        ),
        wanted,
        "an undeclared longer field cannot erase a qualified owner"
    );
    assert_eq!(
        resolve_seed_target(
            &census,
            request,
            &[
                settings,
                weak,
                ("data/seed/cobalt-archive.lino", "settings\n  data-policy\n"),
            ]
        ),
        None,
        "equal explicit owners cannot authorize lexical fallback"
    );
    for request in [
        "Update scarlet data-policy.",
        "Update cobalt with \"data-policy\".",
    ] {
        assert_eq!(
            resolve_seed_target(&census, request, &[settings, weak]),
            None,
            "{request}"
        );
    }
    let wrong_kind =
        WorkspaceCensus::compile(&[("src/config.rs", "pub fn COBALT_CONFIG_LINO() {}")]);
    assert_eq!(
        resolve_seed_target(&wrong_kind, "Update cobalt data-policy.", &[settings, weak]),
        None
    );
}
