Parent: #710

Audit verdict: still-broken.

Current evidence
- tests/unit/issue_403.rs proves the Russian interval problem is formalized and solved.
- No production path translates that proof into a requested programming language.

Acceptance
- Represent the proof independently from its presentation language.
- Translate one proof into at least two programming languages through the general code-translation path.
- Compile or execute generated proof programs where the environment supports it.
- Add regression coverage for all registered supported natural languages.
