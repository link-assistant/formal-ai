extern crate alloc;

pub mod agent;
pub mod agentic_coding;
pub mod algorithm_discovery;
pub mod anthropic;
pub mod arithmetic;
pub mod associative_package;
pub mod associative_persistence;
pub mod attachment_context;
pub(crate) mod calculation;
pub(crate) mod calculation_time;
pub(crate) mod calculation_word_problem;
pub mod change_request;
pub mod client_contract_learning;
pub mod client_integrations;
pub(crate) mod code_editing;
pub(crate) mod coding;
pub mod computer_use;
pub(crate) mod concepts;
pub mod context_capacity;
pub mod conversation_context;
pub mod cue_lexicon;
pub mod definition_merge;
pub mod dialog_conversation;
pub mod dialog_log;
pub mod document_formats;
pub mod draft_portfolio;
pub mod dreaming;
pub mod dreaming_application;
pub mod dreaming_runtime;
pub mod engine;
pub(crate) mod engine_assistant_name;
pub(crate) mod engine_responses;
pub mod entity_resolution;
pub mod event_log;
pub mod external_benchmarks;
pub mod fact_checking;
pub mod file_legality;
pub mod formal_system;
pub(crate) mod fuzzy;
pub mod gemini;
pub mod github_logs;
pub mod google_trends_catalog;
pub mod google_trends_learning;
pub mod intent_formalization;
pub mod issue_report;
pub mod json_lino;
pub mod knowledge;
pub mod language;
pub mod language_adoption;
pub mod language_frontier;
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
pub mod memory_program;
pub mod memory_query_language;
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
pub(crate) mod number_constraints;
pub mod option_evidence;
pub mod option_network;
pub mod orchestration;
pub mod probability;
pub(crate) mod program_coreference;
pub mod program_plan;
pub mod program_skill_gap;
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
pub mod recursive_execution;
pub mod relative_meta_logic;
pub mod repair_strategy;
pub mod requirement_contradiction;
pub(crate) mod responses_stream;
pub mod route_method_alias;
pub(crate) mod rule_synthesis;
pub(crate) mod rule_synthesis_portfolio;
pub mod search_fusion;
pub mod search_fusion_grammar;
pub mod search_fusion_learning;
pub mod seed;
pub mod selection;
pub mod self_ast_census;
pub mod self_explanation;
pub mod self_healing;
pub mod self_improvement;
pub mod self_source_links;
pub mod sequences;
pub mod server;
pub mod shared_dialog;
pub mod shared_memory;
pub mod skill_compiler;
pub mod skill_ledger;
pub mod skill_procedure;
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
pub mod source_fetch;
pub mod source_research;
pub mod statement_audit;
pub mod statement_verification;
pub mod storage_policy;
pub mod substitution;
pub mod summarization;
pub mod task_decomposition;
pub mod telegram;
pub mod telegram_runtime;
pub mod thinking;
pub mod translation;
pub(crate) mod unknown_opener;
pub mod web_engine_core;
pub mod web_search_core;
pub mod web_search_fusion_core;
pub mod world_model;
pub mod world_model_atoms;
pub mod world_model_context;
pub(crate) mod world_model_cycles;
pub mod world_model_dialog;

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
pub use client_contract_learning::{
    learn_client_contracts, load_observations, observe_proxy_transcript, ClientContractFinding,
    ClientContractLearningReport, ClientContractObservation, ClientContractProposal, DeliveryMode,
};
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
pub use fact_checking::{
    AuditScope, ContextAudit, EvidenceTrace, FactCheckError, FactChecker, ProbabilityBasis,
    RefutationAttempt, RefutationOutcome, RefutationStage, StatementVerification,
};
pub use formal_system::FormalSystem;
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
pub use learning_adoption_ledger::{google_trends_adoption_ledger, AdoptionLedger, AdoptionPair};
pub use learning_cycle::{
    google_trends_learning_cycle, parse_frontier_record, recorded_google_trends_frontier,
    run_learning_cycle, BlockedClass, CandidateSurface, FrontierItem, HeldOutTest,
    LearningCycleRun,
};
pub use learning_ledger::{
    approved_lesson_for, canonical_ledger, HumanApproval, LearningLedger, LedgerEntry,
    PromotionRejected,
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
    ChatMessage, FunctionCall, MessageContent, MessageContentPart, ResponseCustomToolCall,
    ResponseFunctionToolCall, ResponseObject, ResponseOutputContent, ResponseOutputItem,
    ResponseOutputMessage, ResponseUsage, ResponseWebSearchAction, ResponseWebSearchToolCall,
    ResponsesRequest, TokenUsage, ToolCall,
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
pub use search_fusion::{
    execute_search_fusion, FormalizedSearchObservation, FusedSearchStatement,
    NormalizedSearchSource, SearchFusionAnswer, SearchFusionExecution, SearchObservationOrigin,
    SearchSourceClassification,
};
pub use search_fusion_learning::{
    execute_search_fusion_with_recipe, SearchFusionLearningApproval, SearchFusionLearningError,
    SearchFusionLearningFrontier, SearchFusionLearningGate, SearchFusionLearningObservation,
    SearchFusionRecipeCandidate, SearchFusionRecipeLedger, SEARCH_FUSION_TASK_FAMILY,
};
pub use seed::{
    agent_info, canonical_model_id, concepts as seed_concepts, environment_directory,
    environment_records, intent_routing, language_rules, merged_bundle, model_aliases,
    multilingual_responses, operation_vocabulary, parse_bundle, projects_registry, prompt_patterns,
    render_response, resolve_model_id, response_for, seed_files, supported_languages,
    try_resolve_model_id, EnvironmentDirectory, EnvironmentRecord, IntentRouting, LocalizedProject,
    MigrationFlow, ModelAliasRegistry, OperationLanguageForms, OperationTrigger,
    OperationVocabulary, ProjectRecord, ProjectStatement, ProjectsRegistry,
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
    learn_from_reported_conversation, learn_rules_from_unknown_traces,
    learning_trace_from_symbolic_answer, BenchmarkGateReport, LearnedRuleAdoption,
    LearnedRuleProposal, LearningRejection, LearningRun, ReportedLearning, UnknownTrace,
};
pub use self_source_links::{
    owned_file_count, owned_manifest, owned_manifest_content_id, owned_manifest_notation,
    owned_source_files, owned_total_bytes, SourceLinks, SourceModuleDigest, SourceModuleProjection,
};
pub use sequences::{
    balanced_convert, compress, detect_palindrome, detect_period, detect_repetition,
    infer_grid_patterns, infer_sequence_patterns, CompressionResult, CompressionStep, Doublet,
    Grid, GridPatternReport, GridSymmetry, GridTransform, LinkAddress, LinkFrequenciesCache,
    LinkFrequency, RepetitionPattern, SequenceIndex, SequencePattern, SequencePatternReport,
    SequenceStore, SymbolTable,
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
pub use skill_procedure::{
    compile_procedure, compile_procedure_with_ledger, extract_compiled_procedure_artifact,
    ApprovedProcedureLesson, CompiledProcedure, ProcedureArtifactError, ProcedureCapabilityLedger,
    ProcedureCapabilityLesson, ProcedureCompileError, ProcedureHost, ProcedureLearnedSurface,
    ProcedureLearningApproval, ProcedureLearningCandidate, ProcedureLearningError,
    ProcedureLearningGate, ProcedureLearningObservation, ProcedureLearningProposal,
    ProcedureRequirement, ProcedureRun, ProcedureRunError, ProcedureStep, ProcedureTrigger,
    StepOutcome, PROCEDURE_CONFORMANCE_TRIGGER,
};
pub use solver::{
    solve, solve_with_history, BlueprintComposition, ConversationRole, ConversationTurn,
    ExecutionSurface, SolverConfig, UniversalSolver,
};
pub use solver_handlers::{
    answer_memory_recall, execute_memory_query, execute_memory_query_with_options,
    try_web_search_with_client, MemoryQueryExecution,
};
pub use solver_helpers::humanize_url;
pub use source_fetch::{
    sha256_hex, CachedSourceClient, CurlSourceTransport, FetchError, SourceCapture, SourceTransport,
};
pub use source_research::{
    execute_option_research, execute_source_research, execute_statement_research,
    OptionResearchExecution, ResearchFailure, ResearchPage, SourceResearchExecution,
    StatementResearchExecution,
};
pub use statement_verification::{
    assess_market_price_claims, extract_market_price_claims, CapturedStatementEvidence,
    MarketPriceAssessment, MarketPriceClaim, StatementVerificationExecution,
    StatementVerificationPlan,
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
    apply_compound_words, apply_semantic_primes, classify_sentence, deduplicate, deformalize,
    describe_project, describe_readme, execute_captured_gathering, execute_multi_source_summary,
    formalize, formalize_dialog, formalize_markdown, formalize_repository_directory,
    formalize_repository_file, formalize_repository_resource, gather, generate_chat_title,
    is_valid_identifier, label_for_mode, merge_into_context, merge_into_formal_context, rank,
    recheck, strip_markdown_noise, summarize, summarize_dialog, summarize_repository_file,
    summarize_repository_resource, to_identifier, to_topic, CapturedGatheringFailure,
    CapturedGatheringReport, CapturedSourceMetadata, CapturedSourceObservation, Contradiction,
    DedupReport, DialogTurn, EmbeddedGrammarFormalization, FetchRecord, FetchedSource,
    GatheringPlan, GatheringReport, IdentifierBudget, ImportanceScore, MergeLink, MergedContext,
    MergedStatement, MetaLanguageFormalization, MultiSourceSummaryExecution, NamingConvention,
    Polarity, RankedStatement, RecheckReport, RecheckedStatement, RepositoryDirectoryFormalization,
    RepositoryEntry, RepositoryFileFormalization, RepositoryResourceFormalization, SourceCache,
    SourceProvider, SourcedStatement, Statement, StatementKind, StatementSignature,
    StatementVariant, SummarizationConfig, SummarizationMode, Verdict,
    DEFAULT_IDENTIFIER_MAX_LENGTH, DEFAULT_IDENTIFIER_MAX_WORDS, DEFAULT_MAX_STATEMENTS,
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
    assess_arithmetic_claim, detect_language as detect_prompt_language,
    evaluate_arithmetic_expression, normalize_prompt as normalize_prompt_text, tokenize_prompt,
    ArithmeticClaimAssessment, ArithmeticClaimOutcome,
};
pub use web_search_core::{
    build_request_evidence as build_web_search_request_evidence, default_search_plan_ids,
    execute_duckduckgo_search, parse_rrf_input, reciprocal_rank_fusion, serialize_rrf_output,
    FusedEntry, ProviderCategory, ProviderRanking, ProviderSpec, SearchExecution,
    WEB_SEARCH_CONCURRENCY_PER_CATEGORY, WEB_SEARCH_PROVIDERS, WEB_SEARCH_PROVIDER_LIMIT,
    WEB_SEARCH_PROVIDER_REGISTRY, WEB_SEARCH_RRF_K,
};
pub use world_model::{
    Action, Context, ContextAccessEvent, ContextAccessEventKind, ContextDiff, Dependency,
    GeneralMemoryCommitError, GeneralMemoryPermission, LinkConflict, Prediction, RecalculatedLink,
    RecalculationReport, Statement as WorldStatement, StatementChange, WorldModel,
};
pub use world_model_atoms::{classify as classify_utterance, state_atom, UtteranceKind};
pub use world_model_context::{
    ContextHierarchy, ContextHierarchyError, ExternalLookup, InheritancePolicy, ParentContext,
    ReferenceResolution, ReferenceResolutionKind,
};
pub use world_model_dialog::{
    ActionForecast, DialogueWorldModel, SyncEvent, SyncEventKind, WorldModelMode,
};
