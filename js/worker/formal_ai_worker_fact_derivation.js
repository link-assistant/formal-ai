// Fact derivation delegates to the canonical crate factory.
// Helpers retain their global API and resolve lazily after all worker modules load.
const FACT_CAPTURES_FILE = "fact-captures.lino";
const FACT_REALIZATION_FILE = "fact-realization.lino";
const FACT_DERIVATION_HELPERS = Object.freeze([
  "factSeedRoot",
  "factChildren",
  "factChildText",
  "factList",
  "factCapturedEntities",
  "factRealizationRules",
  "factSurface",
  "factDefinite",
  "factAcronym",
  "factInflect",
  "factForm",
  "factRealize",
  "factSourceUrl",
  "factLabelSurfaces"
]);
for (const name of FACT_DERIVATION_HELPERS) {
  Object.defineProperty(self, name, {
    configurable: true, enumerable: true,
    get: () => crateModule("crate/fact_derivation.mjs")[name],
    set(value) {
      Object.defineProperty(self, name, {
        value, writable: true, configurable: true, enumerable: true,
      });
    },
  });
}

function derivedCaptureFacts(raw) {
  return crateModule("crate/fact_derivation.mjs").deriveFactsFromTexts(
    seedRawText(raw, FACT_CAPTURES_FILE),
    seedRawText(raw, FACT_REALIZATION_FILE), factRelationProperty,
  );
}
