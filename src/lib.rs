extern crate alloc;

pub mod agent;
pub mod agentic_coding;
pub mod anthropic;
pub mod arithmetic;
pub mod associative_package;
pub(crate) mod calculation;
pub(crate) mod calculation_time;
pub(crate) mod calculation_word_problem;
pub(crate) mod code_editing;
pub(crate) mod coding;
pub(crate) mod concepts;
pub mod cue_lexicon;
pub mod document_formats;
pub mod engine;
pub(crate) mod engine_assistant_name;
pub(crate) mod engine_responses;
pub mod event_log;
pub(crate) mod fuzzy;
pub mod github_logs;
pub mod intent_formalization;
pub mod json_lino;
pub mod knowledge;
pub mod language;
pub mod link_store;
pub(crate) mod links_format;
pub mod links_query;
pub mod memory;
pub mod memory_sync;
pub mod meta_construction;
pub(crate) mod meta_core;
pub mod meta_frame;
pub(crate) mod meta_method_dispatch;
pub mod meta_reasoning;
pub mod meta_self_improvement;
pub mod method_registry;
pub mod probability;
pub(crate) mod program_coreference;
pub mod program_plan;
pub mod proof_engine;
pub mod protocol;
pub(crate) mod protocol_memory;
pub(crate) mod protocol_policy;
pub mod recipe_interpreter;
pub mod relative_meta_logic;
pub mod route_method_alias;
pub(crate) mod rule_synthesis;
pub mod seed;
pub mod selection;
pub mod self_improvement;
pub mod server;
pub mod shared_dialog;
pub mod skill_compiler;
pub mod skill_ledger;
pub mod solution_evidence;
pub mod solver;
pub(crate) mod solver_diagnostics;
pub(crate) mod solver_dispatch;
pub(crate) mod solver_formalization;
pub(crate) mod solver_handler_docs;
pub(crate) mod solver_handler_how;
pub(crate) mod solver_handler_oracle;
pub(crate) mod solver_handler_units;
pub(crate) mod solver_handlers;
pub(crate) mod solver_handlers_policy;
pub(crate) mod solver_helpers;
pub(crate) mod solver_synthesis;
pub(crate) mod solver_terminal;
pub(crate) mod solver_unknown_reasoning;
pub mod substitution;
pub mod summarization;
pub mod telegram;
pub mod telegram_runtime;
pub mod thinking;
pub mod translation;
pub(crate) mod unknown_opener;
pub mod web_engine_core;
pub mod web_search_core;

pub use agent::{
    parse_agent_plan, run_agent_plan, AgentAction, AgentActionKind, AgentActionStatus,
    AgentCommandResult, AgentError, AgentRun, AgentRunStatus, AgentWorkspace, AgentWorkspaceConfig,
    PlannedAgentAction,
};
pub use anthropic::{
    anthropic_message_sse, create_anthropic_message_with_solver,
    create_anthropic_message_with_solver_and_memory, AnthropicContentBlock, AnthropicMessage,
    AnthropicMessageInput, AnthropicMessagesRequest, AnthropicUsage,
};
pub use associative_package::{
    default_associative_packages, default_package_store, AssociativePackage, PackageDependency,
    PackageHandler, PackageImportError, PackageInstallError, PackagePermission,
    PackagePermissionDecision, PackageReplay, PackageStore, PackageTrigger,
};
pub use document_formats::{
    canonical_document_format_label, convert_document_format, cross_format_document_concepts,
    document_format_capabilities, document_package_is_recognized, document_profile_is_recognized,
    supported_document_formats, DocumentConversion, DocumentFormatCapabilities,
    DOCUMENT_FORMAT_ENGINE,
};
pub use engine::{
    humanize_meta_identifier, knowledge_links_notation, naturalize_thinking_step,
    thinking_language_label, FormalAiEngine, SymbolicAnswer, ThinkingStep, DEFAULT_MODEL,
};
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
pub use knowledge::{
    cache_capacity, within_cache_capacity, CodingOracle, KnowledgeSource, OracleSnippet,
    KNOWLEDGE_CACHE_FLOOR,
};
pub use language::{detect as detect_language, Language};
#[cfg(feature = "doublets-native")]
pub use link_store::DoubletsLinkStore;
pub use link_store::{
    default_native_link_store, memory_event_to_link_record, memory_events_to_link_records,
    selected_link_store_backend, validate_memory_links_notation, DefaultNativeLinkStore,
    DoubletLink, LinkRecord, LinkStore, LinkStoreBackend, LinkStoreError,
};
pub use links_query::{
    parse_links_query, run_links_query, run_links_query_against, EdgePattern, Field, Filter,
    FilterOp, LinksQuery, LinksQueryError, LinksQueryResult, NodePattern,
};
pub use memory::{
    export_bundle as export_memory_bundle, export_full_memory as export_memory_full,
    export_links_notation as export_memory_links_notation, extract_memory_from_bundle,
    import_full_memory as import_memory_full, parse_links_notation as parse_memory_links_notation,
    suggest_migrations as suggest_memory_migrations, BundleInfo, MemoryEvent, MemoryStore,
    ParsedBundle,
};
pub use memory_sync::{
    configured_memory_path, events_since, merge_event, merge_union_by_id, SyncStore,
};
pub use probability::{
    rank_probability_candidates, symbolic_cosine_similarity, ProbabilityCandidate,
    ProbabilityDecisionPolicy, ProbabilityEvidence, ProbabilityModel, ProbabilityRanking,
    ProbabilityRankingConfig, ProbabilitySourceProvenance, ProbabilityStore,
    RankedProbabilityCandidate, SimilarEvidence,
};
pub use protocol::{
    create_chat_completion, create_chat_completion_with_solver,
    create_chat_completion_with_solver_and_memory, create_response, create_response_with_solver,
    create_response_with_solver_and_memory, ChatChoice, ChatCompletion, ChatCompletionRequest,
    ChatMessage, FunctionCall, MessageContent, MessageContentPart, ResponseFunctionToolCall,
    ResponseObject, ResponseOutputContent, ResponseOutputItem, ResponseOutputMessage,
    ResponseUsage, ResponsesRequest, TokenUsage, ToolCall,
};
pub use seed::{
    agent_info, concepts as seed_concepts, environment_directory, environment_records,
    intent_routing, language_rules, merged_bundle, multilingual_responses, operation_vocabulary,
    parse_bundle, projects_registry, prompt_patterns, response_for, seed_files,
    supported_languages, EnvironmentDirectory, EnvironmentRecord, IntentRouting, LocalizedProject,
    MigrationFlow, OperationLanguageForms, OperationTrigger, OperationVocabulary, ProjectRecord,
    ProjectStatement, ProjectsRegistry,
};
pub use self_improvement::{
    learn_rules_from_unknown_traces, BenchmarkGateReport, LearnedRuleAdoption, LearnedRuleProposal,
    LearningRejection, LearningRun, UnknownTrace,
};
pub use server::{
    handle_api_request, handle_api_request_with_auth, handle_api_request_with_headers, serve,
    ApiAuthConfig, ApiHttpResponse,
};
pub use shared_dialog::{
    convert_shared_dialog_to_demo_memory, parse_shared_dialog, shared_dialog_to_memory_events,
    SharedDialog, SharedDialogError, SharedDialogFormat, SharedDialogMetadata, SharedDialogTurn,
};
pub use skill_compiler::{
    compile_natural_language_skill, CompiledSkillEffect, CompiledSkillExpectedTest,
    CompiledSkillHandlerStub, CompiledSkillInput, CompiledSkillPackage, CompiledSkillPermission,
    CompiledSkillPrecondition, CompiledSkillReplay, CompiledSkillStep, SkillCompileError,
};
pub use solver::{
    solve, solve_with_history, BlueprintComposition, ConversationRole, ConversationTurn,
    ExecutionSurface, SolverConfig, UniversalSolver,
};
pub use solver_handlers::{answer_memory_recall, execute_memory_query, MemoryQueryExecution};
pub use solver_helpers::humanize_url;
pub use substitution::{
    CrudEvent, LinkPattern, SubstitutionAction, SubstitutionGraph, SubstitutionLink,
    SubstitutionRule, SubstitutionRuleError, SubstitutionRuleSet, SubstitutionTrace,
    SubstitutionTraceReport,
};
pub use summarization::{
    apply_compound_words, apply_semantic_primes, classify_sentence, deformalize, describe_project,
    describe_readme, formalize, formalize_dialog, formalize_markdown,
    formalize_repository_directory, formalize_repository_file, formalize_repository_resource,
    generate_chat_title, strip_markdown_noise, summarize, summarize_dialog,
    summarize_repository_file, summarize_repository_resource, to_topic, DialogTurn,
    EmbeddedGrammarFormalization, MetaLanguageFormalization, RepositoryDirectoryFormalization,
    RepositoryEntry, RepositoryFileFormalization, RepositoryResourceFormalization, Statement,
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
