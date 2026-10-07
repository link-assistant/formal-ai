// The github-logs importers of repository history: a port of
// rust/src/history_context/github.rs (issue #1180 R3, R4, R5 and R11).
//
// Issues and pull requests (title plus body as content, labels, trailer
// links, the body's requirement statements and the dated lifecycle
// transitions), their comments and reviews, and Actions runs with their
// failing steps, each mapped to the same memory event the Rust importer
// writes. The caller hands in the captured files as `{name, text}` entries
// (the files `formal-ai github-logs` writes), so the module reads no
// directory itself.

import { patternNumber, requirementStatements, stateTransitions } from './history_context.mjs';

const NUMBER_PLACEHOLDER = '%number%';

/** Mirrors `parse_digits`. */
function parseDigits(text) {
  return /^[0-9]+$/u.test(text) ? Number(text) : null;
}

/**
 * Mirrors `classify_log_file`: the github-logs file shape a name has.
 * @param {string} name
 * @returns {{kind: string, number?: number}}
 */
export function classifyLogFile(name) {
  if (!name.endsWith('.json')) return { kind: 'other' };
  const stem = name.slice(0, -'.json'.length);
  if (stem === 'issues-recent') return { kind: 'issue_list' };
  if (stem === 'pulls-recent') return { kind: 'pull_list' };
  if (stem === 'actions-runs-recent') return { kind: 'run_list' };
  const suffixed = (rest, suffix) => (rest.endsWith(suffix) ? parseDigits(rest.slice(0, -suffix.length)) : null);
  if (stem.startsWith('issue-')) {
    const rest = stem.slice('issue-'.length);
    const comments = suffixed(rest, '-comments');
    if (comments !== null) return { kind: 'issue_comments', number: comments };
    const number = parseDigits(rest);
    if (number !== null) return { kind: 'issue', number };
  }
  if (stem.startsWith('pr-')) {
    const rest = stem.slice('pr-'.length);
    for (const [suffix, kind] of [
      ['-conversation-comments', 'pull_conversation_comments'],
      ['-review-comments', 'pull_review_comments'],
      ['-reviews', 'pull_reviews'],
    ]) {
      const number = suffixed(rest, suffix);
      if (number !== null) return { kind, number };
    }
    const number = parseDigits(rest);
    if (number !== null) return { kind: 'pull', number };
  }
  if (stem.startsWith('run-')) {
    const number = parseDigits(stem.slice('run-'.length));
    if (number !== null) return { kind: 'run', number };
  }
  return { kind: 'other' };
}

function jsonRoot(text) {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

const valueItems = (root) => (Array.isArray(root) ? root : [root]);

/** Mirrors `field_str`: the first present string field among `names`. */
function fieldStr(value, names) {
  for (const name of names) {
    if (typeof value?.[name] === 'string') return value[name];
  }
  return null;
}

/** Mirrors `nested_login`. */
function nestedLogin(value) {
  for (const holder of ['author', 'user']) {
    const login = value?.[holder]?.login;
    if (typeof login === 'string') return login;
  }
  return '';
}

/** Mirrors `label_names`. */
function labelNames(value) {
  return Array.isArray(value?.labels)
    ? value.labels.map((label) => label?.name).filter((name) => typeof name === 'string')
    : [];
}

/** Mirrors `effective_record`. */
function recordRule(rules, kind) {
  return rules.records.find((record) => record.kind === kind) || rules.records[0];
}

/** Mirrors `rule_event`. */
function ruleEvent(rule, id, intent, content, sentAt, conversationId, evidence) {
  return { id, kind: rule.kind, role: rule.role, intent, content, sentAt, conversationId, evidence };
}

/**
 * Mirrors `formalize_issue_or_pull`.
 * @param {object} value one captured issue or pull request
 * @param {boolean} isPull
 * @param {object} rules
 */
export function formalizeIssueOrPull(value, isPull, rules) {
  if (!Number.isInteger(value?.number) || value.number < 0) return null;
  const { number } = value;
  const rule = recordRule(rules, isPull ? 'pull_request' : 'issue');
  const title = fieldStr(value, ['title']) ?? '';
  const body = fieldStr(value, ['body']) ?? '';
  const evidence = [];
  const url = fieldStr(value, ['url']);
  if (url !== null) evidence.push(`url:${url}`);
  for (const label of labelNames(value)) evidence.push(`label:${label}`);
  for (const trailer of rules.trailers) {
    const captured = patternNumber(body, trailer.pattern);
    if (captured === null) continue;
    const filled = trailer.evidence.split(NUMBER_PLACEHOLDER).join(captured);
    if (filled) evidence.push(filled);
  }
  evidence.push(...requirementStatements(body, rules), ...stateTransitions(value, rules));
  return {
    event: ruleEvent(
      rule,
      `${rule.idPrefix}${number}`,
      fieldStr(value, ['state']),
      body ? `${title}\n\n${body}` : title,
      fieldStr(value, ['createdAt', 'created_at']) ?? '',
      `issue-${number}`,
      evidence,
    ),
    updatedAt: fieldStr(value, ['updatedAt', 'updated_at']),
  };
}

/** Mirrors `formalize_review`. */
export function formalizeReview(value, parentNumber, parentIsPull, intent, index, rules) {
  const rule = recordRule(rules, 'review');
  const parentKind = parentIsPull ? 'pr' : 'issue';
  const parent = `${parentKind}-${parentNumber}`;
  const ownId = Number.isInteger(value?.id) ? String(value.id) : String(index);
  const body = fieldStr(value, ['body']) ?? '';
  const state = fieldStr(value, ['state']) ?? '';
  const author = nestedLogin(value);
  let content;
  if (!body && !state) content = author;
  else if (!body) content = `${author}: ${state}`;
  else content = `${author}: ${state}: ${body}`;
  return {
    event: ruleEvent(
      rule,
      `${rule.idPrefix}${parent}-${intent}-${ownId}`,
      intent,
      content,
      fieldStr(value, ['submittedAt', 'submitted_at', 'createdAt', 'created_at']) ?? '',
      parent,
      [`${parentKind}:${parentNumber}`],
    ),
    updatedAt: fieldStr(value, ['updatedAt', 'updated_at']),
  };
}

const contentLength = (event) => (event.content ?? '').length;

/**
 * Mirrors `import_issues_and_pulls`: issues, pull requests, comments and
 * reviews from the captured files, filtered on `updatedAt > since`.
 * @param {Array<{name: string, text: string}>} files
 * @param {string|null} since
 * @param {object} rules
 */
export function importIssuesAndPulls(files, since, rules) {
  const byId = new Map();
  const record = (imported) => {
    const existing = byId.get(imported.event.id);
    if (!existing || contentLength(imported.event) > contentLength(existing)) {
      byId.set(imported.event.id, imported.event);
    }
  };
  const passes = (imported) => {
    if (since === null || since === undefined) return true;
    return imported.updatedAt !== null && imported.updatedAt > since;
  };
  const sorted = [...files].sort((left, right) => (left.name < right.name ? -1 : left.name > right.name ? 1 : 0));
  for (const { name, text } of sorted) {
    const classified = classifyLogFile(name);
    if (['issue_list', 'issue', 'pull_list', 'pull'].includes(classified.kind)) {
      const root = jsonRoot(text);
      if (root === null) continue;
      const isPull = classified.kind === 'pull_list' || classified.kind === 'pull';
      for (const item of valueItems(root)) {
        const imported = formalizeIssueOrPull(item, isPull, rules);
        if (imported && passes(imported)) record(imported);
      }
      continue;
    }
    const batch = {
      issue_comments: [false, 'comment'],
      pull_conversation_comments: [true, 'comment'],
      pull_review_comments: [true, 'review_comment'],
      pull_reviews: [true, 'review_decision'],
    }[classified.kind];
    if (!batch) continue;
    const root = jsonRoot(text);
    const items = Array.isArray(root) ? root : [];
    items.forEach((item, index) => record(formalizeReview(item, classified.number, batch[0], batch[1], index, rules)));
  }
  return [...byId.keys()].sort().map((id) => byId.get(id));
}

/** Mirrors `failing_step_lines`. */
export function failingStepLines(run) {
  const lines = [];
  for (const job of Array.isArray(run?.jobs) ? run.jobs : []) {
    if (fieldStr(job, ['conclusion']) !== 'failure') continue;
    const jobName = fieldStr(job, ['name']) ?? '';
    const failing = (Array.isArray(job.steps) ? job.steps : [])
      .filter((step) => fieldStr(step, ['conclusion']) === 'failure')
      .map((step) => `${jobName}/${fieldStr(step, ['name']) ?? ''}`);
    lines.push(...(failing.length > 0 ? failing : [jobName]));
  }
  return lines;
}

/**
 * Mirrors `import_ci_runs`: Actions runs newer than `sinceRunId`, the richer
 * (longer) capture of a run winning.
 * @param {Array<{name: string, text: string}>} files
 * @param {number|null} sinceRunId
 * @param {object} rules
 */
export function importCiRuns(files, sinceRunId, rules) {
  const rule = recordRule(rules, 'ci_run');
  const byId = new Map();
  const sorted = [...files].sort((left, right) => (left.name < right.name ? -1 : left.name > right.name ? 1 : 0));
  for (const { name, text } of sorted) {
    const { kind } = classifyLogFile(name);
    if (kind !== 'run_list' && kind !== 'run') continue;
    const root = jsonRoot(text);
    if (root === null) continue;
    for (const run of valueItems(root)) {
      const databaseId = run?.databaseId;
      if (!Number.isInteger(databaseId) || databaseId < 0) continue;
      if (sinceRunId !== null && sinceRunId !== undefined && databaseId <= sinceRunId) continue;
      const workflow = fieldStr(run, ['workflowName', 'workflow_name']) ?? '';
      const conclusion = fieldStr(run, ['conclusion']) ?? '';
      const content = [`${workflow}: ${conclusion}`, ...failingStepLines(run)].join('\n');
      const headSha = fieldStr(run, ['headSha', 'head_sha']);
      const event = ruleEvent(
        rule,
        `${rule.idPrefix}${databaseId}`,
        fieldStr(run, ['event']),
        content,
        fieldStr(run, ['createdAt', 'created_at']) ?? '',
        null,
        headSha === null ? [] : [`commit:${headSha}`],
      );
      const existing = byId.get(event.id);
      if (contentLength(event) > (existing ? contentLength(existing) : 0)) byId.set(event.id, event);
    }
  }
  return [...byId.keys()].sort().map((id) => byId.get(id));
}
