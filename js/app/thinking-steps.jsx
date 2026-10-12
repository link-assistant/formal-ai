// Reasoning-trace presentation: naturalised thinking steps, diagnostics
// summaries, step-level overrides and preview filtering by detail level.

import { normalizeThinkingDetailLevel } from "./preferences.jsx";

// Render a single diagnostics detail value as JSON-ish text so the user can
// read the raw payload (PR #134 feedback 4489651616: "I want diagnostics to
// show exactly all steps with expandable requests/responses data, full lino
// data description and so on"). The function never throws — non-serializable
// values fall back to String() — so a diagnostics row can't crash the chat.
export function formatDiagnosticPayload(value) {
  if (value === null || value === undefined) return "(empty)";
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value, null, 2);
  } catch (_error) {
    return String(value);
  }
}

export function truncateDiagnosticDetail(value) {
  const text = formatDiagnosticPayload(value).replace(/\s+/g, " ").trim();
  if (text.length <= 64) return text;
  return `${text.slice(0, 61)}...`;
}

export function summarizeToolCall(call) {
  if (!call || typeof call !== "object") return "";
  const parts = [];
  if (call.inputs && typeof call.inputs === "object") {
    const keys = Object.keys(call.inputs).slice(0, 3);
    if (keys.length > 0) parts.push(`in: ${keys.join(", ")}`);
  }
  if (call.outputs && typeof call.outputs === "object") {
    if (call.outputs.intent) parts.push(`out: ${call.outputs.intent}`);
    else {
      const keys = Object.keys(call.outputs).slice(0, 2);
      if (keys.length > 0) parts.push(`out: ${keys.join(", ")}`);
    }
  }
  return parts.join(" • ");
}

export function humanizeThinkingIdentifier(value) {
  return String(value || "")
    .replace(/^agent_\d+_/i, "")
    .replace(/^try(?=[A-Z])/u, "")
    .replace(/^handle(?=[A-Z])/u, "")
    .replace(/([a-z0-9])([A-Z])/gu, "$1 $2")
    .replace(/[_:.-]+/gu, " ")
    .replace(/\s+/gu, " ")
    .trim()
    .toLowerCase();
}

function thinkingLanguageLabel(value, t) {
  const code = String(value || "").toLowerCase().split(/[-_]/u)[0];
  if (["en", "ru", "zh", "hi"].includes(code)) {
    return t(`message.thinkingLanguage.${code}`);
  }
  return code || t("message.thinkingLanguage.unknown");
}

function thinkingRouteLabel(value, t) {
  const label = humanizeThinkingIdentifier(value);
  if (!label) return t("message.thinkingRoute.reply");
  if (label === "greeting") return t("message.thinkingRoute.greeting");
  if (label === "farewell") return t("message.thinkingRoute.farewell");
  if (label === "unknown") return t("message.thinkingRoute.unknown");
  return t("message.thinkingRoute.generic", { route: label });
}

function thinkingRuleLabel(value, t) {
  const label = humanizeThinkingIdentifier(value);
  if (!label) return t("message.thinkingRule.selected");
  if (label === "greeting") return t("message.thinkingRule.greeting");
  if (label === "farewell") return t("message.thinkingRule.farewell");
  if (label === "unknown") return t("message.thinkingRule.unknown");
  return label;
}

function thinkingToolLabel(value) {
  const label = humanizeThinkingIdentifier(value);
  return label || "local";
}

function truncateThinkingSummary(value) {
  const text = String(value || "").trim();
  if (text.length <= 96) return text;
  return `${text.slice(0, 93).trimEnd()}...`;
}

function summarizeThinkingDetail(value) {
  if (value === null || value === undefined) return "";
  if (typeof value === "string") {
    return truncateThinkingSummary(humanizeThinkingIdentifier(value));
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (Array.isArray(value)) {
    return `${value.length} item(s)`;
  }
  if (typeof value === "object") {
    const keys = Object.keys(value).slice(0, 3).map(humanizeThinkingIdentifier);
    return keys.length > 0 ? keys.join(", ") : "structured data";
  }
  return truncateThinkingSummary(value);
}

// Preserve a concrete detail value verbatim (the user's prompt, the computed
// result, the composed answer) while bounding its length, mirroring the Rust
// `truncate_thinking_detail` helper (600 chars, ellipsis suffix). Unlike
// `summarizeThinkingDetail` this does NOT lowercase or strip punctuation, so the
// real content survives into the naturalized sentence.
//
// Issue #1963 (P2 "Thinking steps are not fully written, some parts are
// omitted."): the cap was raised 120 -> 600 so realistic single-step detail
// renders in full instead of being clipped mid-sentence. Keep this constant in
// sync with the Rust `truncate_thinking_detail` helper.
function thinkingDetailText(detail) {
  if (detail === null || detail === undefined) return "";
  const text = String(detail).trim();
  if (text.length === 0) return "";
  const chars = Array.from(text);
  if (chars.length <= 600) return text;
  return `${chars.slice(0, 599).join("").trimEnd()}…`;
}

// English indefinite article for a phrase, mirroring the Rust `indefinite_article`
// helper so the English "Formalize the request as {article} {task} task." reads
// grammatically. Languages without articles simply ignore the {article} param.
function thinkingIndefiniteArticle(phrase) {
  const first = String(phrase || "").trimStart().charAt(0).toLowerCase();
  return ["a", "e", "i", "o", "u"].includes(first) ? "an" : "a";
}

// Issue #541 (R8): map the formalization operation (the `OP:*` verb) to a plain,
// localized task noun ("greeting", "calculation", "search", …) so the human
// reasoning view can describe what the request was understood as WITHOUT leaking
// the raw Links-notation tuple. The symbolic tuple stays in the diagnostics
// panel; the default trace stays human-readable per R8 ("no special syntax").
const FORMALIZATION_OP_LABEL_KEYS = {
  greet: "formalizeOpGreet",
  farewell: "formalizeOpFarewell",
  express: "formalizeOpExpress",
  compute: "formalizeOpCompute",
  define: "formalizeOpDefine",
  lookup: "formalizeOpLookup",
  search: "formalizeOpSearch",
  procedure: "formalizeOpProcedure",
  identify: "formalizeOpIdentify",
};

function formalizationOpLabel(formalization, t) {
  if (!formalization || typeof formalization !== "object") return "";
  const op = String(formalization.verb || formalization.op || "")
    .replace(/^OP:/i, "")
    .trim()
    .toLowerCase();
  const key = FORMALIZATION_OP_LABEL_KEYS[op];
  return key ? t(`message.thinkingStep.${key}`) : "";
}

// Translate a single structured thinking step into one concrete, human-readable
// sentence in the active UI language. This is stage 2 of the issue #488 pipeline
// ("translate the meta-language description into the target user language"):
// every known step kind threads its *concrete* detail (the prompt, the computed
// result, the looked-up entity, the composed answer) into a localized template,
// so the trace reads as real reasoning rather than generic category labels.
// Unknown kinds fall back to the meta-language `summary` the Rust solver already
// computed, then to a generic humanized label.
function naturalizeThinkingStep(entry, t) {
  const rawStep = String(entry?.step || "step");
  const step = rawStep.replace(/^agent_\d+_/i, "");
  const detail = entry?.detail;
  const value = thinkingDetailText(detail);
  const hasDetail = value.length > 0;

  if (rawStep !== step) {
    return t("message.thinkingStep.agentSubstep", {
      summary: naturalizeThinkingStep({ ...entry, step }, t),
    });
  }

  switch (step) {
    case "impulse":
      return hasDetail
        ? t("message.thinkingStep.impulse", { prompt: value })
        : t("message.thinkingStep.impulsePlain");
    case "detect_language":
      return t("message.thinkingStep.detectLanguage", {
        language: thinkingLanguageLabel(detail, t),
      });
    case "resolve_response_language":
      return t("message.thinkingStep.resolveResponseLanguage", {
        language: thinkingLanguageLabel(detail, t),
      });
    case "formalize": {
      // Issue #541 (R8): keep the human reasoning view free of symbolic syntax.
      // The browser solver formalizes into a Links-notation tuple before the
      // route is known — that tuple lives only in the diagnostics panel. Here we
      // project the operation to a plain task noun ("greeting", "calculation",
      // "search", …). The Rust solver instead reports the resolved task route in
      // `detail` (e.g. "greeting"), which we humanize directly.
      const opLabel = formalizationOpLabel(entry?.formalization, t);
      if (opLabel) {
        return t("message.thinkingStep.formalize", {
          task: opLabel,
          article: thinkingIndefiniteArticle(opLabel),
        });
      }
      if (!hasDetail) return t("message.thinkingStep.formalizePlain");
      const task = humanizeThinkingIdentifier(detail);
      return t("message.thinkingStep.formalize", {
        task,
        article: thinkingIndefiniteArticle(task),
      });
    }
    case "formalize_resolved": {
      // Issue #541 (R8): never surface the resolved (@USER OP:… Q-id) tuple in
      // the human trace. The browser solver only has an opaque resolved id here
      // (its `detail` still embeds the tuple), so fall back to the plain
      // phrasing; a solver that reports a concrete, syntax-free entity name in
      // `detail` keeps it.
      if (entry?.formalization) {
        return t("message.thinkingStep.formalizeResolvedPlain");
      }
      const looksSymbolic = /[()@?]|OP:|->|⇒/.test(value);
      return hasDetail && !looksSymbolic
        ? t("message.thinkingStep.formalizeResolved", {
            entity: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.formalizeResolvedPlain");
    }
    case "clarify_formalization":
      return hasDetail
        ? t("message.thinkingStep.clarifyFormalization", { options: value })
        : t("message.thinkingStep.clarifyFormalizationPlain");
    case "dispatch_handler":
      return hasDetail
        ? t("message.thinkingStep.dispatchHandler", {
            route: thinkingRouteLabel(detail, t),
          })
        : t("message.thinkingStep.dispatchHandlerPlain");
    case "route_attempt":
      return hasDetail
        ? t("message.thinkingStep.routeAttempt", {
            route: thinkingRouteLabel(detail, t),
          })
        : t("message.thinkingStep.routeAttemptPlain");
    case "match_rule":
      return hasDetail
        ? t("message.thinkingStep.matchRule", {
            rule: thinkingRuleLabel(detail, t),
          })
        : t("message.thinkingStep.matchRulePlain");
    case "compute":
      return hasDetail
        ? t("message.thinkingStep.compute", { expression: value })
        : t("message.thinkingStep.computePlain");
    case "compute_engine":
      return hasDetail
        ? t("message.thinkingStep.computeEngine", {
            engine: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.computeEnginePlain");
    case "compute_expression":
      return t("message.thinkingStep.computeExpression", { expression: value });
    case "compute_steps":
      return t("message.thinkingStep.computeSteps", { count: value });
    case "lookup_fact":
      return hasDetail
        ? t("message.thinkingStep.lookupFact", {
            fact: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.lookupFactPlain");
    case "invoke_tool":
      return hasDetail
        ? t("message.thinkingStep.invokeTool", {
            tool: thinkingToolLabel(detail),
          })
        : t("message.thinkingStep.invokeToolPlain");
    case "rule_verification":
      return hasDetail
        ? t("message.thinkingStep.ruleVerification", {
            rule: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.ruleVerificationPlain");
    case "policy_refusal":
      return hasDetail
        ? t("message.thinkingStep.policyRefusal", {
            policy: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.policyRefusalPlain");
    case "rule_construction":
      return t("message.thinkingStep.ruleConstruction");
    case "coreference_binding":
      return t("message.thinkingStep.coreferenceBinding");
    case "modifier_detection":
      return t("message.thinkingStep.modifierDetection");
    case "program_plan":
      return hasDetail
        ? t("message.thinkingStep.programPlan", {
            plan: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.programPlanPlain");
    case "scan_memory":
      return hasDetail
        ? t("message.thinkingStep.scanMemory", { term: value })
        : t("message.thinkingStep.scanMemoryPlain");
    case "deformalize": {
      // Prefer the clean composed answer the browser solver attaches as
      // `answer`; the Rust/API solver already sends the answer text as the
      // detail. The raw worker `detail` is the symbolic projection summary
      // (with the ⇒ glyph) reserved for the diagnostics panel, so it is not
      // used here.
      const answerText = thinkingDetailText(
        entry?.answer !== undefined && entry?.answer !== null
          ? entry.answer
          : detail,
      );
      return answerText
        ? t("message.thinkingStep.deformalize", { answer: answerText })
        : t("message.thinkingStep.deformalizePlain");
    }
    case "agent_plan":
      return hasDetail
        ? t("message.thinkingStep.agentPlan", {
            task: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.agentPlanPlain");
    case "fallback":
      return t("message.thinkingStep.fallback");
    case "http_chat":
      return t("message.thinkingStep.httpChat");
    case "memory":
      return t("message.thinkingStep.memory");
    case "extract_term":
      return t("message.thinkingStep.extractTerm");
    case "group_by_conversation":
      return t("message.thinkingStep.groupByConversation");
    // ---- Browser-only steps (no Rust solver counterpart) ----
    case "user_context":
      return t("message.thinkingStep.userContext", {
        context:
          summarizeThinkingDetail(detail) ||
          t("message.thinkingStep.userContextDefault"),
      });
    case "desktop_shell":
      return t("message.thinkingStep.desktopShell");
    case "trigger_button":
      return t("message.thinkingStep.triggerButton", {
        action: summarizeThinkingDetail(detail) || "button",
      });
    case "apply_message_command":
      return t("message.thinkingStep.applyMessageCommand", {
        command: summarizeThinkingDetail(detail) || "setting",
      });
    case "trigger_message_action":
      return t("message.thinkingStep.triggerMessageAction", {
        action: summarizeThinkingDetail(detail) || "action",
      });
    default: {
      // Unknown step kind: prefer the concrete meta-language summary the Rust
      // solver already computed (issue #488 pipeline stage 1), then fall back to
      // a generic humanized label so nothing renders as a bare identifier.
      const summary = String(entry?.summary || "").trim();
      if (summary) return summary;
      const readableStep = humanizeThinkingIdentifier(step) || "step";
      const readableDetail = summarizeThinkingDetail(detail);
      return t("message.thinkingStep.generic", {
        step: readableStep,
        detail: readableDetail ? `: ${readableDetail}` : "",
      });
    }
  }
}

export function thinkingStepKey(entry) {
  return String(entry?.step || "").replace(/^agent_\d+_/i, "");
}

// Issue #672 (F4): the reasoning hierarchy the solver emits is a proposal, not
// a verdict — a power user watching the same kind of trace all day knows which
// steps deserve to be phases and which are noise. Their edits are recorded as
// an append-only event log and the effective hierarchy is a *projection* of
// that log over the solver's own `level` field. Nothing rewrites the message:
// the entries the worker produced stay byte-identical, which is what keeps a
// re-render, a history reload, or a diagnostics export honest.
const STEP_LEVELS = Object.freeze(["high", "detailed"]);

function normalizeStepLevel(value) {
  const level = String(value || "").trim();
  return STEP_LEVELS.includes(level) ? level : "";
}

/**
 * Append one hierarchy edit. A `level` of "" is the reset event — recorded
 * rather than removing the earlier entry, so the log stays a history.
 */
export function appendStepLevelEvent(events, step, level) {
  const key = thinkingStepKey({ step });
  if (!key) return Array.isArray(events) ? events : [];
  return [
    ...(Array.isArray(events) ? events : []),
    { step: key, level: normalizeStepLevel(level) },
  ];
}

/** Fold the log down to the level currently in force per step: last write wins. */
export function projectStepLevels(events) {
  const levels = new Map();
  for (const event of Array.isArray(events) ? events : []) {
    const key = thinkingStepKey(event);
    if (!key) continue;
    const level = normalizeStepLevel(event.level);
    if (level) levels.set(key, level);
    else levels.delete(key);
  }
  return levels;
}

function filterThinkingEntriesForDetail(
  entries,
  detailLevel,
  stepLevelOverrides,
) {
  const safeEntries = Array.isArray(entries) ? entries.filter(Boolean) : [];
  if (safeEntries.length <= 1) return safeEntries;
  const overrides =
    stepLevelOverrides instanceof Map ? stepLevelOverrides : new Map();
  const level = normalizeThinkingDetailLevel(detailLevel);
  // "detailed" shows everything and "brief" shows only the conclusion, so in
  // both the hierarchy is not consulted and an override cannot change what the
  // user sees. Only the middle, hierarchy-driven granularity projects the log.
  if (level === "detailed") return safeEntries;
  if (level === "brief") return safeEntries.slice(-1);

  // Medium (default) granularity: show the high-level universal-algorithm
  // phases plus the final step, recursively folding composite internals (the
  // calculator trace, memory scans, tool calls) out of view. Prefer the
  // structured `level` field the solver now emits ("high" for phases,
  // "detailed" for nested children); fall back to a step-name allowlist for
  // legacy entries that predate the level metadata.
  const lastIndex = safeEntries.length - 1;
  const hasLevels = safeEntries.some(
    (entry) => typeof entry?.level === "string" && entry.level.length > 0,
  );
  if (hasLevels) {
    const filtered = safeEntries.filter(
      (entry, index) =>
        index === lastIndex ||
        (overrides.get(thinkingStepKey(entry)) || entry?.level) === "high",
    );
    return filtered.length > 0 ? filtered : safeEntries.slice(-1);
  }

  const standardSteps = new Set([
    "impulse",
    "detect_language",
    "resolve_response_language",
    "clarify_formalization",
    "match_rule",
    "fallback",
    "user_context",
    "deformalize",
    "program_plan",
    "desktop_shell",
    "http_chat",
    "memory",
    "agent_plan",
  ]);
  const filtered = safeEntries.filter((entry, index) => {
    if (index === lastIndex) return true;
    const key = thinkingStepKey(entry);
    // A user edit outranks the legacy allowlist in both directions: it can
    // promote a step the allowlist never knew about and demote one it names.
    if (overrides.has(key)) return overrides.get(key) === "high";
    return standardSteps.has(key);
  });
  return filtered.length > 0 ? filtered : safeEntries.slice(-1);
}

function filterThinkingSummariesForDetail(summaries, detailLevel) {
  const safeSummaries = Array.isArray(summaries)
    ? summaries.map((step) => String(step || "").trim()).filter(Boolean)
    : [];
  if (safeSummaries.length <= 1) return safeSummaries;
  const level = normalizeThinkingDetailLevel(detailLevel);
  if (level === "detailed") return safeSummaries;
  if (level === "brief") return safeSummaries.slice(-1);
  return safeSummaries.length > 4
    ? [safeSummaries[0], ...safeSummaries.slice(-3)]
    : safeSummaries;
}

export function buildThinkingPreviewSteps(
  structuredSteps,
  answer,
  source,
  t,
  detailLevel,
  stepLevelOverrides,
) {
  if (Array.isArray(structuredSteps) && structuredSteps.length > 0) {
    return filterThinkingEntriesForDetail(
      structuredSteps,
      detailLevel,
      stepLevelOverrides,
    )
      .map((entry) => naturalizeThinkingStep(entry, t))
      .filter(Boolean);
  }
  return filterThinkingSummariesForDetail(
    [
      t("message.thinkingStep.fallbackNormalize"),
      t("message.thinkingStep.fallbackIntent", {
        intent: humanizeThinkingIdentifier(answer?.intent || "unknown"),
      }),
      t("message.thinkingStep.fallbackRender", {
        source: humanizeThinkingIdentifier(source || "fallback"),
      }),
    ],
    detailLevel,
  );
}

export function buildMessageThinkingPreviewSteps(
  message,
  t,
  detailLevel,
  stepLevelOverrides,
) {
  if (message?.role !== "assistant") return [];
  const diagnosticsSteps = Array.isArray(message.diagnosticsSteps)
    ? message.diagnosticsSteps
    : [];
  if (diagnosticsSteps.length > 0) {
    return buildThinkingPreviewSteps(
      diagnosticsSteps,
      message,
      message.thinkingPreviewSource || message.intent || "local",
      t,
      detailLevel,
      stepLevelOverrides,
    );
  }
  return filterThinkingSummariesForDetail(
    message.thinkingPreviewSteps ?? [],
    detailLevel,
  );
}
