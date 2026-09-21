//! Finding the *official* setup procedure, not the best-ranked page (#1138 B6).
//!
//! Ranking is not authority: a `.gov`/`.edu` preference does not identify a
//! compiler's official source. `data/seed/setup-publishers.lino` pins which host
//! is authoritative for which program, and a lookalike host is refused with a
//! recorded reason.
//!
//! This is recipe stage 3, `look_up_publisher`, and stage 4,
//! `formalize_procedure`: the registry walk names the sources, the pinned
//! publisher names the authority, and what comes back is a procedure carrying a
//! postcondition probe — because a procedure nothing can verify is refused
//! before anything runs.

use std::path::PathBuf;

use super::{Platform, PrerequisiteNeed};
use crate::concept_lookup::LookupOutcome;
use crate::needs::{Need, NeedKind};
use crate::seed::parser::parse_lino;
use crate::source_walk::{LookupBounds, SourceLookup};

/// The seed document pinning one authoritative publisher per program.
const SETUP_PUBLISHERS_LINO: &str = include_str!("../../../data/seed/setup-publishers.lino");

/// Record type of one publisher row.
const RECORD_PUBLISHER: &str = "setup_publisher";

/// Why a candidate host was refused, as a slug rather than prose so every
/// surface renders it in its own language.
const REFUSED_NOT_THE_PINNED_PUBLISHER: &str = "host_is_not_the_pinned_publisher";

/// Why a candidate host was refused when no publisher is pinned at all.
const REFUSED_NO_PINNED_PUBLISHER: &str = "no_pinned_publisher_for_this_program";

/// Typed setup notation was not present in otherwise attributable publisher
/// bytes. Prose is evidence for a human, not an executable process request.
const REFUSED_NO_TYPED_RECIPE: &str = "publisher_payload_has_no_typed_setup_recipe";

/// An install procedure, with the provenance that makes it trustable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupProcedure {
    /// The program this procedure installs.
    pub program: String,
    /// The trusted publisher the procedure came from, by `sources_registry` id.
    pub source_id: String,
    /// The exact URL fetched.
    pub source_url: String,
    /// Content id of the bytes retrieved. Empty when the publisher was
    /// identified but its bytes were not retrieved in this run, which is an
    /// honest state and not a silent success.
    pub content_id: String,
    /// Platform this procedure is valid for.
    pub platform: Platform,
    /// Ordered steps, each with its own postcondition.
    pub steps: Vec<SetupStep>,
    /// The probe that must pass afterwards. Without it the procedure is refused.
    pub postcondition: Option<super::probe::ToolchainProbe>,
}

/// One ordered step of a setup procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupStep {
    /// Human-readable rendering of the documented call, retained as evidence.
    /// Execution uses `program` and `arguments`, never a shell parser.
    pub command: String,
    /// Exact executable to resolve under the granted environment.
    pub program: String,
    /// Exact argument vector. Shell punctuation remains ordinary data.
    pub arguments: Vec<String>,
    /// Where the step is allowed to write. A step outside the workspace root is
    /// refused before execution, whatever the fetched text says.
    pub writes_under: PathBuf,
    /// Expected artifact digest, when the publisher documents one.
    pub digest: Option<String>,
    /// Whether this step needs network access. Process permission does not
    /// imply network permission.
    pub requires_network: bool,
}

/// One row of `data/seed/setup-publishers.lino`: which host is authoritative for
/// which program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupPublisher {
    /// The program the row is authoritative for.
    pub program: String,
    /// The `sources-registry` id of the publisher.
    pub source_id: String,
    /// The host the publisher actually serves from.
    pub host: String,
}

impl SetupPublisher {
    /// The documented URL this publisher's procedure is read from.
    #[must_use]
    pub fn documentation_url(&self) -> String {
        ["https://", self.host.as_str(), "/"].concat()
    }
}

/// Every publisher row declared in seed, in file order.
#[must_use]
pub fn seed_publishers() -> Vec<SetupPublisher> {
    let root = parse_lino(SETUP_PUBLISHERS_LINO);
    root.children
        .iter()
        .filter(|node| node.find_child_value("record_type") == RECORD_PUBLISHER)
        .map(|node| SetupPublisher {
            program: node.find_child_value("program").to_owned(),
            source_id: node.find_child_value("source_id").to_owned(),
            host: node.find_child_value("host").to_owned(),
        })
        .collect()
}

/// Whether an attributable URL names exactly `expected`, not a lookalike whose
/// path or suffix merely contains it.
fn url_has_host(url: &str, expected: &str) -> bool {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let authority = without_scheme.split('/').next().unwrap_or_default();
    let host_port = authority.rsplit('@').next().unwrap_or_default();
    let host = host_port.split(':').next().unwrap_or_default();
    host.eq_ignore_ascii_case(expected)
}

fn safe_program_name(program: &str) -> bool {
    !program.is_empty()
        && program
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.+".contains(&byte))
}

fn arguments(node: &crate::seed::parser::LinoNode) -> Option<Vec<String>> {
    node.children
        .iter()
        .filter(|child| child.name == "argument")
        .map(|child| {
            if child
                .id
                .chars()
                .any(|character| matches!(character, '\0' | '\n' | '\r'))
            {
                None
            } else {
                Some(child.id.clone())
            }
        })
        .collect()
}

fn display_command(program: &str, arguments: &[String]) -> String {
    let mut rendered = program.to_owned();
    for argument in arguments {
        rendered.push(' ');
        rendered.push_str(argument);
    }
    rendered
}

/// Formalize one retrieved first-party document into the deliberately small
/// setup grammar. Ordinary prose cannot cross this boundary: every process is
/// a `program` plus repeated `argument` links, with an explicit write scope and
/// network declaration.
fn procedure_from_sense(
    need: &PrerequisiteNeed,
    publisher: &SetupPublisher,
    sense: &crate::concept_lookup::ConceptSense,
) -> Option<SetupProcedure> {
    if !url_has_host(&sense.source_url, &publisher.host) {
        return None;
    }
    let root = parse_lino(&sense.gloss);
    let recipe = root
        .children
        .iter()
        .find(|node| node.name == "setup_procedure")?;
    if recipe.find_child_value("program") != need.program {
        return None;
    }
    let platform = recipe.find_child_value("platform");
    if !platform.is_empty() && platform != "any" && platform != need.platform.slug() {
        return None;
    }

    let mut steps = Vec::new();
    for node in recipe.children.iter().filter(|node| node.name == "step") {
        let program = node.find_child_value("program");
        if !safe_program_name(program) {
            return None;
        }
        let arguments = arguments(node)?;
        let writes_under = node.find_child_value("writes_under");
        if writes_under.is_empty() {
            return None;
        }
        steps.push(SetupStep {
            command: display_command(program, &arguments),
            program: program.to_owned(),
            arguments,
            writes_under: PathBuf::from(writes_under),
            digest: (!node.find_child_value("digest").is_empty())
                .then(|| node.find_child_value("digest").to_owned()),
            requires_network: matches!(
                node.find_child_value("requires_network"),
                "true" | "required"
            ),
        });
    }
    if steps.is_empty() {
        return None;
    }

    let postcondition = recipe
        .children
        .iter()
        .find(|node| node.name == "postcondition")?;
    let probe_program = postcondition.find_child_value("program");
    if !safe_program_name(probe_program) {
        return None;
    }
    let probe_arguments = arguments(postcondition)?;

    Some(SetupProcedure {
        program: need.program.clone(),
        source_id: sense.source_id.clone(),
        source_url: sense.source_url.clone(),
        content_id: sense.sha256.clone(),
        platform: need.platform,
        steps,
        postcondition: Some(super::probe::ToolchainProbe::new(
            probe_program,
            &probe_arguments
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        )),
    })
}

/// Build a workspace-scoped recipe for a Python package in a pinned Git repository.
///
/// This is a general package recipe: the caller
/// supplies the target capability, repository, immutable revision and import
/// probe, while the publisher registry decides whether that source is trusted.
pub fn pinned_python_repository_procedure(
    program: &str,
    repository_url: &str,
    revision: &str,
    import_module: &str,
    platform: Platform,
) -> Result<SetupProcedure, PublisherRefusal> {
    let Some(publisher) = seed_publishers()
        .into_iter()
        .find(|publisher| publisher.program == program)
    else {
        return Err(PublisherRefusal {
            host: repository_url.to_owned(),
            program: program.to_owned(),
            reason: String::from(REFUSED_NO_PINNED_PUBLISHER),
        });
    };
    let immutable_revision =
        revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit());
    let safe_import = !import_module.is_empty()
        && import_module
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'.');
    if !url_has_host(repository_url, &publisher.host) || !immutable_revision || !safe_import {
        return Err(PublisherRefusal {
            host: repository_url.to_owned(),
            program: program.to_owned(),
            reason: [REFUSED_NOT_THE_PINNED_PUBLISHER, publisher.host.as_str()].join(":"),
        });
    }

    let repository = repository_url.trim_end_matches('/');
    let package = format!("git+{repository}.git@{revision}");
    let create_arguments = vec![
        String::from("-m"),
        String::from("venv"),
        String::from("{prefix}"),
    ];
    let install_arguments = vec![
        String::from("-m"),
        String::from("pip"),
        String::from("install"),
        String::from("--disable-pip-version-check"),
        package,
    ];
    let content_id = crate::engine::stable_id(
        "setup-procedure",
        &[program, repository_url, revision, import_module].join("\u{1f}"),
    );
    let Some(import_probe) = crate::coding::python_render::runtime_template(
        "publisher_python_import",
        &[("module", import_module)],
    ) else {
        return Err(PublisherRefusal {
            host: repository_url.to_owned(),
            program: program.to_owned(),
            reason: String::from("missing_runtime_template:publisher_python_import"),
        });
    };

    Ok(SetupProcedure {
        program: program.to_owned(),
        source_id: publisher.source_id,
        source_url: format!("{repository_url}/tree/{revision}"),
        content_id,
        platform,
        steps: vec![
            SetupStep {
                command: display_command("python3", &create_arguments),
                program: String::from("python3"),
                arguments: create_arguments,
                writes_under: PathBuf::from("."),
                digest: None,
                requires_network: false,
            },
            SetupStep {
                command: display_command("python3", &install_arguments),
                program: String::from("python3"),
                arguments: install_arguments,
                writes_under: PathBuf::from("."),
                digest: None,
                requires_network: true,
            },
        ],
        postcondition: Some(super::probe::ToolchainProbe::new(
            "python3",
            &["-c", &import_probe],
        )),
    })
}

/// Why a candidate procedure was refused, recorded rather than dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherRefusal {
    /// The host that was refused.
    pub host: String,
    /// The program it claimed to publish.
    pub program: String,
    /// Why it was refused.
    pub reason: String,
}

/// What the publisher search observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublisherSearch {
    /// The procedure found, when one carried a postcondition.
    pub procedure: Option<SetupProcedure>,
    /// Registry ids consulted, in consultation order.
    pub consulted: Vec<String>,
    /// Every candidate refused, with its reason.
    pub refusals: Vec<PublisherRefusal>,
    /// The dependency cycle detected, when one was.
    pub cycle: Vec<String>,
}

/// Walk `need`'s requirements and report the chain that returns to a program
/// already on it. An empty result means no cycle.
#[must_use]
pub fn detect_cycle(need: &PrerequisiteNeed) -> Vec<String> {
    let mut chain = vec![need.program.clone()];
    let mut frontier: Vec<String> = need.requires.clone();
    let mut guard = 0_usize;
    while let Some(next) = frontier.pop() {
        guard += 1;
        if guard > 64 {
            break;
        }
        if chain.contains(&next) {
            chain.push(next);
            return chain;
        }
        chain.push(next.clone());
        if let Some(probe) = super::probe::seed_probe_for_program(&next) {
            frontier.extend(probe.requires);
        }
    }
    Vec::new()
}

/// Find `program`'s official setup procedure through the trusted-source
/// registry, deepest-first over the source kinds declared in
/// `data/seed/sources-registry.lino`.
///
/// Bounded by evidence, not by a budget: the search ends when a procedure with a
/// postcondition is found, when the publisher list is exhausted, or when a cycle
/// is detected.
pub fn discover_setup_procedure<L: SourceLookup>(
    need: &PrerequisiteNeed,
    lookup: &mut L,
    bounds: &LookupBounds,
) -> Option<SetupProcedure> {
    search_setup_procedure(need, lookup, bounds).procedure
}

/// The same search, reporting everything it observed rather than only its result.
pub fn search_setup_procedure<L: SourceLookup>(
    need: &PrerequisiteNeed,
    lookup: &mut L,
    bounds: &LookupBounds,
) -> PublisherSearch {
    let cycle = detect_cycle(need);
    if !cycle.is_empty() {
        return PublisherSearch {
            procedure: None,
            consulted: Vec::new(),
            refusals: Vec::new(),
            cycle,
        };
    }

    let mut record = Need::raised(
        NeedKind::Prerequisite,
        &need.program,
        need.platform.slug(),
        "prerequisite_recover",
    );
    record.source_span.clone_from(&need.source_span);

    let pinned = seed_publishers()
        .into_iter()
        .find(|publisher| publisher.program == need.program);

    let mut consulted = Vec::new();
    let mut refusals = Vec::new();
    let mut procedure = None;

    match lookup.lookup(&record, bounds) {
        LookupOutcome::Found(senses) => {
            for sense in senses {
                if !consulted.contains(&sense.source_id) {
                    consulted.push(sense.source_id.clone());
                }
                match pinned.as_ref() {
                    Some(publisher) if url_has_host(&sense.source_url, &publisher.host) => {
                        if procedure.is_none() {
                            procedure = procedure_from_sense(need, publisher, &sense);
                            if procedure.is_none() {
                                refusals.push(PublisherRefusal {
                                    host: sense.source_url.clone(),
                                    program: need.program.clone(),
                                    reason: String::from(REFUSED_NO_TYPED_RECIPE),
                                });
                            }
                        }
                    }
                    Some(publisher) => refusals.push(PublisherRefusal {
                        host: sense.source_url.clone(),
                        program: need.program.clone(),
                        reason: [REFUSED_NOT_THE_PINNED_PUBLISHER, publisher.host.as_str()]
                            .join(":"),
                    }),
                    None => refusals.push(PublisherRefusal {
                        host: sense.source_url.clone(),
                        program: need.program.clone(),
                        reason: String::from(REFUSED_NO_PINNED_PUBLISHER),
                    }),
                }
            }
        }
        LookupOutcome::NotFound { consulted: walked } => {
            for outcome in walked {
                if !consulted.contains(&outcome.source_id) {
                    consulted.push(outcome.source_id.clone());
                }
                match pinned.as_ref() {
                    Some(publisher) if url_has_host(&outcome.detail, &publisher.host) => {}
                    Some(publisher) => refusals.push(PublisherRefusal {
                        host: outcome.detail.clone(),
                        program: need.program.clone(),
                        reason: [REFUSED_NOT_THE_PINNED_PUBLISHER, publisher.host.as_str()]
                            .join(":"),
                    }),
                    None => refusals.push(PublisherRefusal {
                        host: outcome.detail.clone(),
                        program: need.program.clone(),
                        reason: String::from(REFUSED_NO_PINNED_PUBLISHER),
                    }),
                }
            }
        }
    }

    PublisherSearch {
        procedure,
        consulted,
        refusals,
        cycle: Vec::new(),
    }
}
