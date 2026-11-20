import { serverMessage } from '../messages.mjs';
import {
  BREAK, CONTINUE, createMemoryLinks, createNames, decorate, formatLino, group, parseLino, runQuery, sameLink,
} from '@link-foundation/ui-links';

export const EVENT_KINDS = ['create', 'update', 'delete'];
const VARIABLES = new Set(['@id', '@source', '@target', '@old-source', '@old-target']);
export const LINK_LIMIT = 8192;
const QUERY_WORK_LIMIT = 32_768;
const EVENT_LIMIT = 256;
const ACTION_LIMIT = 512;
const CYCLE_LIMIT = 24;

export function parseLinkSubstitution(text, { template = false } = {}) {
  if (typeof text !== 'string' || text.length > 4096) throw new Error(serverMessage('dynamic_meta_substitution_length'));
  const parsed = parseLino(text);
  const document = parsed.length === 1 && typeof parsed[0] !== 'string' && parsed[0].id === null
    ? parsed[0].values : parsed;
  if (document.length !== 2 || document.some((side) => typeof side === 'string' || side.id !== null)) {
    throw new Error(serverMessage('dynamic_meta_substitution_sides'));
  }
  function visit(part, depth) {
    if (typeof part === 'string') {
      if (part.startsWith('@') && (!template || !VARIABLES.has(part))) throw new Error(serverMessage('dynamic_meta_unknown_event_reference'));
      return;
    }
    if (depth > 12 || part.values.length !== 2) throw new Error(serverMessage('dynamic_meta_expected_doublet'));
    if (part.id !== null) visit(part.id, depth + 1);
    for (const item of part.values) visit(item, depth + 1);
  }
  for (const side of document) {
    if (side.values.length > 16) throw new Error(serverMessage('dynamic_meta_pattern_limit'));
    for (const item of side.values) visit(item, 1);
  }
  return document;
}

export const renderLinkSubstitution = (document) => formatLino([group(document)]);

export function linkContext({ links = [], names = [] } = {}) {
  const store = createMemoryLinks();
  for (const { id, from_id, to_id } of links) {
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

function applyQuery(context, document, consume) {
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
      if (base.count() >= LINK_LIMIT) throw new Error(serverMessage('dynamic_meta_link_limit'));
      return base.create();
    },
    createAt(id) {
      consume();
      if (base.count() >= LINK_LIMIT) throw new Error(serverMessage('dynamic_meta_link_limit'));
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
    if (!bindings.has(part)) throw new Error(serverMessage('dynamic_meta_event_value_missing', { reference: part }));
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

/**
 * Each cycle plans against the same committed previous-cycle events and
 * enabled rule set. Pure matching can be split by event. Apply the plan in
 * event and catalog order so overlapping writes have a reproducible outcome.
 */
export function runCascade(context, initial, {
  rules, active, enabled, selfModification, setRule,
}) {
  let work = QUERY_WORK_LIMIT;
  const consume = () => {
    if (--work < 0) throw new Error(serverMessage('dynamic_meta_substitution_work'));
  };
  const trace = [];
  let pending = applyQuery(context, initial, consume);
  let eventCount = pending.length;
  let actionCount = 0;
  if (eventCount > EVENT_LIMIT) throw new Error(serverMessage('dynamic_meta_event_limit'));
  trace.push({ cycle: 0, query: renderLinkSubstitution(initial), events: pending });
  for (let cycle = 1; pending.length && enabled; cycle += 1) {
    const eligible = rules.filter((rule) => active(rule.id));
    const plan = [];
    for (const event of pending) {
      for (const rule of eligible) {
        consume();
        if (!matches(rule, event)) continue;
        if (++actionCount > ACTION_LIMIT) throw new Error(serverMessage('dynamic_meta_trigger_action_limit'));
        plan.push({ event, rule });
      }
    }
    if (!plan.length) break;
    if (cycle > CYCLE_LIMIT) throw new Error(serverMessage('dynamic_meta_cycle_limit'));
    pending = [];
    for (const { event, rule } of plan) {
      if (rule.substitution !== undefined) {
        const query = bindTemplate(rule.substitution, event);
        const events = applyQuery(context, query, consume);
        eventCount += events.length;
        if (eventCount > EVENT_LIMIT) throw new Error(serverMessage('dynamic_meta_event_limit'));
        pending.push(...events);
        trace.push({ cycle, rule: rule.id, event, query: renderLinkSubstitution(query), events });
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
