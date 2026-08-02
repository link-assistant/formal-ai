# Online research and component evaluation

Research was performed on 2026-07-19. Primary/authoritative sources were preferred.

## Translation architecture

- Richens' 1958 interlingual MT paper describes the scalable split into source → interlingua and interlingua → target dictionaries, rather than maintaining every language pair: <https://academic.oup.com/comjnl/article/1/3/144/412182>.
- Barnett et al. describe an interlingua pipeline that captures language-specific distinctions during source analysis and target generation: <https://aclanthology.org/1991.mtsummit-papers.4/>.
- Universal Dependencies provides cross-linguistically consistent syntactic annotation while preserving language-specific extensions: <https://universaldependencies.org/introduction.html>. It is useful background for a future unrestricted sentence parser, but is much larger than the slot-order defect here.

These sources support the repository's existing source formalizer → `MeaningId` → target renderer design. The fix extends that design rather than introducing direct ru→en logic.

## Domain meaning and translation quality

- The Stanford Encyclopedia's Gödel entry explains the theorem and its formal assumptions: <https://plato.stanford.edu/entries/goedel-incompleteness/>.
- LMU's theorem notes give the familiar formulation that a formal system capable of elementary arithmetic is either inconsistent or incomplete: <https://cs.lmu.edu/~ray/notes/godeltheorems/>.
- Russian reference wording describes an effectively axiomatized, consistent formal system with sufficient arithmetic expressiveness: <https://ru.wikipedia.org/wiki/Теоремы_Гёделя_о_неполноте>.

Inference: `any formal system is either incomplete or inconsistent` is the faithful translation of the submitted sentence, but the sentence is broader than Gödel's theorem. Translation must not silently add “effectively axiomatized” or “sufficiently expressive”; instead the semantic node links to the existing, qualified Gödel concept for later reasoning.

## Existing components/libraries

| Component | Fit | Decision |
|---|---|---|
| Existing `WordForm::Slot` + seed roles | Already models prefix, suffix, circumfix, and bare forms in Rust and JS. | Use it; add suffix projection. |
| Existing `TranslationPipeline`/`MeaningId` | Implements language-neutral formalization and deformation plus provenance. | Use it unchanged. |
| Existing Wiktionary/Wikidata adapters | Strong for lexical senses and attested translations; unreliable for an arbitrary full proposition. | Retain as the general lexical fallback, not the proposition renderer. |
| Universal Dependencies | Strong basis for future unrestricted multilingual syntax. | No dependency added for this bounded grammar extension. |
| `nom` (already a dependency) | Could implement a larger grammar but would duplicate seed slot semantics for this case. | Not needed. |

No upstream project defect was found: all failures are in Formal AI's own routing, extraction, formalization, and seed coverage, so no external issue was filed.

## CI client configuration

- OpenCode's custom-provider documentation defines `limit.context` as the maximum accepted input and `limit.output` as the maximum generated output, and states that these fields let the client calculate remaining context: <https://opencode.ai/docs/providers/>.

The unrelated Agent CLI CI failure was therefore fixed by completing Formal AI's local test-model configuration. There was no evidence of an OpenCode implementation defect to report upstream.

## Deterministic browser networking

- MDN documents that `AbortController.abort()` can cancel fetch requests,
  response-body consumption, and streams:
  <https://developer.mozilla.org/en-US/docs/Web/API/AbortController/abort>.
- Playwright's official network guide documents `page.route()` as the mechanism
  for mocking or aborting browser HTTP requests:
  <https://playwright.dev/docs/network#handle-requests>.

These APIs fit the two distinct requirements: production provider calls need a
bounded lifetime, while the desktop-fallback regression must prove behavior
with browser networking deterministically unavailable. No additional runtime
or test library is necessary.
