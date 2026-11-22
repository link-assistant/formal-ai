// Complete native function extraction must retain source-size headroom.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {checkListedFiles} from '../../scripts/check-file-size.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url)).replace(/\/$/u,'');
for (const [owner,helper,name] of [
  ["rust/src/agentic_coding/task_acceptance.rs", "rust/src/agentic_coding/task_acceptance/reports.rs", "acceptance_report"],
  [
    "rust/src/agentic_coding/work_item_file_io.rs",
    "rust/src/agentic_coding/work_item_file_io/command_ownership.rs",
    "owned_command"
  ],
  [
    "rust/src/agentic_coding/positional_edit.rs",
    "rust/src/agentic_coding/positional_edit/clause_insertion.rs",
    "clause_insert"
  ],
  [
    "rust/src/agentic_coding/module_function.rs",
    "rust/src/agentic_coding/module_function/arithmetic_declaration.rs",
    "closed_arithmetic_declaration"
  ],
  [
    "rust/src/agentic_coding/task_acceptance.rs",
    "rust/src/agentic_coding/task_acceptance/workflow_checks.rs",
    "workflow_checks"
  ],
  [
    "rust/src/agentic_coding/general_planner/owned_goals.rs",
    "rust/src/agentic_coding/general_planner/owned_goals/collection_ownership.rs",
    "collection_summary_owns"
  ],
  [
    "rust/src/formalization/proposition_structure.rs",
    "rust/src/formalization/proposition_structure/reader.rs",
    "read_proposition_structure"
  ],
  [
    "rust/src/solver_handlers/statistics.rs",
    "rust/src/solver_handlers/statistics/word_problems.rs",
    "handle_word_problem"
  ],
  [
    "rust/src/engine.rs",
    "rust/src/engine/knowledge_graph.rs",
    "knowledge_graph"
  ],
  [
    "rust/src/meta_reasoner/reasoner.rs",
    "rust/src/meta_reasoner/reasoner/reasoning_steps.rs",
    "meta_reason_core"
  ],
  [
    "rust/src/meta_executor.rs",
    "rust/src/meta_executor/selection.rs",
    "select"
  ],
  [
    "rust/tests/source/solver_handlers/software_project.rs",
    "rust/tests/source/solver_handlers/software_project/plan_rendering.rs",
    "render_plan_response"
  ],
  [
    "rust/tests/source/solver_handlers/mod.rs",
    "rust/tests/source/solver_handlers/translation_dispatch.rs",
    "try_translation"
  ],
  [
    "rust/tests/source/solver_handlers/behavior_rules.rs",
    "rust/tests/source/solver_handlers/behavior_rules/rule_matching.rs",
    "rule_when_then"
  ],
  [
    "rust/tests/source/engine.rs",
    "rust/tests/source/engine/language_answers.rs",
    "language_aware_answer_for"
  ]
]) {
 test(owner+' keeps its complete function below the unchanged size warning',()=>{
  assert.deepEqual(checkListedFiles(root,new Set([owner,helper])),{warnings:[],violations:[],embeddedDataViolations:[],errors:[]});
  const source=readFileSync(root+'/'+owner,'utf8'),child=readFileSync(root+'/'+helper,'utf8');
  const module=helper.split('/').at(-1).slice(0,-3);
  assert(source.includes('mod '+module+';'));
  assert(source.includes('use '+module+'::'+name+';'));
  assert(!new RegExp('fn '+name+'\\b','u').test(source),'the function must have one owner');
  assert(new RegExp('fn '+name+'\\b','u').test(child),'the original function remains reachable');
 });
}
