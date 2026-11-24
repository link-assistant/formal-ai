// Replay the real unchanged-limit scanner against the final large native owners.
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {checkListedFiles} from '../../scripts/check-file-size.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url)).replace(/\/$/u,'');
for(const {owner,helpers} of [
  {
    "owner": "js/wasm-worker/src/lib.rs",
    "helpers": [
      "js/wasm-worker/src/dialogue_rendering.rs"
    ]
  },
  {
    "owner": "rust/src/agentic_coding/shell_command.rs",
    "helpers": [
      "rust/src/agentic_coding/shell_command/prose_classification.rs"
    ]
  },
  {
    "owner": "rust/src/agentic_coding/evidence_record.rs",
    "helpers": [
      "rust/src/agentic_coding/evidence_record/evidence_relevance.rs"
    ]
  },
  {
    "owner": "rust/src/agentic_coding/web_research.rs",
    "helpers": [
      "rust/src/agentic_coding/web_research/answer_selection.rs"
    ]
  },
  {
    "owner": "rust/src/agentic_coding/write_request.rs",
    "helpers": [
      "rust/src/agentic_coding/write_request/write_bindings.rs"
    ]
  },
  {
    "owner": "rust/src/agentic_coding/general_execution.rs",
    "helpers": [
      "rust/src/agentic_coding/general_execution/work_item_delivery.rs"
    ]
  },
  {
    "owner": "rust/tests/source/solver_helpers.rs",
    "helpers": [
      "rust/tests/source/solver_helpers/request_classification.rs"
    ]
  },
  {
    "owner": "rust/tests/source/intent_formalization.rs",
    "helpers": [
      "rust/tests/source/intent_formalization/write_rule_recovery.rs"
    ]
  },
  {
    "owner": "rust/tests/source/seed/meanings.rs",
    "helpers": [
      "rust/tests/source/seed/meanings/meaning_readers.rs"
    ]
  },
  {
    "owner": "rust/src/execution_box/mod.rs",
    "helpers": [
      "rust/src/execution_box/configuration.rs"
    ]
  },
  {
    "owner": "rust/src/agentic_coding/planner.rs",
    "helpers": [
      "rust/src/agentic_coding/planner/request_routes.rs"
    ]
  },
  {
    "owner": "rust/src/agentic_coding/command_reroute.rs",
    "helpers": [
      "rust/src/agentic_coding/command_reroute/recipe_execution.rs"
    ]
  },
  {
    "owner": "rust/src/formalization/text_statements.rs",
    "helpers": [
      "rust/src/formalization/text_statements/types.rs"
    ]
  },
  {
    "owner": "rust/tests/source/solver.rs",
    "helpers": [
      "rust/tests/source/solver/execution.rs"
    ]
  },
  {
    "owner": "rust/examples/generate_grammar_projection_rules/rules.rs",
    "helpers": [
      "rust/examples/generate_grammar_projection_rules/rules/primary_spine.rs",
      "rust/examples/generate_grammar_projection_rules/rules/reverse_spine.rs"
    ]
  }
]) test(owner+' retains source headroom',()=>{
 assert.deepEqual(checkListedFiles(root,new Set([owner,...helpers])),{warnings:[],violations:[],embeddedDataViolations:[],errors:[]});
});
