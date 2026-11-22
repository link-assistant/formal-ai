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
use crate::coding::recurrence::{cache::recurrence_fold, formalize};
use crate::coding::task_spec::{CodingTaskSpec, Example};
use crate::engine::{ExecutionRecipe, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::Language;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};

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
    let directory = source_cache_root();
    let client = CachedSourceClient::new(directory, CurlSourceTransport).with_online(live);
    let stdlib = if live {
        fetch_index(&client).unwrap_or_default()
    } else {
        static OFFLINE_STDLIB: OnceLock<StdlibIndex> = OnceLock::new();
        OFFLINE_STDLIB
            .get_or_init(|| fetch_index(&client).unwrap_or_default())
            .clone()
    };

    discovery_catalog_with_client(spec, log, &client, stdlib, false)
}

/// Discover function signatures, implementations and verification inputs through a
/// caller-supplied client. Offline tests exercise the same path as live reads.
fn discovery_catalog_with_client<T: SourceTransport>(
    spec: &CodingTaskSpec,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
    stdlib: StdlibIndex,
    recurrence_only: bool,
) -> DiscoveryCatalog {
    let preferences = crate::solver_handler_how_synthesis::service_preferences_from_env(log);
    if !crate::seed::source_record("wikifunctions")
        .is_some_and(|record| preferences.allows(&record))
    {
        return DiscoveryCatalog::new(stdlib, Vec::new(), Vec::new());
    }
    let language = spec.prose_language.as_str();
    let wikipedia = crate::seed::source_record("wikipedia").filter(|record| {
        language != "en"
            && record.api_language.iter().any(|locale| locale == language)
            && preferences.allows(record)
    });
    let mut matches: Vec<FunctionMatch> = Vec::new();
    for phrase in research_phrases(spec) {
        let mut found = catalogue_matches(client, log, &phrase, language);
        if let Some(record) = &wikipedia
            && !found
                .iter()
                .any(|candidate| recurrence_match(&phrase, candidate))
            && let Some(label) = english_article_title(client, log, &record.api, language, &phrase)
        {
            for mut matched in catalogue_matches(client, log, &label, "en") {
                if recurrence_match(&label, &matched) {
                    matched.match_label.clone_from(&phrase);
                    matched.match_lang = language.to_owned();
                    found.push(matched);
                }
            }
        }
        for matched in found {
            let exact = recurrence_match(&phrase, &matched);
            if recurrence_only && !exact {
                continue;
            }
            if let Some(index) = matches.iter().position(|existing| {
                existing.page_title == matched.page_title
                    && existing.match_label == matched.match_label
            }) {
                if matched.match_rate > matches[index].match_rate {
                    matches[index] = matched;
                }
            } else {
                log.append(
                    "source:http",
                    format!(
                        "url={};fetched_at={};sha256={};catalog_match={};subject_match={exact}",
                        matched.source_url, matched.fetched_at, matched.sha256, matched.page_title
                    ),
                );
                matches.push(matched);
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
    let mut seen = BTreeSet::new();
    for matched in matches
        .iter()
        .filter(|item| item.match_rate >= 0.65 && seen.insert(item.page_title.as_str()))
        .take(4)
    {
        let Ok(definition) = fetch_function(client, &matched.page_title) else {
            continue;
        };
        let implementations = if recurrence_only {
            Vec::new()
        } else {
            fetch_implementations(
                client,
                &definition.implementation_zids[..definition.implementation_zids.len().min(8)],
            )
            .unwrap_or_default()
        };
        let mut recurrence = None;
        let mut source_tests = Vec::new();
        for implementation_zid in definition.implementation_zids.iter().take(8) {
            let Ok(abstract_implementation) =
                fetch_abstract_implementation(client, implementation_zid)
            else {
                continue;
            };
            let operator_zids = abstract_implementation
                .expression
                .function_zids()
                .into_iter()
                .filter(|zid| zid != &definition.zid)
                .collect::<Vec<_>>();
            if operator_zids.is_empty() || operator_zids.len() > 8 {
                continue;
            }
            let Ok(descriptors) = fetch_function_descriptors(client, &operator_zids) else {
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
            source_tests = fetch_testers(client, &tester_zids)
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
            if !source_tests.is_empty() {
                recurrence = Some(formalized);
                break;
            }
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
                callable_contract: None,
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
    trace_synthesis_stage("synthesis_catalog_ready");
    let mut concepts = discover_over_registry(spec, &catalog, log, live);
    trace_synthesis_stage("synthesis_concepts_ready");
    let mut outcome = composition::compose_for_verification(spec, &concepts);
    trace_synthesis_stage("synthesis_initial_composition_finished");
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
        trace_synthesis_stage("synthesis_sequence_discovery_started");
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
            outcome = composition::compose_for_verification(spec, &concepts);
        }
    }
    (concepts, outcome)
}

pub(crate) fn trace_synthesis_stage(stage: &'static str) {
    if crate::solver_config::flag_enabled("FORMAL_AI_TRACE_REQUESTS") {
        crate::dialog_log::structured_log::event(
            "trace",
            module_path!(),
            || stage.to_owned(),
            || serde_json::json!({"stage":stage}),
        );
    }
}

/// Try publisher recurrences before a catalog template claims a callable request.
pub(crate) fn discover_recurrence_and_compose(
    spec: &CodingTaskSpec,
    log: &mut EventLog,
    live: bool,
) -> Option<(ConceptMap, composition::CompositionOutcome)> {
    let client =
        CachedSourceClient::new(source_cache_root(), CurlSourceTransport).with_online(live);
    let mut catalog =
        discovery_catalog_with_client(spec, log, &client, StdlibIndex::default(), true);
    catalog.functions.retain(|item| item.recurrence.is_some());
    if catalog.functions.is_empty() {
        return None;
    }
    let mut concepts = discover_with_source_depth(spec, &catalog, log, false, 0);
    for need in &mut concepts.needs {
        need.candidates
            .retain(|part| part.kind == "wikifunctions_recurrence");
    }
    let fragments = crate::coding::fragment_catalog::FragmentCatalog::bootstrap();
    let outcome = composition::compose_with_ir(spec, &concepts, &fragments, []);
    Some((concepts, outcome))
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
    let depth = crate::coding::concept_discovery::DiscoveryBounds::default().max_depth;
    discover_with_source_depth(spec, catalog, log, live, depth)
}

fn discover_with_source_depth(
    spec: &CodingTaskSpec,
    catalog: &DiscoveryCatalog,
    log: &mut EventLog,
    live: bool,
    max_depth: usize,
) -> ConceptMap {
    let client =
        CachedSourceClient::new(source_cache_root(), CurlSourceTransport).with_online(live);
    let preferences = crate::solver_handler_how_synthesis::service_preferences_from_env(log);
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
        crate::coding::concept_discovery::DiscoveryBounds {
            max_depth,
            ..Default::default()
        },
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
        |language| language.save_as.to_string(),
    );
    let commands = language.map_or_else(Vec::new, |language| {
        language
            .execution
            .check_command
            .as_deref()
            .into_iter()
            .chain(program.then_some(language.execution.run_command.as_ref()))
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
        crate::seed::ROLE_PROGRAM_LANGUAGE_ALIAS,
        crate::seed::ROLE_REQUEST_FUNCTION_WORD,
        crate::seed::ROLE_STATEMENT_FUNCTION_WORD,
    ];
    let mut phrases = Vec::new();
    if spec.name != crate::coding::task_spec::PROVISIONAL_FUNCTION_NAME {
        phrases.push(spec.name.replace('_', " "));
    }
    for sentence in &spec.requirement_sentences {
        let normalized = crate::engine::normalize_prompt(sentence);
        let words = normalized
            .split_whitespace()
            .filter(|token| {
                token.chars().any(char::is_alphabetic)
                    && !grammatical_roles
                        .iter()
                        .any(|role| lexicon.mentions_role(role, token))
            })
            .take(24)
            .collect::<Vec<_>>();
        for length in 1..=3 {
            phrases.extend(
                words
                    .windows(length)
                    .map(|span| span.join(" "))
                    .filter(|phrase| {
                        let size = phrase.chars().count();
                        (3..=80).contains(&size)
                    }),
            );
        }
    }
    let mut seen = BTreeSet::new();
    phrases
        .into_iter()
        .filter(|phrase| seen.insert(phrase.clone()))
        .take(12)
        .collect()
}

fn recurrence_match(phrase: &str, candidate: &FunctionMatch) -> bool {
    let phrase = recurrence_fold(phrase);
    candidate.match_rate >= 0.65
        && [&candidate.label, &candidate.match_label]
            .into_iter()
            .map(|label| recurrence_fold(label))
            .any(|label| label.chars().count() >= 3 && phrase == label)
}

fn catalogue_matches<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    log: &mut EventLog,
    phrase: &str,
    language: &str,
) -> Vec<FunctionMatch> {
    match search_functions(client, phrase, language) {
        Ok(results) => results,
        Err(error) => {
            log.append(
                "synthesis:source_miss",
                format!("query={phrase};language={language};error={error}"),
            );
            Vec::new()
        }
    }
}

fn english_article_title<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    log: &mut EventLog,
    template: &str,
    language: &str,
    phrase: &str,
) -> Option<String> {
    use crate::seed::percent_encode;
    use serde_json::Value;
    let template = template.replace("{language}", language);
    let url = url::Url::parse(&template).ok()?;
    let api = format!(
        "{}/w/api.php?action=query&format=json&formatversion=2",
        url.origin().ascii_serialization()
    );
    let search = client
        .fetch(&format!(
            "{api}&list=search&srlimit=1&srnamespace=0&srsearch={}",
            percent_encode(phrase)
        ))
        .ok()?;
    let value: Value = serde_json::from_slice(search.bytes()).ok()?;
    let result = value.pointer("/query/search/0")?;
    let title = result.get("title")?.as_str()?;
    if title.chars().count() > 160 {
        return None;
    }
    let subject = recurrence_fold(phrase);
    let highlight = result
        .get("snippet")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .split("<span class=\"searchmatch\">")
        .skip(1)
        .filter_map(|span| span.split_once("</span>").map(|(text, _)| text))
        .any(|text| !text.contains('<') && recurrence_fold(text) == subject);
    if recurrence_fold(title) != subject && !highlight {
        return None;
    }
    let links = client
        .fetch(&format!(
            "{api}&prop=langlinks&lllang=en&titles={}",
            percent_encode(title)
        ))
        .ok()?;
    let value: Value = serde_json::from_slice(links.bytes()).ok()?;
    let page = value.pointer("/query/pages/0")?;
    if page.get("title")?.as_str()? != title {
        return None;
    }
    let label = page
        .get("langlinks")?
        .as_array()?
        .iter()
        .find_map(|link| {
            (link.get("lang")?.as_str()? == "en")
                .then(|| link.get("title")?.as_str())
                .flatten()
        })?
        .trim();
    if label.is_empty() || label.chars().count() > 80 {
        return None;
    }
    for capture in [&search, &links] {
        log.append(
            "source:http",
            format!(
                "url={};sha256={};fetched_at={};catalog_subject={phrase}",
                capture.source_url(),
                capture.sha256(),
                capture.fetched_at()
            ),
        );
    }
    Some(label.to_owned())
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
    crate::solver_config::read_network_allowed()
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
