//! Checked equivalence and entailment between two formal statements, kept
//! apart from retrieval (issue #1487).
//!
//! Shared terms (a similarity score) may propose that two statements are
//! related; only this check may certify that one says what another says, and
//! it answers with a verdict and the reason that justifies it:
//!
//! * [`RelationVerdict::Equivalent`] — the same formal identity (a repeated
//!   mention), or the same ordered relations over the same operands with the
//!   same shape, polarity and repetition (`1 < 2` and `2 is greater than 1`);
//! * [`RelationVerdict::Entailed`] — the later statement drops conjuncts of an
//!   asserted, purely conjunctive earlier one (`1972` from `1969 and 1972`);
//! * [`RelationVerdict::Distinct`] — something the identity keeps differs:
//!   operand order, a connective, a quantifier, negation scope, an explicit
//!   count, or a term the earlier statement does not hold;
//! * [`RelationVerdict::Unresolved`] — the languages differ or a statement
//!   holds structure the reader does not support, so nothing is certified.
//!
//! The browser twin is `js/agentic/crate/statement_relation.mjs`.

use super::proposition_structure::{
    COMPARISON_RULE, PropositionStructure, conjunctive_shape, relation_of,
};
use super::text_statements::{
    Polarity, Statement, content_ids, statement_identity, statement_structure,
};
use super::typed_meaning::typed_meaning_entails;

/// What a later statement is to an earlier one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationVerdict {
    /// It says the same.
    Equivalent,
    /// The earlier statement entails it.
    Entailed,
    /// It says something else.
    Distinct,
    /// Nothing is certified.
    Unresolved,
}

impl RelationVerdict {
    /// The verdict as machine-readable output writes it.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Equivalent => "equivalent",
            Self::Entailed => "entailed",
            Self::Distinct => "distinct",
            Self::Unresolved => "unresolved",
        }
    }
}

/// Why the check reached its verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationReason {
    /// The statements are in different languages.
    Language,
    /// The typed meaning of the earlier statement does not entail the later.
    TypedMeaning,
    /// The formal identities are equal.
    Identity,
    /// A statement holds structure the reader does not support.
    UnsupportedStructure,
    /// The same ordered relations exhaust both statements.
    SameRelations,
    /// A statement is not asserted.
    NotAsserted,
    /// A statement has a disjunction, quantifier or scoped negation.
    NonConjunctive,
    /// The explicit counts of repetition differ.
    Repetition,
    /// A relation of the later statement is not held by the earlier.
    Relations,
    /// The later statement drops conjuncts of the earlier one.
    ConjunctionElimination,
    /// A term of the later statement is not held by the earlier.
    Terms,
}

impl RelationReason {
    /// The reason as machine-readable output writes it.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Language => "different-language",
            Self::TypedMeaning => "typed-meaning-not-entailed",
            Self::Identity => "same-identity",
            Self::UnsupportedStructure => "unsupported-structure",
            Self::SameRelations => "same-relations",
            Self::NotAsserted => "not-asserted",
            Self::NonConjunctive => "non-conjunctive-shape",
            Self::Repetition => "different-repetition",
            Self::Relations => "different-relations",
            Self::ConjunctionElimination => "conjunction-elimination",
            Self::Terms => "terms-not-held",
        }
    }
}

/// A checked relation: the verdict and the reason that justifies it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckedRelation {
    /// What the later statement is to the earlier.
    pub verdict: RelationVerdict,
    /// Why.
    pub reason: RelationReason,
}

/// Whether every term of `statement` is an operand of a relation or a
/// comparison word.
#[must_use]
pub fn exhausted_by_relations(statement: &Statement, structure: &PropositionStructure) -> bool {
    !structure.relations.is_empty()
        && content_ids(statement)
            .iter()
            .all(|id| structure.operands.contains(id) || relation_of(COMPARISON_RULE, id).is_some())
}

/// What `later` is to `earlier`.
#[must_use]
pub fn checked_relation(later: &Statement, earlier: &Statement) -> CheckedRelation {
    let answer = |verdict, reason| CheckedRelation { verdict, reason };
    if later.language != earlier.language {
        return answer(RelationVerdict::Unresolved, RelationReason::Language);
    }
    if !typed_meaning_entails(&earlier.dialogue, &later.dialogue) {
        return answer(RelationVerdict::Distinct, RelationReason::TypedMeaning);
    }
    if statement_identity(later) == statement_identity(earlier) {
        return answer(RelationVerdict::Equivalent, RelationReason::Identity);
    }
    let left = statement_structure(later);
    let right = statement_structure(earlier);
    if !left.unresolved.is_empty() || !right.unresolved.is_empty() {
        return answer(
            RelationVerdict::Unresolved,
            RelationReason::UnsupportedStructure,
        );
    }
    if exhausted_by_relations(later, &left)
        && exhausted_by_relations(earlier, &right)
        && left.relations == right.relations
        && left.repetition == right.repetition
        && left.shape == right.shape
        && later.polarity == earlier.polarity
    {
        return answer(RelationVerdict::Equivalent, RelationReason::SameRelations);
    }
    if later.polarity != Polarity::Asserted || earlier.polarity != Polarity::Asserted {
        return answer(RelationVerdict::Distinct, RelationReason::NotAsserted);
    }
    if !conjunctive_shape(&left) || !conjunctive_shape(&right) {
        return answer(RelationVerdict::Distinct, RelationReason::NonConjunctive);
    }
    if left.repetition != right.repetition {
        return answer(RelationVerdict::Distinct, RelationReason::Repetition);
    }
    if !left
        .relations
        .iter()
        .all(|entry| right.relations.contains(entry))
    {
        return answer(RelationVerdict::Distinct, RelationReason::Relations);
    }
    let ids = content_ids(later);
    let held = content_ids(earlier);
    if !ids.is_empty() && ids.iter().all(|id| held.contains(id)) {
        return answer(
            RelationVerdict::Entailed,
            RelationReason::ConjunctionElimination,
        );
    }
    answer(RelationVerdict::Distinct, RelationReason::Terms)
}

/// Whether the verdict lets `later` be dropped as already said.
#[must_use]
pub const fn certifies_restatement(relation: CheckedRelation) -> bool {
    matches!(
        relation.verdict,
        RelationVerdict::Equivalent | RelationVerdict::Entailed
    )
}
