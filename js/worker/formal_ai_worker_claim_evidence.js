// Claim evidence for the handler classes of issue #1175 R3 that read their
// operand from the dialogue, compose from a specification, or look up a
// subject. Twin of rust/src/capability_routing/claim_evidence.rs; spread into
// CLAIM_EVIDENCE (formal_ai_worker_dispatch.js).
//
// Every kind is the reader the handler itself runs before it answers, so a
// row admits exactly the prompts the handler could answer from structure:
// (b) the earlier turn it continues, (c) the specification a composer builds
// from (a composer without one is admitted to its refusal lane only), and
// (e) the subject a lookup resolves or the shape a policy answers. Each
// evidence function takes (prompt, normalized, history).

/** The user and assistant turns of `history` that carry content. */
function claimEvidenceTurns(history) {
  return (Array.isArray(history) ? history : []).filter((turn) => turn && turn.content);
}

/** The words of `text`: whitespace-separated, each trimmed of edge punctuation (Rust `words`). */
function claimEvidenceWords(text) {
  return String(text || "").split(/\s+/u).map((word) => word.replace(/^[^\p{L}\p{M}\p{N}]+|[^\p{L}\p{M}\p{N}]+$/gu, "")).filter(Boolean);
}

/**
 * Does the prompt address the assistant: a word that is an
 * `assistant_self_reference` surface, or that surface plus an inflectional
 * ending of at most three letters (твоё, तुमने)? Unspaced scripts and
 * multi-word surfaces match as substrings (Rust `addresses_assistant`).
 */
function claimEvidenceAddressesAssistant(normalized) {
  const text = String(normalized || "").toLowerCase();
  const words = claimEvidenceWords(text);
  return wordsForRole(ROLE_ASSISTANT_SELF_REFERENCE).map((surface) => String(surface).toLowerCase()).some((surface) =>
    /\s|[㐀-鿿]/u.test(surface)
      ? text.includes(surface)
      : words.some((word) => word.startsWith(surface) && [...word].length - [...surface].length <= 3));
}

/**
 * Issue #1173 R3: the question without a trailing sentence that only shapes
 * the answer. "What is Rust? Explain briefly." reads as "What is Rust?": the
 * last of several sentences is a shaping directive when it has at most three
 * words, one of them a `rule_brief_request` manner surface, and so names no
 * operand of its own. Twin of Rust `without_answer_shape_directive`; the
 * concept reader (extractConceptQuery) reads through it.
 */
function withoutAnswerShapeDirective(prompt) {
  const text = String(prompt || "").trim();
  const sentences = text.match(/[^.!?。！？]+[.!?。！？]*/gu) || [];
  if (sentences.length < 2) return text;
  const last = sentences[sentences.length - 1].trim().toLowerCase();
  const words = claimEvidenceWords(last);
  const shaping = words.length > 0 && words.length <= 3 && wordsForRole(ROLE_RULE_BRIEF_REQUEST)
    .map((surface) => String(surface).toLowerCase())
    .some((surface) => (/\s|[\u3400-\u9fff]/u.test(surface) ? last.includes(surface) : words.includes(surface)));
  return shaping ? sentences.slice(0, -1).join("").trim() : text;
}

/** Do the brackets of the prompt fail to balance (Rust `unbalanced_brackets`)? */
function claimEvidenceUnbalancedBrackets(prompt) {
  let depth = 0;
  for (const character of String(prompt || "")) {
    if (character === "(") depth += 1;
    else if (character === ")") depth -= 1;
    if (depth < 0) return true;
  }
  return depth !== 0;
}

/** The install steps the conversion reader extracts (tryInstallationConversion). */
function claimEvidenceInstallSteps(prompt, normalized) {
  const format = detectInstallationSourceFormat(prompt, normalized);
  const source = extractInstallationSourceText(prompt, format);
  if (extractInstallationSteps(source, format).length > 0) return true;
  return format === INSTALL_FORMAT_MARKDOWN && source !== String(prompt || "")
    && extractInstallationSteps(prompt, format).length > 0;
}

/** The text after the first colon, trimmed (a supplied payload; Rust `supplied_payload`). */
function claimEvidencePayload(prompt) {
  const text = String(prompt || "");
  const colon = text.indexOf(":");
  return colon === -1 ? "" : text.slice(colon + 1).trim();
}

/** The term a conversation-recall query names (Rust `recognize_recall_query`). */
function claimEvidenceRecallQueryTerm(normalized) {
  const text = String(normalized || "");
  return [ROLE_CONVERSATION_RECALL_QUERY, ROLE_CONVERSATION_RECALL_OTHER_QUERY].some((role) => roleWordForms(role).some((form) => {
    let raw = null;
    if (form.slot === "prefix" && text.startsWith(form.before)) raw = text.slice(form.before.length);
    else if (form.slot === "suffix" && text.endsWith(form.after)) raw = text.slice(0, text.length - form.after.length);
    else if (form.slot === "circumfix" && text.startsWith(form.before) && text.endsWith(form.after)
      && text.length > form.before.length + form.after.length) raw = text.slice(form.before.length, text.length - form.after.length);
    return raw !== null && raw.replace(/[\s`"':\-_.,?!()]+/gu, "") !== "";
  }));
}

/** Whether a seeded summary topic's detection keyword occurs (Rust `SummaryTopicSeeds::pick_topic`). */
function claimEvidenceSummaryTopic(normalized) {
  const text = seedRawText(SEED_RAW, "summary-topics.lino") || "";
  return text.split("\n").filter((line) => /^\s+detection_keywords /u.test(line))
    .some((line) => [...line.matchAll(/"([^"]+)"/gu)].some((match) => String(normalized || "").includes(match[1])));
}

/** A document the originality check reads: an attachment, a text sample or a supplied payload (Rust `names_document_operand`). */
function claimEvidenceDocumentOperand(prompt) {
  return extractDocumentOriginalityAttachmentNames(prompt).length > 0 || hasDocumentOriginalityTextSample(prompt)
    || claimEvidencePayload(prompt) !== "";
}

/** The words of `text` outside the surfaces of `roles` and the seeded function words (Rust `content_beyond_roles`). */
function claimEvidenceContentBeyond(text, roles) {
  const covered = new Set([...roles, "request_function_word", "statement_function_word"]
    .flatMap((role) => wordsForRole(role)).flatMap((surface) => claimEvidenceWords(String(surface).toLowerCase())));
  return claimEvidenceWords(String(text || "").toLowerCase()).some((word) => !covered.has(word));
}

/** Whether the proof request states a claim beyond its directive (Rust `names_stated_claim`). */
function claimEvidenceStatedClaim(claim) {
  return claimEvidenceContentBeyond(claim, ["proof_directive", "proof_request_lead", "proof_claim_scaffold", "proof_marker"]);
}

/**
 * Whether the prompt names a subject other than the assistant: the concept,
 * dictionary or mechanism reader extracts a term with a word beyond the
 * seeded function words (Rust `names_other_subject`).
 */
function claimEvidenceNamesOtherSubject(prompt) {
  const query = extractConceptQuery(prompt);
  const definition = wordDefinitionTerm(prompt);
  const subjects = [query && query.term, definition && (definition.term || definition[0]), extractHowItWorksSubject(prompt, nativeLaneLowercase(prompt))];
  return subjects.some((subject) => typeof subject === "string" && claimEvidenceContentBeyond(subject, []));
}

/** A memory query statement in a dialect the WebAssembly engine reads (Rust `memory_query_worker::detect_dialect`). */
function claimEvidenceMemoryQueryStatement(prompt) {
  const text = String(prompt || "").trim().toLowerCase();
  return ["select ", "insert into ", "update ", "delete from "].some((lead) => text.startsWith(lead))
    || (text.includes("{") && ["query", "mutation", "{"].some((lead) => text.startsWith(lead)) && text.includes("memory"));
}

const CLASS_CLAIM_EVIDENCE = Object.freeze({
  // (b) The earlier turn the follow-up continues.
  dialogue_turn: (prompt, normalized, history) => claimEvidenceTurns(history).length > 0,
  prior_user_request: (prompt, normalized, history) => lastHistoryTurn(history, "user") !== null,
  prior_reply: (prompt, normalized, history) => lastHistoryTurn(history, "assistant") !== null,
  prior_software_project: (prompt, normalized, history) => priorSoftwareProjectDialogue(history) !== null,
  prior_research_request: (prompt, normalized, history) => {
    const user = lastHistoryTurn(history, "user");
    return user !== null && looksLikeResearchPrompt(user);
  },
  prior_procedure: (prompt, normalized, history) => priorProceduralHowToDialogue(history) !== null,
  coreference_antecedent: (prompt, normalized, history) => {
    const previous = normalizePrompt(lastHistoryTurn(history, "user") || "");
    return previous !== "" && matchingCoreferenceAntecedent(previous) !== null;
  },
  prior_program: (prompt, normalized, history) => activeProgramContext(history) != null,
  list_items: (prompt) => parseNumericListNumbers(prompt).length > 0 || parseNumericListQuotedStrings(prompt).length >= 2,
  prior_numeric_list: (prompt, normalized, history) => {
    const inherited = numericListHistoryContext(history);
    return Boolean(inherited.slug) || inherited.codeRequested || inherited.items.length > 0;
  },
  text_operation: (prompt, normalized, history) => parseTextManipulationRequest(prompt, normalized, history) !== null,
  shell_command_operand: (prompt) => shellTransformCommand(String(prompt || "")) !== null
    || shellTransformLoopCommand(String(prompt || "")) !== null,
  // (c) The specification a composer builds from.
  composition_topic: (prompt, normalized) => creativeTopicWords(normalized, detectLanguage(String(prompt || ""))).length > 0,
  advice_topic: (prompt, normalized) => creativeRecords("advice_topic").some((record) => childValue(record, "topic").length > 0
    && creativeChildValues(record, "surface").some((surface) => normalized.includes(surface))),
  cached_destination: (prompt, normalized) => creativeDestinations().some((entry) =>
    entry.surfaces.some((surface) => normalized.includes(surface.toLowerCase()))),
  pattern_constraints: (prompt) => {
    const scan = regexSynthesisScanMentions(String(prompt || "").toLowerCase());
    return regexSynthesisCompose(scan.classes, scan.separators) !== null;
  },
  table_reference: (prompt) => sqlSynthesisTableName(String(prompt || "").toLowerCase().split(/\s+/u)
    .map((token) => token.replace(/^[^\p{L}\p{N}_]+|[^\p{L}\p{N}_]+$/gu, "")).filter((token) => token !== "")) !== null,
  filesystem_object: (prompt) => shellComposeSedSubstitution(prompt) !== null
    || codeTaskTokens(String(prompt || "").toLowerCase()).some((token) => codeTaskMapWords("file_context").includes(token)),
  function_spec: (prompt) => extractPythonFunctionName(prompt) !== "" || Boolean(recurrenceSourceMatch(prompt)),
  script_language: (prompt) => {
    const language = programLanguageFromPrompt(normalizeProgramPrompt(prompt));
    return Boolean(language && writeProgramTemplate("hello_world", language));
  },
  program_task: (prompt) => writeProgramParameters(prompt) != null,
  document_format: (prompt) => documentPlanFormat(String(prompt || "").toLowerCase()) !== null,
  install_steps: (prompt, normalized) => claimEvidenceInstallSteps(prompt, normalized),
  call_expression: (prompt) => /[\p{L}_][\p{L}\p{N}_]*\s*\(/u.test(String(prompt || "")),
  // (e) The subject a lookup resolves, or the shape a policy answers.
  concept_subject: (prompt) => extractConceptQuery(prompt) !== null || wordDefinitionTerm(prompt) !== null,
  definition_merge_term: (prompt) => extractDefinitionMergeTerm(prompt, true) !== null,
  mechanism_subject: (prompt) => Boolean(extractHowItWorksSubject(prompt, nativeLaneLowercase(prompt))),
  procedure_task: (prompt) => extractProceduralHowToTask(normalizePrompt(prompt)) != null,
  search_focus: (prompt, normalized) => Boolean((extractWebSearchRequest(prompt, normalized) || {}).query),
  conversation_topic_subject: (prompt) => handlerRuleSetClaims("conversation_topic", prompt, String(prompt || "").toLowerCase()),
  marketplace_scope: (prompt, normalized) => productSearchCatalogue().marketplaces.some((item) =>
    item.phrases.some((phrase) => normalized.includes(phrase) || String(prompt || "").toLowerCase().includes(phrase))),
  // As names_verifiable_spec: the task reader formalizes the request, or the pattern arm reads a run of atoms.
  verifiable_spec: (prompt) => recogniseBrowserVerifiableTask(prompt) !== null || tryPatternInference(prompt) !== null,
  legality_assessment: (prompt, normalized) => assessLegality(prompt, normalized) !== null,
  assistant_addressee: (prompt, normalized) => claimEvidenceAddressesAssistant(normalized),
  punctuation_only: (prompt) => String(prompt || "").trim() !== "" && !/[\p{L}\p{N}]/u.test(String(prompt || "")),
  unbalanced_brackets: (prompt) => claimEvidenceUnbalancedBrackets(prompt),
  memory_program_reading: (prompt) => compileMemoryProgramOnce(prompt).status !== "not_memory_program",
  // Follow-up round of #1175 R3: the no-input arms record a refusal event, these read the input.
  backticked_term: (prompt) => /`[^`]*\S[^`]*`/u.test(String(prompt || "")),
  name_assignment: (prompt) => Boolean(extractAssistantName(prompt) || extractName(String(prompt || ""))),
  recall_query_term: (prompt, normalized) => claimEvidenceRecallQueryTerm(normalized),
  supplied_payload: (prompt) => claimEvidencePayload(prompt) !== "",
  summary_topic: (prompt, normalized) => claimEvidenceSummaryTopic(normalized),
  brainstorm_category: (prompt, normalized) => ((BRAINSTORM_SEEDS || {}).categories || [])
    .some((category) => containsAny(normalized, category.detectionKeywords)),
  persona_or_topic: (prompt, normalized) => ((PERSONA_SEEDS || {}).personas || []).some((persona) => containsAny(normalized, persona.aliases))
    || ((PERSONA_SEEDS || {}).topics || []).some((topic) => containsAny(normalized, topic.detectionKeywords)),
  document_operand: (prompt) => claimEvidenceDocumentOperand(prompt),
  translation_text: (prompt) => Boolean(extractQuotedPhrase(prompt) || extractUnquotedTranslationSurface(prompt)
    || /`[^`]+`/u.test(String(prompt || "")) || textTransformFreeTextPayload(prompt)),
  triz_precedent: (prompt) => trizRelevantTasks(prompt, trizBenchmarkTasks()).length > 0,
  algorithm_operation: (prompt) => operationMatchesSlug("sort", normalizePrompt(String(prompt || "").toLowerCase())),
  source_reference: (prompt) => firstUrlCandidate(prompt) !== null || /(^|\s)[\w.-]*\/[\w.\/-]+/u.test(String(prompt || "")),
  stated_claim: (prompt) => claimEvidenceStatedClaim(extractProofClaim(normalizePrompt(prompt))),
  assistant_subject: (prompt, normalized) => claimEvidenceAddressesAssistant(normalized) || !claimEvidenceNamesOtherSubject(prompt),
  memory_query_statement: (prompt) => claimEvidenceMemoryQueryStatement(prompt),
  fact_subject: (prompt, normalized) => Boolean(gatedFactRecord(normalized, prompt) || explanationConcept(prompt)
    || tryPromptTextQuestion(prompt, normalized) || tryFactComparison(prompt, normalized)),
  // Native-only readers with no worker twin (the browser has no handler to admit).
  learnable_source: () => false,
});
