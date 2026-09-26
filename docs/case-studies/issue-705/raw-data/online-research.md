# Primary-source prior art for anticipatory access

Research was performed on 2026-08-01. These systems predict future *accesses*,
not user intent, so they inform the bounded prefetch policy rather than provide
a component that can be copied into Formal AI.

## Predictive file systems

- Griffioen and Appleton, “Reducing File System Latency using a Predictive
  Approach,” USENIX 1994:
  <https://www.usenix.org/conference/usenix-summer-1994-technical-conference/reducing-file-system-latency-using-predictive>
  describes using past access patterns to predict and prefetch future files.
- Lei and Duchamp, “An Analytical Approach to File Prefetching,” USENIX 1997:
  <https://www.usenix.org/conference/usenix-1997-annual-technical-conference/analytical-approach-file-prefetching>
  analyzes the latency benefit against prefetch cost and inaccurate
  predictions.
- Yeh, Long, and Brandt, “Performing File Prediction with a Program-Based
  Successor Model,” USENIX 2001:
  <https://www.usenix.org/conference/2001-usenix-annual-technical-conference/design-and-implementation-predictive-file>
  presents a successor-style predictive model whose evidence is prior access
  structure.

Transfer to issue #705: learn transition counts from the append-only history,
prefetch only a bounded high-ranked set, and keep wrong predictions observable
because speculative work has a real cost.

## Web prefetching

- Pitkow and Pirolli, “Mining Longest Repeating Subsequences to Predict World
  Wide Web Surfing,” USITS 1999:
  <https://www.usenix.org/events/usits99/full_papers/pitkow/pitkow_html/>
  studies navigation-sequence prediction and compares its usefulness with
  simpler Markov predictors.

Transfer to issue #705: a first-order model is an inspectable baseline whose
quality can be measured before adding order or complexity. Formal AI therefore
ships a deterministic first-order model and an honest hit ledger, not an
unmeasured claim that a more elaborate predictor is better.

## Deliberate non-transfer

None of the systems supplies Formal AI's semantic class formalization,
meaning/operation expansion, source-consent boundary, Links Notation
provenance, or human-gated learning protocol. No neural sequence model or
probabilistic library is introduced. The implementation reuses the repository's
`ProbabilityEvidence`, source cache, and proposal-only adoption cycle.
