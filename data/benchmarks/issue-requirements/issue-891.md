Parent: #710

Audit verdict: still-broken.

Current evidence
- calculator_delegation covers linear, placeholder, symbolic, polynomial, and several word-problem categories.
- The explicit issue-406 requirement of at least 50 verified equation-type examples is not met or counted by a ratchet.

Acceptance
- Define a machine-readable corpus with at least 50 distinct equation types.
- Run every case through the production solver and verify the result.
- Add a CI ratchet that fails below 50 or on any corpus regression.
- Record category coverage and upstream calculator limitations.
