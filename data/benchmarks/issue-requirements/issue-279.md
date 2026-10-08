Parent: #244
Audit source: PR #245 post-implementation audit on 2026-05-26, after issues #246-#259 and #262 were merged.

## Problem

`REQUIREMENTS.md` R6 asks us to explore Bayesian networks, Markov chains, and similar symbolic/probabilistic methods. The current implementation has deterministic rules and the new temperature-based selection path, but it does not yet store or update probabilistic evidence over the Links Notation graph.

We need a symbolic probabilistic layer that helps rank interpretations and candidate answers without using neural-network inference for reasoning.

## Scope

- Represent probabilistic evidence as link-native records, with provenance and timestamps.
- Add Bayesian/Markov-style ranking helpers over candidate formalizations or answer candidates.
- Integrate ranking with the existing temperature/clarify-vs-guess selection policy.
- Keep deterministic replay: same prompt, same store, same config, and same impulse hash must produce the same selected candidate.
- Surface probability evidence in traces so users can inspect why a candidate outranked another.

## Acceptance criteria

- Tests demonstrate link-native probabilistic evidence creation, update, and replay.
- Candidate ranking changes when new symbolic evidence is added, without modifying neural weights or calling neural inference.
- The clarify-vs-guess policy can consume the probability margin between top candidates.
- Offline mode and cached-source provenance remain respected.
- Documentation explains the supported probabilistic model and its non-neural boundary.

## Requirement links

- `REQUIREMENTS.md` R6
- `VISION.md` universal problem-solving and no-neural-reasoning constraints

