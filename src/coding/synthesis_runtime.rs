//! Runtime assembly for research-backed coding synthesis.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::coding::composition;
use crate::coding::concept_discovery::{CatalogFunction, DiscoveryCatalog};
use crate::coding::discovered_procedures::DiscoveredProcedureLedger;
use crate::coding::function_catalog::oeis::discover_programs;
use crate::coding::function_catalog::python_docs::{StdlibIndex, fetch_index};
use crate::coding::function_catalog::wikifunctions::{
    FunctionMatch, fetch_abstract_implementation, fetch_function, fetch_function_descriptors,
    fetch_implementations, fetch_testers, search_functions,
};
use crate::coding::recurrence::formalize;
use crate::coding::task_spec::{CodingTaskSpec, Example};
use crate::event_log::EventLog;
use crate::language::Language;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport};

pub fn discovery_catalog(
    spec: &CodingTaskSpec,
    log: &mut EventLog,
    live: bool,
) -> DiscoveryCatalog {
    let cache_dir = std::env::var("FORMAL_AI_SOURCE_CACHE_DIR")
        .or_else(|_| std::env::var("FORMAL_AI_CACHE_DIR"))
        .unwrap_or_else(|_| String::from("data"));
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
) -> DiscoveryCatalog {
    let cache_dir = std::env::var("FORMAL_AI_SOURCE_CACHE_DIR")
        .or_else(|_| std::env::var("FORMAL_AI_CACHE_DIR"))
        .unwrap_or_else(|_| String::from("data"));
    let client = CachedSourceClient::new(cache_dir, CurlSourceTransport).with_online(live);
    let sequence_discovery = discover_programs(&client, spec);
    for diagnostic in sequence_discovery.diagnostics {
        log.append("synthesis:source_miss", diagnostic);
    }
    let source_candidates = sequence_discovery
        .programs
        .into_iter()
        .map(|program| {
            log.append(
                "source:http",
                format!(
                    "url={};fetched_at={};sha256={};catalog_match={}",
                    program.source_url, program.fetched_at, program.sha256, program.id
                ),
            );
            crate::coding::concept_discovery::CandidatePart {
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
            }
        })
        .collect();
    catalog.with_source_candidates(source_candidates)
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
    let values = [
        ("source", selected.source.as_str()),
        ("count", count.as_str()),
    ];
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
