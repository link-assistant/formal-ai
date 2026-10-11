// Agentic route for evidence-weighted repository statement audits: the
// JavaScript twin of rust/src/agentic_coding/statement_audit.rs.

import { childrenNamed, parseLino, readText } from './host.mjs';
import { sentences } from './shell_command_policy.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { safeRelativePath } from './write_request.mjs';
import { toAsciiLowercase } from './crate/rust_str.mjs';

/** Mirrors `const STATEMENT_AUDIT_PATH`. */
export const STATEMENT_AUDIT_PATH = 'statement-audit.lino';

/** Mirrors `const STATEMENT_AUDIT_COMMAND`. */
export const STATEMENT_AUDIT_COMMAND = ['formal-ai', 'statement-audit', '--root', '.', '--output', STATEMENT_AUDIT_PATH].join(' ');

/** Mirrors `const STATEMENT_AUDIT_WITH_EVIDENCE_COMMAND`. */
export const STATEMENT_AUDIT_WITH_EVIDENCE_COMMAND = [
  'formal-ai', 'statement-audit', '--root', '.', '--evidence', 'evidence.json', '--output', STATEMENT_AUDIT_PATH,
].join(' ');

/** Mirrors `fn command_for` in rust/src/agentic_coding/statement_audit.rs. */
export function commandFor(prompt) {
  const frame = auditRoleFrame(prompt);
  if (frame?.explicitOutput) return frame.complete ? (registeredAuditCommand(frame.output) ?? '') : '';
  return toAsciiLowercase(prompt).includes('evidence.json') ? STATEMENT_AUDIT_WITH_EVIDENCE_COMMAND : STATEMENT_AUDIT_COMMAND;
}

/** Mirrors `fn is_statement_audit_task` in rust/src/agentic_coding/statement_audit.rs. */
export function isStatementAuditTask(prompt) {
  const frame = auditRoleFrame(prompt);
  if (frame?.explicitOutput) return frame.complete && registeredAuditCommand(frame.output) !== null;
  const lower = prompt.toLowerCase();
  if (lower.includes(STATEMENT_AUDIT_PATH) || lower.includes('statement audit')) return true;
  const repositoryScope = lower.includes('repository') || lower.includes('repo');
  const statementScope = lower.includes('statements') || lower.includes('requirements');
  const assessment = ['audit', 'probability', 'probabilities', 'weigh'].some((word) => lower.includes(word));
  return repositoryScope && statementScope && assessment;
}

/** A complete source-owned CLI action; every unquoted clause needs one role. */
export function ownedStatementAuditOutput(prompt) {
  const frame = auditRoleFrame(prompt);
  return frame?.complete && frame.output !== null && safeRelativePath(frame.output)
    ? frame.output : null;
}

function auditRoleFrame(prompt) {
  if (quotedSegmentSpans(prompt).length !== 0) return null;
  const document = parseLino(readText('data/seed/statement-audit-registry.lino') ?? '');
  const roots = document.name === 'statement_audit_registry' ? [document]
    : childrenNamed(document, 'statement_audit_registry');
  if (roots.length !== 1) return null;
  const contracts = childrenNamed(roots[0], 'request-contract');
  if (contracts.length !== 1 || contracts[0].id !== 'repository-audit') return null;
  const roles = childrenNamed(contracts[0], 'role');
  const clauses = sentences(prompt);
  if (roles.length === 0) return null;
  let complete = clauses.length === roles.length;
  const seen = new Set();
  let output = null;
  for (const clause of clauses) {
    const matches = roles.flatMap(role => {
      const patterns = childrenNamed(role, 'pattern');
      if (patterns.length !== 1) return [];
      const match = new RegExp(patterns[0].id, 'iu').exec(clause.text);
      return match !== null && match[0] === clause.text ? [{role, match}] : [];
    });
    if (matches.length !== 1 || seen.has(matches[0].role.id)) {
      complete = false;
      continue;
    }
    const {role, match} = matches[0];
    seen.add(role.id);
    if (role.id === 'deliver') output = match[1] ?? null;
  }
  return {
    output,
    complete: complete && seen.size === roles.length && output !== null,
    explicitOutput: seen.has('audit') && (seen.has('deliver') || (seen.has('weigh') && seen.has('persist'))),
  };
}

function registeredAuditCommand(output) {
  if (!safeRelativePath(output)) return null;
  const document = parseLino(readText('data/seed/statement-audit-registry.lino') ?? '');
  const roots = document.name === 'statement_audit_registry' ? [document] : childrenNamed(document, 'statement_audit_registry');
  if (roots.length !== 1) return null;
  const contracts = childrenNamed(roots[0], 'request-contract');
  if (contracts.length !== 1 || contracts[0].id !== 'repository-audit') return null;
  const templates = childrenNamed(contracts[0], 'command-template');
  if (templates.length !== 1 || templates[0].children.length !== 0 || templates[0].id.split('{output}').length !== 2) return null;
  return templates[0].id.replace('{output}', output);
}
