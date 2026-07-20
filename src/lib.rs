extern crate alloc;

pub mod agent;
pub mod agentic_coding;
pub mod anthropic;
pub mod arithmetic;
pub mod associative_package;
pub mod associative_persistence;
pub mod attachment_context;
pub(crate) mod calculation;
pub(crate) mod calculation_time;
pub(crate) mod calculation_word_problem;
pub mod change_request;
pub mod client_integrations;
pub(crate) mod code_editing;
pub(crate) mod coding;
pub(crate) mod concepts;
pub mod context_capacity;
pub mod cue_lexicon;
pub mod dialog_log;
pub mod document_formats;
pub mod dreaming;
pub mod dreaming_application;
pub mod dreaming_runtime;
pub mod engine;
pub(crate) mod engine_assistant_name;
pub(crate) mod engine_responses;
pub mod event_log;
pub(crate) mod fuzzy;
pub mod gemini;
pub mod github_logs;
pub mod google_trends_catalog;
pub mod google_trends_learning;
pub mod intent_formalization;
pub mod json_lino;
pub mod knowledge;
pub mod language;
pub mod learning_adoption_ledger;
pub mod learning_cycle;
pub mod learning_ledger;
pub mod lexeme_import;
pub mod link_store;
pub(crate) mod links_format;
pub mod links_query;
pub mod links_substitution_query;
pub(crate) mod mcp;
pub mod memory;
pub mod memory_sync;
pub mod meta_construction;
pub(crate) mod meta_core;
pub mod meta_frame;
pub(crate) mod meta_method_dispatch;
pub mod meta_reasoning;
pub mod meta_self_improvement;
pub mod method_registry;
pub(crate) mod network_endpoint;
pub mod normal_markov;
pub mod option_evidence;
pub mod option_network;
pub mod probability;
pub(crate) mod program_coreference;
pub mod program_plan;
pub mod promotion;
pub mod proof_engine;
pub mod protocol;
pub(crate) mod protocol_memory;
pub(crate) mod protocol_policy;
pub(crate) mod protocol_responses;
pub mod proxy;
pub mod question_generation;
pub mod rebuild_plan;
pub mod recipe_interpreter;
pub mod relative_meta_logic;
pub mod repair_strategy;
pub mod requirement_contradiction;
pub(crate) mod responses_stream;
pub mod route_method_alias;
pub(crate) mod rule_synthesis;
pub mod seed;
pub mod selection;
pub mod self_ast_census;
pub mod self_explanation;
pub mod self_healing;
pub mod self_improvement;
pub mod self_source_links;
pub mod server;
pub mod shared_dialog;
pub mod shared_memory;
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
pub(crate) mod solver_search;
pub(crate) mod solver_synthesis;
pub(crate) mod solver_terminal;
pub(crate) mod solver_unknown_reasoning;
pub mod statement_audit;
pub mod statement_verification;
pub mod storage_policy;
pub mod substitution;
pub mod summarization;
pub mod telegram;
pub mod telegram_runtime;
pub mod thinking;
pub mod translation;
pub(crate) mod unknown_opener;
pub mod web_engine_core;
pub mod web_search_core;
pub mod world_model;

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
pub use associative_persistence::{
    AssociativeMemory, PersistedExpression, RetentionWeights, ScoredExpression,
};
pub use change_request::{canonical_change_request, AcceptedChange, ChangeRejected, ChangeRequest};
pub use client_integrations::{run_with_formal_ai, ClientProtocol, WithFormalAiArgs};
pub use document_formats::{
    canonical_document_format_label, convert_document_format, cross_format_document_concepts,
    document_format_capabilities, document_package_is_recognized, document_profile_is_recognized,
    supported_document_formats, DocumentConversion, DocumentFormatCapabilities,
    DOCUMENT_FORMAT_ENGINE,
};
pub use dreaming::{
    apply_dreaming_plan, compose_recipe_with_amendments, plan_memory_dreaming,
    render_dreaming_plan, DreamingAction, DreamingActionKind, DreamingConfig, DreamingDurability,
    DreamingEventObservation, DreamingOutcome, DreamingPlan, DreamingSynthesizedTask,
    LearnedRequirement, MetaAlgorithmAmendment, TopicFrequency,
};
pub use dreaming_application::{
    amended_answer, apply_retained_amendments, replay_answer_with_amendments, retained_amendments,
    solve_with_amendment_records, solve_with_standing_requirements, topic_matches,
    RetainedAmendment,
};
pub use dreaming_runtime::{
    core_is_idle, dreaming_disabled, run_core_dreaming_once, ForegroundActivity,
};
pub use engine::{
    humanize_meta_identifier, knowledge_links_notation, naturalize_thinking_step,
    render_thinking_steps, thinking_language_label, thinking_narrative, FormalAiEngine,
    SymbolicAnswer, ThinkingStep, DEFAULT_MODEL,
};
pub use event_log::{Event, EventLog};
pub use github_logs::{
    collect_github_logs, collect_github_logs_with_runner, github_log_capture_plan,
    render_github_log_plan, GithubLogCapture, GithubLogCapturedFile, GithubLogCollectionSummary,
    GithubLogCollectorConfig,
};
pub use google_trends_catalog::{
    google_trends_catalog, parse_google_trends_rss, render_google_trends_snapshot_lino,
    GoogleTrendNewsItem, GoogleTrendPromptAnswer, GoogleTrendPromptVariant, GoogleTrendTopic,
    GoogleTrendsCatalog, GoogleTrendsParseError, GOOGLE_TRENDS_TOP_LIMIT,
};
pub use google_trends_learning::{
    trending_learning_report, TrendingFrontierEntry, TrendingLearningReport,
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
pub use learning_adoption_ledger::{
    google_trends_adoption_ledger, AdoptionLedger, AdoptionPair,
};
pub use learning_cycle::{
    google_trends_learning_cycle, parse_frontier_record, recorded_google_trends_frontier,
    run_learning_cycle, BlockedClass, CandidateSurface, FrontierItem, HeldOutTest,
    LearningCycleRun,
};
pub use learning_ledger::{
    canonical_ledger, HumanApproval, LearningLedger, LedgerEntry, PromotionRejected,
};
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
    export_links_notation, export_links_notation as export_memory_links_notation,
    extract_memory_from_bundle, import_full_memory as import_memory_full,
    parse_links_notation as parse_memory_links_notation, seed_cache_events,
    suggest_migrations as suggest_memory_migrations, write_locked_atomic, BundleInfo, MemoryEvent,
    MemoryStore, ParsedBundle,
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
pub use promotion::{
    apply_promotions, demonstration_promotion_proposals, demonstration_promotion_run,
    parse_promotion_proposals, promotions_from_learning_run, render_promotion_proposals,
    replay_promotion_gates, replay_promotion_gates_with, AppliedSeedEdit, GateCommandOutput,
    PromotionApplyOutcome, PromotionBranchPlan, PromotionOutcome, PromotionProposal,
    PromotionRatchet, PromotionRecord, PromotionRun, SeedEdit, LEARNED_PROGRAM_RULES_SEED_FILE,
};
pub use protocol::{
    create_chat_completion, create_chat_completion_with_solver,
    create_chat_completion_with_solver_and_memory, create_response, create_response_with_solver,
    create_response_with_solver_and_memory, ChatChoice, ChatCompletion, ChatCompletionRequest,
    ChatMessage, FunctionCall, MessageContent, MessageContentPart, ResponseFunctionToolCall,
    ResponseObject, ResponseOutputContent, ResponseOutputItem, ResponseOutputMessage,
    ResponseUsage, ResponseWebSearchAction, ResponseWebSearchToolCall, ResponsesRequest,
    TokenUsage, ToolCall,
};
pub use proxy::{
    run_proxy, summarize_proxy_exchange, ProxyConfig, ProxyExchangeLog, ProxyToolCallLog,
};
pub use question_generation::{
    generated_question_answers, question_lexicon_summary, question_lexicon_summary_for_language,
    GeneratedQuestion, GeneratedQuestionAnswer, GeneratedQuestionAnswerStream,
    GeneratedQuestionClass, LogicalMeaningClass, QuestionAcceptance, QuestionGenerationConfig,
    QuestionGenerator, QuestionGrammarClass, QuestionLexiconSummary, QuestionWord,
};
pub use rebuild_plan::{canonical_rebuild_plan, ReattachArtifact, RebuildPlan, RebuildStep};
pub use relative_meta_logic::{
    Aggregator, RelativeEvidence, SourceTier, Stance, StatementAssessment, TruthValue,
    ASSUMED_TRUE_PRIOR,
};
pub use repair_strategy::{canonical_strategies, RepairStrategy, RepairTarget};
pub use seed::{
    agent_info, canonical_model_id, concepts as seed_concepts, environment_directory,
    environment_records, intent_routing, language_rules, merged_bundle, model_aliases,
    multilingual_responses, operation_vocabulary, parse_bundle, projects_registry, prompt_patterns,
    resolve_model_id, response_for, seed_files, supported_languages, try_resolve_model_id,
    EnvironmentDirectory, EnvironmentRecord, IntentRouting, LocalizedProject, MigrationFlow,
    ModelAliasRegistry, OperationLanguageForms, OperationTrigger, OperationVocabulary,
    ProjectRecord, ProjectStatement, ProjectsRegistry,
};
pub use self_ast_census::{
    drift_report, scan_symbols, CensusDrift, CensusFidelity, CensusResolution, ModuleCensus,
    SymbolSpan, WorkspaceCensus,
};
pub use self_explanation::{
    canonical_explanation, Citation, CitationKind, ExplanationSection, SystemExplanation,
};
pub use self_healing::{
    canonical_case, canonical_failure_trace, RepairCase, RepairOutcome, SourceRoundTrip,
};
pub use self_improvement::{
    learn_rules_from_unknown_traces, BenchmarkGateReport, LearnedRuleAdoption, LearnedRuleProposal,
    LearningRejection, LearningRun, UnknownTrace,
};
pub use self_source_links::{
    owned_file_count, owned_manifest, owned_manifest_content_id, owned_manifest_notation,
    owned_source_files, owned_total_bytes, SourceLinks, SourceModuleDigest, SourceModuleProjection,
};
pub use server::{
    enable_http_agent_mode_for_current_process, handle_api_request, handle_api_request_with_auth,
    handle_api_request_with_headers, serve, ApiAuthConfig, ApiHttpResponse,
};
pub use shared_dialog::{
    convert_shared_dialog_to_demo_memory, parse_shared_dialog, shared_dialog_to_memory_events,
    SharedDialog, SharedDialogError, SharedDialogFormat, SharedDialogMetadata, SharedDialogTurn,
};
pub use shared_memory::{
    ensure_shared_memory_file, resolve_memory_path_from, shared_memory_path, MEMORY_PATH_ENV,
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
pub use statement_verification::{
    assess_market_price_claims, extract_market_price_claims, MarketPriceAssessment,
    MarketPriceClaim,
};
pub use storage_policy::{
    apply_auto_free_space_for_write, apply_auto_free_space_with_snapshot, auto_free_space_choice,
    auto_free_space_enabled, auto_free_space_preference_path, measure_storage,
    persist_auto_free_space_choice, plan_for_real_storage, AutoFreeSpaceChoice, StorageSnapshot,
};
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
pub use world_model::{
    Action, Context, ContextDiff, Dependency, LinkConflict, Prediction, RecalculationReport,
    Statement as WorldStatement, StatementChange, WorldModel,
};
