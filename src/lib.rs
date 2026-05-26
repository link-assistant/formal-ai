extern crate alloc;

pub mod arithmetic;
pub mod associative_package;
pub(crate) mod calculation;
pub(crate) mod concepts;
pub mod engine;
pub(crate) mod engine_assistant_name;
pub(crate) mod engine_hello_world;
pub mod event_log;
pub(crate) mod fuzzy;
pub mod github_logs;
pub mod intent_formalization;
pub mod language;
pub mod link_store;
pub(crate) mod links_format;
pub mod memory;
pub mod probability;
pub mod proof_engine;
pub mod protocol;
pub mod seed;
pub mod server;
pub mod skill_compiler;
pub mod solver;
pub(crate) mod solver_formalization;
pub(crate) mod solver_handler_docs;
pub(crate) mod solver_handler_how;
pub(crate) mod solver_handler_units;
pub(crate) mod solver_handlers;
pub(crate) mod solver_handlers_policy;
pub(crate) mod solver_helpers;
pub(crate) mod solver_unknown_reasoning;
pub mod summarization;
pub mod telegram;
pub mod telegram_runtime;
pub mod translation;
pub(crate) mod unknown_opener;
pub mod web_engine_core;
pub mod web_search_core;

pub use associative_package::{
    default_associative_packages, default_package_store, AssociativePackage, PackageDependency,
    PackageHandler, PackageImportError, PackageInstallError, PackagePermission,
    PackagePermissionDecision, PackageReplay, PackageStore, PackageTrigger,
};
pub use engine::{knowledge_links_notation, FormalAiEngine, SymbolicAnswer, DEFAULT_MODEL};
pub use event_log::{Event, EventLog};
pub use github_logs::{
    collect_github_logs, collect_github_logs_with_runner, github_log_capture_plan,
    render_github_log_plan, GithubLogCapture, GithubLogCapturedFile, GithubLogCollectionSummary,
    GithubLogCollectorConfig,
};
pub use intent_formalization::{
    formalize_intent, impulse_id_for, IntentFormalization, IntentFormalizationCache,
    IntentFormalizationCacheEntry, IntentKind,
};
pub use language::{detect as detect_language, Language};
#[cfg(feature = "doublets-native")]
pub use link_store::DoubletsLinkStore;
pub use link_store::{
    default_native_link_store, memory_event_to_link_record, memory_events_to_link_records,
    selected_link_store_backend, validate_memory_links_notation, DefaultNativeLinkStore,
    DoubletLink, LinkRecord, LinkStore, LinkStoreBackend, LinkStoreError,
};
pub use memory::{
    export_bundle as export_memory_bundle, export_full_memory as export_memory_full,
    export_links_notation as export_memory_links_notation, extract_memory_from_bundle,
    import_full_memory as import_memory_full, parse_links_notation as parse_memory_links_notation,
    suggest_migrations as suggest_memory_migrations, BundleInfo, MemoryEvent, MemoryStore,
    ParsedBundle,
};
pub use probability::{
    rank_probability_candidates, ProbabilityCandidate, ProbabilityEvidence, ProbabilityModel,
    ProbabilityRanking, ProbabilityRankingConfig, ProbabilitySourceProvenance, ProbabilityStore,
    RankedProbabilityCandidate,
};
pub use protocol::{
    create_chat_completion, create_chat_completion_with_solver, create_response,
    create_response_with_solver, ChatChoice, ChatCompletion, ChatCompletionRequest, ChatMessage,
    MessageContent, MessageContentPart, ResponseObject, ResponseOutputContent,
    ResponseOutputMessage, ResponseUsage, ResponsesRequest, TokenUsage,
};
pub use seed::{
    agent_info, concepts as seed_concepts, environment_directory, environment_records,
    intent_routing, language_rules, merged_bundle, multilingual_responses, parse_bundle,
    projects_registry, prompt_patterns, response_for, seed_files, EnvironmentDirectory,
    EnvironmentRecord, IntentRouting, LocalizedProject, MigrationFlow, ProjectRecord,
    ProjectStatement, ProjectsRegistry,
};
pub use server::{
    handle_api_request, handle_api_request_with_auth, handle_api_request_with_headers, serve,
    ApiAuthConfig, ApiHttpResponse,
};
pub use skill_compiler::{
    compile_natural_language_skill, CompiledSkillEffect, CompiledSkillExpectedTest,
    CompiledSkillHandlerStub, CompiledSkillInput, CompiledSkillPackage, CompiledSkillPermission,
    CompiledSkillPrecondition, CompiledSkillReplay, CompiledSkillStep, SkillCompileError,
};
pub use solver::{
    solve, solve_with_history, ConversationRole, ConversationTurn, ExecutionSurface, SolverConfig,
    UniversalSolver,
};
pub use solver_helpers::humanize_url;
pub use summarization::{
    apply_compound_words, apply_semantic_primes, classify_sentence, deformalize, describe_project,
    describe_readme, formalize, formalize_dialog, formalize_markdown, generate_chat_title,
    strip_markdown_noise, summarize, summarize_dialog, to_topic, DialogTurn, Statement,
    StatementKind, SummarizationConfig, SummarizationMode, DEFAULT_MAX_STATEMENTS,
};
pub use telegram::{
    handle_telegram_webhook, parse_get_updates_response, telegram_html_from_markdown,
    ParsedUpdatesBatch, TelegramPollingConfig, TelegramPollingError, TelegramPollingReply,
    TelegramReplyParameters, TelegramWebhookError, TelegramWebhookReply,
};
pub use telegram_runtime::{
    run_telegram_polling, run_telegram_polling_with_transport, run_telegram_webhook_server,
    CurlTelegramTransport, TelegramPollingRuntimeError, TelegramTransport,
};
pub use unknown_opener::unknown_answer_variation_for;
pub use web_engine_core::{
    detect_language as detect_prompt_language, evaluate_arithmetic_expression,
    normalize_prompt as normalize_prompt_text, tokenize_prompt,
};
pub use web_search_core::{
    build_request_evidence as build_web_search_request_evidence, default_search_plan_ids,
    parse_rrf_input, reciprocal_rank_fusion, serialize_rrf_output, FusedEntry, ProviderCategory,
    ProviderRanking, ProviderSpec, WEB_SEARCH_CONCURRENCY_PER_CATEGORY, WEB_SEARCH_PROVIDERS,
    WEB_SEARCH_PROVIDER_LIMIT, WEB_SEARCH_PROVIDER_REGISTRY, WEB_SEARCH_RRF_K,
};
