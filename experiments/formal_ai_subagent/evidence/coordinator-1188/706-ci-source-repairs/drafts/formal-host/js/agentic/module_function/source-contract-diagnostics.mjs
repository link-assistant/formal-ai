import { readText } from "../host.mjs";
import { parseLinoRoot, findChildValue } from "../write_lino.mjs";
/** Seed text reports an unmet condition; it never grants effect authority. */
export function sourceContractDiagnostic(code, language = 'en') {
  const root = parseLinoRoot(readText('data/seed/meanings-source-authoring-grammar.lino'));
  const meanings = root.children.filter(node => node.name === 'meanings');
  if (meanings.length !== 1) throw Error('MissingSourceDiagnostic');
  const records = meanings[0].children.filter(node => node.name === code && findChildValue(node, 'role') === 'source-contract-diagnostic');
  if (records.length !== 1) throw Error('MissingSourceDiagnostic');
  const lexemes = records[0].children.filter(node => node.name === 'lexeme' && node.id === language);
  if (lexemes.length !== 1) throw Error('MissingSourceDiagnosticLanguage');
  const surfaces = lexemes[0].children.filter(node => node.name === 'surface');
  if (surfaces.length !== 1) throw Error('AmbiguousSourceDiagnostic');
  const text = findChildValue(surfaces[0], 'text');
  if (text === '') throw Error('MissingSourceDiagnosticText');
  return text;
}
