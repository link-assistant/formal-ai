// A cardinal summary describes a collection; it is not an artifact obligation.
// Only existing seeded number and file-noun surfaces can own the entire clause.
import { meaningsWithRole } from '../crate/seed_meanings.mjs';
import { safeRelativePath } from '../write_request.mjs';

export function collectionSummaryOwns(goals) {
  if (!Array.isArray(goals) || goals.length < 2 || goals[0].kind !== 'unsupported') return false;
  const remainder = goals.slice(1);
  if (remainder.some(goal => goal.kind !== 'literal_file')) return false;
  const targets = remainder.map(goal => {
    if (typeof goal.target !== 'string' || !safeRelativePath(goal.target)) return null;
    return goal.target.split('/').filter(part => part !== '.').join('/').normalize('NFC').toLowerCase();
  });
  if (targets.some(target => target === null || target === '')
    || new Set(targets).size !== targets.length) return false;
  const clause = goals[0].clause.replace(/[\s.!?。！？।;；]+$/u, '').trim().normalize('NFC').toLowerCase();
  const nounMeanings = meaningsWithRole('file-read-object-noun');
  for (const cardinal of meaningsWithRole('cardinal_number_word')) {
    const values = cardinal.lexemes.flatMap(lexeme => lexeme.words)
      .map(word => word.text).filter(text => /^[0-9]{1,9}$/u.test(text)).map(Number);
    if (new Set(values).size !== 1 || values[0] !== targets.length) continue;
    for (const lexeme of cardinal.lexemes) {
      const nouns = nounMeanings.flatMap(meaning => meaning.lexemes)
        .filter(noun => noun.language === lexeme.language).flatMap(noun => noun.words);
      for (const word of lexeme.words) for (const noun of nouns) {
        const number = word.text.normalize('NFC').toLowerCase();
        const object = noun.text.normalize('NFC').toLowerCase();
        if (clause === number + ' ' + object
          || lexeme.language === 'zh' && clause === number + object) return true;
      }
    }
  }
  return false;
}
