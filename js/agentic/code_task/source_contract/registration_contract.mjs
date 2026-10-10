import { childrenNamed, parseLino, readText } from '../../host.mjs';
import { roleWordForms } from '../../write_lexicon.mjs';
import { safeRelativePath } from '../../write_request.mjs';
import { rustIdentifierIsValid } from '../identifier_domain.mjs';
import { sourceDescriptionContract } from '../source_contract.mjs';

/** Mirrors source_registration_contract: owned prefix evidence retains every unconsumed Need. */
export function sourceRegistrationContract(task, sourceFor) {
  const forms = roleWordForms('coding_module_registration_action').filter(form => form.slot === 'bare').map(form => form.text);
  if (forms.length === 0) return null;
  const escaped = text => text.replace(/[.*+?^\x24{}()|[\]\\]/g, '\\$&');
  const slots = {
    'source-clause': '.+?',
    'registration-action': '(?:' + forms.map(escaped).join('|') + ')',
    'registration-path': '[^ \\t\\n\\r\\v\\f,;:]+\\.rs'
  };
  const root = parseLino(readText('data/seed/source-authoring-grammar.lino') ?? '');
  for (const form of childrenNamed(root, 'composition')) {
    const template = childrenNamed(form, 'pattern')[0]?.id;
    if (typeof template !== 'string' || !template.startsWith('^') || !template.endsWith('$')) continue;
    const seen = [];
    let missing = false;
    const pattern = template.slice(0, -1).replace(/\{([a-z]+(?:-[a-z]+)*)\}/gu, (_, role) => {
      if (!Object.hasOwn(slots, role) || seen.includes(role)) { missing = true; return '(?!)'; }
      seen.push(role);
      return '(' + slots[role] + ')';
    });
    if (missing || seen.length !== Object.keys(slots).length) continue;
    let matched;
    try { matched = new RegExp(pattern, 'diu').exec(task); } catch { continue; }
    if (matched === null) continue;
    const captures = Object.fromEntries(seen.map((role, index) => [role, {text: matched[index + 1], span: matched.indices[index + 1]}]));
    const declaration = captures['source-clause'];
    const artifact = sourceFor(declaration.text);
    if (artifact === null || !artifact.path.endsWith('.rs')) continue;
    const module = artifact.path.split('/').at(-1).slice(0, -3);
    if (!rustIdentifierIsValid(module)) continue;
    const source = sourceDescriptionContract(declaration.text, artifact);
    const registration = captures['registration-path'];
    if (source === null || declaration.span[0] !== 0 || !registration.text.endsWith('.rs')
      || !safeRelativePath(registration.text) || registration.text === artifact.path) continue;
    const end = matched.indices[0][1];
    return {unit:'utf16', full:[0,end], wholeRequestConsumed:end === task.length, declaration:source,
      registration:{action:captures['registration-action'], target:registration, module,
        span:[declaration.span[1],end]}, remainingSpan:[end,task.length],
      source:task, compilation:'pending', unknownEffects:['module_initialization','tool_execution'],
      condition:'both-source-owned-operations-and-entire-request-before-effects', whitespaceProfile:'ASCII'};
  }
  return null;
}
