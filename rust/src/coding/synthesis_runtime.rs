//! Runtime assembly for research-backed coding synthesis.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::coding::composition;
use crate::coding::concept_discovery::{CatalogFunction, ConceptMap, DiscoveryCatalog};
use crate::coding::discovered_procedures::DiscoveredProcedureLedger;
use crate::coding::function_catalog::oeis::discover_programs;
use crate::coding::function_catalog::python_docs::{StdlibIndex, fetch_index};
use crate::coding::function_catalog::wikifunctions::{
    FunctionMatch, fetch_abstract_implementation, fetch_function, fetch_function_descriptors,
    fetch_implementations, fetch_testers, search_functions,
};
use crate::coding::recurrence::formalize;
use crate::coding::task_spec::{CodingTaskSpec, Example};
use crate::engine::{ExecutionRecipe, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::Language;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport};

/// Where every capture is read from and written to.
///
/// `FORMAL_AI_SOURCE_CACHE_DIR`, then `FORMAL_AI_CACHE_DIR`, then `data`. It is
/// a function rather than three lines repeated per caller because the universal
/// loop and the coding path must share one cache: two roots would be two
/// different answers to the same question about the same word (issue #1138,
/// plan 01 L11).
#[must_use]
pub fn source_cache_root() -> String {
    std::env::var("FORMAL_AI_SOURCE_CACHE_DIR")
        .or_else(|_| std::env::var("FORMAL_AI_CACHE_DIR"))
        .unwrap_or_else(|_| String::from("data"))
}

pub fn discovery_catalog(
    spec: &CodingTaskSpec,
    log: &mut EventLog,
    live: bool,
) -> DiscoveryCatalog {
    let cache_dir = source_cache_root();
    let client = CachedSourceClient::new(cache_dir, CurlSourceTransport).with_online(live);
    let stdlib = if live {
        fetch_index(&client).unwrap_or_default()
    } else {
        static OFFLINE_STDLIB: OnceLock<StdlibIndex> = OnceLock::new();
        OFFLINE_STDLIB
            .get_or_init(|| fetch_index(&client).unwrap_or_default())
            .clone()
    };

    let phrases = research_phrases(spec);
    let mut matches: Vec<FunctionMatch> = Vec::new();
    for phrase in phrases {
        for language in [spec.prose_language.as_str(), "en"] {
            match search_functions(&client, &phrase, language) {
                Ok(found) => {
                    for matched in found {
                        if let Some(existing) = matches
                            .iter_mut()
                            .find(|existing| existing.page_title == matched.page_title)
                        {
                            if matched.match_rate > existing.match_rate {
                                *existing = matched;
                            }
                        } else {
                            log.append(
                                "source:http",
                                format!(
                                    "url={};fetched_at={};sha256={};catalog_match={}",
                                    matched.source_url,
                                    matched.fetched_at,
                                    matched.sha256,
                                    matched.page_title
                                ),
                            );
                            matches.push(matched);
                        }
                    }
                }
                Err(error) => {
                    log.append("synthesis:source_miss", error.to_string());
                }
            }
            if language == "en" || spec.prose_language == "en" {
                break;
            }
        }
    }

    let requirement_text = spec.requirement_sentences.join(" ");
    matches.sort_by(|left, right| {
        right
            .match_rate
            .total_cmp(&left.match_rate)
            .then_with(|| {
                token_overlap(&requirement_text, &right.label)
                    .cmp(&token_overlap(&requirement_text, &left.label))
            })
            .then_with(|| left.page_title.cmp(&right.page_title))
    });

    let mut functions = Vec::new();
    for matched in matches
        .iter()
        .filter(|item| item.match_rate >= 0.65)
        .take(4)
    {
        let Ok(definition) = fetch_function(&client, &matched.page_title) else {
            continue;
        };
        let implementations =
            fetch_implementations(&client, &definition.implementation_zids).unwrap_or_default();
        let mut recurrence = None;
        let mut source_tests = Vec::new();
        for implementation_zid in definition.implementation_zids.iter().take(8) {
            let Ok(abstract_implementation) =
                fetch_abstract_implementation(&client, implementation_zid)
            else {
                continue;
            };
            let operator_zids = abstract_implementation
                .expression
                .function_zids()
                .into_iter()
                .filter(|zid| zid != &definition.zid)
                .collect::<Vec<_>>();
            let Ok(descriptors) = fetch_function_descriptors(&client, &operator_zids) else {
                continue;
            };
            let Ok(formalized) = formalize(&definition, &abstract_implementation, &descriptors)
            else {
                continue;
            };
            let tester_zids = definition
                .tester_zids
                .iter()
                .take(8)
                .cloned()
                .collect::<Vec<_>>();
            source_tests = fetch_testers(&client, &tester_zids)
                .unwrap_or_default()
                .into_iter()
                .filter(|test| test.function_zid == definition.zid && test.arguments.len() == 1)
                .filter(|test| {
                    test.arguments[0]
                        .parse::<u64>()
                        .is_ok_and(|argument| argument <= 20)
                })
                .take(4)
                .map(|test| Example {
                    arguments: test.arguments,
                    expected: test.expected,
                })
                .collect();
            recurrence = Some(formalized);
            break;
        }
        functions.push(CatalogFunction {
            definition,
            implementations,
            recurrence,
            source_tests,
        });
    }
    DiscoveryCatalog::new(stdlib, matches, functions)
}

pub fn extend_with_sequence_programs(
    catalog: DiscoveryCatalog,
    spec: &CodingTaskSpec,
    log: &mut EventLog,
    live: bool,
) -> (DiscoveryCatalog, Vec<crate::coding::program_ir::ProgramIr>) {
    let cache_dir = std::env::var("FORMAL_AI_SOURCE_CACHE_DIR")
        .or_else(|_| std::env::var("FORMAL_AI_CACHE_DIR"))
        .unwrap_or_else(|_| String::from("data"));
    let client = CachedSourceClient::new(cache_dir, CurlSourceTransport).with_online(live);
    let sequence_discovery = discover_programs(&client, spec);
    for diagnostic in sequence_discovery.diagnostics {
        log.append("synthesis:source_miss", diagnostic);
    }
    let mut source_candidates = Vec::new();
    let mut programs = Vec::new();
    for program in sequence_discovery.programs {
        log.append(
            "source:http",
            format!(
                "url={};fetched_at={};sha256={};catalog_match={}",
                program.source_url, program.fetched_at, program.sha256, program.id
            ),
        );
        if let Some(ir) = program.program_ir {
            programs.push(ir);
        } else {
            source_candidates.push(crate::coding::concept_discovery::CandidatePart {
                id: program.id,
                kind: "source_program".to_owned(),
                label: program.composition,
                language: Some("python".to_owned()),
                code: Some(program.source),
                callable_name: Some(program.callable_name),
                source_tests: Vec::new(),
                license: program.license,
                source_url: program.source_url,
                sha256: program.sha256,
                fetched_at: program.fetched_at,
                score: 1.0,
            });
        }
    }
    (catalog.with_source_candidates(source_candidates), programs)
}

/// Discover candidate parts, compose them, and expand to sequence sources only
/// when the first bounded pass cannot verify a draft.
pub fn discover_and_compose(
    spec: &CodingTaskSpec,
    log: &mut EventLog,
    live: bool,
) -> (ConceptMap, composition::CompositionOutcome) {
    let mut catalog = discovery_catalog(spec, log, live);
    let mut concepts = discover_over_registry(spec, &catalog, log, live);
    let mut outcome = composition::compose(spec, &concepts);
    if outcome.selected.is_none() {
        let cache_dir = source_cache_root();
        let client = CachedSourceClient::new(&cache_dir, CurlSourceTransport).with_online(live);
        let ledger = crate::coding::fragment_catalog::FragmentLedger::new(&cache_dir);
        let fragment_catalog = crate::coding::fragment_catalog::FragmentCatalog::bootstrap()
            .with_rediscovered(&ledger);
        let absent_seeds = crate::coding::fragment_catalog::FragmentCatalog::absent_seed_files(
            crate::coding::fragment_catalog::bootstrap_seed_directory(),
        );
        if !absent_seeds.is_empty() {
            log.append("bootstrap_absent", absent_seeds.join(", "));
        }
        let lookup_bounds = crate::source_walk::LookupBounds::default();
        let elaboration_bounds = crate::coding::program_ir::ElaborationBounds {
            max_candidates: 64,
            max_depth: 4,
        };
        let phrases = concepts
            .needs
            .iter()
            .map(|need| need.phrase().to_owned())
            .collect::<Vec<_>>();
        let mut programs = Vec::new();
        for phrase in phrases {
            for (_, steps) in crate::procedure_text::retrieve_procedure(
                &phrase,
                &spec.prose_language,
                &client,
                &lookup_bounds,
                log,
            ) {
                programs.extend(crate::coding::program_ir::elaborate(
                    spec,
                    &steps,
                    &fragment_catalog,
                    elaboration_bounds,
                ));
            }
        }
        programs.sort_by_key(crate::coding::program_ir::ProgramIr::content_id);
        programs.dedup_by(|left, right| left.content_id() == right.content_id());
        if !programs.is_empty() {
            log.append(
                "synthesis:procedure_elaboration",
                format!("candidate_count={}", programs.len()),
            );
            outcome = composition::compose_with_ir(spec, &concepts, &fragment_catalog, programs);
        }
    }
    if outcome.selected.is_none() {
        let (extended, programs) = extend_with_sequence_programs(catalog, spec, log, live);
        catalog = extended;
        if !programs.is_empty() {
            let cache_dir = source_cache_root();
            let ledger = crate::coding::fragment_catalog::FragmentLedger::new(&cache_dir);
            let fragments = crate::coding::fragment_catalog::FragmentCatalog::bootstrap()
                .with_rediscovered(&ledger);
            outcome = composition::compose_with_ir(spec, &concepts, &fragments, programs);
        }
        if outcome.selected.is_none() && !catalog.source_candidates.is_empty() {
            concepts = discover_over_registry(spec, &catalog, log, live);
            outcome = composition::compose(spec, &concepts);
        }
    }
    (concepts, outcome)
}

/// Discover with the registry lookup under it, so a word the seed does not
/// contain is *asked about* rather than skipped (issue #1138, plan 01 L10).
///
/// The lookup is the same one the universal loop builds — one cache root, one
/// registry, one set of bounds — and every source it consulted is written to
/// the event log, including the ones that could not serve the language, so an
/// absent gloss stays attributable.
pub fn discover_over_registry(
    spec: &CodingTaskSpec,
    catalog: &DiscoveryCatalog,
    log: &mut EventLog,
    live: bool,
) -> ConceptMap {
    let client =
        CachedSourceClient::new(source_cache_root(), CurlSourceTransport).with_online(live);
    let preferences = crate::how_to_guide::ServicePreferences::default();
    let mut availability = crate::service_accessibility::ServiceAccessibilityCache::new(
        std::path::Path::new(&source_cache_root()).join("cache/service-accessibility"),
    );
    let bounds = crate::source_walk::LookupBounds::default();
    let now = crate::service_accessibility::unix_now();
    let mut lookup = crate::concept_lookup::RegistrySourceLookup::new(
        &client,
        &preferences,
        &mut availability,
        bounds,
        &spec.prose_language,
        now,
    );
    let map = crate::coding::concept_discovery::discover_with_lookup(
        spec,
        catalog,
        &mut lookup,
        crate::coding::concept_discovery::DiscoveryBounds::default(),
    );
    for row in lookup.outcomes() {
        log.append(
            "concept_lookup:source",
            format!(
                "source={};status={};detail={}",
                row.source_id, row.status, row.detail
            ),
        );
    }
    map
}

/// Preserve a verified artifact as a typed execution recipe so every agent
/// protocol can select its own advertised write and execution capabilities.
pub fn attach_execution_recipe(
    mut answer: SymbolicAnswer,
    spec: &CodingTaskSpec,
    selected: composition::VerifiedDraft,
) -> SymbolicAnswer {
    let program = spec.artifact_shape == crate::coding::task_spec::ArtifactShape::Program;
    let language = crate::coding::program_language_by_slug(&spec.language);
    let path = language.map_or_else(
        || format!("main.{}", spec.language),
        |language| language.save_as.to_owned(),
    );
    let commands = language.map_or_else(Vec::new, |language| {
        language
            .execution
            .check_command
            .into_iter()
            .chain(program.then_some(language.execution.run_command))
            .map(str::to_owned)
            .collect()
    });
    answer.execution_recipe = Some(Box::new(ExecutionRecipe {
        language: spec.language.clone(),
        source: selected.source,
        path,
        supporting_files: Vec::new(),
        commands,
    }));
    answer
}

fn research_phrases(spec: &CodingTaskSpec) -> Vec<String> {
    let lexicon = crate::seed::lexicon();
    let grammatical_roles = [
        crate::seed::ROLE_CODING_REQUEST_VERB,
        crate::seed::ROLE_CODING_REQUEST_OBJECT,
        crate::seed::ROLE_PROGRAM_SYNTHESIS_SUBJECT,
        crate::seed::ROLE_PROGRAM_SYNTHESIS_ACTION,
        crate::seed::ROLE_PROGRAM_SYNTHESIS_DOMAIN,
    ];
    let mut phrases = Vec::new();
    for sentence in &spec.requirement_sentences {
        phrases.push(sentence.clone());
        let normalized = crate::engine::normalize_prompt(sentence);
        let content = normalized
            .split_whitespace()
            .filter(|token| {
                !grammatical_roles
                    .iter()
                    .any(|role| lexicon.mentions_role(role, token))
            })
            .collect::<Vec<_>>();
        if !content.is_empty() {
            phrases.push(content.join(" "));
        }
        phrases.extend(
            content
                .into_iter()
                .filter(|token| token.chars().count() >= 4)
                .map(str::to_owned),
        );
    }
    if spec.name != "discovered_function" {
        phrases.push(spec.name.replace('_', " "));
    }
    let mut seen = BTreeSet::new();
    phrases
        .into_iter()
        .filter(|phrase| seen.insert(phrase.clone()))
        .take(8)
        .collect()
}

fn token_overlap(left: &str, right: &str) -> usize {
    let tokens = |value: &str| {
        crate::engine::normalize_prompt(value)
            .split_whitespace()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    };
    tokens(left).intersection(&tokens(right)).count()
}

pub fn live_fetch_enabled() -> bool {
    std::env::var("FORMAL_AI_LIVE_FETCH").is_ok_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

pub fn procedure_ledger() -> Option<DiscoveredProcedureLedger> {
    std::env::var("FORMAL_AI_CACHE_DIR")
        .ok()
        .filter(|path| !path.trim().is_empty())
        .map(DiscoveredProcedureLedger::new)
}

pub fn response_language(spec: &CodingTaskSpec) -> Language {
    crate::language::from_slug(&spec.prose_language).unwrap_or(Language::English)
}

fn localized_template(intent: &str, language: Language) -> String {
    crate::seed::response_for(intent, language.slug())
        .or_else(|| crate::seed::response_for(intent, Language::English.slug()))
        .unwrap_or_default()
}

pub fn research_trail_line(language: Language, trail: &str) -> String {
    let values = [("trail", trail)];
    crate::seed::render_response("coding_synthesis_research_trail", language.slug(), &values)
        .or_else(|| {
            crate::seed::render_response(
                "coding_synthesis_research_trail",
                Language::English.slug(),
                &values,
            )
        })
        .unwrap_or_default()
}

pub fn render_answer(selected: &composition::VerifiedDraft, language: Language) -> String {
    let count = selected.assertion_count.to_string();
    let source = selected.source.trim_end();
    let values = [("source", source), ("count", count.as_str())];
    let mut body =
        crate::seed::render_response("coding_synthesis_verified_answer", language.slug(), &values)
            .or_else(|| {
                crate::seed::render_response(
                    "coding_synthesis_verified_answer",
                    Language::English.slug(),
                    &values,
                )
            })
            .unwrap_or_default();
    if !selected.source_urls.is_empty() {
        let _ = write!(
            body,
            "\n{}",
            localized_template("coding_synthesis_sources_heading", language)
        );
        for (index, url) in selected.source_urls.iter().enumerate() {
            let license = selected
                .source_licenses
                .get(index)
                .map_or("source-declared", String::as_str);
            let _ = write!(body, "\n- {url} ({license})");
        }
    }
    body
}
