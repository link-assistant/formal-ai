import { serverMessage } from '../messages.mjs';
import {
  BREAK, CONTINUE, createMemoryLinks, createNames, decorate, exists, formatLino, getLink, group, parseLino, runQuery, sameLink,
} from '@link-foundation/ui-links';

export const EVENT_KINDS = ['create', 'update', 'delete'];
const VARIABLES = new Set(['@id', '@source', '@target', '@old-source', '@old-target']);
export const LINK_LIMIT = 8192;
const QUERY_WORK_LIMIT = 32_768;
const EVENT_LIMIT = 256;
const ACTION_LIMIT = 512;
const CYCLE_LIMIT = 24;

export function parseLinkSubstitution(text, { template = false } = {}) {
  if (typeof text !== 'string' || text.length > 4096) throw new Error(serverMessage('dynamic-meta-substitution-length'));
  const parsed = parseLino(text);
  const document = parsed.length === 1 && typeof parsed[0] !== 'string' && parsed[0].id === null
    ? parsed[0].values : parsed;
  if (document.length !== 2 || document.some((side) => typeof side === 'string' || side.id !== null)) {
    throw new Error(serverMessage('dynamic-meta-substitution-sides'));
  }
  function visit(part, depth) {
    if (typeof part === 'string') {
      if (part.startsWith('@') && (!template || !VARIABLES.has(part))) throw new Error(serverMessage('dynamic-meta-unknown-event-reference'));
      return;
    }
    if (depth > 12 || part.values.length !== 2) throw new Error(serverMessage('dynamic-meta-expected-doublet'));
    if (part.id !== null) visit(part.id, depth + 1);
    for (const item of part.values) visit(item, depth + 1);
  }
  for (const side of document) {
    if (side.values.length > 16) throw new Error(serverMessage('dynamic-meta-pattern-limit'));
    for (const item of side.values) visit(item, 1);
  }
  return document;
}

export const renderLinkSubstitution = (document) => formatLino([group(document)]);

export function linkContext({ links = [], names = [] } = {}, consume = () => {}) {
  const store = createMemoryLinks();
  for (const { id, from_id, to_id } of links) {
    consume();
    store.createAt(id);
    store.update(id, from_id, to_id);
  }
  return { links: store, names: createNames(names) };
}

export function snapshot(context) {
  const links = [];
  context.links.each((link) => { links.push(link); });
  return { links, names: context.names.entries() };
}

function applyQuery(context, document, consume, reserve) {
  // Bound the candidate enumeration inside the upstream join as well as the
  // number of distinct outputs. The caller discards the staged graph on error.
  const base = context.links;
  const bounded = decorate(base, {
    each(handler, query) {
      consume();
      return base.each((link) => {
        consume();
        return handler(link) === BREAK ? BREAK : CONTINUE;
      }, query);
    },
    create() {
      consume();
      if (base.count() >= LINK_LIMIT) throw new Error(serverMessage('dynamic-meta-link-limit'));
      if (reserve) {
        let address = reserve();
        while (exists(base, address)) address = reserve();
        return base.createAt(address);
      }
      return base.create();
    },
    createAt(id) {
      consume();
      if (base.count() >= LINK_LIMIT) throw new Error(serverMessage('dynamic-meta-link-limit'));
      return base.createAt(id);
    },
  });
  // The upstream query reader treats a two-pattern first side as a wrapper.
  // Give it an explicit wrapper even when the input used two top-level sides.
  return runQuery({ links: bounded, base, names: context.names }, [group(document)])
    .filter(({ before, after }) => !sameLink(before, after))
    .map(({ before, after }) => ({
      kind: before === null ? 'create' : after === null ? 'delete' : 'update',
      before,
      after,
    }));
}

function eventBindings({ before, after }) {
  const current = after ?? before;
  return new Map([
    ['@id', current.id], ['@source', current.from_id], ['@target', current.to_id],
    ...(before ? [['@old-source', before.from_id], ['@old-target', before.to_id]] : []),
  ]);
}

function bindTemplate(document, event) {
  const bindings = eventBindings(event);
  const reference = (part) => {
    if (!part.startsWith('@')) return part;
    if (!bindings.has(part)) throw new Error(serverMessage('dynamic-meta-event-value-missing', { reference: part }));
    return String(bindings.get(part));
  };
  function bind(part) {
    if (typeof part === 'string') return reference(part);
    return { id: part.id === null ? null : reference(part.id), values: part.values.map(bind) };
  }
  return document.map(bind);
}

function matches({ when }, event) {
  const current = event.after ?? event.before;
  return when.event === event.kind &&
    (when.source === undefined || when.source === current.from_id) &&
    (when.target === undefined || when.target === current.to_id);
}

function editConflict() {
  return new Error(serverMessage('dynamic-meta-substitution-conflict'));
}

function applyCycle(context, plan, consume, eventBudget) {
  const graph = snapshot(context);
  const occupied = new Set(graph.links.map(({ id: address }) => address));
  let next = 1;
  function reserve() {
    consume();
    while (occupied.has(next)) next += 1;
    return next++;
  }

  // Plan the queries before exposing any writes from this cycle.
  const changes = plan.map((entry) => {
    if (entry.rule.substitution === undefined) return entry;
    const query = bindTemplate(entry.rule.substitution, entry.event);
    const staged = linkContext(graph, consume);
    const allocated = new Set();
    const events = applyQuery(staged, query, consume, () => {
      const address = reserve();
      allocated.add(address);
      return address;
    });
    eventBudget -= events.length;
    if (eventBudget < 0) throw new Error(serverMessage('dynamic-meta-event-limit'));
    const created = events.filter(({ before }) => before === null).map(({ after }) => ({
      address: after.id,
      name: staged.names.getName(after.id),
      anonymous: allocated.has(after.id) ? `${after.from_id}:${after.to_id}` : null,
    }));
    return { ...entry, query: renderLinkSubstitution(query), events, created };
  });

  const anonymous = new Set();
  const events = changes.flatMap((entry) => entry.events ?? []);
  for (const entry of changes) {
    if (entry.events === undefined) continue;
    for (const { address, name, anonymous: pair } of entry.created) {
      if (name !== null) {
        if (context.names.getByName(name) !== null) throw editConflict();
        context.names.setName(address, name);
      } else if (pair !== null) {
        if (anonymous.has(pair)) throw editConflict();
        anonymous.add(pair);
      }
    }
    for (const { before, after } of entry.events) {
      const address = (before ?? after).id;
      if (!sameLink(getLink(context.links, address), before)) throw editConflict();
      if (after === null) {
        context.links.delete(address);
        context.names.removeName(address);
      } else {
        if (before === null) context.links.createAt(address);
        context.links.update(address, after.from_id, after.to_id);
      }
    }
  }
  if (context.links.count() > LINK_LIMIT) throw new Error(serverMessage('dynamic-meta-link-limit'));
  for (const { before, after } of events) {
    if (after === null) {
      if (context.links.count({ from_id: before.id }) || context.links.count({ to_id: before.id })) throw editConflict();
    } else if ([after.from_id, after.to_id].some((part) => part !== 0 && !exists(context.links, part))) {
      throw editConflict();
    }
  }
  return { changes, events };
}

/**
 * Each cycle plans against the same previous-cycle graph, events and rules.
 * Apply edits in event and catalog order and reject conflicting results.
 */
export function runCascade(context, initial, {
  rules, active, enabled, selfModification, setRule,
}) {
  let work = QUERY_WORK_LIMIT;
  const consume = () => {
    if (--work < 0) throw new Error(serverMessage('dynamic-meta-substitution-work'));
  };
  const trace = [];
  let pending = applyQuery(context, initial, consume);
  let eventCount = pending.length;
  let actionCount = 0;
  if (eventCount > EVENT_LIMIT) throw new Error(serverMessage('dynamic-meta-event-limit'));
  trace.push({ cycle: 0, query: renderLinkSubstitution(initial), events: pending });
  for (let cycle = 1; pending.length && enabled; cycle += 1) {
    const eligible = rules.filter((rule) => active(rule.id));
    const plan = [];
    for (const event of pending) {
      for (const rule of eligible) {
        consume();
        if (!matches(rule, event)) continue;
        if (++actionCount > ACTION_LIMIT) throw new Error(serverMessage('dynamic-meta-trigger-action-limit'));
        plan.push({ event, rule });
      }
    }
    if (!plan.length) break;
    if (cycle > CYCLE_LIMIT) throw new Error(serverMessage('dynamic-meta-cycle-limit'));
    const prepared = applyCycle(context, plan, consume, EVENT_LIMIT - eventCount);
    pending = prepared.events;
    eventCount += pending.length;
    for (const { event, rule, query, events } of prepared.changes) {
      if (events !== undefined) {
        trace.push({ cycle, rule: rule.id, event, query, events });
      } else {
        const { target, enabled: value } = rule.setRule;
        if (selfModification) setRule(target, value);
        trace.push({
          cycle, rule: rule.id, event, target,
          decision: selfModification ? (value ? 'enabled' : 'disabled') : 'self-modification-disabled',
        });
      }
    }
  }
  return trace;
}
