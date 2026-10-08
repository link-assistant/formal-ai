// The issue-title convention of rust/src/issue_report.rs (`ReportTurn`,
// `TitleSettings`, `issue_title`, `truncate_words`), kept apart from
// ./issue_report.mjs (the fenced-block helper another port owns).

import { agentInfoValue } from './seed_agent_info.mjs';

/** Mirrors `TITLE_MAX_LENGTH`. */
export const TITLE_MAX_LENGTH = 120;
const TITLE_JOIN = '` + `';

const charCount = (text) => Array.from(text).length;
const splitWhitespace = (text) => text.split(/\p{White_Space}+/u).filter(Boolean);
const WHITESPACE = /^\p{White_Space}$/u;

/**
 * Mirrors `ReportTurn::new` (with `report_invoking` settable).
 * @param {string} role
 * @param {string} content
 */
export function reportTurn(role, content, reportInvoking = false) {
  return { role, content, intent: '', reported: false, report_invoking: reportInvoking };
}

/** Mirrors `TitleSettings::from_seed`. */
export function titleSettingsFromSeed() {
  return {
    prefix: agentInfoValue('issue_report_title_prefix') ?? '',
    default_title: agentInfoValue('issue_report_default_title') ?? '',
  };
}

/** Mirrors `fn issue_title` in rust/src/issue_report.rs. */
export function issueTitle(turns, settings) {
  const subjects = titleSubjects(turns);
  if (!subjects.length) return settings.default_title;
  const first = subjects[0];
  const last = subjects[subjects.length - 1];
  if (last !== first) {
    const combined = `${settings.prefix}\`${first}${TITLE_JOIN}${last}\``;
    if (charCount(combined) <= TITLE_MAX_LENGTH) return combined;
  }
  const budget = Math.max(0, TITLE_MAX_LENGTH - (charCount(settings.prefix) + 2));
  return `${settings.prefix}\`${truncateWords(first, budget)}\``;
}

/** Mirrors `fn title_subjects`. */
function titleSubjects(turns) {
  const subjects = turns
    .filter((turn) => turn.role.toLowerCase() === 'user')
    .map((turn) => [splitWhitespace(turn.content).join(' '), turn.report_invoking])
    .filter(([text]) => text !== '');
  while (subjects.length > 1 && subjects[subjects.length - 1][1]) subjects.pop();
  const out = [];
  for (const [text] of subjects) if (out[out.length - 1] !== text) out.push(text);
  return out;
}

/** Mirrors `fn truncate_words` in rust/src/issue_report.rs. */
export function truncateWords(text, max) {
  const trimmed = text.replace(/^\p{White_Space}+|\p{White_Space}+$/gu, '');
  const chars = Array.from(trimmed);
  if (chars.length <= max) return trimmed;
  const head = chars.slice(0, Math.max(0, max - 1));
  let boundary = -1;
  for (let index = head.length - 1; index >= 0; index -= 1) {
    if (WHITESPACE.test(head[index])) {
      boundary = index;
      break;
    }
  }
  const cut = boundary >= 0 && boundary >= Math.floor(max / 2) ? head.slice(0, boundary).join('') : head.join('');
  return `${cut.replace(/\p{White_Space}+$/u, '')}…`;
}
