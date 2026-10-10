import { rustIdentifierIsValid } from './identifier_domain.mjs';
import { cached, childrenNamed, parseLino, readText } from '../host.mjs';
import { roleWordForms } from '../write_lexicon.mjs';
import { firstRoleMatch, words } from '../crate/seed_meanings.mjs';
/** Mirrors source_description_contract: conditional full-request consumption, not purity or compiler certification. */
/** Mirrors source_whitespace_supported: an explicit consumed-grammar profile. */
export function sourceWhitespaceSupported(text) {
  if (typeof text !== 'string') return false;
  for (const character of text) {
    if (/[\p{White_Space}\uFEFF]/u.test(character) && !/[ \u0009-\u000D]/u.test(character)) return false;
  }
  return true;
}

export function sourceDescriptionContract(task, artifact) {
  if (!sourceWhitespaceSupported(task)) return null;
  const declaration = /^(pub )?fn ([A-Za-z_][A-Za-z_0-9]*)\(\) -> i64 \{\n    (-?\d+)\n\}\n$/.exec(artifact.content);
  if (declaration === null || !rustIdentifierIsValid(declaration[2])) return null;
  const integer = BigInt(declaration[3]);
  if (integer < -(1n << 63n) || integer >= (1n << 63n)) return null;
  const escape = (text) => text.replace(/[.*+?^\x24{}()|[\]\\]/g, '\\$&');
  const surfaces = (role, slot) => roleWordForms(role).filter((form) => form.slot === slot)
    .map((form) => (slot === 'prefix' ? form.before : form.text).trim()).filter(Boolean);
  const alternatives = (values) => values.length === 0 ? null : '(?:' + [...values].sort((left, right) => right.length - left.length).map(escape).join('|') + ')';
  const role = (name, slot) => alternatives(surfaces(name, slot));
  const meaning = (name, concept) => {
    const selected = firstRoleMatch(name, concept);
    return selected === null ? null : alternatives(words(selected).filter((surface) => !surface.includes('…')));
  };
  const roles = [role('coding_request_verb', 'bare'), role('file_declared_noun', 'bare'),
    role('file_write_content_lead', 'prefix'), alternatives(cached('source-contract-articles', () => childrenNamed(parseLino(readText('data/seed/intent-routing.lino')), 'article').map(node => node.id.trim()))),
    role('coding_visibility', 'bare'), meaning('program_language_alias', 'rust'),
    meaning('program_kind', 'function'), role('coding_name_slot', 'prefix'), role('coding_return_action', 'bare')];
  if (roles.some((value) => value === null)) return null;
  const visibility = declaration[1] === undefined ? '()' : '(' + roles[4] + ')\\s+';
  const pattern = '^\\s*(' + roles[0] + ')\\s+(?:(' + roles[1] + ')\\s+)?(' + escape(artifact.path) + ')\\s+(' + roles[2] + ')\\s+'
    + '(?:(' + roles[3] + ')\\s+)?' + visibility + '(' + roles[5] + ')\\s+(' + roles[6] + ')\\s+(' + roles[7] + ')\\s+('
    + escape(declaration[2]) + ')\\s+(' + roles[8] + ')\\s+(' + escape(declaration[3]) + ')[\\s.!?。！？।]*$';
  const matched = new RegExp(pattern, 'diu').exec(task);
  if (matched === null || matched[3] !== artifact.path || matched[10] !== declaration[2] || matched[12] !== declaration[3]) return null;
  const labels = ['coding_request_verb', 'file_declared_noun', 'path', 'file_write_content_lead',
    'article', 'coding_visibility', 'program_language_alias', 'program_kind', 'coding_name_slot', 'identifier', 'coding_return_action', 'value'];
  return { unit: 'utf16', full: matched.indices[0], source: task,
    captures: labels.map((role, index) => matched.indices[index + 1] === undefined ? null : { role, span: matched.indices[index + 1], text: matched[index + 1] }).filter((capture) => capture !== null),
    output: { path: artifact.path, identifier: declaration[2], value: declaration[3], content: artifact.content },
    unknownEffects: ['module_initialization', 'tool_execution'], compilation: 'pending', wholeRequestConsumed: true, whitespaceProfile: 'ASCII'  };
}
