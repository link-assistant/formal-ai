---
bump: patch
---

### Fixed
- Statement identity keeps the operand order, ordered comparisons (`<` versus `<=`), the and/or shape, quantifier and negation scope, and explicit counts (`twice`) in both the JavaScript and native readers (issue #1487). `2 is less than 1.` no longer restates `1 is less than 2.`, `a - b` and `b - a` and `true or false` and `true and false` keep distinct identities and statement-merge signatures, and summarization drops a sentence only when a checked relation (equivalent or entailed) certifies it. Shared terms alone no longer certify a restatement. Checked paraphrases (`2 is greater than 1.`, `1 < 2`) and repeated mentions still unify.
- A proof request binds to the exact theorem proposition. `Prove there are not infinitely many primes.` is refuted (`proof_binding:negated`, `proof_outcome:disproven`) instead of being answered with the positive proof, and a negation whose scope is not read gets an honest partial plan.
- The intent-formalization cache answers only the request it was formalized from, so `a < b` and `a <= b` no longer share a cached formalization.
- Program synthesis withholds a candidate when the requirement holds structure that no discovered coding part reads (`twice`, `not`, `or`, an operator). The requirement keeps an open `structure:` obligation (`synthesis:obligation:unresolved_structure=…`), so `Return twice the number of vowels` no longer receives the program for `Return the number of vowels`.
