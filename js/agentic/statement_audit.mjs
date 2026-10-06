// Agentic route for evidence-weighted repository statement audits: the
// JavaScript twin of rust/src/agentic_coding/statement_audit.rs.

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
  return toAsciiLowercase(prompt).includes('evidence.json') ? STATEMENT_AUDIT_WITH_EVIDENCE_COMMAND : STATEMENT_AUDIT_COMMAND;
}

/** Mirrors `fn is_statement_audit_task` in rust/src/agentic_coding/statement_audit.rs. */
export function isStatementAuditTask(prompt) {
  const lower = prompt.toLowerCase();
  if (lower.includes(STATEMENT_AUDIT_PATH) || lower.includes('statement audit')) return true;
  const repositoryScope = lower.includes('repository') || lower.includes('repo');
  const statementScope = lower.includes('statements') || lower.includes('requirements');
  const assessment = ['audit', 'probability', 'probabilities', 'weigh'].some((word) => lower.includes(word));
  return repositoryScope && statementScope && assessment;
}
