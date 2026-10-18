// The worker route delegates to the maintained generated crate factory.
// Markdown segmentation, seeded obligation and restatement policy have one owner:
// js/agentic/crate/requirement_extraction.mjs.
function extractRequirements(text) {
  return crateModule("crate/requirement_extraction.mjs").extractRequirements(text);
}
