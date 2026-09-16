//! Issue #1138 B6 (plan 06, L6): the publisher, not the best-ranked page.
//!
//! Ranking is not authority: a `.gov`/`.edu` preference does not identify a
//! compiler's official source. A lookalike host is refused and the refusal is
//! recorded; a procedure with no postcondition is refused before anything runs;
//! an exhausted search names every source it consulted; a dependency cycle
//! terminates with the cycle named; and two dependents of the same toolchain
//! install it once.

use std::path::PathBuf;

use formal_ai::concept_lookup::LookupOutcome;
use formal_ai::needs::Need;
use formal_ai::prerequisite::install::InstallGrant;
use formal_ai::prerequisite::probe::{ProbeVerdict, ToolchainProbe};
use formal_ai::prerequisite::publisher::{SetupProcedure, SetupStep, search_setup_procedure};
use formal_ai::prerequisite::{Platform, PrerequisiteError, PrerequisiteNeed, RecoveryOutcome, recover};
use formal_ai::prerequisite::install::install_scoped;
use formal_ai::prerequisite::ledger::ToolchainLedger;
use formal_ai::source_walk::{LookupBounds, SourceLookup, WalkSourceOutcome};

const REQUIREMENT: &str =
    "The build for this project needs a compiler that is not on this machine. Find out which one, \
get it, and then run the project's own test command.";

/// A lookup whose registry order and hosts are supplied by the test, so the
/// refusal rules are exercised without reaching the network.
struct FixtureLookup {
    /// Registry ids in the order the fixture serves them.
    order: Vec<String>,
    /// Hosts the fixture ranks, best first.
    hosts: Vec<String>,
    /// Every need the fixture was asked about, in order.
    asked: Vec<String>,
}

impl FixtureLookup {
    fn new(order: &[&str], hosts: &[&str]) -> Self {
        Self {
            order: order.iter().map(|id| (*id).to_owned()).collect(),
            hosts: hosts.iter().map(|host| (*host).to_owned()).collect(),
            asked: Vec::new(),
        }
    }
}

impl SourceLookup for FixtureLookup {
    fn lookup(&mut self, need: &Need, _bounds: &LookupBounds) -> LookupOutcome {
        self.asked.push(need.subject.clone());
        LookupOutcome::NotFound {
            consulted: self
                .order
                .iter()
                .zip(self.hosts.iter().cycle())
                .map(|(id, host)| WalkSourceOutcome {
                    source_id: id.clone(),
                    status: String::from("no_items"),
                    detail: host.clone(),
                    pages: 1,
                    items: 0,
                })
                .collect(),
        }
    }
}

fn need_for(program: &str, requires: &[&str]) -> PrerequisiteNeed {
    PrerequisiteNeed {
        program: program.to_owned(),
        source_span: REQUIREMENT.to_owned(),
        observed: ProbeVerdict::Missing {
            exit_code: Some(127),
            stderr: format!("{program}: command not found"),
        },
        platform: Platform::observed(),
        requires: requires.iter().map(|r| (*r).to_owned()).collect(),
    }
}

fn temp_root(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("formal-ai-issue-1138-publisher-{tag}"))
}

/// A host that merely looks official is refused, and the refusal is an event the
/// answer can quote, not a silent skip.
#[test]
fn a_lookalike_host_is_refused() {
    let mut lookup = FixtureLookup::new(
        &["lookalike", "kotlin_official"],
        &["kotlin-lang.example.com", "kotlinlang.org"],
    );
    let search = search_setup_procedure(&need_for("kotlinc", &[]), &mut lookup, &LookupBounds::default());
    assert!(
        search
            .refusals
            .iter()
            .any(|refusal| refusal.host.contains("example.com")),
        "the lookalike host must be refused and the refusal recorded: {:?}",
        search.refusals
    );
    assert!(
        search
            .procedure
            .as_ref()
            .is_none_or(|procedure| !procedure.source_url.contains("example.com")),
        "a lookalike host may never supply the selected procedure"
    );
}

/// A procedure nothing can verify is refused before a single step runs.
#[test]
fn a_procedure_without_a_postcondition_is_refused() {
    let procedure = SetupProcedure {
        program: String::from("zig"),
        source_id: String::from("zig_official"),
        source_url: String::from("https://ziglang.org/learn/getting-started/"),
        content_id: String::from("0".repeat(64)),
        platform: Platform::observed(),
        steps: vec![SetupStep {
            command: String::from("tar -xf zig.tar.xz"),
            writes_under: temp_root("no-postcondition"),
            digest: None,
        }],
        postcondition: None,
    };
    let grant = InstallGrant::Allowed {
        programs: vec![String::from("zig")],
        root: temp_root("no-postcondition"),
    };
    assert_eq!(
        install_scoped(&procedure, &grant),
        Err(PrerequisiteError::NoPostcondition),
        "without a postcondition probe nothing could verify the install, so it is refused"
    );
}

/// An exhausted search is attributable: it names every source it consulted, in
/// consultation order.
#[test]
fn exhausted_search_names_every_source_consulted() {
    let mut lookup = FixtureLookup::new(
        &["publisher_index", "package_registry", "distribution_docs"],
        &["example.org"],
    );
    let mut ledger = ToolchainLedger::new(temp_root("exhausted").join("toolchain-ledger.lino"));
    let outcome = recover(
        &need_for("gleam", &[]),
        &InstallGrant::Refused,
        &mut lookup,
        &LookupBounds::default(),
        &mut ledger,
    );
    match outcome {
        RecoveryOutcome::NotFound { consulted } => assert_eq!(
            consulted,
            vec![
                String::from("publisher_index"),
                String::from("package_registry"),
                String::from("distribution_docs"),
            ],
            "exhaustion names every source consulted, in order"
        ),
        other => panic!("an exhausted search must report NotFound, got {other:?}"),
    }
}

/// `a` requires `b` requires `a` terminates, and the cycle is named.
#[test]
fn a_cycle_is_detected_and_reported() {
    let mut lookup = FixtureLookup::new(&["cycle_fixture"], &["example.org"]);
    let mut need = need_for("alpha", &["beta"]);
    need.requires.push(String::from("alpha"));
    let search = search_setup_procedure(&need, &mut lookup, &LookupBounds::default());
    assert!(
        !search.cycle.is_empty(),
        "a dependency cycle must be detected and reported, not looped on"
    );
    assert!(
        search.cycle.contains(&String::from("alpha")),
        "the reported cycle names the program it returns to: {:?}",
        search.cycle
    );
}

/// `kotlinc` and `scalac` both need a JDK. It is installed once and the ledger
/// records one setup for it.
#[test]
fn two_dependents_share_one_setup() {
    let mut lookup = FixtureLookup::new(&["jdk_official"], &["jdk.java.net"]);
    let root = temp_root("shared-jdk");
    let ledger_path = root.join("toolchain-ledger.lino");
    let _ = std::fs::remove_dir_all(&root);
    let mut ledger = ToolchainLedger::new(&ledger_path);
    let grant = InstallGrant::Allowed {
        programs: vec![
            String::from("java"),
            String::from("kotlinc"),
            String::from("scalac"),
        ],
        root: root.clone(),
    };
    let bounds = LookupBounds::default();

    let _ = recover(&need_for("kotlinc", &["java"]), &grant, &mut lookup, &bounds, &mut ledger);
    let _ = recover(&need_for("scalac", &["java"]), &grant, &mut lookup, &bounds, &mut ledger);

    let java_records = ledger
        .records()
        .into_iter()
        .filter(|record| record.program == "java")
        .count();
    assert_eq!(
        java_records, 1,
        "two dependents of the same toolchain install it once"
    );

    let probe: ToolchainProbe = ledger
        .record_for("java")
        .expect("the shared setup is retained")
        .postcondition;
    assert_eq!(
        probe.program, "java",
        "the shared record keeps the postcondition probe that verified it"
    );
}
