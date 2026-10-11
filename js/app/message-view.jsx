// Chat message rendering: the Message component and its reasoning preview,
// diagnostics, approval and step-hierarchy panels.

import React from "react";
import { ConversationSummaryDetails } from "./conversation-summary-details.js";
import { copyTextToClipboard, enhanceCodeBlocks, markdownHtml } from "./markdown-render.jsx";
import {
  DESKTOP_TOOL_I18N_KEYS, DESKTOP_TOOL_OPTIONS, desktopToolGrantState,
} from "./preferences.jsx";
import {
  buildMessageThinkingPreviewSteps, formatDiagnosticPayload, humanizeThinkingIdentifier,
  summarizeToolCall, thinkingStepKey, truncateDiagnosticDetail,
} from "./thinking-steps.jsx";

const { createElement: h, useCallback, useEffect, useMemo, useRef, useState } = React;

// Issue #153: dedicated renderer for the formalize / formalize_resolved
// diagnostics step. Keeps the SVO layout consistent regardless of source
// language and shows the canonical id prefixes (`Q`, `WP:`, `WT:`, `OP:`,
// `@USER`) so reviewers can verify the symbolic mapping. The verb slot
// labels the SVO triple in the user's UI language.
function FormalizationView({ formalization, t }) {
  if (!formalization) return null;
  return <div className="formalization-view" data-testid="formalization">{formalization.raw ? <div className="formalization-raw"><code>{formalization.raw}</code>
    <span className="formalization-arrow" aria-hidden="true">{"→"}</span><code className="formalization-tuple">{formalization.tuple}</code>
    </div> : <code className="formalization-tuple">{formalization.tuple}</code>}<div className="formalization-svo"><span className="formalization-svo-label">
    {t("message.formalizationSubjectVerbObject")}</span><ol className="formalization-svo-list"><li><span className="formalization-slot">{"S"}</span><code>
    {formalization.subject || ""}</code></li><li><span className="formalization-slot">{"V"}</span><code>{formalization.verb || ""}</code></li><li>
    <span className="formalization-slot">{"O"}</span><code>{formalization.object || ""}</code></li></ol></div></div>;
}

// Issue #180: format the unified link-notation projection for an HTTP
// exchange so the user can see the formalization step alongside the raw
// request and response in diagnostics mode.
function formatHttpExchangeAsLinks(exchange) {
  if (!exchange || typeof exchange !== "object") return "";
  const lines = [];
  const id = exchange.id || `http:${exchange.method || "GET"}:${exchange.url || ""}`;
  lines.push(`(${id}: kind http_exchange)`);
  if (exchange.provider) lines.push(`(${id}: provider ${exchange.provider})`);
  if (exchange.phase) lines.push(`(${id}: phase ${exchange.phase})`);
  if (exchange.method) lines.push(`(${id}: method ${exchange.method})`);
  if (exchange.url) lines.push(`(${id}: url ${exchange.url})`);
  if (typeof exchange.status === "number") {
    lines.push(`(${id}: status ${exchange.status})`);
  }
  if (typeof exchange.elapsedMs === "number") {
    lines.push(`(${id}: elapsed_ms ${exchange.elapsedMs})`);
  }
  if (typeof exchange.responseBytes === "number") {
    lines.push(`(${id}: response_bytes ${exchange.responseBytes})`);
  }
  if (exchange.error) {
    const safeError = String(exchange.error).replace(/[()]/g, " ");
    lines.push(`(${id}: error ${safeError})`);
  }
  return lines.join("\n");
}

// Issue #180: render the worker's per-provider summary and the raw HTTP
// exchange list (request URL, status, elapsed time, response snippet,
// unified Links Notation projection) for each search-providing message.
function DiagnosticsHttpPanel({ providers, exchanges, t }) {
  if (
    (!Array.isArray(providers) || providers.length === 0) &&
    (!Array.isArray(exchanges) || exchanges.length === 0)
  ) {
    return null;
  }
  const safeExchanges = Array.isArray(exchanges) ? exchanges : [];
  return <div className="diagnostics-http" data-testid="diagnostics-http">{Array.isArray(providers) && providers.length > 0 ? <div className="diagnostics-http-section"><strong className="diagnostics-section-label">{t("message.diagnosticsProviders")}</strong><ul className="diagnostics-http-provider-list">{providers.map((entry, index) => <li key={`${entry.id || "provider"}-${index}`} className={`diagnostics-http-provider ${entry.ok ? "is-ok" : "is-error"}`} data-testid="diagnostics-http-provider">{t("message.diagnosticsProviderRow", {
          label: entry.label || entry.id || "(provider)",
          status: entry.ok ? t("message.diagnosticsProviderOk") : `${t("message.diagnosticsProviderError")}: ${entry.error || "(unknown)"}`,
          count: typeof entry.count === "number" ? entry.count : 0,
          elapsed: typeof entry.elapsedMs === "number" ? entry.elapsedMs : 0
        })}</li>)}</ul></div> : null}<div className="diagnostics-http-section"><strong className="diagnostics-section-label">{t("message.diagnosticsHttp")}</strong>
          {safeExchanges.length === 0 ? <p className="diagnostics-http-empty">{t("message.diagnosticsHttpEmpty")}</p> : <ol className="diagnostics-http-list">
          {safeExchanges.map((exchange, index) => <li key={`${exchange.id || index}`} className="diagnostics-http-item">
          <details className="diagnostics-detail" data-testid="diagnostics-http-exchange"><summary><span className="diagnostics-step-name">
          {`${exchange.method || "GET"} ${exchange.provider ? `[${exchange.provider}] ` : ""}`}</span><span className="diagnostics-step-summary">{exchange.url || "(no url)"}</span>
          <span className="diagnostics-http-status">{t("message.diagnosticsHttpStatus", {
                status: typeof exchange.status === "number" ? exchange.status : "—",
                elapsed: typeof exchange.elapsedMs === "number" ? exchange.elapsedMs : 0,
                bytes: typeof exchange.responseBytes === "number" ? exchange.responseBytes : 0
              })}</span></summary><div className="diagnostics-detail-body"><div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.diagnosticsHttpRequest")}</span><pre className="diagnostics-payload">{formatDiagnosticPayload({
                  method: exchange.method || "GET",
                  url: exchange.url || "",
                  headers: exchange.requestHeaders || {},
                  body: exchange.requestBody || null,
                  provider: exchange.provider || "",
                  phase: exchange.phase || ""
                })}</pre></div><div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.diagnosticsHttpResponse")}</span><pre className="diagnostics-payload">{formatDiagnosticPayload({
                  status: exchange.status ?? null,
                  ok: !!exchange.ok,
                  elapsedMs: exchange.elapsedMs ?? null,
                  responseBytes: exchange.responseBytes ?? null,
                  finalUrl: exchange.finalUrl || "",
                  contentType: exchange.contentType || "",
                  responseSnippet: exchange.responseSnippet || "",
                  error: exchange.error || ""
                })}</pre></div><div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.diagnosticsHttpUnified")}</span><pre className="diagnostics-payload diagnostics-http-links">{formatHttpExchangeAsLinks(exchange)}</pre></div></div></details></li>)}</ol>}</div></div>;
}

// Issue #488: while the worker is still computing the answer there is no live
// per-step stream to display, but the pending bubble should still feel alive —
// an expert reasoning out loud surfaces "what they're working on right now"
// every couple of seconds. The hook accumulates a fixed sequence of generic
// expert-shaped phases (read → formalize → look up → compose) over ~2 s steps
// and stops once the answer arrives. Each new phase appends to the visible
// steps array so the rotated-scrolling animation in `ThinkingPreview` re-fires
// the same way it would for real solver steps.
// Issue #541 (R6): honour the OS "reduce motion" accessibility preference. When
// the user has asked for reduced motion we skip the staged reveal entirely and
// show the answer at once, matching the existing prefers-reduced-motion CSS.
function usePrefersReducedMotion() {
  const query = "(prefers-reduced-motion: reduce)";
  const getInitial = () =>
    typeof window !== "undefined" && typeof window.matchMedia === "function"
      ? window.matchMedia(query).matches
      : false;
  const [reduced, setReduced] = useState(getInitial);
  useEffect(() => {
    if (
      typeof window === "undefined" ||
      typeof window.matchMedia !== "function"
    ) {
      return undefined;
    }
    const media = window.matchMedia(query);
    const handler = (event) => setReduced(event.matches);
    if (typeof media.addEventListener === "function") {
      media.addEventListener("change", handler);
      return () => media.removeEventListener("change", handler);
    }
    // Safari < 14 fallback.
    media.addListener(handler);
    return () => media.removeListener(handler);
  }, []);
  return reduced;
}

// Issue #541 (R5/R6): stage the reveal of a freshly produced assistant message —
// reasoning steps first (each new step re-triggers the rotated-scroll animation
// in ThinkingPreview), then the answer body — across a minimum animation budget
// so the deterministic engine's instant answers still *feel* considered. Returns
// the count of currently revealed steps and whether the body is shown yet. With
// budgetMs<=0, reduced motion, or no steps it is an immediate no-op (everything
// shown at once). The first ~72% of the budget unveils the steps; the body is
// held back until the full budget elapses, satisfying R6's "only when we
// scrolled to the last thinking step can we show the message itself".
//
// Issue #672 (F3): the budget is a global preference, but the right value for
// it is per-message — a user who is happy to watch the reasoning fill in for
// most answers still wants THIS one now. `skip()` is that one-shot override:
// it ends the staged reveal for this message immediately (clearing the pending
// timers) without touching the preference, so the next message animates as
// before. It is deliberately additive to the reduced-motion path rather than a
// replacement for it — an OS-level reduced-motion preference must keep
// suppressing the animation without the user having to click anything.
function useMessageReveal(stepCount, budgetMs) {
  const reducedMotion = usePrefersReducedMotion();
  const [skipped, setSkipped] = useState(false);
  const active = budgetMs > 0 && stepCount > 0 && !reducedMotion && !skipped;
  // The staged reveal plays exactly once — when the freshly produced message
  // first appears. Once it has played out (or if it never applied) we latch
  // "done" so that a later change in step count — e.g. the user toggling the
  // reasoning-detail setting on an already-revealed message — snaps straight to
  // "show everything" instead of replaying the animation. Replaying would set
  // `bodyShown` back to false (the `.is-revealing` rule is `display:none`, so
  // the answer would briefly vanish) and re-scroll the steps from the first
  // one, which is jarring when the user is just adjusting how much detail to see.
  const doneRef = useRef(!active);
  const [revealedSteps, setRevealedSteps] = useState(active ? 1 : stepCount);
  const [bodyShown, setBodyShown] = useState(!active);
  useEffect(() => {
    if (!active || doneRef.current) {
      setRevealedSteps(stepCount);
      setBodyShown(true);
      return undefined;
    }
    // Reserve the final slice of the budget for the body fade; spread the rest
    // across the steps so even a single step occupies a perceptible beat.
    const stepsWindow = budgetMs * 0.72;
    const perStep = stepsWindow / stepCount;
    setRevealedSteps(1);
    setBodyShown(false);
    const timers = [];
    for (let index = 1; index < stepCount; index += 1) {
      timers.push(
        setTimeout(
          () => setRevealedSteps(index + 1),
          Math.round(perStep * index),
        ),
      );
    }
    timers.push(
      setTimeout(() => {
        setBodyShown(true);
        doneRef.current = true;
      }, Math.round(budgetMs)),
    );
    return () => timers.forEach((timer) => clearTimeout(timer));
  }, [active, stepCount, budgetMs]);
  // Issue #672 (F3). Latching `doneRef` matters as much as flipping `skipped`:
  // without it a later step-count change (the reasoning-detail setting, say)
  // would take the "not done yet" path and restart the animation the user just
  // asked to end. Flipping `active` to false tears down the pending timers
  // through the effect's own cleanup, so nothing is left to fire.
  const skip = useCallback(() => {
    doneRef.current = true;
    setSkipped(true);
  }, []);
  return { active, revealedSteps, bodyShown, skip };
}

function usePendingThinkingPhases(isActive, t) {
  const [phaseIndex, setPhaseIndex] = useState(0);
  const phrases = useMemo(
    () => [
      t("message.thinkingStep.pendingReading"),
      t("message.thinkingStep.pendingFormalizing"),
      t("message.thinkingStep.pendingDispatching"),
      t("message.thinkingStep.pendingComposing"),
      t("message.thinkingStep.working"),
    ],
    [t],
  );
  useEffect(() => {
    if (!isActive) {
      setPhaseIndex(0);
      return undefined;
    }
    if (phaseIndex >= phrases.length - 1) {
      return undefined;
    }
    const timer = setTimeout(() => setPhaseIndex((value) => value + 1), 1800);
    return () => clearTimeout(timer);
  }, [isActive, phaseIndex, phrases.length]);
  if (!isActive) return [];
  return phrases.slice(0, phaseIndex + 1);
}

// Issue #488: render the pending assistant message — while processing, the
// thinking preview IS the visible part of the message (no separate "working"
// caption), and the preview pulls from a hook that adds expert-shaped phases
// over time so the rotated-scrolling animation actually has something to rotate
// even though the worker itself does not yet stream per-step messages.
export function PendingAssistantBubble({ t }) {
  const pendingPhases = usePendingThinkingPhases(true, t);
  return <article className="message assistant pending"><div className="avatar" aria-hidden="true">{"FA"}</div><div className="message-body"><ThinkingPreview steps={pendingPhases} t={t} isPending={true} /></div></article>;
}

// Issue #676 (R8): map the resolved intent to a per-intent narrative catalog
// key, mirroring the Rust `thinking_narrative`. Grouped so related routes share
// one human headline (all lookups read the same, all web tools read the same).
const THINKING_NARRATIVE_KEYS = {
  greeting: "narrativeGreeting",
  wellbeing: "narrativeWellbeing",
  assistant_free_time: "narrativeAssistantFreeTime",
  farewell: "narrativeFarewell",
  gratitude: "narrativeGratitude",
  thanks: "narrativeGratitude",
  courtesy_response: "narrativeGratitude",
  courtesy: "narrativeGratitude",
  identity: "narrativeIdentity",
  assistant_name: "narrativeIdentity",
  set_assistant_name: "narrativeIdentity",
  recall_name: "narrativeIdentity",
  naming: "narrativeIdentity",
  assistant_naming: "narrativeIdentity",
  self_facts: "narrativeIdentity",
  who_is_question: "narrativeIdentity",
  calculation: "narrativeCalculation",
  arithmetic: "narrativeCalculation",
  calculation_error: "narrativeCalculation",
  object_counting: "narrativeCalculation",
  fact_lookup: "narrativeLookup",
  fact_query: "narrativeLookup",
  concept_lookup: "narrativeLookup",
  concept_lookup_in_context: "narrativeLookup",
  known_facts: "narrativeLookup",
  wikipedia_lookup: "narrativeLookup",
  wikipedia_article_question: "narrativeLookup",
  definition_merge: "narrativeLookup",
  translation: "narrativeTranslation",
  web_search: "narrativeWeb",
  http_fetch: "narrativeWeb",
  url_navigate: "narrativeWeb",
  write_program: "narrativeCode",
  software_project_plan: "narrativeCode",
  software_project_implementation: "narrativeCode",
  algorithm: "narrativeCode",
  test_status: "narrativeTests",
  self_healing: "narrativeSelfHealing",
  self_heal: "narrativeSelfHealing",
  meta_explanation: "narrativeMetaExplanation",
  learn_from_source: "narrativeLearn",
  clarification: "narrativeClarification",
  unknown: "narrativeUnknown",
  fallback: "narrativeUnknown",
  no_match: "narrativeUnknown",
};

// Issue #676 (R8): produce the single human, first-person headline that leads a
// thinking trace ("You asked how I'm doing, so I told you and offered to
// help."). Unknown routes still get a human sentence via the generic template.
// Returns "" only when there is no intent to summarize (e.g. the pending
// placeholder), so callers can skip the headline entirely.
function thinkingNarrative(intent, t) {
  const route = String(intent || "").trim().toLowerCase();
  if (!route) return "";
  const key = THINKING_NARRATIVE_KEYS[route];
  if (key) return t(`message.thinkingStep.${key}`);
  return t("message.thinkingStep.narrativeGeneric", {
    task: humanizeThinkingIdentifier(route),
  });
}

function ThinkingPreview({ steps, t, isPending = false, narrative = "" }) {
  const [expanded, setExpanded] = useState(false);
  const safeSteps = Array.isArray(steps)
    ? steps.map((step) => String(step || "").trim()).filter(Boolean)
    : [];
  // Issue #488: track the index of the current step so a change in the latest
  // step triggers the rotated-scrolling CSS animation (current step slides up
  // into place; the previous step half-shows above with the gradient fade).
  const lastIndex = safeSteps.length - 1;
  const current = lastIndex >= 0 ? safeSteps[lastIndex] : "";
  const previous = lastIndex > 0 ? safeSteps[lastIndex - 1] : "";
  // Use a stable but per-step key so React re-mounts the current/previous
  // <p> nodes when the step changes — that re-mount is what re-triggers the
  // CSS `@keyframes thinking-rotate-in` animation.
  const animationKey = `${lastIndex}-${current}`;
  if (safeSteps.length === 0) return null;

  return <section className={["thinking-preview", expanded ? "is-expanded" : "is-collapsed", isPending ? "is-pending" : ""].filter(Boolean).join(" ")} data-testid="thinking-preview" data-pending={isPending ? "true" : null} aria-label={t("message.thinking")} aria-live={isPending ? "polite" : null}><div className="thinking-preview-header"><strong className="thinking-preview-title">{
      // Issue #488: show a subtle "live" affordance while pending so the user
      // understands the trace is updating in real time (the dot pulses via
      // CSS; the visible label stays unchanged for screen readers).
      isPending ? <span className="thinking-preview-live-dot" aria-hidden="true" data-testid="thinking-preview-live-dot" /> : null}{t("message.thinking")}</strong>
        <button type="button" className="thinking-preview-toggle" data-testid="thinking-preview-toggle" aria-expanded={expanded ? "true" : "false"}
        onClick={() => setExpanded(value => !value)}>
        {expanded ? t("message.thinkingCollapse") : t("message.thinkingExpand")}</button></div>
        {narrative ? <p className="thinking-preview-narrative" data-testid="thinking-narrative">{narrative}
        </p> : null}{expanded ? <ol className="thinking-preview-list" data-testid="thinking-expanded-list">{safeSteps.map((step, index) => <li key={`${index}-${step}`}>{step}
        </li>)}</ol> : <div className="thinking-preview-collapsed" data-testid="thinking-collapsed">
        {previous ? <p key={`prev-${animationKey}`} className="thinking-preview-previous" data-testid="thinking-preview-previous" aria-label={t("message.thinkingPrevious")}>
        {previous}</p> : null}<p key={`curr-${animationKey}`} className="thinking-preview-current" data-testid="thinking-preview-current" aria-label={t("message.thinkingCurrent")}>
        {current}</p></div>}</section>;
}

export function DesktopPermissionPanel({
  grants,
  mode,
  onDecision,
  onGrantAll,
  hasPendingTask = false,
  testId = "desktop-permission-panel",
  t,
}) {
  const active = mode !== "chat";
  const tr = typeof t === "function" ? t : (key) => key;
  const stateLabel = (state) =>
    state === "granted"
      ? tr("permissions.state.granted")
      : state === "declined"
        ? tr("permissions.state.declined")
        : tr("permissions.state.undecided");
  // Issue #541 (R9): the original issue text says "After permissions are
  // granted nothing happens, the message for granting permissions should also
  // include button to grant all permissions and switch to agent mode, which
  // when clicked should actually evaluate pending task for execution."
  //
  // We render that affordance as a primary CTA above the per-tool rows so the
  // user can opt-in with a single click without scrolling through every
  // grant/decline button. The button label changes when a task is queued
  // ("...and run pending task") so it is honest about what will happen.
  const grantAllLabel = hasPendingTask
    ? tr("permissions.action.grantAllAndRun")
    : tr("permissions.action.grantAll");
  return <section className="permission-panel" data-testid={testId} data-mode={mode}><div className="permission-panel-header"><strong>{tr("permissions.panel.title")}</strong><span>
    {active ? tr("permissions.panel.active") : tr("permissions.panel.saved")}</span></div>{onGrantAll ? <div className="permission-panel-grant-all">
    <button type="button" className="permission-button permission-button-grant-all" data-testid={`${testId}-grant-all`} data-has-pending-task={hasPendingTask ? "true" : "false"}
    onClick={() => onGrantAll()}>
    {grantAllLabel}</button></div> : null}<div className="permission-tool-list">{DESKTOP_TOOL_OPTIONS.map(tool => {
      const state = desktopToolGrantState(grants, tool);
      const granted = state === "granted";
      const declined = state === "declined";
      const i18nKey = DESKTOP_TOOL_I18N_KEYS[tool] || tool;
      return <div key={tool} className="permission-tool-row" data-testid={`${testId}-row-${tool}`}><div className="permission-tool-copy"><strong>
        {tr(`permissions.tool.${i18nKey}.label`)}</strong><span>{tr(`permissions.tool.${i18nKey}.description`)}</span></div>
        <span className={`permission-state permission-state-${state}`} data-testid={`${testId}-state-${tool}`}>{stateLabel(state)}</span><div className="permission-actions">
        <button type="button" className="permission-button" data-testid={`${testId}-grant-${tool}`} aria-pressed={granted ? "true" : "false"}
        onClick={() => onDecision && onDecision(tool, true)}>
        {tr("permissions.action.grant")}</button>
        <button type="button" className="permission-button permission-button-secondary" data-testid={`${testId}-decline-${tool}`} aria-pressed={declined ? "true" : "false"}
        onClick={() => onDecision && onDecision(tool, false)}>
        {tr("permissions.action.decline")}</button></div></div>;
    })}</div></section>;
}

function CommandApprovalPanel({ approval, status, onApprove, onDeny, t }) {
  if (!approval) {
    return null;
  }
  const tr = typeof t === "function" ? t : (key) => key;
  const currentStatus = status || approval.status || "pending";
  const pending = currentStatus === "pending";
  const command = String(approval.command || "");
  const statusKeys = {
    pending: "permissions.command.status.pending",
    running: "permissions.command.status.running",
    approved: "permissions.command.status.approved",
    denied: "permissions.command.status.denied",
  };
  const statusLabel = statusKeys[currentStatus]
    ? tr(statusKeys[currentStatus])
    : currentStatus;
  return <section className="command-approval-panel" data-testid="command-approval" data-status={currentStatus}><div className="command-approval-copy"><strong>
    {tr("permissions.command.title")}</strong><code>{command}</code><span className={`command-approval-status command-approval-status-${currentStatus}`}>{statusLabel}</span></div>
    <div className="command-approval-actions">
    <button type="button" className="permission-button" data-testid="command-approve" disabled={!pending} onClick={() => pending && onApprove && onApprove(approval)}>
    {tr("permissions.command.approve")}</button>
    <button type="button" className="permission-button permission-button-secondary" data-testid="command-deny" disabled={!pending}
    onClick={() => pending && onDeny && onDeny(approval)}>
    {tr("permissions.command.deny")}</button></div></section>;
}

// Issue #672 (F4): the hierarchy editor for a single reasoning step. It is a
// right-click menu because it is a power-user affordance on a Diagnostics-mode
// surface: putting two buttons on every step would clutter the trace for the
// far larger group of users who only read it.
function StepHierarchyMenu({ menu, currentLevel, overridden, onSelect, t }) {
  const close = menu?.onClose;
  useEffect(() => {
    if (!menu) return undefined;
    const dismiss = () => close && close();
    const onKey = (event) => {
      if (event.key === "Escape") dismiss();
    };
    // `pointerdown` rather than `click`: the menu must not survive the press
    // that starts an interaction somewhere else on the page.
    window.addEventListener("pointerdown", dismiss);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", dismiss);
      window.removeEventListener("keydown", onKey);
    };
  }, [menu, close]);
  if (!menu) return null;
  return <div className="step-hierarchy-menu" data-testid="step-hierarchy-menu" data-step={menu.step} role="menu" aria-label={t("message.stepLevel.title")}
    style={{ top: `${menu.y}px`, left: `${menu.x}px` }} onPointerDown={event => event.stopPropagation()}>
    <button type="button" role="menuitem" data-testid="step-hierarchy-bump" disabled={currentLevel === "high"} onClick={() => onSelect(menu.step, "high")}>
    {t("message.stepLevel.bump")}</button>
    <button type="button" role="menuitem" data-testid="step-hierarchy-demote" disabled={currentLevel === "detailed"} onClick={() => onSelect(menu.step, "detailed")}>
    {t("message.stepLevel.demote")}</button>{overridden ? <button type="button" role="menuitem" data-testid="step-hierarchy-reset" onClick={() => onSelect(menu.step, "")}>
    {t("message.stepLevel.reset")}</button> : null}</div>;
}

export function Message({
  message,
  conversationMessages,
  diagnosticsMode,
  reportIssueUrl,
  stepLevelOverrides,
  onEditStepLevel,
  thinkingDetailLevel,
  minMessageAnimationMs = 0,
  renderPermissionPanel,
  commandApprovals,
  onApproveCommand,
  onDenyCommand,
  t,
}) {
  const evidence = diagnosticsMode ? (message.evidence ?? []) : [];
  const thinkingSteps = diagnosticsMode ? (message.thinkingSteps ?? []) : [];
  const overrides =
    stepLevelOverrides instanceof Map ? stepLevelOverrides : new Map();
  const thinkingPreviewSteps = buildMessageThinkingPreviewSteps(
    message,
    t,
    thinkingDetailLevel,
    overrides,
  );
  // Issue #672 (F4): `{ step, x, y }` while the right-click menu is open.
  const [stepMenu, setStepMenu] = useState(null);
  const closeStepMenu = useCallback(() => setStepMenu(null), []);
  const handleStepContextMenu = useCallback(
    (event, step) => {
      if (typeof onEditStepLevel !== "function") return;
      const key = thinkingStepKey({ step });
      if (!key) return;
      event.preventDefault();
      setStepMenu({
        step: key,
        x: event.clientX,
        y: event.clientY,
        onClose: closeStepMenu,
      });
    },
    [onEditStepLevel, closeStepMenu],
  );
  const handleStepLevelSelect = useCallback(
    (step, level) => {
      setStepMenu(null);
      if (typeof onEditStepLevel === "function") onEditStepLevel(step, level);
    },
    [onEditStepLevel],
  );
  const diagnosticsSteps = diagnosticsMode
    ? (message.diagnosticsSteps ?? [])
    : [];
  const diagnosticsToolCalls = diagnosticsMode
    ? (message.diagnosticsToolCalls ?? [])
    : [];
  // Issue #180: surface raw HTTP request/response bodies and the per-provider
  // outcomes inside the diagnostics panel so the user can audit every network
  // call the worker performed on their behalf.
  const diagnosticsPayload = diagnosticsMode ? message.diagnostics : null;
  const diagnosticsProviders = Array.isArray(diagnosticsPayload?.providers)
    ? diagnosticsPayload.providers
    : [];
  const diagnosticsHttp = Array.isArray(diagnosticsPayload?.httpExchanges)
    ? diagnosticsPayload.httpExchanges
    : [];
  const reportLabel =
    message.intent === "unknown"
      ? t("buttons.reportMissingRule")
      : t("buttons.reportIssue");
  const [iframeFullscreen, setIframeFullscreen] = useState(false);
  // Issue #330: progressive syntax highlighting + per-code-block copy buttons.
  const markdownRef = useRef(null);
  const [markdownCopied, setMarkdownCopied] = useState(false);

  // Issue #541 (R5/R6): stage the reveal of this message — reasoning steps
  // first, then the answer body — across the minimum animation budget. Only a
  // freshly produced answer carries `animateReveal`; hydrated history shows at
  // once (budget 0 -> immediate no-op).
  const revealBudgetMs = message.animateReveal ? minMessageAnimationMs : 0;
  const reveal = useMessageReveal(thinkingPreviewSteps.length, revealBudgetMs);
  const revealedThinkingSteps = reveal.active
    ? thinkingPreviewSteps.slice(0, reveal.revealedSteps)
    : thinkingPreviewSteps;
  const bodyRevealClass = reveal.active
    ? reveal.bodyShown
      ? " is-revealed"
      : " is-revealing"
    : "";

  // React 19 compares `dangerouslySetInnerHTML` by object identity (React 18
  // compared the inner `__html` string). A fresh `markdownHtml(...)` object on
  // every render would therefore make React re-assign `innerHTML` each pass,
  // wiping the `.code-block` wrappers that `enhanceCodeBlocks` grafts in below.
  // Memoising by `message.content` keeps the object stable while the text is
  // unchanged, so the out-of-band enhancements survive unrelated re-renders.
  const markdownContent = useMemo(
    () => markdownHtml(message.content),
    [message.content],
  );

  useEffect(() => {
    if (!iframeFullscreen) {
      return undefined;
    }
    const handleKeyDown = (event) => {
      if (event.key === "Escape") {
        setIframeFullscreen(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [iframeFullscreen]);

  // Highlight and wrap code fences after marked renders the message body. The
  // enhancement is idempotent, so re-running it on content changes is safe.
  useEffect(() => {
    enhanceCodeBlocks(markdownRef.current, t);
  }, [message.content, t]);

  const handleCopyMarkdown = useCallback(async () => {
    const ok = await copyTextToClipboard(message.content);
    if (ok) {
      setMarkdownCopied(true);
      setTimeout(() => setMarkdownCopied(false), 1600);
    }
  }, [message.content]);

  return <article className={`message ${message.role}`} data-testid="chat-message" data-demo-label={message.demoLabel || null}
    data-skip-animation={reveal.active && !reveal.bodyShown ? "available" : null}>
    <div className="avatar" aria-hidden="true">{message.role === "user" ? "Y" : "FA"}</div><div className="message-body"><div className="message-meta"><strong>
    {message.role === "user" ? t("message.author.user") : message.author}</strong><time>{message.sentAt}</time>{diagnosticsMode && message.intent ? <span className="intent">
    {`intent:${message.intent}`}</span> : null}
    <button type="button" className={`message-copy-button${markdownCopied ? " is-copied" : ""}`} data-testid="copy-markdown-button" data-copied={markdownCopied ? "true" : null}
    onClick={handleCopyMarkdown} aria-label={t("message.copyMarkdownTitle")} title={t("message.copyMarkdownTitle")}>
    <span className="copy-button-label">{markdownCopied ? t("message.copyMarkdownDone") : t("message.copyMarkdown")}</span></button></div>{
    // Issue #488: render thinking ABOVE the answer body. Reasoning logically
    // precedes the answer (and during streaming it is the only visible part of
    // the message), so it belongs at the top of the message body, not below it.
    // Issue #541 (R6): during the staged reveal only the steps unveiled so far
    // are shown, so the trace visibly fills in before the answer appears.
    revealedThinkingSteps.length ? <ThinkingPreview steps={revealedThinkingSteps} t={t} narrative={thinkingNarrative(message.intent, t)} /> : null}{
    // Issue #672 (F3): a one-shot per-message override of the global animation
    // budget. Only offered while the reveal is actually withholding the answer,
    // so it never lingers as dead chrome on a settled message.
    reveal.active && !reveal.bodyShown ? <button type="button" className="skip-animation" data-testid="message-skip-animation" onClick={reveal.skip}
      title={t("message.skipAnimation")}>
      {t("message.skipAnimation")}</button> : null}
      <div ref={markdownRef} className={`markdown-body${bodyRevealClass}`} aria-hidden={reveal.active && !reveal.bodyShown ? "true" : null} data-testid="message-markdown-body"
      dangerouslySetInnerHTML={markdownContent} />
      <ConversationSummaryDetails message={message} messages={conversationMessages} t={t}
        className={`markdown-body${bodyRevealClass}`} ariaHidden={reveal.active && !reveal.bodyShown} />
      {message.permissionPanel && typeof renderPermissionPanel === "function" ? <div className="message-permission-panel">
      {renderPermissionPanel("desktop-permission-panel-message")}
      </div> : null}{message.commandApproval ? <CommandApprovalPanel approval={message.commandApproval}
      status={commandApprovals && commandApprovals[message.commandApproval.id] && commandApprovals[message.commandApproval.id].status} onApprove={onApproveCommand}
      onDeny={onDenyCommand} t={t} /> : null}{message.iframeUrl ? <div className={`fetch-iframe-container${iframeFullscreen ? " is-fullscreen" : ""}`}
      data-testid="fetch-iframe-container">
      <div className="fetch-iframe-header"><span className="fetch-iframe-url">{message.iframeUrl}</span><div className="fetch-iframe-actions">
      <a href={message.iframeUrl} target="_blank" rel="noopener noreferrer" className="fetch-iframe-open fetch-iframe-control" aria-label={t("fetch.openInNewTab")}
      title={t("fetch.openInNewTab")}>
      {"↗"}</a>
      <button type="button" className="fetch-iframe-toggle fetch-iframe-control" onClick={() => setIframeFullscreen(prev => !prev)}
      aria-label={iframeFullscreen ? t("fetch.minimize") : t("fetch.fullscreen")} aria-pressed={iframeFullscreen ? "true" : "false"}
      title={iframeFullscreen ? t("fetch.minimize") : t("fetch.fullscreen")}>
      {iframeFullscreen ? "⤡" : "⛶"}</button></div></div><iframe className="fetch-iframe" src={message.iframeUrl} title={t("fetch.frameTitle", {
        url: message.iframeUrl
      })} sandbox="allow-scripts allow-same-origin allow-forms allow-popups" loading="lazy" data-testid="fetch-iframe" />
        </div> : null}{evidence.length ? <div className="evidence-list">{evidence.map(item => <span key={item}>{item}</span>)}
        </div> : null}{thinkingSteps.length ? <div className="thinking-steps"><strong>{t("message.thinking")}</strong><ol>{thinkingSteps.map(item => <li key={item}>{item}</li>)}
        </ol></div> : null}{diagnosticsSteps.length ? <div className="diagnostics-steps" data-testid="diagnostics-steps"><strong>{t("message.diagnosticsSteps")}</strong>
        <ol className="diagnostics-step-list">{diagnosticsSteps.map((entry, index) => <li key={`${entry.step}-${index}`} className="diagnostics-step">
        <details className="diagnostics-detail" data-testid="diagnostics-step" data-step={entry.step} data-level={overrides.get(thinkingStepKey(entry)) || entry.level || null}
        data-solver-level={entry.level || null} data-level-override={overrides.get(thinkingStepKey(entry)) || null}
        onContextMenu={event => handleStepContextMenu(event, entry.step)}>
        <summary title={t("message.stepLevel.hint")}><span className="diagnostics-step-name">{entry.formalization ? t("message.formalization") : entry.step}</span>
        <span className="diagnostics-step-summary">{entry.formalization ? truncateDiagnosticDetail(entry.formalization.tuple) : truncateDiagnosticDetail(entry.detail)}</span>
        </summary><div className="diagnostics-detail-body">
        {entry.formalization ? <FormalizationView formalization={entry.formalization} t={t} /> : <pre className="diagnostics-payload">{formatDiagnosticPayload(entry.detail)}</pre>}
        </div></details></li>)}</ol>
        <StepHierarchyMenu menu={stepMenu}
        currentLevel={stepMenu ? overrides.get(stepMenu.step) || (diagnosticsSteps.find(entry => thinkingStepKey(entry) === stepMenu.step) || {}).level || "" : ""}
        overridden={stepMenu ? overrides.has(stepMenu.step) : false} onSelect={handleStepLevelSelect} t={t} />
        </div> : null}{diagnosticsToolCalls.length ? <div className="diagnostics-tools" data-testid="diagnostics-tools"><strong>{t("message.diagnosticsTools")}</strong>
        <ol className="diagnostics-tool-list">{diagnosticsToolCalls.map((call, index) => <li key={`${call.tool || "tool"}-${index}`} className="diagnostics-tool">
        <details className="diagnostics-detail" data-testid="diagnostics-tool"><summary><span className="diagnostics-tool-name">{call.tool || "(tool)"}</span>
        <span className="diagnostics-tool-summary">{summarizeToolCall(call)}</span></summary><div className="diagnostics-detail-body"><div className="diagnostics-tool-section">
        <span className="diagnostics-section-label">{t("message.toolInputs")}</span><pre className="diagnostics-payload">{formatDiagnosticPayload(call.inputs)}</pre></div>
        <div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.toolOutputs")}</span><pre className="diagnostics-payload">
        {formatDiagnosticPayload(call.outputs)}</pre></div>{Array.isArray(call.steps) && call.steps.length > 0 ? <div className="diagnostics-tool-section">
        <span className="diagnostics-section-label">{t("message.toolReasoning")}</span><ol className="diagnostics-tool-reasoning">
        {call.steps.map((s, j) => <li key={`${call.tool}-step-${j}`}>{`${s.step}: ${s.detail}`}</li>)}</ol></div> : null}</div></details></li>)}</ol>
        </div> : null}{diagnosticsPayload ? <DiagnosticsHttpPanel providers={diagnosticsProviders} exchanges={diagnosticsHttp} t={t} /> : null}{reportIssueUrl ? <div
        className="message-actions" data-testid="detected-failure-report">
        <span>{t("message.detectedFailureReport")}</span><a href={reportIssueUrl} target="_blank" rel="noopener noreferrer">{reportLabel}</a></div> : null}</div></article>;
}
