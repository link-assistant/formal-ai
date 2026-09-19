//! Issue #1138 B6 (plan 06, L6): the publisher, not the best-ranked page.
//!
//! Ranking is not authority: a `.gov`/`.edu` preference does not identify a
//! compiler's official source. A lookalike host is refused and the refusal is
//! recorded; a procedure with no postcondition is refused before anything runs;
//! an exhausted search names every source it consulted; a dependency cycle
//! terminates with the cycle named; and two dependents of the same toolchain
//! install it once.

use std::path::PathBuf;

use formal_ai::concept_lookup::{ConceptSense, LookupOutcome};
use formal_ai::needs::Need;
use formal_ai::prerequisite::install::InstallGrant;
use formal_ai::prerequisite::install::install_scoped;
use formal_ai::prerequisite::ledger::ToolchainLedger;
use formal_ai::prerequisite::probe::{ProbeVerdict, ToolchainProbe};
use formal_ai::prerequisite::publisher::pinned_python_repository_procedure;
use formal_ai::prerequisite::publisher::{SetupProcedure, SetupStep, search_setup_procedure};
use formal_ai::prerequisite::{
    Platform, PrerequisiteError, PrerequisiteNeed, RecoveryOutcome, recover,
};
use formal_ai::relative_meta_logic::SourceTier;
use formal_ai::source_walk::{LookupBounds, SourceLookup, WalkSourceOutcome};

const REQUIREMENT: &str = "The build for this project needs a compiler that is not on this machine. Find out which one, \
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
    /// Retrieved senses, when this fixture represents a successful source walk.
    senses: Option<Vec<ConceptSense>>,
}

impl FixtureLookup {
    fn new(order: &[&str], hosts: &[&str]) -> Self {
        Self {
            order: order.iter().map(|id| (*id).to_owned()).collect(),
            hosts: hosts.iter().map(|host| (*host).to_owned()).collect(),
            asked: Vec::new(),
            senses: None,
        }
    }

    fn found(senses: Vec<ConceptSense>) -> Self {
        Self {
            order: Vec::new(),
            hosts: Vec::new(),
            asked: Vec::new(),
            senses: Some(senses),
        }
    }
}

impl SourceLookup for FixtureLookup {
    fn lookup(&mut self, need: &Need, _bounds: &LookupBounds) -> LookupOutcome {
        self.asked.push(need.subject.clone());
        if let Some(senses) = self.senses.clone() {
            return LookupOutcome::Found(senses);
        }
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

fn setup_sense(source_url: &str, source_id: &str, gloss: &str) -> ConceptSense {
    ConceptSense {
        surface: String::from("kotlinc"),
        lemma: String::from("kotlinc"),
        language: String::from("en"),
        gloss: gloss.to_owned(),
        part_of_speech: String::from("procedure"),
        synonyms: Vec::new(),
        source_id: source_id.to_owned(),
        source_url: source_url.to_owned(),
        sha256: String::from("7".repeat(64)),
        fetched_at: String::from("2026-09-16T00:00:00Z"),
        cached: true,
        tier: SourceTier::OriginalFirstParty,
        license_name: String::from("Apache-2.0"),
        license_url: String::from("https://www.apache.org/licenses/LICENSE-2.0"),
        depth: 0,
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
    let search = search_setup_procedure(
        &need_for("kotlinc", &[]),
        &mut lookup,
        &LookupBounds::default(),
    );
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

/// A trusted lookup result can carry a machine-readable procedure. The
/// formalizer preserves exact argv boundaries and the source bytes' digest;
/// it does not turn surrounding prose into shell text.
#[test]
fn a_trusted_retrieved_recipe_becomes_executable_steps() {
    let recipe = r#"
setup_procedure kotlin_cli
  program kotlinc
  platform any
  step create_environment
    program python3
    argument -m
    argument venv
    argument {prefix}
    writes_under .
    requires_network false
  step retrieve_package
    program python3
    argument -m
    argument pip
    argument install
    argument kotlin-compiler
    requires_network true
    writes_under .
  postcondition compiler_responds
    program kotlinc
    argument -version
"#;
    let mut lookup = FixtureLookup::found(vec![setup_sense(
        "https://kotlinlang.org/docs/command-line.html",
        "kotlin_official",
        recipe,
    )]);

    let search = search_setup_procedure(
        &need_for("kotlinc", &[]),
        &mut lookup,
        &LookupBounds::default(),
    );
    let procedure = search
        .procedure
        .expect("a first-party structured recipe should formalize");

    assert_eq!(procedure.content_id, "7".repeat(64));
    assert_eq!(procedure.steps.len(), 2);
    assert_eq!(procedure.steps[0].program, "python3");
    assert_eq!(
        procedure.steps[0].arguments,
        vec![
            String::from("-m"),
            String::from("venv"),
            String::from("{prefix}")
        ]
    );
    assert!(procedure.steps[1].requires_network);
    assert_eq!(
        procedure.postcondition,
        Some(ToolchainProbe::new("kotlinc", &["-version"]))
    );
}

/// Authority alone is not executable consent: ordinary prose from the right
/// host is evidence, but only the typed recipe grammar can become processes.
#[test]
fn trusted_prose_is_not_executed_as_a_recipe() {
    let mut lookup = FixtureLookup::found(vec![setup_sense(
        "https://kotlinlang.org/docs/command-line.html",
        "kotlin_official",
        "Download the compiler and run whichever setup command your system needs.",
    )]);
    let search = search_setup_procedure(
        &need_for("kotlinc", &[]),
        &mut lookup,
        &LookupBounds::default(),
    );

    assert!(search.procedure.is_none());
    assert!(
        search
            .refusals
            .iter()
            .any(|refusal| refusal.reason == "publisher_payload_has_no_typed_setup_recipe")
    );
}

/// A pinned Python repository is a reusable recipe family, not a workflow-only
/// special case. Mutable branches and unregistered hosts remain refusals.
#[test]
fn a_pinned_python_repository_recipe_is_reproducible() {
    let revision = "f7bbbb2ccdf479001d6467c9e34af59e44a840f9";
    let procedure = pinned_python_repository_procedure(
        "swebench-harness",
        "https://github.com/SWE-bench/SWE-bench",
        revision,
        "swebench.harness.run_evaluation",
        Platform::observed(),
    )
    .expect("the registry pins GitHub as this prerequisite's publisher");

    assert_eq!(procedure.source_id, "github");
    assert!(procedure.source_url.ends_with(revision));
    assert_eq!(procedure.steps.len(), 2);
    assert!(!procedure.steps[0].requires_network);
    assert!(procedure.steps[1].requires_network);
    assert!(
        pinned_python_repository_procedure(
            "swebench-harness",
            "https://github.com/SWE-bench/SWE-bench",
            "main",
            "swebench.harness.run_evaluation",
            Platform::observed(),
        )
        .is_err(),
        "a mutable revision is never accepted as a reproducible procedure"
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
            program: String::from("tar"),
            arguments: vec![String::from("-xf"), String::from("zig.tar.xz")],
            writes_under: temp_root("no-postcondition"),
            digest: None,
            requires_network: false,
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
    let recipe = r#"
setup_procedure jdk
  program java
  platform any
  step retrieve_jdk
    program curl
    argument https://jdk.java.net/archive/
    writes_under .
    requires_network true
  postcondition java_responds
    program java
    argument -version
"#;
    let mut lookup = FixtureLookup::found(vec![setup_sense(
        "https://jdk.java.net/archive/",
        "jdk_official",
        recipe,
    )]);
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

    let _ = recover(
        &need_for("kotlinc", &["java"]),
        &grant,
        &mut lookup,
        &bounds,
        &mut ledger,
    );
    let _ = recover(
        &need_for("scalac", &["java"]),
        &grant,
        &mut lookup,
        &bounds,
        &mut ledger,
    );

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
