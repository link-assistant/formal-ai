extern crate alloc;

pub(crate) mod arithmetic;
pub(crate) mod calculation;
pub(crate) mod concepts;
pub mod engine;
pub(crate) mod engine_hello_world;
pub mod event_log;
pub mod github_logs;
pub mod language;
pub mod memory;
pub mod protocol;
pub mod seed;
pub mod server;
pub mod solver;
pub(crate) mod solver_handler_how;
pub(crate) mod solver_handler_units;
pub(crate) mod solver_handlers;
pub(crate) mod solver_handlers_policy;
pub(crate) mod solver_helpers;
pub mod telegram;
pub mod telegram_runtime;
pub mod web_search_core;

pub use engine::{knowledge_links_notation, FormalAiEngine, SymbolicAnswer, DEFAULT_MODEL};
pub use event_log::{Event, EventLog};
pub use github_logs::{
    collect_github_logs, collect_github_logs_with_runner, github_log_capture_plan,
    render_github_log_plan, GithubLogCapture, GithubLogCapturedFile, GithubLogCollectionSummary,
    GithubLogCollectorConfig,
};
pub use language::{detect as detect_language, Language};
pub use memory::{
    export_bundle as export_memory_bundle, export_full_memory as export_memory_full,
    export_links_notation as export_memory_links_notation, extract_memory_from_bundle,
    import_full_memory as import_memory_full, parse_links_notation as parse_memory_links_notation,
    suggest_migrations as suggest_memory_migrations, BundleInfo, MemoryEvent, MemoryStore,
    ParsedBundle,
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
    prompt_patterns, response_for, seed_files, EnvironmentDirectory, EnvironmentRecord,
    IntentRouting, MigrationFlow,
};
pub use server::{handle_api_request, serve, ApiHttpResponse};
pub use solver::{
    solve, solve_with_history, ConversationRole, ConversationTurn, SolverConfig, UniversalSolver,
};
pub use solver_helpers::humanize_url;
pub use telegram::{
    handle_telegram_webhook, parse_get_updates_response, telegram_html_from_markdown,
    ParsedUpdatesBatch, TelegramPollingConfig, TelegramPollingError, TelegramPollingReply,
    TelegramReplyParameters, TelegramWebhookError, TelegramWebhookReply,
};
pub use telegram_runtime::{
    run_telegram_polling, run_telegram_polling_with_transport, run_telegram_webhook_server,
    CurlTelegramTransport, TelegramPollingRuntimeError, TelegramTransport,
};
pub use web_search_core::{
    build_request_evidence as build_web_search_request_evidence, default_search_plan_ids,
    parse_rrf_input, reciprocal_rank_fusion, serialize_rrf_output, FusedEntry, ProviderCategory,
    ProviderRanking, ProviderSpec, WEB_SEARCH_CONCURRENCY_PER_CATEGORY, WEB_SEARCH_PROVIDERS,
    WEB_SEARCH_PROVIDER_LIMIT, WEB_SEARCH_PROVIDER_REGISTRY, WEB_SEARCH_RRF_K,
};
