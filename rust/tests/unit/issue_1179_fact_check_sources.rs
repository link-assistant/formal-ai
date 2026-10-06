//! Issue #1179 (E144): fact checking over code, Git, GitHub, and the internet
//! with relative meta logic.
//!
//! These tests pin the two halves the fact_check module owns:
//! - claims, not keys: a structured line becomes an exclusive claim only when
//!   it is a top-level fact field, so the 8,179 YAML step-key false
//!   positives cannot recur (R1, R2);
//! - evidence from every context: the code collector's scope tracking and
//!   certification gate (R4), the Git collector's bounded history queries
//!   (R5), and the web tiering that demotes aggregator reposts (R6/R7).
//!
//! The module registers as `formal_ai::fact_check` (a one-line `pub mod`
//! the integration wiring adds to `rust/src/lib.rs`); everything offline —
//! no network, no `gh`, and the Git tests skip when no `git` is on PATH.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use formal_ai::fact_check::code_source::{
    CallSiteCount, CodeQuery, code_evidence_for, count_call_sites, parse_code_query,
};
use formal_ai::fact_check::git_source::{
    GitQuery, blame_commit_for_line, git_history_evidence_for, git_log_for_path,
};
use formal_ai::fact_check::web_source::{
    is_external_statement, researchable, tier_for_url, url_host, web_query_for,
};
use formal_ai::fact_check::{
    DocumentType, claim_fields, fact_check, is_sequence_item_key, predicate_is_excluded,
    prior_for_context, sequence_item_keys, structured_claim, tier_for_context,
};
use formal_ai::relative_meta_logic::{SourceTier, Stance};

static TEMP_IDS: AtomicU32 = AtomicU32::new(0);

fn temp_repo(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-1179-{label}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("temp dir");
    path
}

/// R1: a YAML sequence item never becomes an exclusive claim.
#[test]
fn sequence_item_lines_carry_no_claim() {
    let workflow = ".github/workflows/ci.yml";
    assert!(is_sequence_item_key("- name: build"));
    assert!(is_sequence_item_key("  - name: build"));
    assert!(structured_claim(workflow, "- name: build").is_none());
    assert!(structured_claim(workflow, "- run: cargo test").is_none());
    assert!(structured_claim(workflow, "- uses: actions/checkout@v4").is_none());
    assert!(structured_claim(workflow, "- id: install").is_none());
}

/// R1: repeated step keys under sibling items cannot collide, because none of
/// them claims anything.
#[test]
fn sibling_sequence_keys_never_collide() {
    let workflow = ".github/workflows/ci.yml";
    let first = structured_claim(workflow, "- name: build");
    let second = structured_claim(workflow, "- name: test");
    assert!(first.is_none() && second.is_none());
}

/// R1: a top-level fact field does become a claim.
#[test]
fn top_level_fact_fields_become_claims() {
    let manifest = "Cargo.toml";
    let claim = structured_claim(manifest, "version = \"0.347.0\"")
        .expect("manifest version is a stated fact");
    assert_eq!(claim.subject, manifest);
    assert_eq!(claim.predicate, "version");
    assert_eq!(claim.value, "0.347.0");

    let chart = "chart.yaml";
    let claim = structured_claim(chart, "version: 1.2.3").expect("chart version is a fact");
    assert_eq!(claim.predicate, "version");
    assert_eq!(claim.value, "1.2.3");
}

/// R1: structure that is not a stated fact stays structure — dependency names,
/// nested keys, section headers, comments.
#[test]
fn non_fact_structure_stays_structure() {
    let manifest = "Cargo.toml";
    assert!(structured_claim(manifest, "serde = \"1.0\"").is_none());
    assert!(structured_claim(manifest, "[dependencies]").is_none());
    assert!(structured_claim(manifest, "# a comment").is_none());
    // A nested key keeps its indentation and so is not a document fact.
    assert!(structured_claim(manifest, "  version = \"9.9\"").is_none());
    // The allow-list decides: `edition` is a manifest fact field.
    assert!(structured_claim(manifest, "edition = \"2021\"").is_some());
}

/// R2: predicates carrying a YAML list-item key are excluded from
/// contradiction weighing, so legacy-format audits read clean.
#[test]
fn yaml_list_item_predicates_are_excluded() {
    assert!(predicate_is_excluded("- name"));
    assert!(predicate_is_excluded("- run"));
    assert!(predicate_is_excluded("- uses"));
    assert!(predicate_is_excluded("- id"));
    assert!(!predicate_is_excluded("version"));
    assert!(!predicate_is_excluded("name"));
}

/// The exclusion registry is data: the seed rows load with their keys.
#[test]
fn exclusion_seed_rows_load() {
    let keys = sequence_item_keys();
    for expected in ["name", "run", "uses", "id", "with", "env"] {
        assert!(keys.contains(&expected.to_owned()), "missing {expected}");
    }
    let manifest_fields = claim_fields(DocumentType::Manifest);
    assert!(manifest_fields.contains(&"version".to_owned()));
    assert!(manifest_fields.contains(&"edition".to_owned()));
    // `any` rows apply to every document type.
    let workflow_fields = claim_fields(DocumentType::Workflow);
    assert!(workflow_fields.contains(&"version".to_owned()));
    assert!(workflow_fields.contains(&"name".to_owned()));
}

/// R3-adjacent priors: code above tests above generated status above docs
/// above notes, read from the seed.
#[test]
fn source_priors_are_ordered_by_trust() {
    let code = prior_for_context("code");
    let tests = prior_for_context("tests");
    let generated = prior_for_context("generated_status");
    let docs = prior_for_context("docs");
    let notes = prior_for_context("notes");
    assert!(
        code > tests && tests > generated && generated > docs && docs > notes,
        "code {code} > tests {tests} > generated {generated} > docs {docs} > notes {notes}"
    );
    assert_eq!(tier_for_context("code"), SourceTier::OriginalFirstParty);
    // An unregistered context falls back to the audit defaults.
    assert_eq!(
        tier_for_context("unregistered_context"),
        SourceTier::IndependentCorroboration
    );
    assert!((0.0..=1.0).contains(&prior_for_context("unregistered_context")));
}

/// "What to build" item 3: a verdict from a declared prior and collected
/// evidence — contradicting trusted evidence lowers the posterior below the
/// prior.
#[test]
fn fact_check_weighs_evidence_from_the_declared_prior() {
    let contradiction = formal_ai::relative_meta_logic::RelativeEvidence::new(
        "code:src/lib.rs",
        SourceTier::OriginalFirstParty,
        Stance::Contradicts,
        0.9,
    );
    let verdict = fact_check(
        "Status: planned, nothing implemented",
        "docs",
        &[contradiction],
    );
    let prior = prior_for_context("docs");
    let posterior = verdict.assessment.posterior.get();
    assert!(
        posterior < prior,
        "posterior {posterior} must fall below prior {prior}"
    );

    let empty = fact_check("an unchallenged statement", "notes", &[]);
    assert_eq!(
        empty.assessment.posterior.get(),
        prior_for_context("notes"),
        "no evidence leaves the prior unchanged"
    );
}

/// R4: the scope tracker splits production from `#[cfg(test)]` references.
#[test]
fn call_sites_split_by_test_scope() {
    let source = "\
pub fn used_from_cli() {}

pub fn caller() {
    used_from_cli();
}

#[cfg(test)]
mod tests {
    #[test]
    fn calls_it() {
        used_from_cli();
    }
}
";
    let count = count_call_sites(source, "used_from_cli");
    assert_eq!(
        count,
        CallSiteCount {
            production: 2,
            test: 1
        },
        "definition, production call, and one test call — word boundaries exclude neither the definition nor the call"
    );
}

/// R4: symbol matching is whole-word, so `run` does not match `running`.
#[test]
fn symbol_matching_is_whole_word() {
    let count = count_call_sites("let running = true;", "run");
    assert_eq!(count.production, 0);
    let count = count_call_sites("run();", "run");
    assert_eq!(count.production, 1);
}

/// R4: the bounded queries are recognized from the seeded phrasings.
#[test]
fn code_queries_parse_from_seeded_phrases() {
    let query = parse_code_query(
        "The audit asks whether `research_coding_skill_gap` is called outside tests today.",
    )
    .expect("seeded phrasing");
    assert_eq!(
        query,
        CodeQuery::FunctionCalledOutsideTests {
            symbol: "research_coding_skill_gap".to_owned()
        }
    );
    assert_eq!(query.query_id(), "function_called_outside_tests");

    let query = parse_code_query(
        "does the symbol `parse_lino` exist in the file `rust/src/seed/parser.rs`?",
    )
    .expect("seeded phrasing");
    assert_eq!(
        query,
        CodeQuery::SymbolExistsAtPath {
            symbol: "parse_lino".to_owned(),
            path: "rust/src/seed/parser.rs".to_owned()
        }
    );
    assert!(parse_code_query("what is the weather").is_none());
}

/// R4: the collector certifies every inspected file through the sole AST
/// engine — with the engine on, a real fixture answers; with it off, the
/// collector answers nothing rather than guess.
#[test]
fn code_collector_certifies_before_answering() {
    let root = temp_repo("code");
    fs::create_dir_all(root.join("src")).expect("src dir");
    fs::write(
        root.join("src/lib.rs"),
        "pub fn production_fn() {}\nfn main() {\n    production_fn();\n}\n",
    )
    .expect("fixture");
    let query = CodeQuery::FunctionCalledOutsideTests {
        symbol: "production_fn".to_owned(),
    };
    let evidence = code_evidence_for(&query, &root);
    #[cfg(feature = "meta-language")]
    {
        assert_eq!(evidence.len(), 1, "{evidence:?}");
        assert_eq!(evidence[0].stance, Stance::Supports);
        assert_eq!(evidence[0].tier, SourceTier::OriginalFirstParty);
    }
    #[cfg(not(feature = "meta-language"))]
    {
        assert!(
            evidence.is_empty(),
            "without the engine there is no certification, so no evidence"
        );
    }
}

/// R5: Git history answers "when was this last true" from a real bounded
/// repository; the tests skip when `git` is unavailable.
fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "user.email=audit@example.com",
            "-c",
            "user.name=statement audit",
        ])
        .args(args)
        .output()
        .expect("git invocation");
    assert!(
        status.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );
}

/// R5: the newest revision containing the value supports the claim; a value
/// no revision ever contained contradicts it.
#[test]
fn git_history_supports_and_contradicts() {
    if !git_available() {
        return;
    }
    let root = temp_repo("git");
    git(&root, &["init", "-q"]);
    fs::write(
        root.join("STATUS.md"),
        "Status: planned, nothing implemented\n",
    )
    .expect("fixture");
    git(&root, &["add", "STATUS.md"]);
    git(&root, &["commit", "-q", "-m", "plan recorded"]);
    fs::write(root.join("STATUS.md"), "Status: implemented in PR #1139\n").expect("fixture");
    git(&root, &["add", "STATUS.md"]);
    git(&root, &["commit", "-q", "-m", "implemented"]);

    let history = git_log_for_path(&root, "STATUS.md");
    assert_eq!(history.len(), 2, "bounded follow-log of both revisions");
    assert_eq!(history[0].subject, "implemented");

    let query = GitQuery::ValueLastTrue {
        path: "STATUS.md".to_owned(),
        needle: "implemented in PR #1139".to_owned(),
    };
    let evidence = git_history_evidence_for(&query, &root);
    assert_eq!(evidence.len(), 1, "{evidence:?}");
    assert_eq!(evidence[0].stance, Stance::Supports);
    assert_eq!(evidence[0].tier, SourceTier::OriginalFirstParty);

    let query = GitQuery::ValueLastTrue {
        path: "STATUS.md".to_owned(),
        needle: "a value no revision ever held".to_owned(),
    };
    let evidence = git_history_evidence_for(&query, &root);
    assert_eq!(evidence.len(), 1, "{evidence:?}");
    assert_eq!(evidence[0].stance, Stance::Contradicts);

    let blame = blame_commit_for_line(&root, "STATUS.md", 1);
    assert!(blame.is_some_and(|commit| !commit.is_empty()));
}

/// R6/R7: aggregator reposts carry no weight; any other host corroborates.
#[test]
fn web_tiering_demotes_aggregators() {
    assert_eq!(
        tier_for_url("https://medium.com/some-post"),
        SourceTier::Unoriginal
    );
    assert_eq!(
        tier_for_url("https://www.reddit.com/r/rust"),
        SourceTier::Unoriginal
    );
    assert_eq!(
        tier_for_url("https://www.rust-lang.org/"),
        SourceTier::IndependentCorroboration
    );
    assert_eq!(url_host("https://example.com/a/b"), "example.com");
    assert_eq!(url_host("not-a-url"), "not-a-url");
}

/// R7: only external statements are worth internet queries, and the query is
/// a quoted phrase.
#[test]
fn only_external_statements_research() {
    assert!(is_external_statement(
        "The upstream crate published version 0.22 last Tuesday."
    ));
    assert!(!is_external_statement(
        "The parser lives in `rust/src/seed/parser.rs`."
    ));
    assert!(!is_external_statement(
        "See docs/case-studies/issue-1179/README.md for the audit."
    ));
    assert!(researchable(
        "The upstream crate published version 0.22 last Tuesday."
    ));
    assert!(!researchable("It shipped."));
    assert_eq!(
        web_query_for("The crate is released. The release notes say little."),
        "\"The release notes say little\""
    );
}
