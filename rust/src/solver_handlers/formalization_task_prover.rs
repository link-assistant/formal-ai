// ---------------------------------------------------------------------------
// The theorem-prover step (issue #1186 R4)
// ---------------------------------------------------------------------------
//
// Each `prover` record of `data/seed/formal-targets.lino` turns its target's
// rendering into a self-contained compile unit: the declarations of the
// domain (the `export_target rml` record's `domain`), of every predicate and
// of every object constant the clause names, then the rendered theorem. When
// the record's `binary` is found in PATH it runs once on that unit, written
// under the system temporary directory with the record's extension, and the
// honesty block states the exit status and the unit's path in the answer
// language; otherwise it states that the binary was not found. No prover
// found means the block opens with "No theorem prover was invoked".
//
// The Lean and Rocq texts themselves come from the seed-grammar exporter
// (`render_clause`); `ClauseExporter` is the seam where the
// relative-meta-logic crate takes over in process once it is published
// (link-foundation/relative-meta-logic#185).
//
// Mirrored by `formalProverUnit`/`formalProverChecks` in
// `js/worker/formal_ai_worker_formalization_targets.js` and, for the process
// itself, by `js/server/prover-host.mjs`.

/// Which exporter renders a clause into a formal target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClauseExporter {
    /// The `formal_language` templates of `data/seed/formal-targets.lino`.
    SeedTemplates,
}

impl ClauseExporter {
    /// The exporter this build renders with. The relative-meta-logic crate
    /// is not published, so the seed templates are the only exporter today.
    #[must_use]
    pub const fn active() -> Self {
        Self::SeedTemplates
    }

    /// The exporter's name, as recorded in the derivation.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SeedTemplates => "seed_templates",
        }
    }

    /// Render `clause` into the target `slug`.
    #[must_use]
    pub fn export(self, clause: &QuantifiedClause, slug: &str) -> Option<String> {
        match self {
            Self::SeedTemplates => render_clause(clause, slug),
        }
    }
}

/// One `prover` record of `data/seed/formal-targets.lino`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProverRecord {
    /// The record id (`lean`, `rocq`).
    pub id: String,
    /// The `formal_language` slug whose rendering the prover checks.
    pub target: String,
    /// The executable looked up in PATH.
    pub binary: String,
    /// The name the honesty block uses for the prover.
    pub label: String,
    /// The compile unit's file extension.
    pub extension: String,
    domain_declaration: String,
    predicate: String,
    predicate_with_object: String,
    constant: String,
}

/// The seed's prover records, in seed order, read once.
#[must_use]
pub fn prover_records() -> &'static [ProverRecord] {
    static RECORDS: OnceLock<Vec<ProverRecord>> = OnceLock::new();
    RECORDS.get_or_init(|| {
        let tree = parse_lino(TARGETS);
        target_records(&tree)
            .filter(|record| record.name == "prover")
            .map(|record| {
                let value = |name: &str| record.find_child_value(name).to_owned();
                ProverRecord {
                    id: record.id.clone(),
                    target: value("target"),
                    binary: value("binary"),
                    label: value("label"),
                    extension: value("extension"),
                    domain_declaration: value("domain_declaration"),
                    predicate: value("predicate"),
                    predicate_with_object: value("predicate_with_object"),
                    constant: value("constant"),
                }
            })
            .collect()
    })
}

/// The compile unit `prover` checks for `clause`: declarations, a blank
/// line, the rendered theorem. `None` when the target renders nothing.
#[must_use]
pub fn prover_unit(clause: &QuantifiedClause, prover: &ProverRecord) -> Option<String> {
    let rendered = ClauseExporter::active().export(clause, &prover.target)?;
    let domain = rml_templates().domain.as_str();
    // Apply the target language's identifier quoting rule to names that are
    // not plain identifiers (e.g. Cyrillic names in Lean get «…»).
    let target_language = grammar().formal.iter().find(|l| l.slug == prover.target);
    let apply = |name: &str| -> String {
        target_language.map_or_else(|| name.to_owned(), |lang| apply_identifier_rule(lang, name))
    };
    let mut lines = vec![fill(&prover.domain_declaration, &[("domain", domain)])];
    let mut declared: Vec<&str> = Vec::new();
    let mut constants: Vec<&str> = Vec::new();
    for predicate in clause.antecedent.iter().chain([&clause.consequent]) {
        if !declared.contains(&predicate.name.as_str()) {
            let template = if predicate.object.is_some() {
                &prover.predicate_with_object
            } else {
                &prover.predicate
            };
            let quoted_name = apply(&predicate.name);
            lines.push(fill(
                template,
                &[("name", &quoted_name), ("domain", domain)],
            ));
            declared.push(&predicate.name);
        }
        if let Some(object) = predicate.object.as_deref()
            && !constants.contains(&object)
        {
            constants.push(object);
        }
    }
    for constant in constants {
        let quoted_constant = apply(constant);
        lines.push(fill(
            &prover.constant,
            &[("constant", &quoted_constant), ("domain", domain)],
        ));
    }
    Some(format!("{}\n\n{rendered}\n", lines.join("\n")))
}

/// The executable `binary` names in some directory of the PATH-shaped list
/// `path_value` (looked up, not run).
#[must_use]
pub fn prover_command_in(path_value: &std::ffi::OsStr, binary: &str) -> Option<std::path::PathBuf> {
    std::env::split_paths(path_value)
        .filter(|directory| !directory.as_os_str().is_empty())
        .map(|directory| directory.join(binary))
        .find(|candidate| candidate.is_file())
}

/// What one prover did for one answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProverRun {
    /// The binary was not found in PATH, or the target rendered nothing.
    Absent,
    /// The binary ran on the compile unit: its exit status (`None` when it
    /// was killed by a signal or could not be spawned) and the unit's path.
    Ran {
        command: String,
        exit: Option<i32>,
        source_path: String,
    },
}

/// `clause_<FNV-1a hex>`: one unit always lands on one file name, and the
/// name is a valid module identifier for every prover (Rocq rejects a dash).
#[must_use]
pub fn prover_file_stem(unit: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in unit.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("clause_{hash:016x}")
}

/// Write `unit` under `directory` with `extension` and run `command` on it.
#[must_use]
pub fn run_prover_with(
    command: &std::path::Path,
    unit: &str,
    directory: &std::path::Path,
    extension: &str,
) -> ProverRun {
    let source_path = directory.join(format!("{}.{extension}", prover_file_stem(unit)));
    let written = std::fs::create_dir_all(directory)
        .and_then(|()| std::fs::write(&source_path, unit))
        .is_ok();
    let exit = if written {
        std::process::Command::new(command)
            .arg(&source_path)
            .current_dir(directory)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .ok()
            .and_then(|status| status.code())
    } else {
        None
    };
    ProverRun::Ran {
        command: command.display().to_string(),
        exit,
        source_path: source_path.display().to_string(),
    }
}

/// Every seeded prover's run for `clause`, searching `path_value` and
/// writing units under `directory`.
#[must_use]
pub fn prover_runs_with(
    clause: &QuantifiedClause,
    path_value: &std::ffi::OsStr,
    directory: &std::path::Path,
) -> Vec<(&'static ProverRecord, ProverRun)> {
    prover_records()
        .iter()
        .map(|prover| {
            let run = prover_unit(clause, prover)
                .zip(prover_command_in(path_value, &prover.binary))
                .map_or(ProverRun::Absent, |(unit, command)| {
                    run_prover_with(&command, &unit, directory, &prover.extension)
                });
            (prover, run)
        })
        .collect()
}

/// The runs against this process's PATH and temporary directory.
fn prover_runs(clause: &QuantifiedClause) -> Vec<(&'static ProverRecord, ProverRun)> {
    let path_value = std::env::var_os("PATH").unwrap_or_default();
    let directory = std::env::temp_dir().join("formal-ai-prover");
    prover_runs_with(clause, &path_value, &directory)
}

/// The honesty block's prover slots in `language`: `prover_summary`, then
/// one `<target>_check` per prover.
#[must_use]
pub fn prover_check_slots(
    runs: &[(&ProverRecord, ProverRun)],
    language: &str,
) -> Vec<(String, String)> {
    let mut ran: Vec<&str> = Vec::new();
    let mut slots = Vec::new();
    for (prover, run) in runs {
        let sentence = match run {
            ProverRun::Absent => fill(
                &response("formalization_prover_absent", language),
                &[("label", &prover.label)],
            ),
            ProverRun::Ran {
                exit, source_path, ..
            } => {
                ran.push(&prover.label);
                let exit = exit.map_or_else(|| String::from("none"), |code| code.to_string());
                fill(
                    &response("formalization_prover_present", language),
                    &[
                        ("label", &prover.label),
                        ("source", source_path),
                        ("exit", &exit),
                    ],
                )
            }
        };
        slots.push((format!("{}_check", prover.target), sentence));
    }
    let summary = if ran.is_empty() {
        response("formalization_prover_none", language)
    } else {
        fill(
            &response("formalization_prover_invoked", language),
            &[("provers", &ran.join(", "))],
        )
    };
    let mut all = vec![(String::from("prover_summary"), summary)];
    all.extend(slots);
    all
}

/// Run the provers for `clause`, record the step as a derivation fragment,
/// and return the honesty slots in `language`.
fn record_prover_fragment(
    log: &mut EventLog,
    clause: &QuantifiedClause,
    language: &str,
) -> Vec<(String, String)> {
    let runs = prover_runs(clause);
    let invoked = runs
        .iter()
        .any(|(_, run)| matches!(run, ProverRun::Ran { .. }));
    let mut fields: Vec<(String, String)> = vec![
        (String::from("stage"), String::from("prover")),
        (
            String::from("exporter"),
            ClauseExporter::active().name().to_owned(),
        ),
    ];
    for (prover, run) in &runs {
        let status = match run {
            ProverRun::Absent => String::from("absent"),
            ProverRun::Ran { exit, .. } => {
                format!(
                    "ran:{}",
                    exit.map_or_else(|| String::from("none"), |code| code.to_string())
                )
            }
        };
        fields.push((prover.binary.clone(), status));
    }
    fields.push((String::from("invoked"), invoked.to_string()));
    let borrowed: Vec<(&str, &str)> = fields
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    log.append_fields(crate::derivation::FORMALIZE_FRAGMENT_KIND, &borrowed);
    prover_check_slots(&runs, language)
}

/// The honesty block in `language`: the prover slots, then the rml step.
fn honesty_block(prover_slots: &[(String, String)], rml_check: &str, language: &str) -> String {
    let mut slots: Vec<(&str, &str)> = prover_slots
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    slots.push(("rml_check", rml_check));
    fill(&response("formalization_honesty", language), &slots)
}
