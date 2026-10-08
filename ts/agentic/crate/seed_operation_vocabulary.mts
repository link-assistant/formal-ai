// `crate::seed::operation_vocabulary` (rust/src/seed/operation_vocabulary.rs),
// read from data/seed/operation-vocabulary.lino.

import { cached, readText } from '../host.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';

const splitCombo = (raw) => raw.split('+').map((part) => part.trim()).filter((part) => part !== '');

/** Mirrors `fn operation_vocabulary`: `[{canonical, languages: Map, inverse_of, exclusions}]`. */
export function operationVocabulary() {
  return cached('operation-vocabulary', () => {
    const operations = [];
    const root = parseRoot(readText('data/seed/operation-vocabulary.lino')).children[0];
    if (!root) return operations;
    for (const operationNode of (root.children || []).filter((child) => child.name === 'operation')) {
      const languages = new Map();
      const exclusions = [];
      for (const languageNode of (operationNode.children || []).filter((child) => child.name === 'language')) {
        const forms = { phrases: [], combos: [] };
        for (const entry of languageNode.children || []) {
          if (entry.name === 'phrase') forms.phrases.push(entry.value);
          else if (entry.name === 'combo') forms.combos.push(splitCombo(entry.value));
          else if (entry.name === 'exclude') exclusions.push(entry.value);
        }
        languages.set(languageNode.value, forms);
      }
      const inverse = findChildValue(operationNode, 'inverse');
      operations.push({ canonical: operationNode.value, languages, inverse_of: inverse === '' ? null : inverse, exclusions });
    }
    return operations;
  });
}

const formsMatch = (forms, normalized) => forms.phrases.some((phrase) => normalized.includes(phrase))
  || forms.combos.some((combo) => combo.length > 0 && combo.every((token) => normalized.includes(token)));

/** Mirrors `OperationTrigger::matches`. */
export function operationMatches(operation, normalized) {
  return !operation.exclusions.some((phrase) => normalized.includes(phrase))
    && [...operation.languages.values()].some((forms) => formsMatch(forms, normalized));
}

/** Mirrors `OperationVocabulary::matches`. */
export function vocabularyMatches(canonical, normalized) {
  return operationVocabulary().some((operation) => operation.canonical === canonical && operationMatches(operation, normalized));
}

/** Mirrors `OperationVocabulary::detect`. */
export function detectOperations(normalized) {
  return operationVocabulary().filter((operation) => operationMatches(operation, normalized)).map((operation) => operation.canonical);
}
