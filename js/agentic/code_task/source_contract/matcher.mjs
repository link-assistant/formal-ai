import { sourceDeclarationSlots } from './declaration.mjs';
import { sourceWhitespaceSupported } from '../source_contract.mjs';
import { childrenNamed, parseLino, readText } from '../../host.mjs';
export function matchSourceDescription(task, artifact) {
  const root = sourceGrammarRoot(parseLino(readText('data/seed/source-authoring-grammar.lino') ?? ''));
  if (root === null) return null;
  const forms = childrenNamed(root, 'form').map(form => Object.fromEntries(['language', 'kind', 'pattern'].map(name => [name, childrenNamed(form, name)[0]?.id ?? childrenNamed(root, name)[0]?.id])));
  if (!sourceWhitespaceSupported(task)) return null;
  const declaration = sourceDeclarationSlots(artifact.content);
  if (declaration === null) return null;
  const { kind } = declaration;
  const slots = { path: artifact.path, ...declaration.slots };
  const escaped = text => text.replace(/[.*+?^\x24{}()|[\]\\]/g, '\\$&');
  for (const form of forms) {
    if (form.kind !== kind || typeof form.pattern !== 'string' || typeof form.language !== 'string') continue;
    const seen = [];
    let missing = false;
    const pattern = form.pattern.replace(/\{([a-z]+(?:-[a-z]+)*)\}/gu, (_, role) => {
      if (!Object.hasOwn(slots, role) || seen.includes(role)) {
        missing = true;
        return '(?!)';
      }
      seen.push(role);
      return '(' + escaped(slots[role]) + ')';
    });
    if (missing || seen.length !== Object.keys(slots).length) continue;
    let expression;
    try {
      expression = new RegExp(pattern, 'diu');
    } catch {
      continue;
    }
    const match = expression.exec(task);
    if (match === null || match[0] !== task || seen.some((role, index) => match[index + 1] !== slots[role])) continue;
    return {
      unit: 'utf16',
      full: match.indices[0],
      language: form.language,
      kind,
      captures: seen.map((role, index) => ({
        role,
        span: match.indices[index + 1],
        text: match[index + 1]
      })),
      output: {
        ...slots,
        content: artifact.content
      },
      wholeRequestConsumed: true,
      compilation: 'pending',
      unknownEffects: ['module_initialization', 'tool_execution'],
      whitespaceProfile: 'ASCII'
    };
  }
  return null;
}

/** Select exactly one named grammar container; unrelated document order is inert. */
export function sourceGrammarRoot(document) {
  const roots = document?.name === 'source-authoring-grammar'
    ? [document] : childrenNamed(document, 'source-authoring-grammar');
  return roots.length === 1 ? roots[0] : null;
}
