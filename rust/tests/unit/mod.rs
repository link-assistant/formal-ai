#[path = "../fixtures/first-workspace-call.rs"]
mod first_workspace_call;

#[path = "../fixtures/observed-plan-tools.rs"]
mod observed_plan_tools;

#[path = "issue_1066_ladder_capability/tool_workspace.rs"]
mod tool_workspace;

#[allow(dead_code)]
#[path = "../fixtures/observed-plan-event.rs"]
mod observed_plan_event;

mod agent_cli_entry_point;
mod architect_notes;
mod architecture_docs;
mod assistant_name;
mod benchmark_release_capability;
mod bulk_lexeme_import;
mod caller_framing_context;
mod candidate_solution_portfolios;
mod ci_gates;
mod claude_code_away_recap;
mod cli_paths;
mod coding_discovery;
mod coding_ladder_rungs;
mod compatibility_research_evidence;
mod concept_lookup_response_language;
mod concept_sense_ledger;
mod concise_lexemes;
mod conversational_variations;
mod courtesy_response;
mod custom_formalization_subject;
mod data_files;
mod dialog_log;
mod docker_runtime;
mod docs_benchmarks;
mod docs_requirements;
mod document_generation_plan;
mod document_originality_check;
mod documentation_issue_citations;
mod documentation_status;
mod dreaming_runtime;
mod exit_code_verification;
mod fetched_page_answer_extraction;
mod formal_ai;
mod formal_projection_round_trips;
mod generated_question_ordering;
mod github_logs;
mod grounded_local_action;
mod hive_mind_full_circle;
mod installation_conversion;
mod interval_number_riddle;
mod issue_1021_behaviour_range;
mod issue_1021_bounded_autonomy;
mod issue_1021_closed_circle;
mod issue_1021_contribution_artifacts;
mod issue_1021_write_path;
mod issue_1066_agent_ladder;
mod issue_1066_hollow_answers;
mod issue_1066_ladder_capability;
mod issue_1066_node_verifier;
mod issue_1066_self_development;
mod issue_1069_identifier_rename;
mod issue_1073_reasoning_standard;
mod issue_1085_metric_restatement;
mod issue_1085_python_signature;
mod issue_1085_self_authored_backlog;
mod issue_1085_upstream_frontier;
mod issue_1085_upstream_prompt_transfer;
mod issue_1095_continuation_cue;
mod issue_1105_report_flow;
mod issue_1133_hive_mind_three_runs;
mod issue_1138_concept_lookup;
mod issue_1138_formalization_depth;
mod issue_1138_retrieval_method;
mod issue_1138_round_trip_projection;
mod issue_1138_rust_projection;
mod issue_1138_segmentation;
mod issue_1138_source_walk_parity;
mod issue_1138_three_source_roots;
mod issue_1138_translation_tool;
mod issue_1138_universal_loop_lookup;
mod issue_498_google_trends;
mod issue_499_learn_from_source;
mod issue_531_algorithm_discovery;
mod issue_531_concepts_probe;
mod issue_649_world_model;
mod issue_656_promotion;
mod issue_661_repository_audit;
mod issue_671_matrix_coverage;
mod issue_673_self_ast_census;
mod issue_676_thinking_narrative;
mod issue_680_intent_routing;
mod issue_701_dreaming_amendment_class;
mod issue_701_learning_adoption;
mod issue_702_nested_contexts;
mod issue_702_world_model_dialog;
mod issue_702_world_state_chat;
mod issue_705_anticipation;
mod issue_706_any_language;
mod issue_708_memory_query_language_browser;
mod issue_708_self_hosting;
mod issue_709_search_fusion;
mod issue_709_search_fusion_learning;
mod issue_715_link_substitution_query;
mod issue_715_links_substitution_query;
mod issue_715_normal_markov;
mod issue_715_renderer_artifacts;
mod issue_781_option_evidence;
mod issue_781_option_network;
mod issue_819_followup;
mod issue_823_recursive_learning;
mod issue_835_file_legality;
mod issue_844_production_pipeline;
mod issue_844_statement_merge;
mod issue_844_statement_ranking;
mod issue_845_fact_checking;
mod issue_848_workspace_change_learning;
mod issue_885_document_fact_checking;
mod issue_889_thinking_seed;
mod issue_896_component_boundaries;
mod issue_906_language_router;
mod issue_918_handler_rules_batch;
mod issue_918_handler_rules_batch_2;
mod issue_920_question_necessity;
mod issue_922_method_learning;
mod issue_931_local_transport_contract;
mod issue_932_box_language_projects;
mod issue_932_self_authoring;
mod issue_933_answer_parity;
mod issue_933_self_authoring;
mod issue_936_substitution_compiler;
mod issue_940_research_documents;
mod issue_962_word_operator_parity;
mod issue_988_stock_rust_install;
mod issue_991_how_to_synthesis;
mod issue_991_incremental_decomposition;
mod issue_991_merge_conflict_policy;
mod language_parity_lib_suite;
mod lino_location;
mod local_path_discovery;
mod local_report_export;
mod local_surface;
mod market_price_claims;
mod memory_learning;
mod memory_maintenance;
mod memory_retention_origin;
mod minimal_core_and_seed_metadata;
mod multilingual_variations;
mod offline_replay;
mod page_formalization_route;
mod process_composition;
mod proof_language_independence;
mod proof_request;
mod proof_request_configuration;
mod proxy;
mod pull_request_1188_fix_gaps;
mod pull_request_1188_function_recipe;
mod pull_request_1188_line_anchor;
mod pull_request_1188_line_moves;
mod pull_request_1188_line_operations;
mod pull_request_1188_line_removal;
mod pull_request_1188_quoted_payload_command;
mod pull_request_1188_quoted_payload_cues;
mod pull_request_1188_quoted_payload_safety;
mod pull_request_1188_replace_semantics;
mod pull_request_1188_setting_verb;
mod pull_request_1188_subagent_gaps;
mod pull_request_1188_teach_e;
mod pull_request_1188_teach_f;
mod pull_request_1188_typo_discovery;
mod pull_request_1188_unnamed_language;
mod pull_request_1188_unquoted_output;
mod pull_request_1188_workspace_search;
mod report_output_location;
mod requirement_listing_route;
mod requirement_span_integrity;
mod researched_coding_procedures;
mod semantic_intent_routing;
mod server_route_manifest;
mod shared_dialog;
mod shared_memory_isolation;
mod software_project;
#[path = "../fixtures/source-cache-seed-contract.rs"]
mod source_cache_seed_contract;
mod source_module_contracts;
mod source_provenance;
mod specification;
mod symbolic_kernel_coverage;
mod task_ladder_false_greens;
mod terminal_command_intent;
mod test_status;
mod thinking_detail_length;
mod tool_failure_observations;
mod tool_scope;
mod total_closure;
mod tui_integration_metadata;
mod unresolved_request_research;
mod user_journeys;
mod web_requests;
mod web_search_by_reasoning;

#[path = "ci-cd/mod.rs"]
mod ci_cd;
mod hive_mind_hello_world;
mod issue_1088_evidence_index;
mod issue_1090_manual_column_retired;
mod issue_1101_documentation_question_parity;
mod issue_1138_bounded_file_analysis;
mod issue_1138_conversation_container;
mod issue_1138_execution_box;
mod issue_1138_family_migration;
mod issue_1138_handler_promotions;
mod issue_1138_held_out_toolchain;
mod issue_1138_js_meta;
mod issue_1138_js_tokenizer;
mod issue_1138_learned_items_change_answers;
mod issue_1138_learning_ratchet;
mod issue_1138_obligation_evidence;
mod issue_1138_place_timezone;
mod issue_1138_review_time_gate;
mod issue_1138_selection_heuristics;
mod issue_1138_solve_cli;
mod issue_1138_source_capabilities;
mod issue_1138_store_read_path;
mod issue_1138_surface_honesty;
mod issue_1138_telegram_execution;
mod issue_1138_uniform_dispatch;
mod issue_1163_1164_captured_pages;
mod issue_1163_formal_knowledge_paths;
mod issue_1163_web_formalize;
mod issue_1164_code_example_knowledge;
mod issue_1164_pascal_from_documentation;
mod issue_1165_discovery_production;
mod issue_1165_documentation_route;
mod issue_1165_documented_commands;
mod issue_1166_executor_gaps;
mod issue_1166_obligation_routing;
mod issue_1167_compose_render_parse;
mod issue_1168_latest_versions;
mod issue_1168_workflow_render_parity;
mod issue_1169_dependency_currency;
mod issue_1171_llm_task_parity;
mod issue_1172_live_fact;
mod issue_1172_word_definition;
mod issue_1173_fallback_executes_search;
mod issue_1175_routing;
mod issue_1176_quantities_dates;
mod issue_1176_wikidata_grounding;
mod issue_1179_fact_check_sources;
mod issue_1180_repository_qa;
mod issue_1181_boolean_environment_parser;
mod issue_1182_duplicate_gate;
mod issue_1182_links_notation_conformance;
mod issue_1184_derivation_records;
mod issue_1187_credentials;
mod issue_1188_dependency_summarization;
mod issue_1188_round_trip_translation;
mod issue_1188_self_translation_corpus;
mod issue_1188_web_formalization;
mod issue_483_small_model_fallback;
mod issue_491_least_action;
mod issue_668_associative_package_sharing;
mod issue_669_memory_sync;
mod issue_700_si_units;
mod issue_800_product_search;
mod issue_836_legality_warning;
mod issue_861_telemetry;
mod issue_872_app_store_search;
mod issue_901_triz_solver;
mod issue_954_module_map;
mod r1017_small_math_tasks;
mod verifiable_task;

// Readers for the split REQUIREMENTS.md register and the archived CHANGELOG.md.
#[path = "../support/assembled_docs.rs"]
mod assembled_docs;

#[path = "agentic-coding/mod.rs"]
mod agentic_coding;

#[path = "capability-routing/mod.rs"]
mod capability_routing;

#[path = "memory/mod.rs"]
mod memory;

#[path = "prerequisite/mod.rs"]
mod prerequisite;

#[path = "protocol/mod.rs"]
mod protocol;

#[path = "repository-workspace/mod.rs"]
mod repository_workspace;

#[path = "seed/mod.rs"]
mod seed;

#[path = "sequences/mod.rs"]
mod sequences;

#[path = "solver/mod.rs"]
mod solver;

#[path = "web-engine-core/mod.rs"]
mod web_engine_core;

#[path = "../fixtures/source-read-provenance.rs"]
mod source_read_provenance;
