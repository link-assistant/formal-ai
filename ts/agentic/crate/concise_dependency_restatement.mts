// R1188-U21: concise generated wording is accepted only by the existing formal model.
// The original dependency graph and rendering remain unchanged and available.
import {summarizeByDependency} from './dependency_summarization.mjs';
import {deformalizeStatement, formalizeText, readWords, WordKind} from './text_formalization.mjs';
import {quoteFault, quotedSegmentSpans} from './normal_markov.mjs';

/** Ordered typed facts; opaque names, numbers and unknown surfaces stay exact. */
export function restatementFacts(statement) {
  const term = (value) => value === null ? null : {
    kind: value.kind, id: value.id,
    surface: value.surface,
  };
  return {language:statement.language,polarity:statement.polarity,
    subject:term(statement.subject),terms:statement.terms.map(term),unknown:[...statement.unknown]};
}

/** This verifies the maintained formal model; it does not assert unmodeled semantic equivalence. */
export function verifiedRestatement(statement) {
  const originalText = statement.text;
  const originalFacts = restatementFacts(statement);
  const retained = (reason) => ({text:originalText,originalText,originalFacts,rewritten:false,reason});
  if (quoteFault(originalText) !== null || quotedSegmentSpans(originalText).length > 0) {
    return retained('quoted-or-malformed-source');
  }
  if (readWords(originalText,statement.language,true).filter((word) => word.kind === WordKind.Negation).length > 1) {
    return retained('multiple-negation-scopes');
  }
  const candidate = deformalizeStatement(statement,statement.language);
  if (Array.from(candidate).length >= Array.from(originalText).length) return retained('not-shorter');
  const recovered = formalizeText(candidate,statement.language);
  if (recovered.length !== 1 || JSON.stringify(restatementFacts(recovered[0])) !== JSON.stringify(originalFacts)) {
    return retained('formal-facts-not-preserved');
  }
  return {text:candidate,originalText,originalFacts,rewritten:true,reason:'verified-shorter-formal-restatement'};
}

/** Additive API: no selection, dependency, duplicate, source fact or legacy text mutation. */
export function summarizeWithRestatement(text,language) {
  const summary = summarizeByDependency(text,language);
  const restatements = summary.kept.map((index) => {
    const node = summary.statements[index];
    const restatement = verifiedRestatement(node.statement);
    const closing = node.closing;
    const rendered = closing !== '' && !restatement.text.endsWith(closing)
      ? restatement.text + closing : restatement.text;
    return {index,dependsOn:[...node.dependsOn],dependents:[...node.dependents],...restatement,rendered};
  });
  const generatedText = restatements.map((item) => item.rendered).join(' ');
  const recoveredFacts = formalizeText(generatedText,language).map(restatementFacts);
  const keptFacts = summary.kept.map((index) => restatementFacts(summary.statements[index].statement));
  const coreRestatementVerified = JSON.stringify(recoveredFacts) === JSON.stringify(keptFacts);
  return {...summary,restatements,coreRestatementVerified,
    restatedText:coreRestatementVerified ? generatedText : summary.text,
    preservationScope:'ordered typed formal facts and original graph; unmodeled semantics unknown'};
}
