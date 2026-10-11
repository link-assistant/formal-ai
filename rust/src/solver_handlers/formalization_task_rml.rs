// ---------------------------------------------------------------------------
// The relative-meta-logic export step (issue #1186 R4/R6)
// ---------------------------------------------------------------------------
//
// The relative-meta-logic crate is not published (link-foundation/
// relative-meta-logic#185), so it cannot run in process. Its command-line
// exporter can: `rml export lean <file.lino> -o <file.lean>` lowers RML's
// typed, non-probabilistic fragment to Lean 4. A universal conditional
// clause has a direct shape there — each predicate a `Pi` from the domain
// `U` to `Prop`, an object a constant of `U`, and the clause itself one
// declaration whose type binds the variable, then one hypothesis per
// antecedent, ending in the consequent. The existential and negative
// readings need a binder the export subset does not have, so they are
// reported as outside it rather than approximated.
//
// The command is the executable `FORMAL_AI_RML` names, else `rml` found in
// PATH. When present it is run once per answer on a file under the system
// temporary directory, and its exit status and output path are recorded as
// a `formalize:fragment` derivation event, which `formal-ai explain` prints.
//
// The fragment's declaration shapes are the `export_target rml` templates of
// `data/seed/formal-targets.lino`. Mirrored by `formalRmlSource` in
// `js/worker/formal_ai_worker_formalization_targets.js` (the browser has no
// process to run, so its step is never run there).

/// The `export_target rml` templates of `data/seed/formal-targets.lino`.
#[derive(Debug, Default)]
struct RmlTemplates {
    domain: String,
    domain_declaration: String,
    predicate: String,
    predicate_with_object: String,
    constant: String,
    atom: String,
    atom_with_object: String,
    hypothesis: String,
    clause: String,
}

/// The seed's RML export templates, read once.
fn rml_templates() -> &'static RmlTemplates {
    static TEMPLATES: OnceLock<RmlTemplates> = OnceLock::new();
    TEMPLATES.get_or_init(|| {
        let tree = parse_lino(TARGETS);
        target_records(&tree)
            .find(|record| record.name == "export_target" && record.id == "rml")
            .map_or_else(RmlTemplates::default, |record| {
                let value = |name: &str| record.find_child_value(name).to_owned();
                RmlTemplates {
                    domain: value("domain"),
                    domain_declaration: value("domain_declaration"),
                    predicate: value("predicate"),
                    predicate_with_object: value("predicate_with_object"),
                    constant: value("constant"),
                    atom: value("atom"),
                    atom_with_object: value("atom_with_object"),
                    hypothesis: value("hypothesis"),
                    clause: value("clause"),
                }
            })
    })
}

/// An identifier safe in RML's notation: whitespace and delimiters become `_`.
fn rml_identifier(text: &str) -> String {
    text.trim()
        .chars()
        .map(|character| {
            if character.is_whitespace() || "():\"'".contains(character) {
                '_'
            } else {
                character
            }
        })
        .collect()
}

/// One applied predicate as an RML term, from the seed's atom templates.
fn rml_atom(templates: &RmlTemplates, predicate: &AppliedPredicate, variable: &str) -> String {
    let name = rml_identifier(&predicate.name);
    predicate.object.as_ref().map_or_else(
        || fill(&templates.atom, &[("name", &name), ("variable", variable)]),
        |object| {
            fill(
                &templates.atom_with_object,
                &[
                    ("name", &name),
                    ("variable", variable),
                    ("object", &rml_identifier(object)),
                ],
            )
        },
    )
}

/// The RML source of a universal conditional clause, if it has one.
///
/// A reading outside the export subset (existential, negative), or a seed
/// without the `export_target rml` record, is `None`.
#[must_use]
pub fn rml_source(clause: &QuantifiedClause) -> Option<String> {
    let templates = rml_templates();
    if clause.quantifier != "forall" || templates.clause.is_empty() {
        return None;
    }
    let domain = templates.domain.as_str();
    let variable = rml_identifier(&clause.variable);
    let second = format!("{variable}{variable}");
    let mut lines = vec![fill(&templates.domain_declaration, &[("domain", domain)])];
    let mut declared: Vec<String> = Vec::new();
    let mut object_constants: Vec<String> = Vec::new();
    for predicate in clause.antecedent.iter().chain([&clause.consequent]) {
        let name = rml_identifier(&predicate.name);
        if !declared.contains(&name) {
            let template = if predicate.object.is_some() {
                &templates.predicate_with_object
            } else {
                &templates.predicate
            };
            lines.push(fill(
                template,
                &[
                    ("name", &name),
                    ("domain", domain),
                    ("variable", &variable),
                    ("second", &second),
                ],
            ));
            declared.push(name);
        }
        if let Some(object) = &predicate.object {
            let constant = rml_identifier(object);
            if !object_constants.contains(&constant) {
                object_constants.push(constant);
            }
        }
    }
    for constant in &object_constants {
        lines.push(fill(
            &templates.constant,
            &[("constant", constant), ("domain", domain)],
        ));
    }
    let mut body = rml_atom(templates, &clause.consequent, &variable);
    for (index, predicate) in clause.antecedent.iter().enumerate().rev() {
        let atom = rml_atom(templates, predicate, &variable);
        let number = (index + 1).to_string();
        body = fill(
            &templates.hypothesis,
            &[("atom", &atom), ("index", &number), ("body", &body)],
        );
    }
    lines.push(fill(
        &templates.clause,
        &[("domain", domain), ("variable", &variable), ("body", &body)],
    ));
    Some(format!("{}\n", lines.join("\n")))
}

/// What the `rml` step did for one answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RmlExport {
    /// The clause is outside the export subset; nothing was run.
    OutsideSubset,
    /// No `rml` command was found; nothing was run.
    Absent,
    /// The command ran: its exit status (`None` when it was killed by a
    /// signal or could not be spawned) and the source and output paths.
    Ran {
        command: String,
        exit: Option<i32>,
        source_path: String,
        output_path: String,
    },
}

/// The `rml` command: the executable `FORMAL_AI_RML` names, else `rml`
/// found in PATH (looked up, not run).
fn rml_command() -> Option<std::path::PathBuf> {
    if let Some(named) = std::env::var_os("FORMAL_AI_RML") {
        let path = std::path::PathBuf::from(named);
        return path.is_file().then_some(path);
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|directory| directory.join("rml"))
        .find(|candidate| candidate.is_file())
}

/// A stable short name for a source text (`FNV-1a`), so one clause always
/// exports to the same file.
fn rml_file_stem(source: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in source.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("clause-{hash:016x}")
}

/// Run `command export lean` on `source`, written under `directory`.
#[must_use]
pub fn run_rml_export_with(
    command: &std::path::Path,
    source: &str,
    directory: &std::path::Path,
) -> RmlExport {
    let stem = rml_file_stem(source);
    let source_path = directory.join(format!("{stem}.lino"));
    let output_path = directory.join(format!("{stem}.lean"));
    let written = std::fs::create_dir_all(directory)
        .and_then(|()| std::fs::write(&source_path, source))
        .is_ok();
    let exit = if written {
        std::process::Command::new(command)
            .arg("export")
            .arg("lean")
            .arg(&source_path)
            .arg("-o")
            .arg(&output_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .ok()
            .and_then(|status| status.code())
    } else {
        None
    };
    RmlExport::Ran {
        command: command.display().to_string(),
        exit,
        source_path: source_path.display().to_string(),
        output_path: output_path.display().to_string(),
    }
}

/// The `rml` step for one clause: outside the subset, absent, or run.
fn rml_export(clause: &QuantifiedClause) -> RmlExport {
    let Some(source) = rml_source(clause) else {
        return RmlExport::OutsideSubset;
    };
    let Some(command) = rml_command() else {
        return RmlExport::Absent;
    };
    let directory = std::env::temp_dir().join("formal-ai-rml");
    run_rml_export_with(&command, &source, &directory)
}

/// Record the step as a derivation fragment and return the honesty
/// sentence that states it.
fn record_rml_fragment(log: &mut EventLog, clause: &QuantifiedClause) -> String {
    let export = rml_export(clause);
    match &export {
        RmlExport::OutsideSubset => {
            log.append_fields(
                crate::derivation::FORMALIZE_FRAGMENT_KIND,
                &[
                    ("stage", "rml_export"),
                    ("status", "outside_subset"),
                    ("quantifier", &clause.quantifier),
                    ("invoked", "false"),
                ],
            );
            crate::seed::report_text(
                "formalization_rml_outside_subset",
                &[("quantifier", &clause.quantifier)],
            )
        }
        RmlExport::Absent => {
            log.append_fields(
                crate::derivation::FORMALIZE_FRAGMENT_KIND,
                &[
                    ("stage", "rml_export"),
                    ("status", "absent"),
                    ("invoked", "false"),
                ],
            );
            crate::seed::report_text("formalization_rml_absent", &[])
        }
        RmlExport::Ran {
            command,
            exit,
            source_path,
            output_path,
        } => {
            let exit = exit.map_or_else(|| String::from("none"), |code| code.to_string());
            log.append_fields(
                crate::derivation::FORMALIZE_FRAGMENT_KIND,
                &[
                    ("stage", "rml_export"),
                    ("status", "ran"),
                    ("invoked", "true"),
                    ("command", command),
                    ("exit", &exit),
                    ("source", source_path),
                    ("output", output_path),
                ],
            );
            crate::seed::report_text(
                "formalization_rml_ran",
                &[
                    ("exit", &exit),
                    ("source", source_path),
                    ("output", output_path),
                ],
            )
        }
    }
}
