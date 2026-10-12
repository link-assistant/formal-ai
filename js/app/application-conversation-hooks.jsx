// App hooks for conversation messages, interface commands and agent plans.

import React from "react";
import { answerHasDetectedFailure } from "./detected-failure.js";
import { attachmentMemoryRecord } from "./attachments.jsx";
import {
  createMessage, deriveConversationTitle, generateConversationId,
} from "./conversations.jsx";
import { normalizeAssistantName } from "./interface-commands.jsx";
import { recordMemoryEvent } from "./memory-events.jsx";
import {
  DESKTOP_TOOL_OPTIONS, PREFERENCE_DEFAULTS, normalizeAnimationBudgetMs,
  normalizeBlueprintComposition, normalizeChatStyle, normalizeComposerAction,
  normalizeComposerStyle, normalizeDefinitionFusion, normalizeMode, normalizePreferredLanguage,
  normalizeResponseLanguageMode, normalizeSliderPreference, normalizeThemePreference,
  normalizeThinkingDetailLevel, normalizeToolbarIconPack, normalizeUiSkin,
} from "./preferences.jsx";
import { buildThinkingPreviewSteps } from "./thinking-steps.jsx";
import { normalizeUiLanguagePreference } from "./user-context.jsx";

const { useCallback, useEffect } = React;

// Appends user/system/assistant messages to the active conversation and
// the append-only memory log; tracks agent onboarding and pending tasks.
export function useConversationMessages({
  workerRef, setMessages, t, thinkingDetailLevel, agentMode, setAgentOnboardingSeen,
  setDesktopToolGrants, setCurrentConversationId, currentConversationRef, conversationTitlesRef,
  demoConversationIdRef, refreshConversations, demoModeRef, agentOnboardingSeenRef,
  pendingAgentTaskRef, setHasPendingAgentTask,
}) {
  // Issue #27: assign every appended event to the current conversation, lazily
  // minting a fresh id on the first user message of a brand-new chat. The
  // returned object is { conversationId, conversationTitle } so the caller can
  // reuse it for follow-up records within the same turn (assistant reply,
  // reasoning steps, tool calls).
  const ensureConversation = useCallback((seedText) => {
    // Issue #541 (R4): when demo mode is active, route every persisted event
    // into a dedicated demo conversation. We never touch
    // `currentConversationRef`/`setCurrentConversationId` from this path so the
    // user's real thread is preserved exactly as they left it — toggling demo
    // off restores their conversation untouched.
    if (demoModeRef.current) {
      if (!demoConversationIdRef.current) {
        demoConversationIdRef.current = generateConversationId();
      }
      const id = demoConversationIdRef.current;
      let title = conversationTitlesRef.current.get(id);
      if (!title) {
        title = t("buttons.demoOn") || "Demo";
        conversationTitlesRef.current.set(id, title);
      }
      return {
        conversationId: id,
        conversationTitle: title,
        isNew: false,
        isDemo: true,
      };
    }
    let id = currentConversationRef.current;
    let isNew = false;
    if (!id) {
      id = generateConversationId();
      isNew = true;
      currentConversationRef.current = id;
      setCurrentConversationId(id);
    }
    let title = conversationTitlesRef.current.get(id);
    if (!title && seedText) {
      title = deriveConversationTitle(seedText);
      conversationTitlesRef.current.set(id, title);
    }
    return { conversationId: id, conversationTitle: title || "", isNew };
  }, [t]);

  const appendUserMessage = useCallback((text, extra = {}) => {
    const { conversationId, conversationTitle, isDemo } = ensureConversation(text);
    const message = createMessage("user", text, extra);
    const memoryAttachments = Array.isArray(extra.attachments)
      ? extra.attachments.map(attachmentMemoryRecord)
      : [];
    setMessages((current) => [...current, message]);
    recordMemoryEvent({
      kind: "message",
      role: "user",
      content: text,
      sentAt: new Date().toISOString(),
      demoLabel: extra.demoLabel,
      attachments:
        memoryAttachments.length > 0
          ? JSON.stringify(memoryAttachments)
          : undefined,
      conversationId,
      conversationTitle,
      // Issue #541 (R4): flag demo turns so the sidebar can hide the demo
      // conversation and never list it alongside the user's real threads.
      isDemo: isDemo ? true : undefined,
    });
  }, [ensureConversation]);

  const appendSystemMessage = useCallback((content, extra = {}) => {
    const { conversationId, conversationTitle, isDemo } = ensureConversation("");
    const message = createMessage("system", content, {
      author: "formal-ai system",
      ...extra,
    });
    setMessages((current) => [...current, message]);
    recordMemoryEvent({
      kind: "message",
      role: "system",
      content,
      intent: extra.intent,
      sentAt: new Date().toISOString(),
      conversationId,
      conversationTitle,
      // Issue #541 (R4): flag demo turns so the sidebar can hide them.
      isDemo: isDemo ? true : undefined,
    }).then(() => {
      refreshConversations();
    });
    return message;
  }, [ensureConversation, refreshConversations]);

  const showAgentOnboarding = useCallback(() => {
    if (agentOnboardingSeenRef.current) {
      return false;
    }
    agentOnboardingSeenRef.current = true;
    setAgentOnboardingSeen(true);
    appendSystemMessage(
      [
        t("permissions.onboarding.intro"),
        t("permissions.onboarding.perTool"),
        t("permissions.onboarding.modes"),
      ].join("\n\n"),
      {
        intent: "agent_permission_onboarding",
        permissionPanel: true,
      },
    );
    return true;
  }, [appendSystemMessage, t]);

  const setDesktopToolGrant = useCallback((tool, granted) => {
    if (!DESKTOP_TOOL_OPTIONS.includes(tool)) {
      return;
    }
    setDesktopToolGrants((current) => ({
      ...current,
      [tool]: Boolean(granted),
    }));
  }, []);

  // Issue #541 (R9): record a shell command that was deferred because the user
  // was not in Agent mode (or had not granted `shell`). The "Grant all" button
  // on the permission panel reads this and replays the command.
  const capturePendingAgentTask = useCallback((command) => {
    const safeCommand = String(command || "").trim();
    if (!safeCommand) {
      return;
    }
    pendingAgentTaskRef.current = { kind: "shell", command: safeCommand };
    setHasPendingAgentTask(true);
  }, []);

  const clearPendingAgentTask = useCallback(() => {
    pendingAgentTaskRef.current = null;
    setHasPendingAgentTask(false);
  }, []);

  useEffect(() => {
    if (agentMode) {
      showAgentOnboarding();
    }
  }, [agentMode, showAgentOnboarding]);

  const appendAssistantMessage = useCallback((answer) => {
    const source = answer.source || (workerRef.current ? "worker" : "fallback");
    const solverEvidence = Array.isArray(answer.evidence) ? answer.evidence : [];
    const evidence = answer.intent
      ? [`intent:${answer.intent}`, `source:${source}`, ...solverEvidence]
      : solverEvidence;
    const structuredSteps = Array.isArray(answer.steps) ? answer.steps : [];
    const structuredToolCalls = Array.isArray(answer.toolCalls)
      ? answer.toolCalls
      : [];
    const detectedFailure = answerHasDetectedFailure(answer);
    const thinkingSteps = structuredSteps.length > 0
      ? structuredSteps.map((entry) => `${entry.step}: ${entry.detail}`)
      : [
          "Normalize prompt text",
          `Select symbolic intent ${answer.intent || "unknown"}`,
          `Render deterministic answer from ${source}`,
        ];
    const thinkingPreviewSteps = buildThinkingPreviewSteps(
      structuredSteps,
      answer,
      source,
      t,
      thinkingDetailLevel,
    );
    const message = createMessage("assistant", answer.content, {
      intent: answer.intent,
      evidence,
      thinkingSteps,
      thinkingPreviewSteps,
      thinkingPreviewSource: source,
      diagnosticsSteps: structuredSteps,
      diagnosticsToolCalls: structuredToolCalls,
      detectedFailure,
      // Issue #180: forward the web_search diagnostics envelope so the
      // diagnostics panel can show raw HTTP request/response exchanges and
      // the per-provider success/failure status.
      diagnostics: answer.diagnostics || null,
      derivationId: answer.derivation_id || answer.derivationId || null,
      iframeUrl: answer.iframeUrl || null,
      // Issue #541 (R5/R6): mark this as a freshly produced answer so the
      // Message component stages its reasoning-then-body reveal across the
      // minimum animation budget. Hydrated history (rebuilt from memory events)
      // never carries this flag, so reloads render instantly.
      animateReveal: true,
    });
    setMessages((current) => [...current, message]);
    const sentAt = new Date().toISOString();
    const { conversationId, conversationTitle, isDemo } = ensureConversation("");
    // Issue #541 (R4): flag every persisted record of this turn (reasoning
    // step, tool call, and the message itself) so the sidebar can hide the
    // demo conversation. Set undefined rather than false for non-demo turns
    // so we don't litter the IndexedDB log with redundant keys.
    const demoFlag = isDemo ? true : undefined;
    if (Array.isArray(answer.steps)) {
      answer.steps.forEach((entry) => {
        recordMemoryEvent({
          kind: "reasoning",
          role: "assistant",
          content: `${entry.step}: ${entry.detail}`,
          intent: answer.intent,
          sentAt,
          conversationId,
          conversationTitle,
          isDemo: demoFlag,
        });
      });
    }
    if (Array.isArray(answer.toolCalls)) {
      answer.toolCalls.forEach((call) => {
        recordMemoryEvent({
          kind: "tool_call",
          role: "assistant",
          tool: call.tool,
          inputs: call.inputs,
          outputs: call.outputs,
          content: `tool:${call.tool}`,
          sentAt,
          conversationId,
          conversationTitle,
          isDemo: demoFlag,
        });
      });
    }
    recordMemoryEvent({
      kind: "message",
      role: "assistant",
      content: answer.content,
      intent: answer.intent,
      evidence,
      iframeUrl: answer.iframeUrl || null,
      detectedFailure,
      sentAt,
      conversationId,
      conversationTitle,
      isDemo: demoFlag,
    }).then(() => {
      // Refresh the sidebar so a brand-new conversation appears immediately.
      refreshConversations();
    });
  }, [ensureConversation, refreshConversations, t, thinkingDetailLevel]);

  return {
    ensureConversation, appendUserMessage, appendSystemMessage, showAgentOnboarding,
    setDesktopToolGrant, capturePendingAgentTask, clearPendingAgentTask, appendAssistantMessage,
  };
}

// Conversation history, chat-driven interface commands and agent plans.
export function useInterfaceCommands({
  messages, setUiLanguagePreference, setResponseLanguage, setPreferredLanguage, setDemoMode,
  setDiagnosticsMode, setThinkingDetailLevel, setMinMessageAnimationMs, setSidebarCollapsed,
  setShowDeletedConversations, setGreetingVariations, setGuessProbability, setTemperature,
  setFollowUpProbability, setDefinitionFusion, setBlueprintComposition, setExperimentalOcr,
  setAssociativeProjectPromotion, setThemePreference, setUiSkin, setChatStyle, setComposerStyle,
  setComposerAction, setToolbarIconPack, setLocationPreference, setAssistantName, setMode,
  refreshConversations, triggerAttachFiles, requestAnswer, appendAssistantMessage,
}) {
  const conversationHistory = useCallback(
    () =>
      messages
        .filter((message) => ["user", "assistant"].includes(message.role))
        .map((message) => ({
          role: message.role,
          content: message.content,
          intent: message.intent,
          evidence: message.evidence,
        })),
    [messages],
  );

  const applyInterfaceCommand = useCallback(
    (command) => {
      if (!command) return;
      if (command.kind === "trigger" && command.action === "attach_files") {
        triggerAttachFiles();
        return;
      }
      if (command.kind !== "set_preference") {
        return;
      }
      switch (command.key) {
        case "diagnosticsMode":
          setDiagnosticsMode(Boolean(command.value));
          break;
        case "demoMode":
          setDemoMode(Boolean(command.value));
          break;
        case "agentMode":
          // Issue #513: the legacy natural-language "agent mode on/off" command
          // maps onto the three-way mode (preserving full-auto when already set).
          setMode((current) =>
            command.value
              ? current === "fullAuto"
                ? "fullAuto"
                : "agent"
              : "chat",
          );
          break;
        case "mode":
          setMode(normalizeMode(command.value));
          break;
        case "greetingVariations":
          setGreetingVariations(Boolean(command.value));
          break;
        case "definitionFusion":
          setDefinitionFusion(normalizeDefinitionFusion(command.value));
          break;
        case "blueprintComposition":
          setBlueprintComposition(
            normalizeBlueprintComposition(command.value),
          );
          break;
        case "thinkingDetailLevel":
          setThinkingDetailLevel(normalizeThinkingDetailLevel(command.value));
          break;
        case "minMessageAnimationMs":
          setMinMessageAnimationMs(normalizeAnimationBudgetMs(command.value));
          break;
        case "experimentalOcr":
          setExperimentalOcr(Boolean(command.value));
          break;
        case "associativeProjectPromotion":
          setAssociativeProjectPromotion(Boolean(command.value));
          break;
        case "theme":
          setThemePreference(normalizeThemePreference(command.value));
          break;
        case "uiLanguage":
          setUiLanguagePreference(normalizeUiLanguagePreference(command.value));
          break;
        case "responseLanguage":
          setResponseLanguage(normalizeResponseLanguageMode(command.value));
          break;
        case "preferredLanguage":
          setPreferredLanguage(normalizePreferredLanguage(command.value));
          break;
        case "uiSkin":
          setUiSkin(normalizeUiSkin(command.value));
          break;
        case "chatStyle":
          setChatStyle(normalizeChatStyle(command.value));
          break;
        case "composerStyle":
          setComposerStyle(normalizeComposerStyle(command.value));
          break;
        case "composerAction":
          setComposerAction(normalizeComposerAction(command.value));
          break;
        case "temperature":
          setTemperature(
            normalizeSliderPreference(command.value, PREFERENCE_DEFAULTS.temperature),
          );
          break;
        case "guessProbability":
          setGuessProbability(
            normalizeSliderPreference(
              command.value,
              PREFERENCE_DEFAULTS.guessProbability,
            ),
          );
          break;
        case "followUpProbability":
          setFollowUpProbability(
            normalizeSliderPreference(
              command.value,
              PREFERENCE_DEFAULTS.followUpProbability,
            ),
          );
          break;
        case "toolbarIconPack":
          setToolbarIconPack(normalizeToolbarIconPack(command.value));
          break;
        case "location":
          setLocationPreference(String(command.value || "").slice(0, 80));
          break;
        case "assistantName":
          setAssistantName(normalizeAssistantName(command.value));
          break;
        case "sidebarCollapsed":
          setSidebarCollapsed(Boolean(command.value));
          break;
        case "showDeletedConversations":
          setShowDeletedConversations(Boolean(command.value));
          refreshConversations(Boolean(command.value));
          break;
        default:
          break;
      }
    },
    [refreshConversations, triggerAttachFiles],
  );

  // Issue #27: agent mode — run a decomposed task plan and merge the per-step
  // results into a single assistant message. Each step calls the same solver
  // the chat path uses, so deterministic intents (greeting, identity,
  // arithmetic, concept lookup, etc.) behave identically; the difference is
  // surface presentation, not solver semantics.
  const runAgentPlan = useCallback(
    async (steps, history) => {
      const lines = [];
      lines.push(`## Agent plan (${steps.length} steps)`);
      steps.forEach((step, index) => {
        lines.push(`${index + 1}. ${step}`);
      });
      lines.push("");
      const aggregatedSteps = [];
      const aggregatedToolCalls = [];
      const aggregatedEvidence = [];
      let detectedFailure = false;
      const workingHistory = Array.isArray(history) ? history.slice() : [];
      for (let index = 0; index < steps.length; index += 1) {
        const step = steps[index];
        aggregatedSteps.push({
          step: "agent_plan",
          detail: `${index + 1}/${steps.length} ${step}`,
        });
        const answer = await requestAnswer(step, workingHistory);
        detectedFailure = detectedFailure || answerHasDetectedFailure(answer);
        lines.push(`### Step ${index + 1}: ${step}`);
        lines.push(answer.content || "(no output)");
        lines.push("");
        if (Array.isArray(answer.steps)) {
          answer.steps.forEach((entry) => {
            aggregatedSteps.push({
              step: `agent_${index + 1}_${entry.step}`,
              detail: entry.detail,
            });
          });
        }
        if (Array.isArray(answer.toolCalls)) {
          aggregatedToolCalls.push(...answer.toolCalls);
        }
        if (Array.isArray(answer.evidence)) {
          aggregatedEvidence.push(
            ...answer.evidence.map((item) => `step_${index + 1}:${item}`),
          );
        }
        workingHistory.push({ role: "user", content: step });
        workingHistory.push({ role: "assistant", content: answer.content || "" });
      }
      appendAssistantMessage({
        intent: "agent_plan",
        content: lines.join("\n").trim(),
        confidence: 0.85,
        evidence: ["rule:agent_mode", `steps:${steps.length}`, ...aggregatedEvidence],
        steps: aggregatedSteps,
        toolCalls: aggregatedToolCalls,
        detectedFailure,
      });
    },
    [requestAnswer, appendAssistantMessage],
  );

  return { conversationHistory, applyInterfaceCommand, runAgentPlan };
}
