import {citations,resolvableSegments,resolves} from '../../scripts/check-twin-citations.mjs';
// Run the maintained scanner against the native regression module owners.
import assert from 'node:assert/strict';
import {existsSync, readFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import YAML from 'yaml';
import {checkListedFiles} from '../../scripts/check-file-size.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url)).replace(/\/$/u, '');
for (const [owner, child] of [
  ['rust/tests/unit/agentic-coding/agentic_general_planner', 'command_capture'],
  ['rust/tests/unit/ci-cd/job_budget_fit', 'standalone_suites'],
  ['rust/tests/unit/ci-cd/pinned_tool_images', 'image_policy'],
]) {
  test(owner + ' retains file-size headroom and a reachable regression group', () => {
    const files = new Set([owner + '.rs', owner + '/' + child + '.rs']);
    const result = checkListedFiles(root, files);
    assert.deepEqual(result, {warnings: [], violations: [], embeddedDataViolations: []});
    assert.match(readFileSync(root + '/' + owner + '.rs', 'utf8'), new RegExp('^mod ' + child + ';$', 'mu'));
    assert.match(readFileSync(root + '/' + owner + '/' + child + '.rs', 'utf8'), /^#\[test\]$/mu);
  });
}

test('the hosted native targets execute every extracted regression owner', () => {
  for (const [target, owners] of [
    ['coding_contracts', ['agentic-coding/agentic_general_planner']],
    ['workflow_contracts', ['ci-cd/job_budget_fit', 'ci-cd/pinned_tool_images', 'ci-cd/release_publishing']],
  ]) {
    const source = readFileSync(root + '/rust/tests/unit/' + target + '.rs', 'utf8');
    for (const owner of owners) {
      assert(source.includes('#[path = "' + owner + '.rs"]'), target + ' omits ' + owner);
    }
    assert(readFileSync(root + '/scripts/ci-native-regressions.sh', 'utf8').includes('--test ' + target));
  }
});

test('both core checkout attempts retain the original upstream-report fixtures', () => {
  const workflow = YAML.parse(readFileSync(root + '/.github/workflows/native-type-check.yml', 'utf8'));
  const attempts = workflow.jobs.clippy.steps.filter(step => step.uses === 'actions/checkout@v7');
  assert.equal(attempts.length, 2);
  for (const step of attempts) {
    assert(step.with['sparse-checkout'].split('\n').includes('/dev/log/issues/1079/pulls/1080/upstream-reports/'));
  }
});

for (const [owner, child] of [
  ['rust/tests/unit/ci-cd/workflow_release', 'workflow_policy'],
  ['rust/tests/unit/data_files', 'text_rows'],
  ['rust/tests/unit/specification/benchmarks', 'evaluation'],
]) {
  test(owner + ' keeps native policy module headroom', () => {
    const result = checkListedFiles(root, new Set([owner + '.rs', owner + '/' + child + '.rs']));
    assert.deepEqual(result, {warnings: [], violations: [], embeddedDataViolations: []});
    assert.match(readFileSync(root + '/' + owner + '.rs', 'utf8'), new RegExp('^mod ' + child + ';$', 'mu'));
    assert.match(readFileSync(root + '/' + owner + '/' + child + '.rs', 'utf8'), /^(?:pub\(super\) )?fn /mu);
  });
}

test('the standalone data-files Cargo target resolves its extracted helper', () => {
  const cargo = readFileSync(root + '/rust/Cargo.toml', 'utf8');
  const target = cargo.split('[[test]]').find(block => /^name = "data_files"$/mu.test(block));
  assert(target, 'the data-files target must remain registered');
  const cratePath = /^path = "([^"]+)"$/mu.exec(target)?.[1];
  assert(cratePath, 'the target must identify its real Rust crate root');
  const crateFile = resolve(root, 'rust', cratePath);
  const declaration = /(?:#\[path = "([^"]+)"\]\n)?mod text_rows;/u.exec(readFileSync(crateFile, 'utf8'));
  assert(declaration, 'the crate root must retain its text-row helpers');
  const child = resolve(dirname(crateFile), declaration[1] ?? 'text_rows.rs');
  assert(existsSync(child), 'the crate-root module declaration resolves to a missing file: ' + child);
});

for (const [owner, child, names] of [
  ['rust/tests/unit/ci-cd/release_publishing', 'artifact_contracts', [
    'desktop_linux_artifact_names_are_normalized_for_download_contract',
    'release_scripts_check_configured_release_artifacts',
  ]],
  ['rust/tests/integration/with_formal_ai', 'global_configuration', [
    'with_formal_ai_global_configures_idempotently_and_undo_restores_backups',
  ]],
]) {
  test(owner + ' retains headroom and original native test names', () => {
    assert.deepEqual(checkListedFiles(root, new Set([owner + '.rs', owner + '/' + child + '.rs'])),
      {warnings: [], violations: [], embeddedDataViolations: []});
    const source = readFileSync(root + '/' + owner + '.rs', 'utf8');
    const extracted = readFileSync(root + '/' + owner + '/' + child + '.rs', 'utf8');
    for (const name of names) {
      assert(source.includes('#[test]\nfn ' + name + '() {\n    ' + child + '::' + name + '();\n}'));
      assert(extracted.includes('pub(super) fn ' + name + '() {'));
    }
  });
}

test('Hive commit expectations retain headroom without moving native test names', () => {
  const owner = 'rust/tests/unit/issue_1133_hive_mind_three_runs/extended';
  assert.deepEqual(checkListedFiles(root, new Set([owner + '.rs', owner + '/commit_expectations.rs'])),
    {warnings: [], violations: [], embeddedDataViolations: []});
  const source = readFileSync(root + '/' + owner + '.rs', 'utf8');
  assert.match(source, /^mod commit_expectations;$/mu);
  const helpers = readFileSync(root + '/' + owner + '/commit_expectations.rs', 'utf8');
  for (const name of ['commit_phase_prompt', 'expected_owned_recipe_commit', 'assert_named_branch_push']) {
    assert(helpers.includes('pub(super) fn ' + name + '('));
    assert(source.includes(name));
  }
  assert(!helpers.includes('#[test]'), 'only fixture helpers move out of the original test module');
});


test('procedure fixtures retain headroom while assertion bodies remain extractable', () => {
  const owner = 'rust/tests/unit/specification/reasoning_paths_procedures';
  assert.deepEqual(checkListedFiles(root, new Set([owner + '.rs', owner + '/fixtures.rs'])),
    {warnings: [], violations: [], embeddedDataViolations: []});
  const source = readFileSync(root + '/' + owner + '.rs', 'utf8');
  assert.match(source, /^mod fixtures;$/mu);
  for (const name of ['INSTALL_CASES', 'SURFACE_CASES', 'TYPO_CASES']) assert(source.includes('fixtures::' + name));
});


test('documentation requirement helpers retain headroom and the original assertion owners', () => {
  const owner = 'rust/tests/unit/docs_requirements';
  assert.deepEqual(checkListedFiles(root, new Set([owner + '.rs', owner + '/text_haystack.rs'])),
    {warnings: [], violations: [], embeddedDataViolations: []});
  const source = readFileSync(root + '/' + owner + '.rs', 'utf8');
  assert.match(source, /^mod text_haystack;$/mu);
  assert.match(source, /^fn repository_text_avoids_deferred_labels_requested_by_issue_103\(\) \{/mu);
  const helper = readFileSync(root + '/' + owner + '/text_haystack.rs', 'utf8');
  assert(helper.includes('.args(["ls-files", "-z"])'));
  assert(!helper.includes('#[test]'), 'all original documentation assertions remain in the native owner');
});


// A #[path]-loaded owner resolves an unqualified child beside its source file,
// rather than in the ordinary stem directory. Extracted helpers need an
// explicit relative path so the focused and original native targets agree.
function explicitHelperPath(owner, child, source) {
  const declaration = new RegExp('(?:^#\\[path = "([^"]+)"\\]\\n)?^mod ' + child + ';$', 'mu').exec(source);
  assert(declaration?.[1], owner + ' must explicitly locate ' + child);
  const stem = owner.slice(owner.lastIndexOf('/') + 1);
  assert.equal(declaration[1], stem + '/' + child + '.rs');
  const file = resolve(root, dirname(owner), declaration[1]);
  assert(existsSync(file), 'missing extracted native module: ' + file);
  return file;
}

test('path-loaded native owners explicitly resolve every extracted helper', () => {
  const pairs = [
    ["rust/tests/unit/agentic-coding/agentic_general_planner", "command_capture"],
    ["rust/tests/unit/ci-cd/job_budget_fit", "standalone_suites"],
    ["rust/tests/unit/ci-cd/job_budget_fit", "link_recheck"],
    ["rust/tests/unit/ci-cd/job_budget_fit", "release_preflight"],
    ["rust/tests/unit/ci-cd/pinned_tool_images", "image_policy"],
    ["rust/tests/unit/ci-cd/pinned_tool_images", "agent_cli"],
    ["rust/tests/unit/ci-cd/release_publishing", "artifact_contracts"],
    ["rust/tests/unit/ci-cd/workflow_release", "workflow_policy"],
    ["rust/tests/unit/specification/benchmarks", "evaluation"],
    ["rust/tests/unit/specification/reasoning_paths_procedures", "fixtures"],
    ["rust/tests/unit/docs_requirements", "text_haystack"],
    ["rust/tests/integration/with_formal_ai", "global_configuration"],
  ];
  for (const [owner, child] of pairs) {
    const source = readFileSync(root + '/' + owner + '.rs', 'utf8');
    explicitHelperPath(owner, child, source);
    const attribute = '#[path = "' + owner.slice(owner.lastIndexOf('/') + 1) + '/' + child + '.rs"]\n';
    assert.throws(() => explicitHelperPath(owner, child, source.replace(attribute, '')),
      /must explicitly locate/u, 'removing the path must reproduce the compile admission failure');
    assert.throws(() => explicitHelperPath(owner, child, source.replace(attribute, '#[path = "missing.rs"]\n')),
      /AssertionError/u, 'a wrong explicit path must also be rejected');
  }
});


test('capability routing retains file-size headroom with all public tool entry points', () => {
  const owner = 'rust/src/agentic_coding/capability_router';
  assert.deepEqual(checkListedFiles(root, new Set([owner + '.rs', owner + '/tool_classification.rs'])),
    {warnings: [], violations: [], embeddedDataViolations: []});
  const source = readFileSync(root + '/' + owner + '.rs', 'utf8');
  const classification = readFileSync(root + '/' + owner + '/tool_classification.rs', 'utf8');
  for (const name of ['tool_for', 'shell_command_tool', 'classify_tool', 'workspace_creation_tool', 'is_workspace_creation_tool']) {
    assert(source.includes(name), 'the original parent must expose ' + name);
    assert(classification.includes('pub(in crate::agentic_coding) fn ' + name));
    const citation=citations(readFileSync(root+'/js/agentic/capability_router.mjs','utf8'))
      .find(item=>item.symbol==='fn '+name);
    assert(citation,'the JavaScript twin must cite '+name);
    assert.equal(citation.path,owner+'/tool_classification.rs');
    assert(resolves(classification,resolvableSegments(citation.symbol)));

  }
  assert(source.includes('pub(super) fn is_hosted_research_tool('));
});

for (const [owner, child, names] of [
  [
    "rust/tests/unit/specification/openai_compatibility",
    "streaming_diagnostics",
    [
      "streaming_chat_completion_emits_reasoning_content_delta",
      "streaming_responses_emit_reasoning_summary_events",
      "authenticated_routes_accept_bearer_token",
      "authenticated_routes_reject_missing_bearer_token",
      "rate_limit_metadata_is_exposed",
      "responses_api_attaches_trace_link"
    ]
  ],
  [
    "rust/tests/unit/specification/external_benchmarks",
    "upstream_records",
    [
      "upstream_records_are_parsed_into_gradable_cases",
      "a_full_slice_runs_as_offset_shards_that_add_up"
    ]
  ],
  [
    "rust/tests/unit/specification/self_hosting_metric",
    "release_policy",
    [
      "release_pipeline_and_ledger_remain_pinned_to_the_metric"
    ]
  ],
  [
    "rust/tests/unit/specification/text_manipulation",
    "framed_operands",
    [
      "apostrophes_in_english_prose_do_not_swallow_the_quoted_operands",
      "apostrophe_delimited_operands_are_recognized_in_every_supported_language",
      "chained_operations_run_in_the_order_the_request_states_them",
      "counting_questions_ask_the_seeded_count_of_the_framed_unit",
      "line_prefix_filter_reads_its_argument_and_composes_with_counts",
      "counting_a_seeded_character_class_counts_its_members"
    ]
  ],
  [
    "rust/tests/unit/issue_844_statement_merge",
    "evidence_wording",
    [
      "the_evidence_wording_comes_from_the_seed_for_every_supported_language"
    ]
  ]
]) {
  test(owner + ' keeps warning-free native acceptance owners and original test names', () => {
    assert.deepEqual(checkListedFiles(root, new Set([owner + '.rs', owner + '/' + child + '.rs'])),
      {warnings: [], violations: [], embeddedDataViolations: []});
    const source = readFileSync(root + '/' + owner + '.rs', 'utf8');
    const extracted = readFileSync(root + '/' + owner + '/' + child + '.rs', 'utf8');
    const declaration = owner.endsWith('/openai_compatibility')
      ? 'mod ' + child + ';'
      : '#[path = "' + owner.split('/').at(-1) + '/' + child + '.rs"]\nmod ' + child + ';';
    assert(source.includes(declaration), 'the actual parent resolves the extracted module');
    for (const name of names) {
      assert(source.includes('#[test]\nfn ' + name + '() {\n    ' + child + '::' + name + '();\n}'));
      assert(extracted.includes('pub(super) fn ' + name + '() {'));
    }
  });
}

for (const [owner, child, names] of [
  [
    "rust/tests/integration/issue_703_orchestration",
    "client_contracts",
    [
      "external_agent_fixture_process",
      "seed_registry_pins_each_clients_exact_native_resume_contract"
    ]
  ],
  [
    "rust/tests/integration/issue_703_orchestration_followup",
    "council_corrections",
    [
      "council_results_are_formalized_summarized_and_cross_checked",
      "a_proven_false_claim_resumes_the_same_native_session_with_parent_evidence"
    ]
  ],
  [
    "rust/tests/unit/semantic_intent_routing",
    "directory_prompts",
    [
      "directory_listing_routes_shell_variations_in_every_supported_language"
    ]
  ],
  [
    "rust/tests/unit/memory_maintenance",
    "storage_access",
    [
      "auto_free_space_for_write_stops_at_target_with_nonzero_incoming_bytes",
      "recall_counts_access_and_dreaming_treats_read_data_as_used"
    ]
  ],
  [
    "rust/tests/unit/issue_1066_ladder_capability",
    "textual_requests",
    [
      "textual_rendering_language_is_not_a_gui_capability_request",
      "a_file_the_request_asks_for_is_never_opened_for_reading"
    ]
  ],
  [
    "rust/tests/unit/issue_918_handler_rules_batch",
    "algorithm_answers",
    [
      "the_algorithm_rule_set_answers_unchanged_in_every_prompt_language"
    ]
  ],
  [
    "rust/tests/unit/coding_discovery/no_memorization",
    "relation_schema",
    [
      "the_relation_seed_declares_how_a_gloss_is_read_and_never_what_a_word_means"
    ]
  ],
  [
    "rust/tests/unit/capability-routing/issue_1175_claim_routing",
    "claim_table",
    [
      "the_claim_rows_are_read_from_the_capability_table"
    ]
  ],
  [
    "rust/tests/unit/pull_request_1188_teach_f",
    "edit_contracts",
    [
      "g69_an_addition_that_quotes_no_text_earns_a_question",
      "g70_no_seed_surface_holds_a_seeded_contraction_written_apart",
      "g70_a_provider_denial_written_with_a_contraction_is_a_failure",
      "g78_a_replace_whose_new_text_holds_the_old_is_a_verified_edit"
    ]
  ]
]) {
  test(owner + ' retains its native test inventory below the size warning', () => {
    assert.deepEqual(checkListedFiles(root, new Set([owner + '.rs', owner + '/' + child + '.rs'])),
      {warnings: [], violations: [], embeddedDataViolations: []});
    const source = readFileSync(root + '/' + owner + '.rs', 'utf8');
    const file = explicitHelperPath(owner, child, source);
    const helper = readFileSync(file, 'utf8');
    assert(!helper.includes('#[test]'), 'native test identities must remain in the original module');
    for (const name of names) {
      const wrappers = [...source.matchAll(new RegExp('^#\\[test\\]\\nfn ' + name + '\\(\\) \\{\\n([\\s\\S]*?)^\\}', 'gmu'))];
      assert.equal(wrappers.length, 1, 'the test must have one original owner');
      assert.equal(wrappers[0][1].replace(/\s/gu, ''), child + '::' + name + '();',
        'the original named test must execute exactly its assertion body');
      assert(helper.includes('pub(super) fn ' + name + '() {'));
    }
    const declaration = '#[path = "' + owner.split('/').at(-1) + '/' + child + '.rs"]\n';
    assert.throws(() => explicitHelperPath(owner, child, source.replace(declaration, '')),
      /must explicitly locate/u, 'losing the real child resolution must be detected');
  });
}
