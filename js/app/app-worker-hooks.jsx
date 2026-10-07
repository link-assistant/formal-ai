// App hooks around the answer engines: worker boot, browser runtime loading,
// live preference refs and prompt routing (desktop engine, API or worker).

import React from "react";
import { enhanceWithDesktopReadOnlyTool } from "./desktop-read-only-tools.js";
import { withAssetVersion } from "./app-constants.jsx";
import {
  chatAnswerFromAgentProviderResult, desktopBridge, desktopMessages,
  requestDesktopAgentProvider, requestDesktopAnswer, requestDesktopToolCall,
} from "./desktop-bridge.jsx";
import { normalizeAssistantName } from "./interface-commands.jsx";
import { localFallbackAnswer } from "./local-fallback.jsx";
import { waitForMemoryWrites } from "./memory-events.jsx";
import {
  desktopToolRouterGrants, persistPreferences, serializeDesktopToolGrants,
} from "./preferences.jsx";

const { useCallback, useEffect, useRef, useState } = React;

export function browserRuntimeStatusKey(state) {
  const status = state && state.status;
  return status === "loading" || status === "ready" || status === "failed"
    ? status
    : "available";
}

// Boots the formal-ai worker and owns the opt-in browser runtime loader.
export function useFormalAiWorker({
  workerRef, pendingResponses, setWorkerState, setWorkerReady, setEngineUnavailable,
  diagnosticJsFallback, browserRuntimeState, setBrowserRuntimeState, uiLanguagePreference,
  responseLanguage, preferredLanguage, demoMode, diagnosticsMode, thinkingDetailLevel,
  minMessageAnimationMs, contextPanelWidth, sidebarMenuCollapsed, sidebarDesktopCollapsed,
  sidebarPromptsCollapsed, sidebarToolsCollapsed, sidebarTraceCollapsed,
  sidebarConversationsCollapsed, sidebarSettingsCollapsed, sidebarCollapsed,
  showDeletedConversations, greetingVariations, guessProbability, temperature,
  followUpProbability, definitionFusion, blueprintComposition, experimentalOcr,
  externalServices, associativeProjectPromotion, themePreference, uiSkin, glassOpacity,
  chatStyle, composerStyle, composerAction, toolbarIconPack, locationPreference, assistantName,
  sidebarServicesCollapsed, mode, agentMode, agentOnboardingSeen, desktopToolGrants,
  currentConversationId,
}) {
  useEffect(() => {
    persistPreferences({
      demoMode,
      diagnosticsMode,
      contextPanelWidth,
      sidebarMenuCollapsed,
      sidebarDesktopCollapsed,
      sidebarServicesCollapsed,
      sidebarPromptsCollapsed,
      sidebarToolsCollapsed,
      sidebarTraceCollapsed,
      sidebarConversationsCollapsed,
      sidebarSettingsCollapsed,
      sidebarCollapsed,
      showDeletedConversations,
      greetingVariations,
      guessProbability,
      temperature,
      followUpProbability,
      definitionFusion,
      blueprintComposition,
      thinkingDetailLevel,
      minMessageAnimationMs,
      experimentalOcr,
      ...externalServices,
      associativeProjectPromotion,
      theme: themePreference,
      uiSkin,
      glassOpacity,
      chatStyle,
      composerStyle,
      composerAction,
      toolbarIconPack,
      location: locationPreference,
      assistantName: normalizeAssistantName(assistantName),
      currentConversationId,
      mode,
      agentMode,
      agentOnboardingSeen,
      desktopToolGrants: serializeDesktopToolGrants(desktopToolGrants),
      uiLanguage: uiLanguagePreference,
      responseLanguage,
      preferredLanguage,
    });
  }, [
    demoMode,
    diagnosticsMode,
    contextPanelWidth,
    sidebarMenuCollapsed,
    sidebarDesktopCollapsed,
    sidebarServicesCollapsed,
    sidebarPromptsCollapsed,
    sidebarToolsCollapsed,
    sidebarTraceCollapsed,
    sidebarConversationsCollapsed,
    sidebarSettingsCollapsed,
    sidebarCollapsed,
    showDeletedConversations,
    greetingVariations,
    guessProbability,
    temperature,
    followUpProbability,
    definitionFusion,
    blueprintComposition,
    thinkingDetailLevel,
    minMessageAnimationMs,
    experimentalOcr,
    externalServices,
    associativeProjectPromotion,
    themePreference,
    uiSkin,
    glassOpacity,
    chatStyle,
    composerStyle,
    composerAction,
    toolbarIconPack,
    locationPreference,
    assistantName,
    currentConversationId,
    mode,
    agentMode,
    agentOnboardingSeen,
    desktopToolGrants,
    uiLanguagePreference,
    responseLanguage,
    preferredLanguage,
  ]);

  useEffect(() => {
    // Issue #934: the diagnostic JS fallback is a dev-only override the worker
    // reads from its own URL, so it must be appended before the script loads.
    const workerUrl = withAssetVersion("worker/formal_ai_worker.js");
    const worker = new Worker(
      diagnosticJsFallback
        ? `${workerUrl}${workerUrl.includes("?") ? "&" : "?"}jsfallback=1`
        : workerUrl,
    );
    workerRef.current = worker;
    worker.onmessage = (event) => {
      if (event.data.kind === "ready") {
        setWorkerState(event.data.mode);
        setWorkerReady(true);
        if (event.data.mode === "engine unavailable") {
          setEngineUnavailable(String(event.data.engineError || "unknown error"));
        }
        return;
      }
      if (event.data.kind === "engine_unavailable") {
        setEngineUnavailable(String(event.data.error || "unknown error"));
        setWorkerReady(true);
        return;
      }

      const requestId = event.data.requestId;
      const resolver = pendingResponses.current.get(requestId);
      if (resolver) {
        pendingResponses.current.delete(requestId);
        resolver(event.data);
      }
    };

    return () => worker.terminate();
  }, []);

  // #670/#1138: Pyodide is never fetched during page or worker startup. This
  // handler is bound only to the size-labelled button in Settings, making the
  // network and storage cost an explicit user choice.
  const loadBrowserRuntime = useCallback(() => {
    const worker = workerRef.current;
    if (!worker || browserRuntimeState.status === "loading") return;
    setBrowserRuntimeState({ status: "loading", error: "" });
    const requestId = `browser-runtime-${Date.now()}-${Math.random().toString(16).slice(2)}`;
    pendingResponses.current.set(requestId, (result) => {
      const status = result?.probe?.status === "ready" ? "ready" : "failed";
      setBrowserRuntimeState({ status, error: String(result?.error || "") });
    });
    worker.postMessage({ kind: "browser_runtime_load", requestId });
  }, [browserRuntimeState.status]);

  return { loadBrowserRuntime };
}

// Mirrors live preference state into refs so async callbacks (worker
// requests, desktop calls) always read the current values.
export function useLatestValueRefs({
  uiLanguagePreference, responseLanguage, preferredLanguage, demoMode, diagnosticsMode,
  greetingVariations, guessProbability, temperature, followUpProbability, definitionFusion,
  blueprintComposition, experimentalOcr, externalServices, associativeProjectPromotion,
  themePreference, uiSkin, chatStyle, composerStyle, composerAction, locationPreference,
  assistantName, desktopStatus, mode, agentMode, agentOnboardingSeen, desktopToolGrants,
  commandApprovals,
}) {
  const greetingVariationsRef = useRef(greetingVariations);
  useEffect(() => {
    greetingVariationsRef.current = greetingVariations;
  }, [greetingVariations]);

  const diagnosticsModeRef = useRef(diagnosticsMode);
  useEffect(() => {
    diagnosticsModeRef.current = diagnosticsMode;
  }, [diagnosticsMode]);

  const demoModeRef = useRef(demoMode);
  useEffect(() => {
    demoModeRef.current = demoMode;
  }, [demoMode]);

  const guessProbabilityRef = useRef(guessProbability);
  useEffect(() => {
    guessProbabilityRef.current = guessProbability;
  }, [guessProbability]);

  const temperatureRef = useRef(temperature);
  useEffect(() => {
    temperatureRef.current = temperature;
  }, [temperature]);

  const followUpProbabilityRef = useRef(followUpProbability);
  useEffect(() => {
    followUpProbabilityRef.current = followUpProbability;
  }, [followUpProbability]);

  const definitionFusionRef = useRef(definitionFusion);
  useEffect(() => {
    definitionFusionRef.current = definitionFusion;
  }, [definitionFusion]);

  const blueprintCompositionRef = useRef(blueprintComposition);
  useEffect(() => {
    blueprintCompositionRef.current = blueprintComposition;
  }, [blueprintComposition]);

  const experimentalOcrRef = useRef(experimentalOcr);
  useEffect(() => {
    experimentalOcrRef.current = experimentalOcr;
  }, [experimentalOcr]);

  // Issue #444: mirror the external trusted-service toggles into a ref so the
  // worker prefs payload (assembled outside React render) reads the live values.
  const externalServicesRef = useRef(externalServices);
  useEffect(() => {
    externalServicesRef.current = externalServices;
  }, [externalServices]);

  const associativeProjectPromotionRef = useRef(associativeProjectPromotion);
  useEffect(() => {
    associativeProjectPromotionRef.current = associativeProjectPromotion;
  }, [associativeProjectPromotion]);

  const agentModeRef = useRef(agentMode);
  useEffect(() => {
    agentModeRef.current = agentMode;
  }, [agentMode]);

  // Issue #513: mirror the three-way operating mode for the worker prefs payload.
  const modeRef = useRef(mode);
  useEffect(() => {
    modeRef.current = mode;
  }, [mode]);

  const agentOnboardingSeenRef = useRef(agentOnboardingSeen);
  useEffect(() => {
    agentOnboardingSeenRef.current = agentOnboardingSeen;
  }, [agentOnboardingSeen]);

  const desktopToolGrantsRef = useRef(desktopToolGrants);
  useEffect(() => {
    desktopToolGrantsRef.current = desktopToolGrants;
  }, [desktopToolGrants]);

  const commandApprovalsRef = useRef(commandApprovals);
  useEffect(() => {
    commandApprovalsRef.current = commandApprovals;
  }, [commandApprovals]);

  // Issue #541 (R9): when the chat surface refuses to execute a shell command
  // because the user is not in Agent mode or has not granted `shell`, we stash
  // the original command here so a single click on the permission panel's
  // "Grant all and switch to Agent mode" button can replay it. The ref holds
  // the live value (read inside async callbacks); the state mirrors whether
  // a task is queued so the panel can change its button copy.
  const pendingAgentTaskRef = useRef(null);
  const [hasPendingAgentTask, setHasPendingAgentTask] = useState(false);

  const themePreferenceRef = useRef(themePreference);
  useEffect(() => {
    themePreferenceRef.current = themePreference;
  }, [themePreference]);

  const uiLanguagePreferenceRef = useRef(uiLanguagePreference);
  useEffect(() => {
    uiLanguagePreferenceRef.current = uiLanguagePreference;
  }, [uiLanguagePreference]);

  const responseLanguageRef = useRef(responseLanguage);
  useEffect(() => {
    responseLanguageRef.current = responseLanguage;
  }, [responseLanguage]);

  const preferredLanguageRef = useRef(preferredLanguage);
  useEffect(() => {
    preferredLanguageRef.current = preferredLanguage;
  }, [preferredLanguage]);

  const uiSkinRef = useRef(uiSkin);
  useEffect(() => {
    uiSkinRef.current = uiSkin;
  }, [uiSkin]);

  const chatStyleRef = useRef(chatStyle);
  useEffect(() => {
    chatStyleRef.current = chatStyle;
  }, [chatStyle]);

  const composerStyleRef = useRef(composerStyle);
  useEffect(() => {
    composerStyleRef.current = composerStyle;
  }, [composerStyle]);

  const composerActionRef = useRef(composerAction);
  useEffect(() => {
    composerActionRef.current = composerAction;
  }, [composerAction]);

  const locationPreferenceRef = useRef(locationPreference);
  useEffect(() => {
    locationPreferenceRef.current = locationPreference;
  }, [locationPreference]);

  const assistantNameRef = useRef(assistantName);
  useEffect(() => {
    assistantNameRef.current = assistantName;
  }, [assistantName]);

  const desktopStatusRef = useRef(desktopStatus);
  useEffect(() => {
    desktopStatusRef.current = desktopStatus;
  }, [desktopStatus]);

  return {
    greetingVariationsRef, diagnosticsModeRef, demoModeRef, guessProbabilityRef, temperatureRef,
    followUpProbabilityRef, definitionFusionRef, blueprintCompositionRef, experimentalOcrRef,
    externalServicesRef, associativeProjectPromotionRef, agentModeRef, modeRef,
    agentOnboardingSeenRef, desktopToolGrantsRef, commandApprovalsRef, pendingAgentTaskRef,
    hasPendingAgentTask, setHasPendingAgentTask, themePreferenceRef, uiLanguagePreferenceRef,
    responseLanguageRef, preferredLanguageRef, uiSkinRef, chatStyleRef, composerStyleRef,
    composerActionRef, locationPreferenceRef, assistantNameRef, desktopStatusRef,
  };
}

// Routes a prompt to the desktop engine, the local API or the worker.
export function useAnswerRequest({
  workerRef, pendingResponses, setDesktopAgentStream, currentConversationRef, userContextRef,
  greetingVariationsRef, diagnosticsModeRef, demoModeRef, guessProbabilityRef, temperatureRef,
  followUpProbabilityRef, definitionFusionRef, blueprintCompositionRef, experimentalOcrRef,
  externalServicesRef, associativeProjectPromotionRef, agentModeRef, modeRef,
  desktopToolGrantsRef, themePreferenceRef, uiLanguagePreferenceRef, responseLanguageRef,
  preferredLanguageRef, uiSkinRef, chatStyleRef, composerStyleRef, composerActionRef,
  locationPreferenceRef, assistantNameRef, desktopStatusRef,
}) {
  const requestAnswer = useCallback(async (text, history = []) => {
    const worker = workerRef.current;
    // Issue #529: snapshot every searchable persistent-memory value (after the
    // previous answer's background writes, e.g. its #1184 derivation record,
    // settle) so the worker sees them; handleMemoryOperation writes back.
    let memory = [];
    let memoryEvents = [];
    if (typeof window !== "undefined" && window.FormalAiMemory) {
      try {
        await waitForMemoryWrites();
        memoryEvents = await window.FormalAiMemory.listEvents();
        memory = await window.FormalAiMemory.collectSearchableValues();
      } catch (_error) {
        memory = []; memoryEvents = [];
      }
    }
    const prefs = {
      greetingVariations: greetingVariationsRef.current,
      diagnosticsMode: diagnosticsModeRef.current,
      demoMode: demoModeRef.current,
      guessProbability: guessProbabilityRef.current,
      temperature: temperatureRef.current,
      followUpProbability: followUpProbabilityRef.current,
      definitionFusion: definitionFusionRef.current,
      blueprintComposition: blueprintCompositionRef.current,
      experimentalOcr: experimentalOcrRef.current,
      // Issue #444: forward every external trusted-service opt-out so the worker
      // can skip a disabled service's live fetch.
      ...externalServicesRef.current,
      associativeProjectPromotion: associativeProjectPromotionRef.current,
      agentMode: agentModeRef.current,
      mode: modeRef.current,
      theme: themePreferenceRef.current,
      uiLanguage: uiLanguagePreferenceRef.current,
      responseLanguage: responseLanguageRef.current,
      preferredLanguage: preferredLanguageRef.current,
      uiSkin: uiSkinRef.current,
      chatStyle: chatStyleRef.current,
      composerStyle: composerStyleRef.current,
      composerAction: composerActionRef.current,
      location: locationPreferenceRef.current,
      assistantName: normalizeAssistantName(assistantNameRef.current),
    };
    const currentDesktopStatus = desktopStatusRef.current;
    let answerPromise;
    if (currentDesktopStatus && currentDesktopStatus.activeEngine !== "out-of-box") {
      const bridge = desktopBridge();
      const requestId = `agent-${Date.now()}-${Math.random().toString(16).slice(2)}`;
      setDesktopAgentStream([]);
      answerPromise = requestDesktopAgentProvider(bridge, {
        requestId,
        sessionKey: currentConversationRef.current || "new-conversation",
        mode: modeRef.current,
        prompt: text,
        systemPrompt: desktopMessages(history, "")
          .filter((entry) => entry.content.trim())
          .map((entry) => `${entry.role}: ${entry.content}`).join("\n"),
        grants: desktopToolRouterGrants(modeRef.current, desktopToolGrantsRef.current),
      }).then((result) => chatAnswerFromAgentProviderResult(result) || {
        intent: "agent_cli_error",
        content: String(result && result.reason || "The selected desktop engine did not return an answer."),
        evidence: [`desktop_engine:${currentDesktopStatus.activeEngine}`],
        steps: [],
        toolCalls: [],
      });
    } else if (currentDesktopStatus && currentDesktopStatus.apiReady && currentDesktopStatus.apiBase) {
      answerPromise = requestDesktopAnswer(text, history, currentDesktopStatus, prefs).catch(() => {
        if (!worker) {
          return localFallbackAnswer(text, history, prefs);
        }
        return new Promise((resolve) => {
          const requestId = `request-${Date.now()}-${Math.random().toString(16).slice(2)}`;
          pendingResponses.current.set(requestId, resolve);
          worker.postMessage({
            prompt: text,
            requestId,
            history,
            prefs,
            userContext: userContextRef.current,
            memory,
            memoryEvents,
          });
        });
      });
    } else if (!worker) {
      answerPromise = Promise.resolve(localFallbackAnswer(text, history, prefs));
    } else {
      answerPromise = new Promise((resolve) => {
        const requestId = `request-${Date.now()}-${Math.random().toString(16).slice(2)}`;
        pendingResponses.current.set(requestId, resolve);
        worker.postMessage({
          prompt: text,
          requestId,
          history,
          prefs,
          userContext: userContextRef.current,
          memory,
          memoryEvents,
        });
      });
    }
    const answer = await answerPromise;
    const bridge = desktopBridge();
    return enhanceWithDesktopReadOnlyTool(answer, (tool, input) =>
      requestDesktopToolCall(bridge, tool, input),
    );
  }, []);

  return { requestAnswer };
}
