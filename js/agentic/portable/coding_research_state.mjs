import { researchQueryIdentifierStart, researchQueryIdentifierPart } from './coding_research_query.mjs';
// Canonical research decisions. Adapters expose observations and perform selected effects.
/**
 * @param {string} language @param {boolean} grammarAvailable @param {boolean} bindingSyntaxValid
 * @param {string} name @param {string[]} parameters @param {boolean} hasCommand @param {boolean} hasTest
 * @param {boolean} moduleSeen @param {boolean} moduleComplete @param {boolean} moduleAbsent
 * @param {boolean} testSeen @param {boolean} testComplete @param {boolean} testError
 * @param {boolean} testSourcePresent @param {boolean} acceptanceProved @param {boolean} readTool
 * @returns {string}
 */
export function codingResearchPreflightKernel(language, grammarAvailable, bindingSyntaxValid, name, parameters,
  hasCommand, hasTest, moduleSeen, moduleComplete, moduleAbsent, testSeen, testComplete, testError,
  testSourcePresent, acceptanceProved, readTool) {
  if (language !== 'javascript' && language !== 'typescript') { return 'gap:UnsupportedGrammar'; }
  if (!grammarAvailable) { return 'gap:GrammarUnavailable'; }
  if (!bindingSyntaxValid) { return 'gap:UnsupportedBinding'; }
  for (let index = 0; index < parameters.length; index += 1) {
    if (parameters[index] === name) { return 'gap:UnsupportedBinding'; }
    for (let other = 0; other < index; other += 1) {
      if (parameters[index] === parameters[other]) { return 'gap:UnsupportedBinding'; }
    }
  }
  if (!hasCommand || !hasTest) { return 'gap:IndependentVerificationRequired'; }
  if (!moduleSeen) {
    if (readTool) { return 'read:module'; }
    return 'gap:ReadToolMissing';
  }
  if (!moduleComplete && !moduleAbsent) { return 'gap:SourceReadIncomplete:module'; }
  if (!testSeen) {
    if (readTool) { return 'read:test'; }
    return 'gap:ReadToolMissing';
  }
  if (!testComplete) { return 'gap:SourceReadIncomplete:test'; }
  if (testError || !testSourcePresent) { return 'gap:IndependentVerificationMissing'; }
  if (!acceptanceProved) { return 'gap:IndependentVerificationUnproved'; }
  return 'discover';
}

/**
 * @param {number} proposals
 * @param {boolean} fetchTool @param {string} nextURL @returns {string}
 */
export function codingResearchDiscoveryKernel(proposals, fetchTool, nextURL) {
  if (proposals > 1) { return 'gap:AmbiguousSourceProcedure'; }
  if (proposals === 0) {
    if (nextURL !== '' && fetchTool) { return 'fetch'; }
    return 'gap:NoValidatedSourceProcedure';
  }
  return 'effects';
}

/**
 * @param {string} moduleSource @param {string} proposedSource
 * @param {boolean} latestWritePresent @param {boolean} latestWritePure @param {boolean} plannedWritePure
 * @param {boolean} rerouteExists @param {string} rerouteKind @param {string} lastOutcome
 * @param {string} receiptGap @param {boolean} runTool @returns {string}
 */
export function codingResearchEffectKernel(moduleSource, proposedSource, latestWritePresent, latestWritePure,
  plannedWritePure, rerouteExists, rerouteKind, lastOutcome, receiptGap, runTool) {
  if (moduleSource.trim() !== '') {
    if (moduleSource !== proposedSource) { return 'gap:ExistingModuleRequiresComposition'; }
    if (lastOutcome === 'missing') {
      if (runTool) { return 'run'; }
      return 'gap:ExecutionToolsMissing';
    }
    if (lastOutcome === 'unreported') { return 'gap:VerificationReceiptIncomplete'; }
    if (lastOutcome === 'succeeded') {
      if (receiptGap !== '') { return 'gap:' + receiptGap; }
      return 'existing:verified';
    }
    return 'existing:failed';
  }
  if (latestWritePresent && !latestWritePure) { return 'gap:UnsupportedRepairSource'; }
  if (!rerouteExists) { return 'gap:ExecutionToolsMissing'; }
  if (!plannedWritePure) { return 'gap:UnsupportedRepairSource'; }
  if (rerouteKind === 'final') {
    if (lastOutcome === 'missing' || lastOutcome === 'unreported') { return 'gap:VerificationReceiptIncomplete'; }
    if (lastOutcome === 'succeeded') {
      if (receiptGap !== '') { return 'gap:' + receiptGap; }
      return 'created:verified';
    }
    return 'created:failed';
  }
  return 'reroute';
}

/**
 * @param {string[]} capabilities @param {boolean[]} succeeded @param {boolean[]} argumentsValid
 * @param {string[]} queries @param {number[]} argumentKeys @param {string} query @returns {number}
 */
export function codingResearchSearchIndex(capabilities, succeeded, argumentsValid, queries, argumentKeys, query) {
  if (capabilities.length !== succeeded.length || capabilities.length !== argumentsValid.length
    || capabilities.length !== queries.length || capabilities.length !== argumentKeys.length) { return -1; }
  for (let index = capabilities.length - 1; index >= 0; index -= 1) {
    if (capabilities[index] === 'search' && succeeded[index] && argumentsValid[index] && argumentKeys[index] === 1 && queries[index] === query) {
      return index;
    }
  }
  return -1;
}

/** @param {string[]} capabilities @returns {number} */
export function codingResearchLatestReadIndex(capabilities) {
  for (let index = capabilities.length - 1; index >= 0; index -= 1) {
    if (capabilities[index] === 'read') { return index; }
  }
  return -1;
}

/**
 * @param {string[]} capabilities @param {boolean[]} argumentsValid @param {string[]} commands
 * @param {number} boundary @param {string} command @returns {number}
 */
export function codingResearchRunIndex(capabilities, argumentsValid, commands, boundary, command) {
  if (capabilities.length !== argumentsValid.length || capabilities.length !== commands.length) { return -1; }
  for (let index = capabilities.length - 1; index > boundary; index -= 1) {
    if (capabilities[index] === 'run' && argumentsValid[index] && commands[index] === command) { return index; }
  }
  return -1;
}

/** Mirrors `coding_research_source_ur_ls`, the actual pinned emitter spelling.
 * @param {string[]} urls @param {boolean[]} secure @returns {string[]} */
export function codingResearchSourceURLs(urls, secure) {
  let accepted = [];
  if (urls.length !== secure.length) { return accepted; }
  for (let index = 0; index < urls.length && accepted.length < 3; index += 1) {
    if (!secure[index]) { continue; }
    let present = false;
    for (let previous = 0; previous < accepted.length; previous += 1) {
      if (accepted[previous] === urls[index]) { present = true; }
    }
    if (!present) { accepted = [...accepted, urls[index]]; }
  }
  return accepted;
}

/** @param {string[]} urls @param {string[]} attempted @returns {string} */
export function codingResearchNextURL(urls, attempted) {
  for (let index = 0; index < urls.length; index += 1) {
    let present = false;
    for (let previous = 0; previous < attempted.length; previous += 1) {
      if (attempted[previous] === urls[index]) { present = true; }
    }
    if (!present) { return urls[index]; }
  }
  return '';
}

/**
 * @param {string[]} names @param {string[][]} characters @param {boolean[]} keywords @returns {boolean}
 */
export function codingResearchBindingsKernel(names, characters, keywords) {
  if (names.length !== characters.length || names.length !== keywords.length) { return false; }
  for (let index = 0; index < names.length; index += 1) {
    const letters = characters[index];
    if (letters.length === 0 || keywords[index] || names[index] === 'eval' || names[index] === 'arguments'
      || !researchQueryIdentifierStart(letters[0])) { return false; }
    for (let at = 1; at < letters.length; at += 1) {
      if (!researchQueryIdentifierPart(letters[at])) { return false; }
    }
  }
  return true;
}

/** @param {number} bytes @param {boolean} formalizerAvailable @returns {string} */
export function codingResearchCaptureDecision(bytes, formalizerAvailable) {
  if (bytes > 131072) { return 'gap:SearchResultTooLarge'; }
  if (!formalizerAvailable) { return 'gap:PageFormalizerMissing'; }
  return 'formalize';
}

/** @param {number} sourceIndex @param {number} bytes @returns {boolean} */
export function codingResearchPageEligible(sourceIndex, bytes) {
  return sourceIndex >= 0 && bytes <= 131072;
}

/** @param {string[]} urls @param {string} url @returns {number} */
export function codingResearchSourceIndex(urls, url) {
  for (let index = 0; index < urls.length; index += 1) {
    if (urls[index] === url) { return index; }
  }
  return -1;
}

/**
 * @param {boolean} syntaxValid @param {boolean} effectsNone @param {boolean} syntaxSupported
 * @param {boolean} contractSupported @param {boolean} callEffectsNone
 * @param {number} declarations @param {number} imports @param {string} name @param {string[]} parameters
 * @param {string[]} locals @param {string[]} exposed @param {string} desiredName
 * @param {string[]} desiredParameters @returns {boolean}
 */
export function codingResearchPureRepairKernel(syntaxValid, effectsNone, syntaxSupported, contractSupported, callEffectsNone, declarations,
  imports, name, parameters, locals, exposed, desiredName, desiredParameters) {
  if (!syntaxValid || !effectsNone || !syntaxSupported || !contractSupported || !callEffectsNone || declarations !== 1 || imports !== 0
    || name !== desiredName || parameters.length !== desiredParameters.length
    || locals.length !== 1 || exposed.length !== 1 || locals[0] !== desiredName || exposed[0] !== desiredName) {
    return false;
  }
  for (let index = 0; index < parameters.length; index += 1) {
    if (parameters[index] !== desiredParameters[index]) { return false; }
  }
  return true;
}

/** @param {boolean} found @param {boolean} available @returns {string} */
export function codingResearchSearchDecision(found, available) {
  if (found) { return 'capture'; }
  if (available) { return 'search'; }
  return 'gap:SearchToolMissing';
}

/** @param {string} desired @param {string} observed @returns {boolean} */
export function codingResearchLanguageEligible(desired, observed) { return desired === observed; }
