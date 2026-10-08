// The policy gates: the browser twin of rust/src/solver_handlers/policy_gates.rs
// `try_policy_gates` (R1188-U29).
//
// The native solver asks the gates after its handler table declines and before
// the unknown branch: inappropriate content, unbounded autonomy without an
// agent opt-in, forget and cache-flush requests, and the agent-mode
// confirmations. Each gate reads its seed roles in
// data/seed/meanings-policy.lino as raw substrings of the lowercased prompt, in
// every supported language (`mentions_role_raw`), logs the events the native
// gate logs, and answers with the seeded response its slug names, spelled with
// `-`, in data/seed/multilingual-responses-policy.lino, the English one for a
// language the seed does not carry. The answer's `solverEvents` and
// `responseLink` reach the native log (formal_ai_worker_solver_events.js), so
// its evidence carries `policy:*`, `agent_mode:*` and `response:policy:*` as
// the native answer does.
//
// The browser has no isolated workspace, so the last gate answers an agent
// request with the opt-in confirmation where the native CLI surface would
// first run `try_agent_workspace_task`, as the native HTTP surface does.

const POLICY_GATE_CONFIDENCE = 0.5;
const POLICY_GATE_FALLBACK_LANGUAGE = "en";

/** The gates in the order the native function asks them; the first that holds answers. */
const POLICY_GATES = Object.freeze([
  {
    slug: "inappropriate_content",
    requires: ["vulgar_content_marker"],
    events: ["policy:inappropriate_content"],
  },
  {
    slug: "bounded_autonomy",
    requires: ["unbounded-autonomy-marker"],
    excludes: ["agent-mode-opt-in-marker"],
    events: ["policy:chat_bounded_autonomy"],
  },
  {
    slug: "add_only_history",
    requires: ["forget-request-marker"],
    events: ["policy:add_only_history"],
  },
  {
    slug: "cache_flush_requires_confirmation",
    requires: ["cache-clearing-action", "cache-reference"],
    events: ["policy:cache_flush_requires_confirmation"],
  },
  {
    slug: "destructive_action_requires_confirmation",
    requires: ["agent-mode-opt-in-marker", "destructive-action-marker"],
    events: ["agent_mode:opted_in", "policy:destructive_action_requires_confirmation"],
  },
  {
    slug: "agent_time_budget",
    requires: ["agent-mode-opt-in-marker", "unbounded-loop-marker"],
    events: ["agent_mode:opted_in", "policy:agent_time_budget"],
  },
  {
    slug: "agent_action",
    requires: ["agent-mode-opt-in-marker"],
    events: ["agent_mode:opted_in", "agent_mode:active", "action_log"],
  },
]);

/**
 * Whether the lowercased prompt holds every role a gate requires and none it
 * excludes.
 * @param {{requires: string[], excludes?: string[]}} gate
 * @param {string} lowered
 * @returns {boolean}
 */
function policyGateHolds(gate, lowered) {
  const mentions = (role) => lexiconMentionsRoleSubstring(role, lowered);
  return gate.requires.every(mentions) && !(gate.excludes || []).some(mentions);
}

/**
 * Whether the prompt opts in to agent mode (`is_agent_request`): the chat
 * handlers decline such a request and leave it to the agent gates.
 * @param {string} prompt
 * @returns {boolean}
 */
function isAgentModeRequest(prompt) {
  return lexiconMentionsRoleSubstring("agent-mode-opt-in-marker", String(prompt || "").toLowerCase());
}

/**
 * The answer of the first gate that holds, or null.
 * @param {string} prompt
 * @param {string} language the turn's language (forced or detected)
 * @returns {object|null}
 */
function tryPolicyGates(prompt, language) {
  const source = String(prompt || "");
  const gate = POLICY_GATES.find((row) => policyGateHolds(row, source.toLowerCase()));
  if (!gate) return null;
  const response = gate.slug.replaceAll("_", "-");
  const content = handlerRulesResponseFor(response, language)
    ?? handlerRulesResponseFor(response, POLICY_GATE_FALLBACK_LANGUAGE);
  if (content === null) return null;
  return {
    intent: `policy_${gate.slug}`,
    content,
    confidence: POLICY_GATE_CONFIDENCE,
    evidence: [`policy_gate:${gate.slug}`, `language:${language}`],
    solverEvents: gate.events.map((kind) => ({ kind, payload: source })),
    responseLink: `response:policy:${gate.slug}`,
  };
}
