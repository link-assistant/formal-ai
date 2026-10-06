// `crate::seed::caller_context_vocabulary` (rust/src/seed/caller_context.rs),
// read from data/seed/caller-context.lino.

import { cached, readText } from '../host.mjs';
import { parseRoot } from './seed_parser.mjs';
import { isAlphanumeric, trimMatches, trimStart } from './rust_str.mjs';

const collectLanguageValues = (group, name) => (group.children || [])
  .filter((child) => child.name === 'language')
  .flatMap((language) => (language.children || []).filter((child) => child.name === name)
    .map((child) => child.value.toLowerCase()));

/** Mirrors `fn caller_context_vocabulary`. */
export function callerContextVocabulary() {
  return cached('caller-context-vocabulary', () => {
    const vocab = {
      injected_blocks: [], fact_statement_copulas: [], question_words: [], subject_leads: [], request_verbs: [],
      policy_leads: [],
    };
    const root = parseRoot(readText('data/seed/caller-context.lino')).children[0];
    if (!root) return vocab;
    for (const group of root.children || []) {
      if (group.name === 'injected_blocks') {
        vocab.injected_blocks = (group.children || []).filter((child) => child.name === 'block').map((block) => ({
          tag: block.value,
          clients: (block.children || []).filter((child) => child.name === 'client').map((child) => child.value),
        }));
      } else if (group.name === 'fact_statement_copulas') {
        vocab.fact_statement_copulas = collectLanguageValues(group, 'copula');
      } else if (group.name === 'question_words') {
        vocab.question_words = collectLanguageValues(group, 'word');
      } else if (group.name === 'subject_leads') {
        vocab.subject_leads = collectLanguageValues(group, 'lead');
      } else if (group.name === 'request_verbs') {
        vocab.request_verbs = collectLanguageValues(group, 'verb');
      } else if (group.name === 'policy_leads') {
        vocab.policy_leads = collectLanguageValues(group, 'lead');
      }
    }
    return vocab;
  });
}

/** Mirrors `const fn is_unspaced_script`. */
function isUnspacedScript(character) {
  const cp = character.codePointAt(0);
  return (cp >= 0x3400 && cp <= 0x9fff) || (cp >= 0xf900 && cp <= 0xfaff);
}

function startsWithWordCharacter(text) {
  const first = Array.from(text)[0];
  return first !== undefined && (isAlphanumeric(first) || first === '_');
}

/** Mirrors `CallerContextVocabulary::copula_in`. */
export function copulaIn(token) {
  const trimmed = trimMatches(token, (character) => !isAlphanumeric(character) && !'—–-'.includes(character));
  return callerContextVocabulary().fact_statement_copulas.find((copula) => copula === trimmed
    || (Array.from(copula).some(isUnspacedScript) && trimmed.includes(copula))) ?? null;
}

/** Mirrors `CallerContextVocabulary::asks_a_question`. */
export function asksAQuestion(sentence) {
  const vocab = callerContextVocabulary();
  return [...vocab.question_words, ...vocab.request_verbs].some((word) => {
    if (Array.from(word).some(isUnspacedScript)) return sentence.includes(word);
    return sentence.split(/[^\p{Alphabetic}\p{N}'’]/u).some((token) => token === word);
  });
}

/** Mirrors `CallerContextVocabulary::policy_lead_clause`. */
export function policyLeadClause(lowercased) {
  const text = trimStart(lowercased);
  for (const lead of callerContextVocabulary().policy_leads) {
    if (!text.startsWith(lead)) continue;
    const rest = text.slice(lead.length);
    if (rest === '' || !startsWithWordCharacter(rest)) return trimStart(rest);
  }
  return null;
}

/** Mirrors `CallerContextVocabulary::opens_with_policy_lead`. */
export function opensWithPolicyLead(lowercased) {
  return policyLeadClause(lowercased) !== null;
}

/** Mirrors `CallerContextVocabulary::carries_policy_lead`. */
export function carriesPolicyLead(lowercased) {
  return opensWithPolicyLead(lowercased)
    || callerContextVocabulary().policy_leads.some((lead) => lowercased.includes(` ${lead} `));
}
