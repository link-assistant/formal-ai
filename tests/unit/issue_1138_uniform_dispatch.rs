//! Issue #1138 B9, plan 09 leaf 12: `try_dispatch` has no name special cases,
//! and every prelude method is ledgered.
//!
//! The four `if name == "…"` sites and the five-armed prelude `match` in
//! `src/meta_method_dispatch.rs` become method-record attributes read from
//! `data/seed/method-registry.lino` and applied uniformly. #959 "How to test"
//! clause 2.
//!
//! Written before the leaf that makes it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::method_registry::{MethodRegistry, MethodSurface};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

#[test]
fn try_dispatch_has_no_name_special_cases() {
    let source = fs::read_to_string(repo_root().join("src/meta_method_dispatch.rs"))
        .expect("meta_method_dispatch.rs readable");
    let special_cases = source.matches("name == \"").count();
    assert_eq!(
        special_cases, 0,
        "src/meta_method_dispatch.rs still branches on {special_cases} method names; \
         plan 09 leaf 12 turns each into a method-record attribute \
         (response_language_variant, definition_fusion_variant, project_lookup_fallback, runtime)"
    );
}

#[test]
fn prelude_methods_are_ledgered() {
    // Plan 09 leaf 5 adds the five prelude methods as `status pending` rows, so
    // `handler_migration_pending` rises once, honestly, from 40 to 45 before it
    // falls. The census is precedence **plus** prelude from that leaf onward.
    let ledger = fs::read_to_string(repo_root().join("data/meta/handler-migration-ledger.lino"))
        .expect("handler migration ledger readable");
    let registry = MethodRegistry::shared();
    let prelude: Vec<&str> = registry
        .methods
        .iter()
        .filter(|method| method.surface == MethodSurface::Prelude)
        .map(|method| method.name.as_str())
        .collect();
    assert_eq!(
        prelude.len(),
        5,
        "there are exactly five prelude methods, got {prelude:?}"
    );
    let missing: Vec<&str> = prelude
        .iter()
        .copied()
        .filter(|name| !ledger.contains(&format!("handler {name}\n")))
        .collect();
    assert!(
        missing.is_empty(),
        "the prelude methods {missing:?} run on every turn and carry no ledger row; \
         plan 09 leaf 5 adds them as `status pending` and widens the census"
    );
}

#[test]
fn the_ledger_no_longer_points_at_a_deleted_kernel_ratchet() {
    // Plan 09 leaf 5 also fixes `data/meta/handler-migration-ledger.lino:4`'s
    // dead reference: the ceilings live in `data/meta/debt-ratchet.lino`.
    let ledger = fs::read_to_string(repo_root().join("data/meta/handler-migration-ledger.lino"))
        .expect("handler migration ledger readable");
    assert!(
        !ledger.contains("data/meta/kernel-ratchet.lino"),
        "the ledger still cites data/meta/kernel-ratchet.lino, which does not exist; \
         the one ledger of ceilings is data/meta/debt-ratchet.lino"
    );
    assert!(
        ledger.contains("data/meta/debt-ratchet.lino"),
        "the ledger must name the file that actually holds its ceilings"
    );
}
