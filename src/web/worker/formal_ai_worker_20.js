// Worker module 21 of 21. Loaded by ../formal_ai_worker.js.
function objectForFormalization(prompt, normalized, match) {
  // For search-style ops we extract the explicit query the same way the web
  // search handler does. For other ops we keep the prompt body that follows
  // the detected verb so the tuple shows what the user is asking about.
  const op = match && match.op;
  if (op === "OP:search" || op === "OP:lookup") {
    const query = extractWebSearchQuery(prompt, normalized);
    if (query) return query;
  }
  if (op === "OP:procedure") {
    const task = extractProceduralHowToTask(normalized);
    if (task) return task.task;
  }
  const haystack = String(normalized || "").toLowerCase();
  for (const { verb } of FORMALIZATION_VERBS) {
    if (haystack.startsWith(verb + " ")) {
      return cleanSearchQuery(normalized.slice(verb.length));
    }
  }
  if (match && typeof match.objectText === "string") {
    return cleanSearchQuery(match.objectText);
  }
  return cleanSearchQuery(normalized || "");
}

function virtualObjectId(term) {
  const trimmed = String(term || "").trim();
  if (!trimmed) return "?";
  return `?${trimmed}`;
}

function formatFormalizationTuple(parts) {
  return `(${parts.filter(Boolean).join(" ")})`;
}

function buildFormalization(prompt, normalized) {
  const match = detectFormalizationMatch(prompt, normalized);
  if (!match || match.ambiguous) {
    const fallback = normalized || "(empty)";
    return {
      raw: String(prompt || ""),
      subject: "@USER",
      verb: "OP:express",
      object: virtualObjectId(fallback),
      tuple: formatFormalizationTuple(["@USER", "OP:express", virtualObjectId(fallback)]),
      needsClarification: Boolean(match && match.ambiguous),
      suggestions: match && match.suggestions ? match.suggestions : [],
      interpretations: [],
    };
  }
  const object = objectForFormalization(prompt, normalized, match);
  return {
    raw: String(prompt || ""),
    subject: "@USER",
    verb: match.op,
    object: virtualObjectId(object),
    tuple: formatFormalizationTuple(["@USER", match.op, virtualObjectId(object)]),
    interpretations: match.interpretations || [],
  };
}

function formalizationDetail(formalization) {
  if (!formalization || typeof formalization !== "object") {
    return String(formalization || "(empty)");
  }
  const arrow = formalization.raw && formalization.tuple ? " -> " : "";
  return `${formalization.raw || ""}${arrow}${formalization.tuple || ""}`.trim();
}

function formalizationClarificationMessage(formalization, language) {
  const suggestions = Array.isArray(formalization && formalization.suggestions)
    ? formalization.suggestions
    : [];
  const rendered = suggestions.length > 0
    ? suggestions.map((item) => `"${item}"`).join(", ")
    : "one of the known commands";
  if (language === "ru") {
    return `Не уверен, как интерпретировать этот запрос. Вы имели в виду ${rendered}?`;
  }
  if (language === "zh") {
    return `我不确定如何解释这个请求。你是指 ${rendered} 吗？`;
  }
  if (language === "hi") {
    return `मुझे पक्का नहीं है कि इस अनुरोध को कैसे समझूं। क्या आपका मतलब ${rendered} था?`;
  }
  return `I am not sure how to interpret that request. Did you mean ${rendered}?`;
}

// Once a handler resolves the search object to a concrete entity, this helper
// folds the resolved id back into the original formalization so the trace
// shows the canonical (@USER OP:search Q<id>) tuple alongside the placeholder.
function resolveFormalizationWithId(formalization, resolvedId) {
  if (!formalization || !resolvedId) return null;
  const next = Object.assign({}, formalization, {
    object: resolvedId,
    tuple: formatFormalizationTuple([
      formalization.subject || "@USER",
      formalization.verb || "OP:express",
      resolvedId,
    ]),
  });
  return next;
}

async function solve(prompt, history, prefs, userContext = {}) {
  const preferences = prefs || {};
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
  const language = detectLanguage(prompt);
  events.push(`language:${language}`);
  steps.push({ step: "detect_language", detail: language });
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

  if (isPunctuationOnlyPrompt(prompt)) {
    events.push("handler:clarification");
    events.push(`clarification:punctuation_only:${String(prompt).trim()}`);
    steps.push({ step: "dispatch_handler", detail: "tryPunctuationOnlyPrompt" });
    const trimmed = String(prompt).trim();
    return finalize(events, steps, toolCalls, {
      intent: "clarification",
      content: `I received only punctuation (\`${trimmed}\`). What would you like me to do next?`,
      confidence: 0.8,
      evidence: [
        "handler:clarification",
        "clarification:punctuation_only",
        `language:${language}`,
      ],
    }, formalizationContext);
  }

  const translation = await tryTranslation(prompt, normalized);
  if (translation) {
    events.push(`handler:${translation.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryTranslation" });
    return finalize(events, steps, toolCalls, translation, formalizationContext);
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

  const capabilities = tryCapabilities(prompt, normalized, preferences, history);
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

  if (isGreetingPrompt(normalized, prompt)) {
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
    const temperature = numericPreference(preferences.temperature, 0.7, 0, 1);
    const randomize = preferences.greetingVariations !== false && temperature > 0;
    return finalize(events, steps, toolCalls, {
      intent: "assistant_free_time",
      content: answerFor("assistant_free_time", language, { randomize: randomize }),
      confidence: 1.0,
      evidence: [
        "rule:assistant_free_time",
        `language:${language}`,
        `variation:${randomize ? "random" : "canonical"}`,
        `temperature:${temperature.toFixed(2)}`,
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
  if (isIdentityPrompt(normalized, prompt)) {
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
  const syncHandlers = [
    { name: "tryLinkNativeSynthesis", run: () => tryLinkNativeSynthesis(prompt, normalized) },
    { name: "tryHistorical", run: () => tryHistorical(prompt, history) },
    {
      name: "tryResearchComparisonTable",
      run: () => tryResearchComparisonTable(prompt, normalized, history),
    },
    {
      name: "tryResearchResultFollowup",
      run: () => tryResearchResultFollowup(prompt, normalized, history),
    },
    { name: "tryBrainstormingRequest", run: () => tryBrainstormingRequest(prompt, normalized) },
    { name: "tryRoleplayRequest", run: () => tryRoleplayRequest(prompt, normalized) },
    { name: "tryKupiSlona", run: () => tryKupiSlona(prompt, normalized) },
    {
      name: "tryCalendarReasoning",
      run: () => tryCalendarReasoning(prompt, normalized, userContext),
    },
    {
      name: "tryProofRequest",
      run: () => tryProofRequest(prompt, normalized, language),
    },
    { name: "tryIncompatibleUnits", run: () => tryIncompatibleUnits(prompt, normalized) },
    // Issue #135/#386: a Playwright-script request is more specific than the
    // generalized write_program recognizer. Since #386 taught
    // writeProgramParameters to recognize a program_kind ("скрипт"/"script")
    // requested by a program_request verb ("написать"/"write"), a bare
    // "напиши … playwright скрипт" now also looks like a write_program with no
    // task — so the Playwright handler must win first. This mirrors the Rust
    // dispatch, where try_playwright_script runs ahead of the SPECIALIZED_HANDLERS
    // group that owns write_program (src/solver.rs).
    {
      name: "tryPlaywrightScript",
      run: () => tryPlaywrightScript(prompt, preferences, language),
    },
    {
      name: "tryWriteProgramCoreference",
      run: () => {
        const hit = writeProgram();
        return hit &&
          hit.intent === "write_program" &&
          hit.evidence?.some((link) => link.startsWith("write_program_coreference_rewrite:"))
          ? hit
          : null;
      },
    },
    {
      name: "tryProgramBlueprintFromPrompt",
      run: () =>
        tryProgramBlueprintFromPrompt(
          prompt,
          responseLanguage,
          preferences.blueprintComposition,
        ),
    },
    { name: "tryTextManipulation", run: () => tryTextManipulation(prompt, normalized, history) },
    // Issue #395: a concrete "<operation> these numbers in <language>, give me
    // the code and the result" must produce generated code plus the
    // deterministically-computed result. It runs before program synthesis and
    // arithmetic (either of which would otherwise claim the numeric prompt),
    // mirroring the Rust dispatch where numeric_list precedes arithmetic and
    // program_synthesis.
    { name: "tryNumericList", run: () => tryNumericList(prompt, history) },
    { name: "tryProgramSynthesis", run: () => tryProgramSynthesis(prompt, normalized) },
    {
      name: "tryCompoundInterest",
      run: () => tryCompoundInterest(prompt, normalized, history),
    },
    { name: "tryArithmetic", run: () => tryArithmetic(prompt) },
    { name: "tryJavaScriptExecution", run: () => tryJavaScriptExecution(prompt) },
    // Issue #423: README/install-guide to shell/PowerShell conversion is a
    // narrower script request than generic program synthesis. Keep it ahead of
    // writeProgram so "convert this README to a script" preserves the source
    // instructions instead of producing an unrelated starter script.
    { name: "tryInstallationConversion", run: () => tryInstallationConversion(prompt, normalized) },
    {
      name: "tryWriteProgramConcrete",
      run: () => {
        const hit = writeProgram();
        return hit && hit.intent === "write_program" ? hit : null;
      },
    },
    {
      name: "tryDefinitionMerge",
      run: () => tryDefinitionMerge(prompt, { allowPlainConcept: autoDefinitionFusion }),
    },
    // Issue #341: keep a "test it / run it / show me" step inside the active
    // software-project dialogue instead of letting it resolve as a concept
    // lookup. Must run before tryConceptLookup and trySoftwareProjectRequest.
    {
      name: "trySoftwareProjectFollowup",
      run: () => trySoftwareProjectFollowup(prompt, history),
    },
    { name: "tryConceptLookup", run: () => tryConceptLookup(prompt) },
    { name: "tryWriteProgram", run: () => writeProgram() },
    { name: "trySoftwareProjectRequest", run: () => trySoftwareProjectRequest(prompt, history) },
  ];
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
    return finalize(events, steps, toolCalls, projectLookup);
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

  // Issue #513: recognize terminal-command requests (visible fix for #511)
  // before the unknown fallback, so a shell request returns an agent_suggestion
  // intent in both engines.
  const terminal = tryTerminalCommand(prompt, language, preferences);
  if (terminal) {
    events.push(`handler:${terminal.intent}`);
    steps.push({ step: "dispatch_handler", detail: "tryTerminalCommand" });
    return finalize(events, steps, toolCalls, terminal, formalizationContext);
  }

  events.push("fallback:unknown");
  steps.push({ step: "fallback", detail: "unknown" });
  return finalize(events, steps, toolCalls, {
    intent: "unknown",
    content: unknownAnswerWithVariation(prompt, language),
    confidence: 0.1,
    evidence: ["fallback:unknown", `language:${language}`],
  }, formalizationContext);
}

// Issue #180: every handler hit flows through this helper so the trace shows
// the resolved-formalization fold (when the handler exposes a `formalizedObject`)
// followed by a uniform `deformalize` step that captures how the symbolic
// answer was projected into the natural-language `content`. Keeping the logic
// here means new handlers automatically participate in the architecture
// without having to repeat the bookkeeping.
function applyResolvedFormalization(events, steps, formalizationContext, answer) {
  if (!formalizationContext || !answer || !answer.formalizedObject) return;
  const resolved = resolveFormalizationWithId(
    formalizationContext.initial,
    answer.formalizedObject,
  );
  if (!resolved) return;
  // Skip the extra step when the placeholder already matched the resolved id
  // (e.g. cache hits where the formalization tuple already had a Q-id).
  if (resolved.tuple === formalizationContext.initial.tuple) return;
  formalizationContext.resolved = resolved;
  events.push(`formalization:resolved:${resolved.tuple}`);
  steps.push({
    step: "formalize_resolved",
    detail: formalizationDetail(resolved),
    formalization: {
      raw: resolved.raw,
      subject: resolved.subject,
      verb: resolved.verb,
      object: resolved.object,
      tuple: resolved.tuple,
    },
  });
}

function collectInterpretations(formalizationContext, answer) {
  const combined = [];
  const pushAll = (items) => {
    if (!Array.isArray(items)) return;
    for (const item of items) {
      if (!item || !item.original || !item.corrected) continue;
      combined.push({
        original: String(item.original),
        corrected: String(item.corrected),
      });
    }
  };
  pushAll(
    formalizationContext &&
      formalizationContext.initial &&
      formalizationContext.initial.interpretations,
  );
  pushAll(answer && answer.interpretations);
  const seen = new Set();
  return combined.filter((item) => {
    const key = `${item.original.toLowerCase()}\u0000${item.corrected.toLowerCase()}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function interpretationStatements(interpretations) {
  return interpretations
    .map((item) => `Interpreted "${item.original}" as "${item.corrected}".`)
    .join("\n");
}

function applyVisibleInterpretations(answer, interpretations) {
  if (!answer || interpretations.length === 0) return answer;
  const statements = interpretationStatements(interpretations);
  return Object.assign({}, answer, {
    content: `${statements}\n\n${String(answer.content || "")}`,
    evidence: [
      ...(Array.isArray(answer.evidence) ? answer.evidence : []),
      ...interpretations.map((item) => `interpretation:${item.original}->${item.corrected}`),
    ],
  });
}

function deformalizeProjection(formalizationContext, answer) {
  const tuple =
    (formalizationContext &&
      ((formalizationContext.resolved && formalizationContext.resolved.tuple) ||
        (formalizationContext.initial && formalizationContext.initial.tuple))) ||
    "(@USER OP:express ?)";
  const evidence = Array.isArray(answer.evidence) ? answer.evidence : [];
  const content = String(answer.content || "");
  const firstLine = content.split(/\r?\n/, 1)[0] || "";
  const projection = firstLine.length > 96 ? `${firstLine.slice(0, 96)}…` : firstLine;
  return {
    tuple,
    intent: answer.intent || "unknown",
    contentChars: content.length,
    evidenceCount: evidence.length,
    language:
      (formalizationContext && formalizationContext.language) ||
      answer.language ||
      "",
    summary: `${tuple} ⇒ ${answer.intent || "unknown"}: ${projection}`,
  };
}

// Issue #488: classify each reasoning step into a granularity tier so the
// thinking preview can show only the high-level universal-algorithm phases at
// the default ("standard") granularity and fold the mechanical sub-steps
// (the symbolic formalization tuple, tool probes, calculator reductions, memory
// scans, rule bookkeeping) into the opt-in "detailed" view. This mirrors the
// Rust solver's `ThinkingStep::level` classification (see src/engine.rs and
// src/event_log.rs) so the browser and native engines curate the trace
// identically — the thinking is fully applied to the logic, not just the UI.
const HIGH_LEVEL_THINKING_STEPS = new Set([
  "impulse",
  "detect_language",
  "resolve_response_language",
  "dispatch_handler",
  "match_rule",
  "clarify_formalization",
  "program_plan",
  "compute",
  "deformalize",
  "user_context",
  "fallback",
]);

function thinkingStepLevel(step) {
  const raw = String(step || "");
  // Nested agent sub-reasoning always folds under its composite agent task.
  if (/^agent_\d+_/i.test(raw)) return "detailed";
  return HIGH_LEVEL_THINKING_STEPS.has(raw) ? "high" : "detailed";
}

function withThinkingLevels(steps) {
  if (!Array.isArray(steps)) return [];
  return steps.map((step) =>
    step && typeof step === "object" && !step.level
      ? Object.assign({}, step, { level: thinkingStepLevel(step.step) })
      : step,
  );
}

function finalize(events, steps, toolCalls, answer, formalizationContext) {
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
    toolCalls,
  };
  if (answer.iframeUrl) {
    result.iframeUrl = answer.iframeUrl;
  }
  if (answer.diagnostics) {
    result.diagnostics = answer.diagnostics;
  }
  return result;
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
      hydrateLinoSeedText(SEED_RAW);
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
            start: Number(rule.start),
            end: Number(rule.end),
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

async function init() {
  if (wasm !== undefined) return;
  if (initPromise) return initPromise;
  initPromise = (async () => {
    await loadSeed();
    try {
      const source = await fetch(withAssetVersion("formal_ai_worker.wasm"));
      const bytes = await source.arrayBuffer();
      const module = await WebAssembly.instantiate(bytes, {});
      wasm = module.instance.exports;
    } catch (_error) {
      wasm = null;
      mode = "js fallback";
    }
    postMessage({
      kind: "ready",
      mode,
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
  })();
  return initPromise;
}

self.onmessage = async (event) => {
  await init();
  const data = event.data || {};
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
  const answer = attachUserContext(
    await solve(prompt, history, prefs, userContext),
    userContext,
  );
  postMessage({
    kind: "message",
    requestId: data.requestId,
    intent: answer.intent,
    content: answer.content,
    confidence: answer.confidence,
    evidence: answer.evidence,
    steps: answer.steps,
    toolCalls: answer.toolCalls,
    iframeUrl: answer.iframeUrl || null,
    diagnostics: answer.diagnostics || null,
  });
};

init();
