// `crate::client_contract_learning` (rust/src/client_contract_learning.rs): the
// human-gated learning for real-client verification contracts (issue #671).
// Two independently worded observations of the same capability are enough to
// propose a stable contract amendment; they are never enough to apply it.
// Proposals point back to their evidence and stay `awaiting_human_review`.
//
// The orchestration controller feeds its own recorded sessions through this
// learner (`observe_orchestration_session`), so the part ported here is the
// observation record and `learn_client_contracts` with its review artifact. The
// JSONL transcript loaders (`load_observations`, `observe_proxy_transcript`)
// belong to `formal-ai learn clients` and are not orchestration inputs.

import { stableId } from './engine_stable_identifier.mjs';

const byteOrder = (left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right));
const sorted = (values) => [...values].sort(byteOrder);

/**
 * Mirrors `ClientContractObservation::new` in rust/src/client_contract_learning.rs.
 * @param {string} clientId @param {string} capability @param {string} taskWording
 * @param {'tool_call'|'in_band'} delivery @param {string[]} invokedTools @param {string} evidence
 */
export function clientContractObservation(clientId, capability, taskWording, delivery, invokedTools, evidence) {
  return {
    client_id: clientId,
    capability,
    task_wording: taskWording,
    delivery,
    advertised_tools: [],
    invoked_tools: [...invokedTools],
    observed_contract: {},
    evidence,
  };
}

/** Mirrors `fn normalize` in rust/src/client_contract_learning.rs. */
function normalize(value) {
  return value.split(/\p{White_Space}+/u).filter(Boolean).join(' ').toLowerCase();
}

/** Mirrors `fn quote` in rust/src/client_contract_learning.rs. @param {string} value */
function quote(value) {
  return value.replaceAll('\\', '\\\\').replaceAll('"', "'").replaceAll('\n', '\\n').replaceAll('\r', '\\r');
}

/** Mirrors `fn field` in rust/src/client_contract_learning.rs. */
function field(depth, key, value) {
  return `${'  '.repeat(depth)}${key} "${quote(value)}"\n`;
}

/** Mirrors `fn consistent_delivery` in rust/src/client_contract_learning.rs: the delivery every observation agrees on, or null. */
function consistentDelivery(group) {
  if (group.length === 0) return null;
  const first = group[0].delivery;
  return group.every((observation) => observation.delivery === first) ? first : null;
}

/** Mirrors `fn stable_invoked_tools` in rust/src/client_contract_learning.rs: tools invoked in every observation. */
function stableInvokedTools(group) {
  if (group.length === 0) return new Set();
  const stable = new Set(group[0].invoked_tools);
  for (const observation of group.slice(1)) {
    const tools = new Set(observation.invoked_tools);
    for (const tool of [...stable]) if (!tools.has(tool)) stable.delete(tool);
  }
  return stable;
}

/** Mirrors `fn is_observed_contract_field` in rust/src/client_contract_learning.rs. */
function isObservedContractField(name) {
  return /^[a-z][a-z0-9_]*$/.test(name) && name !== 'file_delivery' && name !== 'required_response_tool';
}

/** Mirrors `fn normalized_contract` in rust/src/client_contract_learning.rs: field to non-empty set of trimmed values. */
function normalizedContract(contract) {
  const out = new Map();
  for (const name of sorted(Object.keys(contract))) {
    if (!isObservedContractField(name)) continue;
    const values = new Set(contract[name].map((value) => value.trim()).filter((value) => value !== ''));
    if (values.size > 0) out.set(name, values);
  }
  return out;
}

/** Mirrors `fn stable_observed_contract` in rust/src/client_contract_learning.rs: values observed in every observation. */
function stableObservedContract(group) {
  if (group.length === 0) return new Map();
  const stable = normalizedContract(group[0].observed_contract);
  for (const observation of group.slice(1)) {
    const contract = normalizedContract(observation.observed_contract);
    for (const [name, values] of [...stable]) {
      const observed = contract.get(name);
      if (!observed) {
        stable.delete(name);
        continue;
      }
      for (const value of [...values]) if (!observed.has(value)) values.delete(value);
      if (values.size === 0) stable.delete(name);
    }
  }
  return stable;
}

/**
 * Mirrors `fn learn_client_contracts` in rust/src/client_contract_learning.rs:
 * learn reusable constraints from independently worded real-client runs. A tool
 * becomes stable only when it was invoked in every observation of a
 * `(client, capability)` group with at least two distinct task wordings.
 * @param {Array<object>} observations
 * @param {Array<object>} integrations the registry (`clientIntegrations()`)
 */
export function learnClientContracts(observations, integrations) {
  const contracts = new Map(integrations.map((integration) => [integration.id, integration]));
  const groups = new Map();
  for (const observation of observations) {
    const key = `${observation.client_id}\0${observation.capability}`;
    if (!groups.has(key)) groups.set(key, { client_id: observation.client_id, capability: observation.capability, items: [] });
    groups.get(key).items.push(observation);
  }
  let independentlyWordedGroups = 0;
  const findings = [];
  const proposals = [];
  const orderedGroups = [...groups.values()].sort((left, right) =>
    byteOrder(left.client_id, right.client_id) || byteOrder(left.capability, right.capability));
  for (const { client_id: clientId, capability, items: group } of orderedGroups) {
    const wordings = new Set(group.map((observation) => normalize(observation.task_wording)));
    if (wordings.size < 2) continue;
    independentlyWordedGroups += 1;
    const stableTools = stableInvokedTools(group);
    const stableContract = stableObservedContract(group);
    const integration = contracts.get(clientId);
    const accepted = new Set(integration ? integration.verification.required_response_tools : []);
    const evidence = sorted(new Set(group.map((observation) => observation.evidence)));
    const observedDelivery = consistentDelivery(group);
    const seededDelivery = integration ? integration.verification.file_delivery : '';
    let status = 'unseeded';
    if (observedDelivery !== null && seededDelivery === observedDelivery) status = 'confirmed';
    else if (observedDelivery !== null && seededDelivery !== '') status = 'contract_drift';
    findings.push({
      client_id: clientId,
      capability,
      wording_count: wordings.size,
      observed_delivery: observedDelivery,
      seeded_delivery: seededDelivery,
      status,
      evidence,
    });
    if (status === 'contract_drift') {
      proposals.push({
        id: stableId('client_contract_proposal', `${clientId}\0${capability}\0file_delivery\0${observedDelivery}`),
        client_id: clientId,
        capability,
        field: 'file_delivery',
        value: observedDelivery,
        evidence,
      });
    }
    for (const tool of sorted(stableTools)) {
      if (accepted.has(tool)) continue;
      proposals.push({
        id: stableId('client_contract_proposal', `${clientId}\0${capability}\0required_response_tool\0${tool}`),
        client_id: clientId,
        capability,
        field: 'required_response_tool',
        value: tool,
        evidence,
      });
    }
    for (const contractField of sorted(stableContract.keys())) {
      for (const value of sorted(stableContract.get(contractField))) {
        proposals.push({
          id: stableId('client_contract_proposal', `${clientId}\0${capability}\0${contractField}\0${value}`),
          client_id: clientId,
          capability,
          field: contractField,
          value,
          evidence,
        });
      }
    }
  }
  return {
    observation_count: observations.length,
    independently_worded_groups: independentlyWordedGroups,
    findings,
    proposals,
    awaiting_human_review: proposals.length > 0,
  };
}

/**
 * Mirrors `ClientContractLearningReport::links_notation` in
 * rust/src/client_contract_learning.rs: the review artifact in Links Notation.
 */
export function learningLinksNotation(report) {
  let out = 'client_contract_learning\n';
  out += field(1, 'issue', '671');
  out += field(1, 'human_gated', 'true');
  out += field(1, 'decision', report.awaiting_human_review ? 'awaiting_human_review' : 'no_reviewable_change');
  out += `  observation_count "${report.observation_count}"\n`;
  out += `  independently_worded_groups "${report.independently_worded_groups}"\n`;
  out += `  proposal_count "${report.proposals.length}"\n`;
  for (const finding of report.findings) {
    out += '  finding\n';
    out += field(2, 'client', finding.client_id);
    out += field(2, 'capability', finding.capability);
    out += `    wording_count "${finding.wording_count}"\n`;
    out += field(2, 'observed_delivery', finding.observed_delivery ?? 'inconsistent');
    out += field(2, 'seeded_delivery', finding.seeded_delivery);
    out += field(2, 'status', finding.status);
    for (const evidence of finding.evidence) out += field(2, 'evidence', evidence);
  }
  for (const proposal of report.proposals) {
    out += '  proposal\n';
    out += field(2, 'id', proposal.id);
    out += field(2, 'client', proposal.client_id);
    out += field(2, 'capability', proposal.capability);
    out += field(2, 'field', proposal.field);
    out += field(2, 'value', proposal.value);
    out += field(2, 'target', 'data/seed/client-integrations.lino');
    out += field(2, 'decision', 'awaiting_human_review');
    for (const evidence of proposal.evidence) out += field(2, 'evidence', evidence);
  }
  return out.trimEnd();
}
