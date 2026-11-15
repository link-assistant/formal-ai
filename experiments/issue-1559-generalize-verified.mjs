// Issue #1559 §4: a statement made in two scopes with a replay-verified task
// becomes one generalized, unscoped amendment.
// Usage: node experiments/issue-1559-generalize-verified.mjs
import { memoryEvent } from '../js/server/memory-store.mjs';
import { applyDreamingPlan, dreamingConfig, planMemoryDreaming } from '../js/server/dreaming-plan.mjs';
import { memoizedReplay, replayAnswerWithAmendments } from '../js/server/dreaming-replay.mjs';

const RULE = 'Always include a LaTeX verification step.';
const input = 'Explain a latex proof by induction';
const events = [
  memoryEvent({ id: 'req-alice', kind: 'message', role: 'user', content: RULE, conversation_title: 'latex', evidence: ['scope:user=alice'] }),
  memoryEvent({ id: 'req-bob', kind: 'message', role: 'user', content: RULE, conversation_title: 'latex', evidence: ['scope:user=bob'] }),
  memoryEvent({ id: 'run', kind: 'test_run', role: 'derived', content: input, inputs: input,
    outputs: replayAnswerWithAmendments(input, [{ id: 'x', topic: 'latex', rule: RULE }]), conversation_title: 'latex' }),
];
const replay = memoizedReplay();
const plan = planMemoryDreaming(events, dreamingConfig(), replay);
const dreamed = applyDreamingPlan(events, plan, replay);
console.log(dreamed.events.filter((e) => e.kind === 'meta_algorithm_amendment').map((e) => [e.id, e.inputs, e.outputs, e.evidence]));
