# Issue 483 Solution Plans

## Plan A: Bundle A Browser LLM

Rejected. Bundling WebLLM, Transformers.js, or model weights would directly
violate the issue requirement that nothing is included in the application
package or web UI and that downloads happen only after explicit opt-in.

## Plan B: Let The Model Emit Links Notation

Rejected. This gives the model authority over anchors, predicates, and terms.
It would make neural output part of the formal source of truth and conflicts
with the "LLMs are never at steering wheel" requirement.

## Plan C: Advisory Picker Over Existing Candidates

Accepted. The formalizer remains responsible for generating candidates. The
model prompt lists those candidates as bounded options. The parser accepts only
an option id, summary, or existing probability target; everything else is
ignored. Accepted advice can only rerank the candidate list, and the normal
formalization selector still computes the final decision.

Implementation steps:

1. Add failing tests for default-off behavior, bounded-option output, hardware
   filtering, and rating sort.
2. Add a small formalization model catalog with explicit VRAM and WebGPU
   feature gates.
3. Add prompt/advice helpers that never synthesize a new
   `FormalizationCandidate`.
4. Add a selection entry point that applies accepted advice and then calls the
   normal formalization selector.
5. Surface the experimental setting in the browser UI, defaulted off.
6. Filter visible models by detected browser hardware and sort them by public
   rating.
7. Preserve research and verification artifacts in this case study.

## Plan D: Runtime Hook With Dynamic Import

Not implemented in this PR because the issue requires no model runtime or
weights in the application package. Dynamic import is the right shape for a
future external/on-demand runtime because it can keep model code out of the
initial bundle, but it must still call the accepted Plan C boundary and must
not download anything before settings opt-in and a formalization task that
needs advice.
