# Issue 923 Solution Plan

## R923-1: Bounded Equality Saturation

Add an optional deterministic e-graph dependency and a general S-expression
decision path. Seed both terms, saturate only within explicit iteration and
node ceilings, emit an auditable certificate on equivalence, and leave every
unproved claim inconclusive. Keep recognized symbolic claims out of the affine
parser even when the optional feature is disabled.

## R923-1: Bounded Rule Inference

Represent range-restricted, function-free positive Datalog as explicit facts,
rules, and one ground query. Compute the least fixed point with clause, arity,
round, fact, and join-substitution ceilings. Prove membership, disprove absence
only after completion, and return inconclusive for parsing or resource
failures.

## R923-2 and R923-3: Pinned Sources And Honest Scores

Mechanically adapt rewrite declarations from a pinned egg test source and
asserted consequences from a pinned Ascent example. Grade only structured proof
events, run the real solver, record exact passed/total values, and raise the
existing monotonic ledger floors. Preserve revision and MIT-license evidence;
do not import Ascent when the generic evaluator is small enough to implement
and audit in-tree.

## R923-4 and R923-5: Regression, Traceability, And Self-Hosting

Begin with the smallest failing equality test, add one regression per behavior
and one live whole-task replay, then run the full repository checks unchanged.
Preserve issue/PR feedback, primary-source research, requirements, release
metadata, and a real Agent-CLI-authored invariant leaf with byte-exact output
and commit trailers.
