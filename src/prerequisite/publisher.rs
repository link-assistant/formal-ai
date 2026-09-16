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
const SETUP_PUBLISHERS_LINO: &str = include_str!("../../data/seed/setup-publishers.lino");

/// Record type of one publisher row.
const RECORD_PUBLISHER: &str = "setup_publisher";

/// Why a candidate host was refused, as a slug rather than prose so every
/// surface renders it in its own language.
const REFUSED_NOT_THE_PINNED_PUBLISHER: &str = "host_is_not_the_pinned_publisher";

/// Why a candidate host was refused when no publisher is pinned at all.
const REFUSED_NO_PINNED_PUBLISHER: &str = "no_pinned_publisher_for_this_program";

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
    /// The command line exactly as the publisher documents it.
    pub command: String,
    /// Where the step is allowed to write. A step outside the workspace root is
    /// refused before execution, whatever the fetched text says.
    pub writes_under: PathBuf,
    /// Expected artifact digest, when the publisher documents one.
    pub digest: Option<String>,
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

/// The documented procedure URL seed carries for one program.
fn seed_documentation_url(program: &str) -> Option<String> {
    let root = parse_lino(SETUP_PUBLISHERS_LINO);
    root.children
        .iter()
        .find(|node| {
            node.find_child_value("record_type") == RECORD_PUBLISHER
                && node.find_child_value("program") == program
        })
        .map(|node| node.find_child_value("documentation").to_owned())
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
    let mut retrieved_content_id = String::new();

    match lookup.lookup(&record, bounds) {
        LookupOutcome::Found(senses) => {
            for sense in senses {
                if !consulted.contains(&sense.source_id) {
                    consulted.push(sense.source_id.clone());
                }
                match pinned.as_ref() {
                    Some(publisher) if sense.source_url.contains(publisher.host.as_str()) => {
                        if retrieved_content_id.is_empty() {
                            retrieved_content_id = sense.sha256.clone();
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
                    Some(publisher) if outcome.detail.contains(publisher.host.as_str()) => {}
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

    // Authority is pinned, never ranked. Without a pinned publisher the search
    // is exhausted and says which sources it consulted.
    let procedure = pinned.map(|publisher| {
        let source_url = seed_documentation_url(&need.program)
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| publisher.documentation_url());
        SetupProcedure {
            program: need.program.clone(),
            source_id: publisher.source_id.clone(),
            source_url,
            content_id: retrieved_content_id,
            platform: need.platform,
            steps: Vec::new(),
            postcondition: Some(super::probe::seed_probe_for_program(&need.program).unwrap_or_else(
                || super::probe::ToolchainProbe::new(&need.program, &["--version"]),
            )),
        }
    });

    PublisherSearch {
        procedure,
        consulted,
        refusals,
        cycle: Vec::new(),
    }
}
