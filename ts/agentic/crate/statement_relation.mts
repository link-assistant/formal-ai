// Issue #1487 — checked equivalence and entailment between two formal
// statements, kept apart from retrieval. Shared terms (a similarity score)
// may propose that two statements are related; only this check may certify
// that one says what another says, and it answers with a verdict and the
// reason that justifies it:
//
// * equivalent  the same formal identity (a repeated mention), or the same
//               ordered relations over the same operands with the same shape,
//               polarity and repetition (`1 < 2` and `2 is greater than 1`);
// * entailed    the later statement drops conjuncts of an asserted,
//               purely conjunctive earlier one (`1972` from `1969 and 1972`);
// * distinct    something the identity keeps differs: operand order,
//               a connective, a quantifier, negation scope, an explicit
//               count, or a term the earlier statement does not hold;
// * unresolved  the languages differ or a statement holds structure the
//               reader does not support, so nothing is certified.
//
// The Rust twin is rust/src/formalization/statement_relation.rs.

import { typedMeaningEntails } from './typed_meaning.mjs';
import { COMPARISON_RULE, conjunctiveShape, relationOf } from './proposition_structure.mjs';
import { Polarity, contentIds, statementIdentity, statementStructure } from './text_formalization.mjs';

/** Mirrors `enum RelationVerdict` in rust/src/formalization/statement_relation.rs. */
export const RelationVerdict = Object.freeze({
  Equivalent: 'equivalent',
  Entailed: 'entailed',
  Distinct: 'distinct',
  Unresolved: 'unresolved',
});

/** Mirrors `enum RelationReason` in rust/src/formalization/statement_relation.rs. */
export const RelationReason = Object.freeze({
  Language: 'different-language',
  TypedMeaning: 'typed-meaning-not-entailed',
  Identity: 'same-identity',
  UnsupportedStructure: 'unsupported-structure',
  SameRelations: 'same-relations',
  NotAsserted: 'not-asserted',
  NonConjunctive: 'non-conjunctive-shape',
  Repetition: 'different-repetition',
  Relations: 'different-relations',
  ConjunctionElimination: 'conjunction-elimination',
  Terms: 'terms-not-held',
});

const same = (left, right) => left.length === right.length && left.every((value, index) => value === right[index]);
const within = (inner, outer) => inner.every((value) => outer.includes(value));

/** Mirrors `fn exhausted_by_relations` in rust/src/formalization/statement_relation.rs: every term is an operand or a comparison word. */
export function exhaustedByRelations(statement, structure) {
  return structure.relations.length > 0 && contentIds(statement)
    .every((id) => structure.operands.includes(id) || relationOf(COMPARISON_RULE, id) !== null);
}

/**
 * Mirrors `fn checked_relation` in rust/src/formalization/statement_relation.rs: what `later` is to `earlier`, as
 * `{verdict, reason}`.
 */
export function checkedRelation(later, earlier) {
  const answer = (verdict, reason) => ({ verdict, reason });
  if (later.language !== earlier.language) return answer(RelationVerdict.Unresolved, RelationReason.Language);
  if (!typedMeaningEntails(earlier.dialogue, later.dialogue)) return answer(RelationVerdict.Distinct, RelationReason.TypedMeaning);
  if (statementIdentity(later) === statementIdentity(earlier)) return answer(RelationVerdict.Equivalent, RelationReason.Identity);
  const left = statementStructure(later);
  const right = statementStructure(earlier);
  if (left.unresolved.length > 0 || right.unresolved.length > 0) return answer(RelationVerdict.Unresolved, RelationReason.UnsupportedStructure);
  if (exhaustedByRelations(later, left) && exhaustedByRelations(earlier, right) && same(left.relations, right.relations)
    && same(left.repetition, right.repetition) && left.shape === right.shape && later.polarity === earlier.polarity) {
    return answer(RelationVerdict.Equivalent, RelationReason.SameRelations);
  }
  if (later.polarity !== Polarity.Asserted || earlier.polarity !== Polarity.Asserted) return answer(RelationVerdict.Distinct, RelationReason.NotAsserted);
  if (!conjunctiveShape(left) || !conjunctiveShape(right)) return answer(RelationVerdict.Distinct, RelationReason.NonConjunctive);
  if (!same(left.repetition, right.repetition)) return answer(RelationVerdict.Distinct, RelationReason.Repetition);
  if (!within(left.relations, right.relations)) return answer(RelationVerdict.Distinct, RelationReason.Relations);
  const laterIdentifiers = contentIds(later);
  if (laterIdentifiers.length > 0 && within(laterIdentifiers, contentIds(earlier))) return answer(RelationVerdict.Entailed, RelationReason.ConjunctionElimination);
  return answer(RelationVerdict.Distinct, RelationReason.Terms);
}

/** Mirrors `fn certifies_restatement` in rust/src/formalization/statement_relation.rs: the verdict lets `later` be dropped as said. */
export function certifiesRestatement(relation) {
  return relation.verdict === RelationVerdict.Equivalent || relation.verdict === RelationVerdict.Entailed;
}
