// Worker module 21 of 21. Loaded by ../formal_ai_worker.js.
function isTargetlessProgramModification(normalized) { // "Reverse it.": Rust `looks_like_ambiguous_program_modification`
  const mentions = (role) => lexiconMentionsRole(role, normalized);
  return mentions(ROLE_PROGRAM_MODIFICATION) && mentions(ROLE_PROGRAM_MODIFICATION_REFERENCE) && !mentions(ROLE_PROGRAM_ARTIFACT);
}
async function solve(prompt, history, prefs, userContext = {}, memory = [], options = {}) {
  // Issue #556: activate the forced response language for the whole replay and
  // always restore the previous value, so a nested follow-up replay never
  // leaks its forced language onto the outer turn's remaining handlers.
  const forced =
    options && isKnownResponseLanguage(options.forcedResponseLanguage)
      ? options.forcedResponseLanguage
      : null;
  const previousForced = setForcedResponseLanguage(forced);
  try {
    // Meanings learned in earlier sessions come back through memory; the
    // ones learned now leave as this answer's memory operation.
    metaImportLearned(memory);
    const answer = await solveImpl(prompt, history, prefs, userContext, memory, options);
    return metaAttachLearned(await metaResolveImpasse(prompt, answer, prefs));
  } finally {
    setForcedResponseLanguage(previousForced);
  }
}
async function solveImpl(prompt, history, prefs, userContext = {}, memory = [], options = {}) {
  const preferences = prefs || {};
  // Issue #556: a response-language follow-up replays the previous request
  // through this whole solver with the requested language forced onto every
  // localizable handler. `forcedResponseLanguage` is the JS mirror of
  // SolverConfig.forced_response_language; when set it overrides the detected
  // message language and doubles as the recursion guard so the replay never
  // re-enters the follow-up handler.
  const forcedResponseLanguage = isKnownResponseLanguage(
    options && options.forcedResponseLanguage,
  )
    ? options.forcedResponseLanguage
    : null;
  const autoDefinitionFusion = definitionFusionByDefault(preferences);
  const steps = [];
  const toolCalls = [];
  const events = [`impulse:${prompt}`];
  steps.push({ step: "impulse", detail: prompt });
  const normalized = normalizePrompt(prompt);
  const formalization = buildFormalization(prompt, normalized);
  events.push(`formalization:${formalization.tuple}`);
  steps.push({
    step: "formalize",
    detail: formalizationDetail(formalization),
    formalization: {
      raw: formalization.raw,
      subject: formalization.subject,
      verb: formalization.verb,
      object: formalization.object,
      tuple: formalization.tuple,
      interpretations: formalization.interpretations || [],
    },
  });
  const language = forcedResponseLanguage || detectLanguage(prompt);
  events.push(`language:${language}`);
  steps.push({ step: "detect_language", detail: language });
  if (forcedResponseLanguage) {
    events.push(`language_to:${forcedResponseLanguage}`);
    steps.push({ step: "force_response_language", detail: forcedResponseLanguage });
  }
  // Issue #324: resolve which language should drive natural-language responses
  // (defaults to the detected message language).
  const responseLanguage = responseLanguageFor(language, preferences, userContext);
  if (responseLanguage !== language) {
    events.push(`response_language:${responseLanguage}`);
    steps.push({ step: "resolve_response_language", detail: responseLanguage });
  }
  // Issue #180: bundle the per-turn formalization context so every
  // handler hit can fold a resolved entity id back into the tuple and
  // every `finalize` call can emit a `deformalize` step that records the
  // symbolic → natural-language projection. The context is mutable so
  // resolvers can update `resolved` as new ids surface.
  const formalizationContext = {
    initial: formalization,
    resolved: null,
    language,
  };
  if (formalization.needsClarification) {
    events.push("formalization:ambiguous");
    steps.push({
      step: "clarify_formalization",
      detail: (formalization.suggestions || []).join(", "),
    });
    return finalize(events, steps, toolCalls, {
      intent: "clarification",
      content: formalizationClarificationMessage(formalization, language),
      confidence: 0.4,
      evidence: ["formalization:ambiguous"],
    }, formalizationContext);
  }

  // 2026-10-06 meta-algorithm doctrine: every turn first opens its words as
  // unknowns and grounds them without network (request definitions, learned
  // chunks, the instruction set's documentation). A program verified against
  // the request's own examples, or one every word of the request grounds,
  // answers here ahead of any canned handler; dictionary lookups wait for the
  // fallback below, so they are spent only when no handler answers.
  let meta = await metaReason(prompt, language, {});
  formalizationContext.meta = meta;
  events.push(`meta:${meta.goal}:${meta.status}`);
  steps.push({ step: "meta_reason", detail: `${meta.goal} ${meta.status}`, derivation: meta.derivationLino });
  // A bare imperative after earlier turns refines their artifact: it never takes the turn ahead of the handlers.
  if (meta.status === "solved" && (meta.program || meta.subgoals) && !(meta.imperative && history?.length) && !isTargetlessProgramModification(normalized)) {
    events.push("handler:meta_reasoner");
    return finalize(events, steps, toolCalls, solverMetaProjection(metaAnswer(meta)), formalizationContext);
  }

  const compound = await FormalAiSeed.solveIndependentQuestions(prompt, history, preferences, userContext, memory, options, solve);
  if (compound) return finalize(["composition:compound_response"], [], compound.toolCalls, compound, formalizationContext);

  const compoundProcedure = await tryGreetingProceduralCompound(prompt, language, preferences);
  if (compoundProcedure) {
    for (const event of compoundProcedure.trace || []) events.push(event);
    events.push(`handler:${compoundProcedure.intent}`);
    steps.push({ step: "decompose_impulse", detail: "greeting+procedural_how_to" });
    steps.push({
      step: "dispatch_handler",
      detail: "tryGreetingProceduralCompound",
    });
    toolCalls.push({
      tool: "procedural_how_to",
      inputs: {
        prompt: compoundProcedure.procedurePrompt || prompt,
        language: compoundProcedure.procedureLanguage || language,
        query: compoundProcedure.query || "",
        wikihowCandidate: compoundProcedure.wikihowCandidate || "",
      },
      outputs: {
        intent: compoundProcedure.intent,
        confidence: compoundProcedure.confidence,
        formalizedObject: compoundProcedure.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, compoundProcedure, formalizationContext);
  }
  const behaviorRule = tryBehaviorRules(prompt, normalized, history, preferences);
  if (behaviorRule) {
    events.push(`handler:${behaviorRule.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryBehaviorRules" });
    return finalize(events, steps, toolCalls, behaviorRule, formalizationContext);
  }
  const memoryEvents = Array.isArray(options.memoryEvents)
    ? options.memoryEvents
    : [];
  const memoryInspection = tryMemoryInspection(
    prompt,
    normalized,
    history,
    memoryEvents,
    language,
  );
  if (memoryInspection) {
    events.push(`handler:${memoryInspection.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryMemoryInspection" });
    return finalize(events, steps, toolCalls, memoryInspection, formalizationContext);
  }
  // The punctuation_only_prompt registry row runs here, ahead of the
  // translation and lookup probes, through the seed rule interpreter
  // (data/seed/handler-rules.lino) so its wording is the seeded response.
  const punctuationOnly = isPunctuationOnlyPrompt(prompt)
    ? tryPunctuationOnlyPrompt(prompt, normalized)
    : null;
  if (punctuationOnly) {
    events.push("handler:clarification");
    events.push(`clarification:punctuation_only:${String(prompt).trim()}`);
    steps.push({ step: "dispatch_handler", detail: "tryPunctuationOnlyPrompt" });
    return finalize(events, steps, toolCalls, punctuationOnly, formalizationContext);
  }
  const translation = await tryTranslation(prompt, normalized);
  if (translation) {
    events.push(`handler:${translation.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryTranslation" });
    return finalize(events, steps, toolCalls, translation, formalizationContext);
  }
  // Skip the response-language follow-up during a forced-language replay: the
  // replay IS the follow-up's body, so re-entering here would recurse forever.
  if (!forcedResponseLanguage) {
    steps.push({ step: "invoke_tool", detail: "response_language_followup" });
    const responseLanguageFollowup = await tryResponseLanguageFollowup(
      prompt,
      normalized,
      history,
      preferences,
    );
    if (responseLanguageFollowup) {
      events.push(`handler:${responseLanguageFollowup.intent}`);
      events.push("handler:response_language_followup");
      steps.push({
        step: "dispatch_handler",
        detail: "tryResponseLanguageFollowup",
      });
      toolCalls.push({
        tool: "response_language_followup",
        inputs: {
          prompt,
          requestedLanguage: detectResponseLanguage(normalized) || "",
        },
        outputs: {
          intent: responseLanguageFollowup.intent,
          confidence: responseLanguageFollowup.confidence,
        },
      });
      return finalize(
        events,
        steps,
        toolCalls,
        responseLanguageFollowup,
        formalizationContext,
      );
    }
    // With no earlier turn to re-render, the terse switch is answered in the named language.
    const demonstration = tryResponseLanguageDemonstration(prompt);
    if (demonstration) return finalizeInlineHandler(events, steps, toolCalls, demonstration, "tryResponseLanguageDemonstration", formalizationContext);
  }
  steps.push({ step: "invoke_tool", detail: "project_lookup" });
  const projectLookup = await tryProjectLookup(prompt, language, preferences);
  if (projectLookup) {
    events.push(`handler:${projectLookup.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryProjectLookup" });
    toolCalls.push({
      tool: "project_lookup",
      inputs: { prompt, language },
      outputs: {
        intent: projectLookup.intent,
        confidence: projectLookup.confidence,
      },
    });
    return finalize(events, steps, toolCalls, projectLookup, formalizationContext);
  }
  steps.push({ step: "invoke_tool", detail: "wikipedia_article_question" });
  const earlyWikiArticleQuestion = await tryWikipediaArticleQuestion(
    prompt,
    language,
    preferences,
  );
  if (earlyWikiArticleQuestion) {
    events.push(`handler:${earlyWikiArticleQuestion.intent}`);
    steps.push({
      step: "dispatch_handler",
      detail: "tryWikipediaArticleQuestion",
    });
    toolCalls.push({
      tool: "wikipedia_article_question",
      inputs: {
        prompt,
        language,
        query: earlyWikiArticleQuestion.query || "",
      },
      outputs: {
        intent: earlyWikiArticleQuestion.intent,
        confidence: earlyWikiArticleQuestion.confidence,
        formalizedObject: earlyWikiArticleQuestion.formalizedObject || "",
      },
    });
    return finalize(
      events,
      steps,
      toolCalls,
      earlyWikiArticleQuestion,
      formalizationContext,
    );
  }
  const githubRepositoryTraffic = tryGithubRepositoryTraffic(normalized, language);
  if (githubRepositoryTraffic) {
    events.push(`handler:${githubRepositoryTraffic.intent}`);
    steps.push({
      step: "dispatch_handler",
      detail: "tryGithubRepositoryTraffic",
    });
    return finalize(
      events,
      steps,
      toolCalls,
      githubRepositoryTraffic,
      formalizationContext,
    );
  }
  const githubRepoInfoRequest = githubRepositoryInfoRequest(prompt, normalized);
  if (githubRepoInfoRequest) {
    steps.push({
      step: "invoke_tool",
      detail: `github_repo_info:${repositorySlug(githubRepoInfoRequest)}`,
    });
    const githubRepoInfo = await tryGithubRepositoryInfo(
      githubRepoInfoRequest,
      language,
      preferences,
    );
    events.push(`handler:${githubRepoInfo.intent}`);
    steps.push({
      step: "dispatch_handler",
      detail: "tryGithubRepositoryInfo",
    });
    toolCalls.push({
      tool: "github_repo_info",
      inputs: {
        prompt,
        language,
        repository: repositorySlug(githubRepoInfoRequest),
      },
      outputs: {
        intent: githubRepoInfo.intent,
        confidence: githubRepoInfo.confidence,
      },
    });
    return finalize(events, steps, toolCalls, githubRepoInfo, formalizationContext);
  }

  const capabilities = !isAssistantFreeTimePrompt(normalized, prompt)
    && tryCapabilities(prompt, normalized, preferences, history);
  if (capabilities) {
    events.push(`handler:${capabilities.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryCapabilities" });
    return finalize(events, steps, toolCalls, capabilities, formalizationContext);
  }
  const architecture = tryArchitectureExplanation(prompt, normalized);
  if (architecture) {
    events.push("handler:meta_explanation");
    steps.push({ step: "dispatch_handler", detail: "tryArchitectureExplanation" });
    return finalize(events, steps, toolCalls, architecture, formalizationContext);
  }
  // Issue #676: "how are you?" small talk is its own wellbeing intent so the
  // reply differs from a bare greeting. It must be checked before the greeting
  // rule because some phrasings ("привет как дела") also contain a greeting cue.
  if (isWellbeingPrompt(normalized, prompt)) {
    events.push("rule:wellbeing");
    steps.push({ step: "match_rule", detail: "wellbeing" });
    const temperature = numericPreference(preferences.temperature, 0.7, 0, 1);
    const randomize = preferences.greetingVariations !== false && temperature > 0;
    return finalize(events, steps, toolCalls, {
      intent: "wellbeing",
      content: answerFor("wellbeing", language, { randomize: randomize }),
      confidence: 1.0,
      evidence: [
        "rule:wellbeing",
        `language:${language}`,
        `variation:${randomize ? "random" : "canonical"}`,
        `temperature:${temperature.toFixed(2)}`,
      ],
    }, formalizationContext);
  }
  if (isGreetingPrompt(normalized, prompt) && !tryRoleplayRequest(prompt, normalized)) {
    events.push("rule:greeting");
    steps.push({ step: "match_rule", detail: "greeting" });
    const temperature = numericPreference(preferences.temperature, 0.7, 0, 1);
    const randomize = preferences.greetingVariations !== false && temperature > 0;
    return finalize(events, steps, toolCalls, {
      intent: "greeting",
      content: answerFor("greeting", language, { randomize: randomize }),
      confidence: 1.0,
      evidence: [
        "rule:greeting",
        `language:${language}`,
        `variation:${randomize ? "random" : "canonical"}`,
        `temperature:${temperature.toFixed(2)}`,
      ],
    }, formalizationContext);
  }
  if (isAssistantFreeTimePrompt(normalized, prompt)) {
    events.push("rule:assistant_free_time");
    steps.push({ step: "match_rule", detail: "assistant_free_time" });
    return finalize(events, steps, toolCalls, {
      intent: "assistant_free_time",
      content: FormalAiSeed.stableResponseVariant(responseEntryFor("assistant_free_time", language), prompt),
      confidence: 1.0,
      evidence: [
        "rule:assistant_free_time",
        `language:${language}`,
        "variation:prompt_stable",
      ],
    }, formalizationContext);
  }
  if (isFarewellPrompt(normalized, prompt)) {
    events.push("rule:farewell");
    steps.push({ step: "match_rule", detail: "farewell" });
    return finalize(events, steps, toolCalls, {
      intent: "farewell",
      content: answerFor("farewell", language),
      confidence: 1.0,
      evidence: ["rule:farewell", `language:${language}`],
    }, formalizationContext);
  }
  if (isTestStatusPrompt(normalized, prompt)) {
    events.push("rule:test_status");
    steps.push({ step: "match_rule", detail: "test_status" });
    return finalize(events, steps, toolCalls, {
      intent: "test_status",
      content: answerFor("test_status", language),
      confidence: 1.0,
      evidence: ["rule:test_status", `language:${language}`],
    });
  }
  if (isCourtesyResponsePrompt(normalized, prompt)) {
    events.push("rule:courtesy_response");
    steps.push({ step: "match_rule", detail: "courtesy_response" });
    const courtesy = courtesyResponseFor(language, preferences);
    return finalize(events, steps, toolCalls, {
      intent: "courtesy_response",
      content: courtesy.content,
      confidence: 1.0,
      evidence: [
        "rule:courtesy_response",
        `language:${language}`,
        `variation:${courtesy.randomize ? "random" : "canonical"}`,
        `temperature:${courtesy.temperature.toFixed(2)}`,
        `follow_up_probability:${courtesy.followUpProbability.toFixed(2)}`,
        `follow_up:${courtesy.followUpIncluded ? "included" : "omitted"}`,
      ],
    });
  }
  const calculatorRateBasis = tryCalculatorRateBasis(normalized, language);
  if (calculatorRateBasis) {
    events.push(`handler:${calculatorRateBasis.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryCalculatorRateBasis" });
    return finalize(events, steps, toolCalls, calculatorRateBasis, formalizationContext);
  }
  const assistantNameMemory = tryAssistantName(prompt, normalized, history);
  if (assistantNameMemory) {
    events.push(`handler:${assistantNameMemory.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryAssistantName" });
    return finalize(events, steps, toolCalls, assistantNameMemory, formalizationContext);
  }
  if (isAssistantNamePrompt(normalized, prompt)) {
    events.push("rule:assistant_name");
    steps.push({ step: "match_rule", detail: "assistant_name" });
    const configuredName = normalizeAssistantNamePreference(preferences.assistantName);
    return finalize(events, steps, toolCalls, {
      intent: "assistant_name",
      content: assistantNameAnswer(language, preferences),
      confidence: 1.0,
      evidence: [
        "rule:assistant_name",
        `language:${language}`,
        `assistant_name:${configuredName ? "configured" : "not_set"}`,
      ],
    }, formalizationContext);
  }
  // Issue #1085: an opinion opener ("what do you think about …") is the
  // opinion_question (and clarification) row natively, which identity must not
  // claim; the rows answer from the synchronous table below, as Rust dispatches.
  if (isIdentityPrompt(normalized, prompt) && !tryOpinionQuestion(prompt, normalized) && !tryClarification(prompt, normalized)) {
    events.push("rule:identity");
    steps.push({ step: "match_rule", detail: "identity" });
    return finalize(events, steps, toolCalls, {
      intent: "identity",
      content: answerFor("identity", language),
      confidence: 1.0,
      evidence: ["rule:identity", `language:${language}`],
    }, formalizationContext);
  }
  // Issue #312: compute the write-program result once so a concrete program
  // request (a known language + task with a template) can take precedence over
  // the concept lookup, while the "unsupported" variant still falls back after
  // the definition/concept handlers. This mirrors the Rust solver, where
  // `SelectedRule::WriteProgram` is promoted above `handle_specialized_pattern`
  // so "напиши программу на Rust" is not answered as a "Rust" encyclopedia entry.
  let writeProgramResult;
  const writeProgram = () => {
    if (writeProgramResult === undefined) {
      writeProgramResult = tryWriteProgram(
        prompt,
        history,
        responseLanguage,
        preferences.blueprintComposition,
      );
    }
    return writeProgramResult;
  };
  const definitionMerge = () => tryDefinitionMerge(prompt, { allowPlainConcept: autoDefinitionFusion });
  const syncHandlers = synchronousHandlerCandidates({
    prompt, normalized, history, memory, memoryEvents, language,
    responseLanguage, preferences, userContext, writeProgram, definitionMerge,
  });
  for (const handler of syncHandlers) {
    const hit = handler.run();
    if (hit) {
      events.push(`handler:${hit.intent}`);
      steps.push({ step: "dispatch_handler", detail: handler.name });
      if (Array.isArray(hit.steps)) {
        for (const step of hit.steps) steps.push(step);
      }
      if (Array.isArray(hit.trace)) {
        for (const event of hit.trace) events.push(event);
      }
      if (hit.intent === "javascript_execution" || hit.intent === "javascript_execution_error") {
        toolCalls.push({
          tool: "eval_js",
          inputs: { prompt },
          outputs: { intent: hit.intent, confidence: hit.confidence },
        });
      }
      if (
        hit.intent === "concept_lookup" ||
        hit.intent === "concept_lookup_in_context" ||
        hit.intent === "definition_merge"
      ) {
        toolCalls.push({
          tool: "concept_lookup",
          inputs: { prompt },
          outputs: { intent: hit.intent, confidence: hit.confidence },
        });
      }
      return finalize(events, steps, toolCalls, hit, formalizationContext);
    }
  }
  const coreferenceFact = tryCoreferenceFactLookup(prompt, normalized, history);
  if (coreferenceFact) {
    events.push(`handler:${coreferenceFact.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryCoreferenceFactLookup" });
    return finalize(events, steps, toolCalls, coreferenceFact, formalizationContext);
  }
  // Real-time fact reasoning: parse structured (relation, subject) queries, hit
  // the 1-week TTL cache, fall back to Wikidata/Wikipedia for any country or
  // entity. Cache warmed from `data/seed/facts.lino` so the test matrix and
  // offline browsers still answer instantly. The legacy substring-based
  // `tryFactLookup` remains as a fallback for non-relation seed facts
  // (e.g. who painted the Mona Lisa) until those are migrated to relations.
  steps.push({ step: "invoke_tool", detail: "fact_query" });
  const factQuery = await tryFactQuery(prompt, normalized, preferences);
  if (factQuery) {
    events.push(`handler:${factQuery.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryFactQuery" });
    if (Array.isArray(factQuery.trace)) {
      for (const event of factQuery.trace) events.push(event);
    }
    toolCalls.push({
      tool: "fact_query",
      inputs: { prompt, language },
      outputs: {
        intent: factQuery.intent,
        confidence: factQuery.confidence,
        formalizedObject: factQuery.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, factQuery, formalizationContext);
  }
  const legacyFact = tryFactLookup(prompt, normalized);
  if (legacyFact) {
    events.push(`handler:${legacyFact.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryFactLookup" });
    return finalize(events, steps, toolCalls, legacyFact, formalizationContext);
  }
  // Issue #1172 R8: the fact_lookup row's last step explains from retrieved pages.
  const explanation = await tryExplanationResearch(prompt);
  if (explanation) {
    events.push(`handler:${explanation.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryExplanationResearch" });
    return finalize(events, steps, toolCalls, explanation, formalizationContext);
  }
  steps.push({ step: "invoke_tool", detail: "http_fetch" });
  const fetched = await tryFetch(prompt);
  if (fetched) {
    events.push(`handler:${fetched.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryFetch" });
    toolCalls.push({
      tool: "http_fetch",
      inputs: { prompt },
      outputs: { intent: fetched.intent, confidence: fetched.confidence, iframeUrl: fetched.iframeUrl || null },
    });
    return finalize(events, steps, toolCalls, fetched, formalizationContext);
  }
  steps.push({ step: "invoke_tool", detail: "url_navigate" });
  const navigated = await tryUrlNavigate(prompt);
  if (navigated) {
    events.push(`handler:${navigated.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryUrlNavigate" });
    toolCalls.push({
      tool: "url_navigate",
      inputs: { prompt },
      outputs: { intent: navigated.intent, confidence: navigated.confidence, iframeUrl: navigated.iframeUrl || null },
    });
    return finalize(events, steps, toolCalls, navigated, formalizationContext);
  }
  steps.push({ step: "invoke_tool", detail: "docs_method_explanation" });
  const docsMethod = tryDocsMethodExplanation(prompt, language);
  if (docsMethod) {
    events.push(`handler:${docsMethod.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryDocsMethodExplanation" });
    toolCalls.push({
      tool: "docs_method_explanation",
      inputs: { prompt, language, project: "pandas", method: "DataFrame.join" },
      outputs: {
        intent: docsMethod.intent,
        confidence: docsMethod.confidence,
        formalizedObject: docsMethod.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, docsMethod, formalizationContext);
  }
  steps.push({ step: "invoke_tool", detail: "procedural_how_to" });
  const procedure = await tryProceduralHowTo(prompt, language, preferences);
  if (procedure) {
    events.push(`handler:${procedure.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryProceduralHowTo" });
    toolCalls.push({
      tool: "procedural_how_to",
      inputs: {
        prompt,
        language,
        query: procedure.query || "",
        wikihowCandidate: procedure.wikihowCandidate || "",
      },
      outputs: {
        intent: procedure.intent,
        confidence: procedure.confidence,
        formalizedObject: procedure.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, procedure, formalizationContext);
  }
  // Issue #444: a bare follow-up that asks for the concrete steps ("Can you
  // give me specific instructions?") carries no "how to" lead-in of its own, so
  // tryProceduralHowTo above returned null. Rebind it to the procedure recovered
  // from the prior turn instead of letting it fall to web search / the unknown
  // opener. Mirrors the procedural_how_to_followup slot in the Rust dispatch
  // table, which sits right after procedural_how_to.
  steps.push({ step: "invoke_tool", detail: "procedural_how_to_followup" });
  const procedureFollowup = await tryProceduralHowToFollowup(prompt, language, history, preferences);
  if (procedureFollowup) {
    events.push(`handler:${procedureFollowup.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryProceduralHowToFollowup" });
    toolCalls.push({
      tool: "procedural_how_to",
      inputs: {
        prompt,
        language,
        query: procedureFollowup.query || "",
        wikihowCandidate: procedureFollowup.wikihowCandidate || "",
      },
      outputs: {
        intent: procedureFollowup.intent,
        confidence: procedureFollowup.confidence,
        formalizedObject: procedureFollowup.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, procedureFollowup, formalizationContext);
  }
  steps.push({ step: "invoke_tool", detail: "document_originality_check" });
  const originality = tryDocumentOriginalityCheck(prompt, language);
  if (originality) {
    events.push(`handler:${originality.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryDocumentOriginalityCheck" });
    if (Array.isArray(originality.attachments)) {
      for (const attachment of originality.attachments) {
        toolCalls.push({
          tool: "read_local_file",
          inputs: { name: attachment },
          outputs: { intent: originality.intent },
        });
      }
    }
    toolCalls.push({
      tool: "web_search",
      inputs: {
        prompt,
        language,
        query: originality.query || "",
        queryKind: "document_originality_check",
      },
      outputs: {
        intent: originality.intent,
        confidence: originality.confidence,
        formalizedObject: originality.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, originality, formalizationContext);
  }
  steps.push({ step: "invoke_tool", detail: "web_search" });
  const webSearch = await tryWebSearch(prompt, language);
  if (webSearch) {
    events.push(`handler:${webSearch.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryWebSearch" });
    toolCalls.push({
      tool: "web_search",
      inputs: { prompt, language, query: webSearch.query || "" },
      outputs: {
        intent: webSearch.intent,
        confidence: webSearch.confidence,
        formalizedObject: webSearch.formalizedObject || "",
      },
    });
    return finalize(events, steps, toolCalls, webSearch, formalizationContext);
  }
  steps.push({ step: "invoke_tool", detail: "wikipedia_lookup" });
  const wiki = await tryWikipediaLookup(prompt, language, preferences);
  if (wiki) {
    events.push(`handler:${wiki.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryWikipediaLookup" });
    toolCalls.push({
      tool: "wikipedia_lookup",
      inputs: {
        prompt,
        language,
        guessProbability: numericPreference(
          preferences.guessProbability,
          0.8,
          0,
          1,
        ),
      },
      outputs: { intent: wiki.intent, confidence: wiki.confidence },
    });
    return finalize(events, steps, toolCalls, wiki, formalizationContext);
  }
  toolCalls.push({
    tool: "wikipedia_lookup",
    inputs: { prompt, language },
    outputs: { intent: "no_match" },
  });
  // Issue #69: "who is X" prompts that were not resolved by the local
  // knowledge base or Wikipedia should still return a question-typed response
  // (not "unknown") and offer a typo correction when the entity name is close
  // to a known variant.
  const whoIs = tryWhoIsQuestion(prompt);
  if (whoIs) {
    events.push(`handler:${whoIs.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryWhoIsQuestion" });
    return finalize(events, steps, toolCalls, whoIs, formalizationContext);
  }
  // Route literal and seeded semantic shell requests before unknown fallback.
  const terminal = claimRouteAdmits("tryTerminalCommand", prompt) ? tryTerminalCommand(prompt, language, preferences) : null;
  if (terminal) {
    events.push(`handler:${terminal.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryTerminalCommand" });
    return finalize(events, steps, toolCalls, terminal, formalizationContext);
  }
  const howItWorks = tryHowItWorks(prompt, history);
  if (howItWorks) return finalizeInlineHandler(events, steps, toolCalls, howItWorks, "tryHowItWorks", formalizationContext);

  if (isTargetlessProgramModification(normalized)) {
    events.push("handler:ambiguous_modification_clarification");
    steps.push({ step: "dispatch_handler", detail: "clarifyProgramModificationTarget" });
    return finalize(events, steps, toolCalls, {
      intent: "ambiguous_modification_clarification",
      content: answerFor("ambiguous_modification_clarification", language),
      confidence: 0.9,
      evidence: ["meaning:program_modification_reference", `language:${language}`],
    }, formalizationContext);
  }

  const learnedResearch = recallAssociativeResearch(prompt, memory);
  if (learnedResearch) {
    events.push("handler:associative_research_memory");
    steps.push({ step: "dispatch_handler", detail: "recallAssociativeResearch" });
    toolCalls.push({
      tool: "associative_memory", inputs: { prompt, associationId: associativeResearchId(prompt) },
      outputs: { intent: learnedResearch.intent, confidence: learnedResearch.confidence } });
    return finalize(events, steps, toolCalls, learnedResearch, formalizationContext);
  }
  const bareTermQuery = extractUnresolvedBareTermSearchQuery(prompt);
  if (bareTermQuery) {
    steps.push({ step: "invoke_tool", detail: "web_search_unresolved_bare_term" });
    const bareTermSearch = await runWebSearchQuery(bareTermQuery, language, "unresolved_bare_term");
    if (bareTermSearch) {
      events.push(`handler:${bareTermSearch.intent}`);
      steps.push({ step: "dispatch_handler", detail: "tryUnresolvedBareTermWebSearch" });
      toolCalls.push({
        tool: "web_search", inputs: { prompt, language, query: bareTermQuery },
        outputs: {
          intent: bareTermSearch.intent, confidence: bareTermSearch.confidence,
          formalizedObject: bareTermSearch.formalizedObject || "" },
      });
      return finalize(events, steps, toolCalls, bareTermSearch, formalizationContext);
    }
  }
  if (meta.goal.startsWith("synthesize") || meta.goal === "decompose") {
    meta = await metaReasonTurn(prompt, language, preferences);
    formalizationContext.meta = meta;
    steps.push({ step: "meta_discover", detail: `${meta.goal} ${meta.status} after ${meta.lookups.length} lookup round(s)`, derivation: meta.derivationLino });
    const discovered = solverMetaProjection(metaAnswer(meta));
    if (discovered) {
      events.push("handler:meta_reasoner");
      return finalize(events, steps, toolCalls, discovered, formalizationContext);
    }
  }
  steps.push({ step: "invoke_tool", detail: "unknown_intent_research" });
  const researchedUnknown = await runWebSearchQuery(prompt, language, "unknown_intent_research", preferences);
  const researchedSources = researchedUnknown?.diagnostics?.fused;
  if (Array.isArray(researchedSources) && researchedSources.length > 0) {
    const associationId = associativeResearchId(prompt);
    researchedUnknown.evidence = [
      ...(researchedUnknown.evidence || []),
      `associative_research:learned:${associationId}`,
      `associative_research:sources:${researchedSources.length}`,
    ];
    researchedUnknown.memoryOperation = associativeResearchMemoryOperation(prompt, researchedUnknown);
    events.push("handler:unknown_intent_research");
    steps.push({ step: "dispatch_handler", detail: "researchUnknownIntent" });
    toolCalls.push({
      tool: "web_search", inputs: { prompt, language, query: prompt, queryKind: "unknown_intent_research" },
      outputs: { intent: researchedUnknown.intent, confidence: researchedUnknown.confidence, associationId } });
    return finalize(events, steps, toolCalls, researchedUnknown, formalizationContext);
  }
  const comparisonGap = tryFactComparisonGap(prompt, normalized);
  if (comparisonGap) return finalizeInlineHandler(events, steps, toolCalls, comparisonGap, "tryFactComparisonGap", formalizationContext);
  events.push("fallback:unknown");
  steps.push({ step: "fallback", detail: "unknown" });
  return finalize(events, steps, toolCalls, {
    intent: "unknown",
    content: unknownAnswerWithVariation(prompt, language),
    confidence: 0.1,
    evidence: ["fallback:unknown", `language:${language}`],
  }, formalizationContext);
}
function finalize(events, steps, toolCalls, answer, formalizationContext) {
  // Issue #1173 R5: no answer may be the canned search-machinery description.
  answer = guardCannedWebSearchAnswer(
    answer,
    (formalizationContext && formalizationContext.language) || "en",
  );
  const interpretations = collectInterpretations(formalizationContext, answer);
  answer = applyVisibleInterpretations(answer, interpretations);
  applyResolvedFormalization(events, steps, formalizationContext, answer);
  const evidence = Array.isArray(answer.evidence) ? answer.evidence : [];
  const projection = deformalizeProjection(formalizationContext, answer);
  events.push(`deformalize:${projection.tuple}:${projection.intent}`);
  // `detail` keeps the symbolic projection summary (with the ⇒ glyph) for the
  // diagnostics panel; `answer` carries the clean composed answer so the
  // human-readable thinking preview can show "Compose the answer: …" (issue
  // #488) without leaking the tuple.
  const answerFirstLine = String(answer.content || "").split(/\r?\n/, 1)[0] || "";
  steps.push({
    step: "deformalize",
    detail: projection.summary,
    projection,
    answer: answerFirstLine,
  });
  const trace = events.map((event) => `trace:${event}`);
  const result = {
    intent: answer.intent,
    content: answer.content,
    confidence: answer.confidence,
    evidence: [...evidence, ...trace],
    steps: withThinkingLevels(steps),
    toolCalls, derivationId: stableBehaviorRuleId("answer", answer.content), // #1184 R1: SymbolicAnswer::derivation_id
  };
  if (formalizationContext && formalizationContext.initial) result.solverEvents = solverEventLog(formalizationContext.initial.raw, answer); // R1013
  if (formalizationContext && formalizationContext.meta) {
    result.derivation = formalizationContext.meta.derivationLino;
  }
  if (answer.iframeUrl) {
    result.iframeUrl = answer.iframeUrl;
  }
  if (answer.diagnostics) {
    result.diagnostics = answer.diagnostics;
  }
  if (answer.programExecution) {
    result.programExecution = answer.programExecution; // R1013
  }
  // Issue #529: carry a natural-language memory write (append/substitution) out
  // to the app so it can apply the read+write transform to persistent storage.
  if (answer.memoryOperation) {
    result.memoryOperation = answer.memoryOperation;
  }
  return result;
}
// Issue #1138 B9, plan 09 leaves 13-15: the browser worker resolves its
// specialized handlers under the same name-keyed vocabulary the seed owns.
// `workerHandlerAliases` holds the slugs whose worker-side implementation keeps
// a genuinely different historical name; every other implemented slug follows
// the mechanical `try` + CamelCase convention. A slug resolving to `null` is a
// row the worker does not run in the synchronous handler phase — the native
// surface runs it, or the browser answers it in the async phase the seed's
// `phase async` note declares. A `@name` value is a per-turn context binding.
function workerHandlerRegistryDefinition() {
  const workerHandlerAliases = {
    conversation_memory: "tryMemoryWrite",
    summarization: "trySummarizeConversation",
    brainstorming: "tryBrainstormingRequest",
    roleplay: "tryRoleplayRequest",
    coreference: "tryCoreferenceFactLookup",
    shell_command_transform: "tryShellCommandTransform",
    software_project: "trySoftwareProjectRequest",
    who_is: "tryWhoIsQuestion",
  };
  const workerHandlers = {
    conversation_control: null, // local recall and behavior rules run above the table
    agentic_continuation: "tryAgenticContinuation", // seed rule interpreter; the agentic runtime resumes above the table
    exact_memory_query: "tryExactMemoryQuery",
    memory_program: "tryMemoryProgram",
    memory_program_gap: "tryMemoryProgramGap",
    current_dialogue_fact_checking: "tryCurrentDialogueFactChecking",
    link_native_synthesis: "tryLinkNativeSynthesis",
    historical: "tryHistorical",
    write_program_coreference: "@writeProgram",
    program_blueprint_from_prompt: "tryProgramBlueprintFromPrompt",
    write_program_concrete: "@writeProgram",
    legality_warning: "tryLegalityWarning",
    page_query_text: "tryPageQueryText",
    http_fetch: null, // phase async
    url_navigate: null, // phase async
    github_repository_traffic: "tryGithubRepositoryTraffic",
    document_originality_check: "tryDocumentOriginalityCheck",
    repository_lineage: null, // native: git history of the working repository (#1180 R10)
    formalization_request: "tryFormalizationRequest",
    product_search: "tryProductSearch",
    web_search: null, // phase async
    learn_from_source: null, // phase async
    research_comparison_table: "tryResearchComparisonTable",
    research_result_followup: "tryResearchResultFollowup",
    docs_method_explanation: "tryDocsMethodExplanation",
    procedural_how_to: null, // phase async: the how-to guide fetches its source registry
    procedural_how_to_followup: null, // phase async
    conversation_memory: workerHandlerAliases.conversation_memory,
    software_project_followup: "trySoftwareProjectFollowup",
    summarization_text: "trySummarizationText",
    summarization: workerHandlerAliases.summarization,
    verifiable_task: "tryVerifiableTask",
    text_manipulation: "tryTextManipulation",
    brainstorm_composition: "tryBrainstormComposition",
    brainstorming: workerHandlerAliases.brainstorming,
    conversation_topic: null, // inline opener machinery in the conversation module
    advice_request: "tryAdviceRequest",
    fact_lookup: "tryFactLookup",
    coreference: workerHandlerAliases.coreference,
    roleplay: workerHandlerAliases.roleplay,
    creative_writing: "tryCreativeWritingRequest",
    translation: "tryTranslation", // async: runs inline ahead of the synchronous table
    text_rewrite: "tryTextRewrite",
    response_language_followup: "tryResponseLanguageFollowup", // async: replays the previous turn inline
    capabilities: "tryCapabilities",
    planning_request: "tryPlanningRequest",
    calendar_reasoning: "tryCalendarReasoning",
    calendar_create_event: "tryCalendarCreateEvent",
    compound_interest: "tryCompoundInterest",
    word_problem: "tryWordProblem",
    numeric_list: "tryNumericList",
    shell_command_transform: workerHandlerAliases.shell_command_transform,
    code_debugging: "tryCodeDebugging",
    regex_synthesis: "tryRegexSynthesis",
    sql_synthesis: "trySqlSynthesis",
    shell_command_compose: "tryShellCommandCompose",
    number_constraint_reasoning: "tryNumberConstraintReasoning",
    code_explanation: "tryCodeExplanation",
    code_review: "tryCodeReview",
    test_generation: "tryTestGeneration",
    code_refactoring: "tryCodeRefactoring",
    format_conversion: "tryFormatConversion",
    program_synthesis: "tryProgramSynthesis",
    arithmetic: "tryArithmetic",
    statistics: "tryStatistics",
    javascript_execution: "tryJavaScriptExecution",
    definition_merge: "@definitionMerge",
    triz_resolution: "tryTrizResolution",
    concept_lookup: "tryConceptLookup",
    who_is: workerHandlerAliases.who_is,
    how_it_works: "tryHowItWorks", // runs inline after the async lookups, so online sources answer first
    meta_explanation: "tryMetaExplanation",
    network_query: "tryNetworkSnapshot", // the snapshot branch; concept introspection and the user filter stay native
    execution_failure: "tryExecutionFailure",
    installation_conversion: "tryInstallationConversion",
    write_script: "tryWriteScript",
    document_generation_plan: "tryDocumentGenerationPlan",
    software_project: workerHandlerAliases.software_project,
    software_project_request: "trySoftwareProjectRequest",
    algorithm: "tryAlgorithm",
    source_refresh: "trySourceRefresh",
    source_conflict: "trySourceConflict",
    clarification: "tryClarification", // seed rule interpreter (handler-rules.lino)
    punctuation_only_prompt: "tryPunctuationOnlyPrompt", // seed rule interpreter, run inline early
    ill_formed: "tryIllFormed", // seed rule interpreter
    physical_action_question: "tryPhysicalActionQuestion", // seed rule interpreter
    kupi_slona: "tryKupiSlona",
    shell_refusal: "tryShellRefusal",
    proof_request: "tryProofRequest",
    opinion_question: "tryOpinionQuestion", // seed rule interpreter
    unit_conversion: "tryUnitConversion",
    incompatible_units: "tryIncompatibleUnits",
  };
  return { workerHandlers, workerHandlerAliases };
}

const WORKER_HANDLER_REGISTRY = workerHandlerRegistryDefinition();

// The registry must stay an exact permutation of the seed's precedence rows:
// the same load-time assertion the native dispatcher runs, on the browser side
// (plan 09 leaves 13-15).
function assertWorkerRegistryPermutation(declared) {
  const registered = Object.keys(WORKER_HANDLER_REGISTRY.workerHandlers);
  const compare = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
  const sortedDeclared = declared.slice().sort(compare);
  const sortedRegistered = registered.slice().sort(compare);
  for (let i = 0; i < sortedDeclared.length; i += 1) {
    if (sortedDeclared[i] !== sortedRegistered[i]) {
      throw new Error(
        `worker handler registry must be an exact permutation of the seed: seed has \
"${sortedDeclared[i]}", registry has "${sortedRegistered[i]}" at position ${i}`,
      );
    }
  }
}

function installWorkerHandlerRegistry(seedText) {
  if (typeof seedText !== "string" || seedText.length === 0) return;
  if (typeof self.FormalAiSeed !== "object" || self.FormalAiSeed === null) return;
  const root = self.FormalAiSeed.parse(seedText);
  const section = root && root.name === "handler_precedence"
    ? root
    : ((root && root.children) || []).find((child) => child.name === "handler_precedence");
  if (!section || !Array.isArray(section.children)) return;
  // Plan 09 leaf 41: rows are `handler <name>` blocks; precedence is their
  // `rank` link, so the permutation asserts row ids, not raw child order.
  assertWorkerRegistryPermutation(
    section.children.filter((child) => child.name === "handler" && child.id).map((child) => child.id),
  );
  for (const [slug, implementation] of Object.entries(WORKER_HANDLER_REGISTRY.workerHandlers)) {
    if (typeof implementation === "string" && implementation.startsWith("@")) {
      if (implementation.length < 2) {
        throw new Error(`worker handler ${slug} declares an empty context binding`);
      }
    } else if (implementation !== null && typeof self[implementation] !== "function") {
      throw new Error(`worker handler ${slug} resolves to missing implementation ${implementation}`);
    }
  }
  for (const [slug, implementation] of Object.entries(WORKER_HANDLER_REGISTRY.workerHandlerAliases)) {
    if (WORKER_HANDLER_REGISTRY.workerHandlers[slug] !== implementation) {
      throw new Error(`worker handler alias ${slug} disagrees with the registry entry`);
    }
  }
}

let seedLoaded = false;
let seedLoadPromise = null;
async function loadSeed() {
  if (seedLoaded) return;
  if (seedLoadPromise) return seedLoadPromise;
  seedLoadPromise = (async () => {
    if (typeof self.FormalAiSeed !== "object" || self.FormalAiSeed === null) {
      seedLoaded = true;
      return;
    }
    try {
      const seed = await self.FormalAiSeed.loadAll();
      SEED_RAW = (seed && seed.raw) || {};
      installBrowserHandlerPrecedence(seed && seed.browserHandlerPrecedence);
      installWorkerHandlerRegistry(SEED_RAW["seed/handler-precedence.lino"]);
      await hydrateLinoSeedAndSourceCaches(SEED_RAW);
      if (seed && seed.responses) {
        const merged = {};
        const intents = new Set(
          Object.keys(MULTILINGUAL_ANSWERS).concat(Object.keys(seed.responses)),
        );
        intents.forEach((intent) => {
          const base = MULTILINGUAL_ANSWERS[intent] || {};
          const next = seed.responses[intent] || {};
          const byLanguage = {};
          const langs = new Set(Object.keys(base).concat(Object.keys(next)));
          langs.forEach((language) => {
            const incoming = next[language];
            if (incoming !== undefined) {
              byLanguage[language] = normalizeEntry(incoming, intent);
            } else {
              byLanguage[language] = normalizeEntry(base[language], intent);
            }
          });
          merged[intent] = byLanguage;
        });
        MULTILINGUAL_ANSWERS = merged;
      }
      if (Array.isArray(seed && seed.concepts) && seed.concepts.length > 0) {
        CONCEPTS = seed.concepts;
      }
      if (
        Array.isArray(seed && seed.conceptContexts) &&
        seed.conceptContexts.length > 0
      ) {
        CONCEPT_CONTEXTS = seed.conceptContexts;
      }
      if (Array.isArray(seed && seed.facts) && seed.facts.length > 0) {
        FACTS = seed.facts;
        warmFactCacheFromSeed();
      }
      if (Array.isArray(seed && seed.projects) && seed.projects.length > 0) {
        PROJECTS = seed.projects;
      }
      if (
        seed &&
        seed.brainstormSeeds &&
        Array.isArray(seed.brainstormSeeds.triggers) &&
        seed.brainstormSeeds.triggers.length > 0
      ) {
        BRAINSTORM_SEEDS = seed.brainstormSeeds;
      }
      if (
        seed &&
        seed.personas &&
        Array.isArray(seed.personas.triggers) &&
        seed.personas.triggers.length > 0
      ) {
        PERSONA_SEEDS = seed.personas;
      }
      if (
        seed &&
        seed.coreferenceSeeds &&
        Array.isArray(seed.coreferenceSeeds.pronouns) &&
        seed.coreferenceSeeds.pronouns.length > 0
      ) {
        COREFERENCE_SEEDS = seed.coreferenceSeeds;
      }
      if (Array.isArray(seed && seed.tools) && seed.tools.length > 0) {
        TOOLS = seed.tools;
      }
      if (seed && seed.agentInfo && typeof seed.agentInfo === "object") {
        AGENT_INFO = Object.assign({}, AGENT_INFO, seed.agentInfo);
      }
      if (
        Array.isArray(seed && seed.languageRules) &&
        seed.languageRules.length > 0
      ) {
        LANGUAGE_RULES = seed.languageRules
          .filter((rule) => rule && rule.language && rule.start && rule.end)
          .map((rule) => ({
            language: rule.language,
            // Issue #706: carry the script, markers and fallback flags so the
            // JS detector stays a mirror of `src/language.rs` for every
            // language the seed registry declares.
            script: rule.script || "",
            start: Number(rule.start),
            end: Number(rule.end),
            markers: Array.isArray(rule.markers) ? rule.markers : [],
            fallback: rule.fallback === true,
            alphabeticOnly: rule.alphabeticOnly === true,
            sourceHost: rule.sourceHost || "",
          }));
      }
      if (
        Array.isArray(seed && seed.promptPatterns) &&
        seed.promptPatterns.length > 0
      ) {
        PROMPT_PATTERNS = seed.promptPatterns;
      }
      if (
        seed &&
        seed.intentRouting &&
        Array.isArray(seed.intentRouting.intents) &&
        seed.intentRouting.intents.length > 0
      ) {
        INTENT_ROUTING = {
          intents: seed.intentRouting.intents,
          articlePrefixes:
            seed.intentRouting.articlePrefixes &&
            seed.intentRouting.articlePrefixes.length
              ? seed.intentRouting.articlePrefixes
              : INTENT_ROUTING.articlePrefixes,
          tracePrefixes:
            seed.intentRouting.tracePrefixes && seed.intentRouting.tracePrefixes.length
              ? seed.intentRouting.tracePrefixes
              : INTENT_ROUTING.tracePrefixes,
        };
      }
    } catch (_error) {
      // Keep fallback tables on error.
    } finally {
      seedLoaded = true;
      seedLoadPromise = null;
    }
  })();
  return seedLoadPromise;
}
let initPromise = null;
// Issue #934: a failed WASM instantiation must surface as a visible
// "engine unavailable" state, never as a silent switch to the JavaScript
// mirror. The mirror stays reachable only behind an explicit diagnostic
// override (?jsfallback=1, written by the app from a dev-only setting) so
// local development can inspect it, and every answer records which engine
// produced it in its trace evidence.
let engineError = "";
let diagnosticJsFallback = false;
try {
  diagnosticJsFallback = /[?&]jsfallback=1\b/.test(
    String(self.location && self.location.search || ""),
  );
} catch (_overrideError) {
  diagnosticJsFallback = false;
}
async function init() {
  if (wasm !== undefined) return;
  if (initPromise) return initPromise;
  initPromise = (async () => {
    await loadSeed();
    try {
      const source = await fetch(withAssetVersion("formal_ai_worker.wasm"));
      if (!source || source.ok === false) {
        throw new Error(`wasm fetch failed: ${source ? source.status : "no response"}`);
      }
      const bytes = await source.arrayBuffer();
      const module = await WebAssembly.instantiate(bytes, {});
      wasm = module.instance.exports;
      // Plan 09 leaf 13: re-check the registry through the same parser the
      // native solver reads the seed with, so the two surfaces cannot drift.
      const seedText = SEED_RAW["seed/handler-precedence.lino"];
      if (typeof seedText === "string" && seedText.length > 0) {
        const declared = String(
          wasmTextCall("engine_handler_precedence", seedText) || "",
        ).split("\n").filter(Boolean);
        assertWorkerRegistryPermutation(declared);
      }
    } catch (error) {
      wasm = null;
      engineError = String((error && error.message) || error);
      mode = diagnosticJsFallback
        ? "js fallback (diagnostic override)"
        : "engine unavailable";
    }
    postMessage({
      kind: "ready",
      mode,
      engineError: wasm ? "" : engineError,
      seed: {
        responseIntents: Object.keys(MULTILINGUAL_ANSWERS),
        conceptCount: CONCEPTS.length,
        conceptContextCount: CONCEPT_CONTEXTS.length,
        factCount: FACTS.length,
        projectCount: PROJECTS.length,
        brainstormCategoryCount: BRAINSTORM_SEEDS.categories.length,
        personaCount: PERSONA_SEEDS.personas.length,
        toolCount: TOOLS.length,
        files: Object.keys(SEED_RAW),
      },
    });
    if (wasm == null && !diagnosticJsFallback) {
      // Loud, not silent: the app turns this into a visible error banner.
      postMessage({ kind: "engine_unavailable", error: engineError });
    }
  })();
  return initPromise;
}
self.onmessage = async (event) => {
  await init();
  const data = event.data || {};
  if (data.kind === "browser_runtime_probe") {
    postMessage({
      kind: "browser_runtime_probe",
      requestId: data.requestId,
      probe: browserExecutionProbe("python"),
    });
    return;
  }
  if (data.kind === "browser_runtime_load") {
    try {
      await loadBrowserPythonRuntime();
      postMessage({
        kind: "browser_runtime_load",
        requestId: data.requestId,
        probe: browserExecutionProbe("python"),
      });
    } catch (error) {
      postMessage({
        kind: "browser_runtime_load",
        requestId: data.requestId,
        probe: browserExecutionProbe("python"),
        error: String(error && error.message || error),
      });
    }
    return;
  }
  if (data.kind === "seed_dump") {
    postMessage({
      kind: "seed_dump",
      requestId: data.requestId,
      raw: SEED_RAW,
      responses: MULTILINGUAL_ANSWERS,
      concepts: CONCEPTS,
      conceptContexts: CONCEPT_CONTEXTS,
      facts: FACTS,
      projects: PROJECTS,
      brainstormSeeds: BRAINSTORM_SEEDS,
      personas: PERSONA_SEEDS,
      tools: TOOLS,
      agentInfo: AGENT_INFO,
      languageRules: LANGUAGE_RULES,
      promptPatterns: PROMPT_PATTERNS,
    });
    return;
  }
  const prompt = data.prompt || "";
  const history = Array.isArray(data.history) ? data.history : [];
  const prefs = (data.prefs && typeof data.prefs === "object") ? data.prefs : {};
  const userContext =
    data.userContext && typeof data.userContext === "object"
      ? data.userContext
      : {};
  // Issue #529: the app passes a snapshot of every searchable persistent-memory
  // value so a natural-language substitution can report how many occurrences it
  // rewrites; the worker stays pure and the app applies the write.
  const memory = Array.isArray(data.memory) ? data.memory : [];
  const memoryEvents = Array.isArray(data.memoryEvents) ? data.memoryEvents : [];
  // Issue #934: engine provenance travels with every answer, and an engine
  // that failed to load refuses to answer instead of quietly degrading.
  const engine = wasm
    ? "wasm"
    : diagnosticJsFallback
      ? "js (diagnostic override)"
      : "unavailable";
  if (wasm == null && !diagnosticJsFallback) {
    postMessage({
      kind: "message",
      requestId: data.requestId,
      intent: "engine_unavailable",
      content:
        "The formal-ai WebAssembly engine could not be loaded, so this demo cannot answer."
        + (engineError ? ` Loader error: ${engineError}.` : "")
        + " Reload the page to retry."
        + " Developers can enable the diagnostic JavaScript fallback in Settings to inspect the mirror implementation.",
      confidence: 0,
      evidence: [`engine:unavailable${engineError ? `:${engineError}` : ""}`],
      steps: [],
      toolCalls: [],
      iframeUrl: null,
      diagnostics: null,
      memoryOperation: null,
      runtimeOffer: null,
      engine,
    });
    return;
  }
  const answer = explainWorkerDerivation(prompt, memoryEvents) || finalizeWorkerDerivation( // #1184 R1184-9
    await executeBrowserCodeRequest(prompt) || attachUserContext(
      await solve(prompt, history, prefs, userContext, memory, { memoryEvents }), userContext));
  postMessage({
    kind: "message",
    requestId: data.requestId,
    intent: answer.intent,
    content: answer.content,
    confidence: answer.confidence,
    evidence: [...(answer.evidence || []), `engine:${engine}`],
    steps: answer.steps,
    toolCalls: answer.toolCalls,
    iframeUrl: answer.iframeUrl || null,
    diagnostics: answer.diagnostics || null,
    memoryOperation: answer.memoryOperation || null,
    derivationRecord: answer.derivationRecord || null,
    runtimeOffer: answer.runtimeOffer || null,
    engine,
  });
};
init();
