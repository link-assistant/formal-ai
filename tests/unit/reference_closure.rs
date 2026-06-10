use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use formal_ai::seed::lexicon;

/// The canonical reserved-role registry, relative to the crate root.
const ROLE_REGISTRY: &str = "data/seed/roles.lino";

// ---------------------------------------------------------------------------
// Issue #398 (PR #399 review comment 4668342875): the seed must be a *closed,
// consistent links network*. Two tree-wide invariants enforce that here:
//
//   1. Reference closure — every `defined-by`/facet target a meaning names must
//      itself be a defined meaning (review point #1 / CI check #1). The earlier
//      `semantic_definition_graph_is_closed` only walked `defined_by`; these
//      tests also close the four semantic facets and the role namespace.
//   2. Role reconciliation — every `role <name>` is declared once in the
//      reserved-role registry `data/seed/roles.lino`, and the registry never
//      drifts from usage (review point #2 / CI check #3). A role is a predicate
//      over meanings, so it gets one definition and cannot silently collide
//      with the meaning graph.
// ---------------------------------------------------------------------------

/// Every slug a loaded meaning declares, including the colon-head roots in
/// `meanings-links-root.lino` (the lexicon parser resolves those, so reusing it
/// keeps this check faithful to what the runtime actually sees).
fn defined_meaning_slugs() -> BTreeSet<String> {
    lexicon()
        .meanings
        .iter()
        .map(|meaning| meaning.slug.clone())
        .collect()
}

/// Distinct `role <name>` values declared by the loaded meanings, mapped to
/// whether the role name is *also* a defined meaning slug.
fn role_usage() -> BTreeMap<String, bool> {
    let lex = lexicon();
    let defined = defined_meaning_slugs();
    let mut roles = BTreeMap::new();
    for meaning in &lex.meanings {
        for role in &meaning.roles {
            roles.insert(role.clone(), defined.contains(role));
        }
    }
    roles
}

/// Parse the reserved-role registry into `name -> kind` ("meaning"/"predicate").
///
/// The file is a flat, generated two-space-indented block (a `<role>` head at
/// indent 2 followed by a single `kind <value>` child at indent 4), so a line
/// reader is both sufficient and immune to the canonical AST shape. The file's
/// canonical-parser validity is covered by
/// `lino_data_files_are_parseable_human_readable_and_bounded`.
fn role_registry() -> BTreeMap<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(ROLE_REGISTRY);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let mut registry = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "roles" {
            continue;
        }
        let indent = line
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        if indent == 2 {
            current = Some(trimmed.to_string());
            registry.entry(trimmed.to_string()).or_default();
        } else if indent == 4 {
            if let Some(role) = &current {
                if let Some(kind) = trimmed.strip_prefix("kind ") {
                    registry.insert(role.clone(), kind.trim().to_string());
                }
            }
        }
    }
    registry
}

#[test]
fn meaning_definition_references_resolve_to_defined_meanings() {
    let lex = lexicon();
    let defined = defined_meaning_slugs();
    let mut dangling = Vec::new();

    for meaning in &lex.meanings {
        for target in &meaning.defined_by {
            if !defined.contains(target) {
                dangling.push(format!("{} defined-by {target}", meaning.slug));
            }
        }
        for facet in &meaning.semantic_facets {
            if !defined.contains(&facet.kind) {
                dangling.push(format!("{} facet kind `{}`", meaning.slug, facet.kind));
            }
            for target in &facet.meanings {
                if !defined.contains(target) {
                    dangling.push(format!("{} {} {target}", meaning.slug, facet.kind));
                }
            }
        }
    }

    assert!(
        dangling.is_empty(),
        "meaning graph has {} reference(s) that resolve to no defined meaning \
         (every defined-by/facet target must be a defined meaning slug):\n{}",
        dangling.len(),
        dangling.join("\n")
    );
}

#[test]
fn every_role_value_is_declared_in_the_registry() {
    let registry = role_registry();
    let mut unregistered: Vec<String> = role_usage()
        .keys()
        .filter(|role| !registry.contains_key(*role))
        .cloned()
        .collect();
    unregistered.sort();

    assert!(
        unregistered.is_empty(),
        "{} role value(s) are used in data/seed/meanings*.lino but not declared \
         in {ROLE_REGISTRY} (run `python3 scripts/generate-role-registry.py`):\n{}",
        unregistered.len(),
        unregistered.join("\n")
    );
}

#[test]
fn role_registry_is_in_lockstep_with_usage() {
    let registry = role_registry();
    let usage = role_usage();

    let stale: Vec<&String> = registry
        .keys()
        .filter(|role| !usage.contains_key(*role))
        .collect();
    assert!(
        stale.is_empty(),
        "{ROLE_REGISTRY} declares role(s) no meaning uses (regenerate the \
         registry): {stale:?}"
    );

    let mut mismatched = Vec::new();
    for (role, is_meaning) in &usage {
        let expected = if *is_meaning { "meaning" } else { "predicate" };
        match registry.get(role) {
            Some(kind) if kind == expected => {}
            Some(kind) => mismatched.push(format!(
                "{role}: registry says `{kind}`, usage says `{expected}`"
            )),
            None => {}
        }
    }
    assert!(
        mismatched.is_empty(),
        "{ROLE_REGISTRY} kind classification drifted from usage \
         (a role name that is also a defined meaning slug is `meaning`, else \
         `predicate`):\n{}",
        mismatched.join("\n")
    );
}
