# Issue 1584 work plan

Objective: implement and verify every criterion of issue 1584 in PR 1585, in the scope
the maintainer's PR comment asks for. That means a full implementation, a generalized
meta check of inputs and outputs in every configuration, and all CI checks green. The
issue has no required sub-issues.

Units and their state:

1. **Declarative grammars** (en, ru, hi, zh, es) and one engine for realization,
   parsing and validation. Done: `cnl_grammar.mjs` and `data/seed/cnl-language-*.lino`.
2. **Profiles, checks and densities**, including the primes floor for any `top-N` and
   the max-precision minimum with its notice. Done: `cnl_output.mjs` and
   `cnl-output.lino`.
3. **Generalized property matrix and reviewed infeasibility ledger.** Done:
   `cnl-grammar-matrix.test.mjs`.
4. **Whole-prompt controlled dialogue** (answers, numbered clarifications, canonical
   tasks) in the worker, HTTP and desktop. Done: `cnl-solver-dialogue.test.mjs` and
   `cnl-output-http.test.mjs`.
5. **Native Rust port and per-language parity fixtures.** Done: `rust/src/cnl_output/`
   and `rust/tests/unit/cnl_parity.rs`.
6. **Documentation, changelog, current-main merge and green CI.** See the PR.

Verification rules:

- Run each JavaScript test file on its own.
- Regenerate worker, TypeScript, registry and embedded projections with the maintained
  generators.
- Run the local gates.
- Preserve the logs of failing CI runs before the next push.
