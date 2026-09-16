//! Issue #1138 B6 (plan 06, L10): the whole family-1 recovery, live, five languages.
//!
//! This is the test a manual install must fail. It drives the held-out program
//! through the actual recovery path — probe, need, publisher lookup,
//! workspace-scoped install under a grant, re-probe, retry — and asserts the two
//! outcomes plan 06 promises: refused without a grant, and recovered with one,
//! with nothing outside the workspace root touched either way.
//!
//! It reaches trusted publishers over the network, so it is ignored by default
//! and run deliberately.

use std::path::PathBuf;

use formal_ai::concept_lookup::RegistrySourceLookup;
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::prerequisite::install::InstallGrant;
use formal_ai::prerequisite::probe::{ProbeVerdict, ToolchainProbe, probe_command};
use formal_ai::prerequisite::{Platform, PrerequisiteNeed, RecoveryOutcome, recover};
use formal_ai::prerequisite::ledger::ToolchainLedger;
use formal_ai::source_walk::LookupBounds;

/// The family-1 prompt, verbatim, in each of the five supported languages.
const PROMPTS: &[(&str, &str)] = &[
    (
        "en",
        "Write a program in Zig that prints the sum of the numbers from one to ten, then actually compile and run it here and show me the real output.",
    ),
    (
        "ru",
        "Напиши на Zig программу, которая печатает сумму чисел от одного до десяти, затем действительно скомпилируй и запусти её здесь и покажи мне настоящий вывод.",
    ),
    (
        "hi",
        "Zig में एक प्रोग्राम लिखो जो एक से दस तक की संख्याओं का योग छापे, फिर उसे यहीं सचमुच संकलित करके चलाओ और मुझे असली आउटपुट दिखाओ।",
    ),
    (
        "zh",
        "用 Zig 写一个打印一到十之和的程序，然后在这里真正编译并运行它，把真实的输出给我看。",
    ),
    (
        "es",
        "Escribe un programa en Zig que imprima la suma de los números del uno al diez, luego compílalo y ejecútalo realmente aquí y muéstrame la salida real.",
    ),
];

/// The held-out program family 1 is written around.
const PROGRAM: &str = "zig";

fn workspace_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("formal-ai-issue-1138-recovery-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the workspace root should be creatable");
    root
}

fn need_for(prompt: &str) -> PrerequisiteNeed {
    PrerequisiteNeed {
        program: String::from(PROGRAM),
        source_span: prompt.to_owned(),
        observed: ProbeVerdict::Missing {
            exit_code: Some(127),
            stderr: format!("{PROGRAM}: command not found"),
        },
        platform: Platform::observed(),
        requires: Vec::new(),
    }
}

#[test]
#[ignore = "network: plan 06 L10"]
fn a_missing_held_out_compiler_is_discovered_installed_and_retried() {
    let bounds = LookupBounds::default();
    let cache = std::env::temp_dir().join("formal-ai-issue-1138-recovery-cache");
    let client = CachedSourceClient::new(&cache, CurlSourceTransport);
    let preferences = ServicePreferences::default();
    let mut availability = ServiceAccessibilityCache::new(&cache);

    for (language, prompt) in PROMPTS {
        // Refused: the default grant installs nothing, but the answer must still
        // name the publisher and the exact command it would have run.
        let refused_root = workspace_root(&format!("refused-{language}"));
        let mut ledger = ToolchainLedger::new(refused_root.join("toolchain-ledger.lino"));
        let mut lookup = RegistrySourceLookup::new(
            &client,
            &preferences,
            &mut availability,
            bounds,
            language,
            0,
        );
        let refused = recover(
            &need_for(prompt),
            &InstallGrant::Refused,
            &mut lookup,
            &bounds,
            &mut ledger,
        );
        match &refused {
            RecoveryOutcome::NotPermitted { procedure } => {
                assert_eq!(procedure.program, PROGRAM, "{language}: the refusal names the program");
                assert!(
                    !procedure.source_url.is_empty(),
                    "{language}: the refusal names the publisher it found"
                );
                assert!(
                    procedure.postcondition.is_some(),
                    "{language}: only a procedure with a postcondition may be offered"
                );
            }
            RecoveryOutcome::NotFound { consulted } => assert!(
                !consulted.is_empty(),
                "{language}: an exhausted search names every source consulted"
            ),
            other => panic!("{language}: nothing may be installed without a grant, got {other:?}"),
        }
        assert!(
            !refused_root.join(".formal-ai/toolchains").exists(),
            "{language}: a refused recovery installs nothing"
        );

        // Granted: the toolchain lands inside the workspace, the postcondition
        // probe passes, and the version is the one that was observed.
        let granted_root = workspace_root(&format!("granted-{language}"));
        let mut ledger = ToolchainLedger::new(granted_root.join("toolchain-ledger.lino"));
        let mut lookup = RegistrySourceLookup::new(
            &client,
            &preferences,
            &mut availability,
            bounds,
            language,
            0,
        );
        let granted = recover(
            &need_for(prompt),
            &InstallGrant::Allowed {
                programs: vec![String::from(PROGRAM)],
                root: granted_root.clone(),
            },
            &mut lookup,
            &bounds,
            &mut ledger,
        );
        match granted {
            RecoveryOutcome::Recovered {
                toolchain,
                retry,
                evidence,
            } => {
                assert!(
                    toolchain.prefix.starts_with(&granted_root),
                    "{language}: the install lands under the workspace root only"
                );
                assert!(
                    !retry.trim().is_empty(),
                    "{language}: recovery names the command to retry"
                );
                assert!(
                    !evidence.is_empty(),
                    "{language}: every step of the recovery appends an observation"
                );
                let reprobe = probe_command(
                    &ToolchainProbe {
                        program: String::from(PROGRAM),
                        argv: vec![String::from("version")],
                        expect: None,
                        requires: Vec::new(),
                    },
                    &granted_root,
                );
                assert!(
                    matches!(reprobe, ProbeVerdict::Present { .. }),
                    "{language}: the postcondition probe must pass after a recovery, got {reprobe:?}"
                );
            }
            RecoveryOutcome::StillMissing { before, after } => panic!(
                "{language}: a successful setup with a failing postcondition is still missing: \
                 {before:?} then {after:?}"
            ),
            other => panic!("{language}: a granted recovery must install, got {other:?}"),
        }
    }
}
