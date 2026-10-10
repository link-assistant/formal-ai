import { rustIdentifierIsValid } from '../identifier_domain.mjs';
import { sourceWhitespaceSupported } from '../source_contract.mjs';
import { childrenNamed, parseLino, readText } from '../../host.mjs';
export function matchSourceDescription(task, artifact) {
  const root = parseLino(readText('data/seed/source-authoring-grammar.lino') ?? '');
  const forms = childrenNamed(root, 'form').map(form => Object.fromEntries(['language', 'kind', 'pattern'].map(name => [name, childrenNamed(form, name)[0]?.id])));
  if (!sourceWhitespaceSupported(task)) return null;
  const integer = /^(pub )?fn ([A-Za-z_][A-Za-z_0-9]*)\(\) -> i64 \{\n    (-?\d+)\n\}\n$/.exec(artifact.content);
  const division = /^(pub )?fn ([A-Za-z_][A-Za-z_0-9]*)\(([A-Za-z_][A-Za-z_0-9]*): (f64)\) -> (f64) \{\n    \3 \/ (-?[0-9]+\.[0-9]+)\n\}\n$/.exec(artifact.content);
  if (integer === null && division === null) return null;
  const declaration = integer ?? division;
  if (declaration[1] === undefined || !rustIdentifierIsValid(declaration[2])) return null;
  const value = integer === null ? division[6] : integer[3];
  if (integer !== null) {
    const numeric = BigInt(value);
    if (numeric < -(1n << 63n) || numeric >= 1n << 63n) return null;
  } else if (!rustIdentifierIsValid(division[3]) || !Number.isFinite(Number(value)) || Number(value) === 0) return null;
  const kind = integer === null ? 'float-division' : 'integer';
  const slots = {
    path: artifact.path,
    identifier: declaration[2],
    value,
    ...(integer === null ? { 'parameter-type': division[4] } : {})
  };
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
