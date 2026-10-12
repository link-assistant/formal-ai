// The App component: owns the chat state and renders the context panel,
// transcript and composer. Feature logic lives in the sibling modules and the
// app-*-hooks.jsx files it composes.

import React from "react";
import { chakra } from "@chakra-ui/react";
import { MenuGlyph, SidebarToggleGlyph } from "./glyphs.jsx";
import { DebuggerView } from "./debugger-view.jsx";
// Issue #825: the shared autocomplete engine backs every own-input-box
// surface — the chat composer here and the free-text settings fields below.
// Ranking, the keyboard contract and the localStorage input history live in
// one place; this file only supplies the suggestion vocabulary and renders.
import {
  COMMAND_SUGGESTIONS, createAutocompleteController, examplePromptSuggestions,
  historySuggestions, recallInputValues, rememberInputValue,
} from "./autocomplete.js";
import { decomposeAgentTask } from "./agent-plan.jsx";
import { EXAMPLE_PROMPTS, SOURCE_CODE_URL } from "./application-constants.js";
import { useConversationMessages, useInterfaceCommands } from "./application-conversation-hooks.jsx";
import { useDesktopCommands, useDesktopIntegration } from "./application-desktop-hooks.jsx";
import { useContextPanelResize, useDocumentEnvironment } from "./application-layout-hooks.jsx";
import { useMemoryActions, useMemoryOperation } from "./application-memory-hooks.jsx";
import {
  browserRuntimeStatusKey, useAnswerRequest, useFormalAiWorker, useLatestValueRefs,
} from "./application-worker-hooks.jsx";
import {
  OCR_DOWNLOAD_WARNING, attachmentOnlyPrompt, buildPromptWithAttachments, useAttachments,
} from "./attachments.jsx";
import {
  conversationToMarkdown, groupConversations, localizeTool, messagesForConversation,
  resizeComposerInput,
} from "./conversations.jsx";
import { useDemoPlayback } from "./demo-mode.jsx";
import {
  DataMigrationNotice, compactUrl, desktopAgentEventLabel, desktopAppVersionLabel,
  desktopBridge, desktopStatusLabel, desktopSurfaceLabel, desktopUpdaterBusy,
  desktopUpdaterStateLabel, normalizeDesktopStatus, serviceStateLabel,
  terminalCommandFromAnswer, vscodeInstallStateLabel,
} from "./desktop-bridge.jsx";
import {
  commandValueLabel, interfaceCommandResponse, normalizeAssistantName,
  recognizeInterfaceCommand, recognizeMemoryAction, sanitizeAssistantNameInput,
} from "./interface-commands.jsx";
import { createIssueUrl, shouldOfferMessageReport } from "./issue-reporting.jsx";
import { copyTextToClipboard } from "./markdown-render.jsx";
import { recordMemoryEvent } from "./memory-events.jsx";
import { DesktopPermissionPanel, Message, PendingAssistantBubble } from "./message-view.jsx";
import {
  CONTEXT_PANEL_MIN_WIDTH, DESKTOP_TOOL_OPTIONS, EXTERNAL_TRUSTED_SERVICES, MODE_LABEL_KEYS,
  MODE_OPTIONS, MODE_TITLE_KEYS, PREFERENCE_DEFAULTS, clampNumber, contextPanelMaxWidth,
  desktopToolGrantCount, formatSliderValue, loadPreferences, normalizeAnimationBudgetMs,
  normalizeBlueprintComposition, normalizeChatStyle, normalizeComposerAction,
  normalizeComposerStyle, normalizeContextPanelWidth, normalizeDefinitionFusion,
  normalizeDesktopToolGrants, normalizeMode, normalizePreferredLanguage,
  normalizeResponseLanguageMode, normalizeSliderPreference, normalizeThemePreference,
  normalizeThinkingDetailLevel, normalizeToolbarIconPack, normalizeUiSkin, settingIsDefault,
} from "./preferences.jsx";
import { buildRecallReport, recognizeRecallQuery } from "./recall-query.jsx";
import { CollapsibleSection, SIDEBAR_SECTION_TEST_IDS } from "./sidebar-section.jsx";
import { appendStepLevelEvent, projectStepLevels } from "./thinking-steps.jsx";
import { ToolbarButton, ToolbarIcon } from "./toolbar-icons.jsx";
import {
  collectUserContext, detectUiLanguage, normalizeUiLanguagePreference, translateUi,
} from "./user-context.jsx";

const { createElement: h, useCallback, useEffect, useMemo, useRef, useState } = React;

export function App() {
  const workerRef = useRef(null);
  const pendingResponses = useRef(new Map());
  const transcriptEndRef = useRef(null);
  const importInputRef = useRef(null);
  const attachmentInputRef = useRef(null);
  const composerInputRef = useRef(null);
  const [messages, setMessages] = useState([]);
  const [prompt, setPrompt] = useState("");
  // Issue #825: composer autocomplete state ({ open, items, activeIndex })
  // mirrors the controller snapshot so React re-renders the listbox.
  const [composerAutocomplete, setComposerAutocomplete] = useState({
    open: false,
    items: [],
    activeIndex: -1,
  });
  const composerHistoryRef = useRef([]);
  const autocompleteControllerRef = useRef(null);
  if (autocompleteControllerRef.current === null) {
    autocompleteControllerRef.current = createAutocompleteController({
      getItems: () => [
        ...COMMAND_SUGGESTIONS,
        ...examplePromptSuggestions(EXAMPLE_PROMPTS),
        ...historySuggestions(composerHistoryRef.current),
      ],
      onComplete: value => {
        setPrompt(value);
        setComposerAutocomplete({ open: false, items: [], activeIndex: -1 });
      },
    });
  }
  // The user's own earlier prompts are the third suggestion source; loaded
  // once per session (localStorage is per-browser and never synced).
  const [assistantNameHistory, setAssistantNameHistory] = useState([]);
  const [locationHistory, setLocationHistory] = useState([]);
  useEffect(() => {
    composerHistoryRef.current = recallInputValues("composerPrompts");
    setAssistantNameHistory(recallInputValues("assistantName"));
    setLocationHistory(recallInputValues("location"));
  }, []);
  const [pending, setPending] = useState(false);
  const [workerState, setWorkerState] = useState("loading worker");
  const [workerReady, setWorkerReady] = useState(false);
  // Issue #934: the engine that failed to load must be visible, not silent.
  const [engineUnavailable, setEngineUnavailable] = useState("");
  const [diagnosticJsFallback, setDiagnosticJsFallback] = useState(() => {
    try {
      return window.localStorage.getItem("formalAiDiagnosticJsFallback") === "1";
    } catch (_error) {
      return false;
    }
  });
  const [browserRuntimeState, setBrowserRuntimeState] = useState({
    status: "available_to_download",
    error: "",
  });
  const [memoryStatus, setMemoryStatus] = useState("");
  const [composerMenuOpen, setComposerMenuOpen] = useState(false);
  const [attachments, setAttachments] = useState([]);
  const [seed, setSeed] = useState({
    raw: {},
    tools: [],
    concepts: [],
    responses: {},
    interfaceCapabilities: [],
  });
  const initialPreferences = useRef(loadPreferences());
  const [uiLanguagePreference, setUiLanguagePreference] = useState(
    normalizeUiLanguagePreference(initialPreferences.current.uiLanguage),
  );
  // Issue #324: which language drives responses, and the pinned language used
  // when the mode is "preferred".
  const [responseLanguage, setResponseLanguage] = useState(
    normalizeResponseLanguageMode(initialPreferences.current.responseLanguage),
  );
  const [preferredLanguage, setPreferredLanguage] = useState(
    normalizePreferredLanguage(initialPreferences.current.preferredLanguage),
  );
  const [i18nRuntimeTick, setI18nRuntimeTick] = useState(0);
  const uiLanguage = detectUiLanguage(uiLanguagePreference);
  const t = useCallback(
    (key, params) => translateUi(key, uiLanguage, params),
    [uiLanguage, i18nRuntimeTick],
  );
  const [demoMode, setDemoMode] = useState(initialPreferences.current.demoMode);
  const [demoPhase, setDemoPhase] = useState("manual");
  const [demoCountdown, setDemoCountdown] = useState(null);
  const [diagnosticsMode, setDiagnosticsMode] = useState(
    Boolean(window.FORMAL_AI_DEBUG_VIEW) || initialPreferences.current.diagnosticsMode,
  );
  const [thinkingDetailLevel, setThinkingDetailLevel] = useState(
    normalizeThinkingDetailLevel(initialPreferences.current.thinkingDetailLevel),
  );
  // Issue #541 (R5): minimum wall-clock budget for the reasoning + reveal
  // animation of a freshly produced answer.
  const [minMessageAnimationMs, setMinMessageAnimationMs] = useState(
    normalizeAnimationBudgetMs(initialPreferences.current.minMessageAnimationMs),
  );
  // Issue #672 (F4): hierarchy edits are stored as the append-only log itself,
  // not as the resulting map. Keeping the events means an edit is a fact that
  // happened ("the user demoted `calculator_eval` at this point"), and the map
  // the UI reads is a projection recomputed from them — the same shape as the
  // links/events model the rest of the system uses. It is deliberately
  // renderer-local and session-scoped: this is a viewing preference about one
  // person's trace, not something to write back into the message record.
  const [stepLevelEvents, setStepLevelEvents] = useState([]);
  const stepLevelOverrides = useMemo(
    () => projectStepLevels(stepLevelEvents),
    [stepLevelEvents],
  );
  const editStepLevel = useCallback((step, level) => {
    setStepLevelEvents((events) => appendStepLevelEvent(events, step, level));
  }, []);
  const [contextPanelWidth, setContextPanelWidth] = useState(
    normalizeContextPanelWidth(initialPreferences.current.contextPanelWidth),
  );
  // Issue #27: sidebar collapse/expand state per section.
  const [sidebarMenuCollapsed, setSidebarMenuCollapsed] = useState(
    initialPreferences.current.sidebarMenuCollapsed,
  );
  const [sidebarDesktopCollapsed, setSidebarDesktopCollapsed] = useState(
    initialPreferences.current.sidebarDesktopCollapsed,
  );
  const [sidebarPromptsCollapsed, setSidebarPromptsCollapsed] = useState(
    initialPreferences.current.sidebarPromptsCollapsed,
  );
  const [sidebarToolsCollapsed, setSidebarToolsCollapsed] = useState(
    initialPreferences.current.sidebarToolsCollapsed,
  );
  const [sidebarTraceCollapsed, setSidebarTraceCollapsed] = useState(
    initialPreferences.current.sidebarTraceCollapsed,
  );
  const [sidebarConversationsCollapsed, setSidebarConversationsCollapsed] = useState(
    initialPreferences.current.sidebarConversationsCollapsed,
  );
  const [sidebarSettingsCollapsed, setSidebarSettingsCollapsed] = useState(
    initialPreferences.current.sidebarSettingsCollapsed,
  );
  // Issue #153: persistent desktop sidebar collapse — separate from the
  // transient `mobileMenuOpen` drawer so wide-screen layouts can dedicate the
  // viewport to chat without losing the user's accordion state.
  const [sidebarCollapsed, setSidebarCollapsed] = useState(
    Boolean(initialPreferences.current.sidebarCollapsed),
  );
  const [showDeletedConversations, setShowDeletedConversations] = useState(
    Boolean(initialPreferences.current.showDeletedConversations),
  );
  const showDeletedConversationsRef = useRef(showDeletedConversations);
  const [greetingVariations, setGreetingVariations] = useState(
    initialPreferences.current.greetingVariations,
  );
  const [guessProbability, setGuessProbability] = useState(
    normalizeSliderPreference(
      initialPreferences.current.guessProbability,
      PREFERENCE_DEFAULTS.guessProbability,
    ),
  );
  const [temperature, setTemperature] = useState(
    normalizeSliderPreference(
      initialPreferences.current.temperature,
      PREFERENCE_DEFAULTS.temperature,
    ),
  );
  const [followUpProbability, setFollowUpProbability] = useState(
    normalizeSliderPreference(
      initialPreferences.current.followUpProbability,
      PREFERENCE_DEFAULTS.followUpProbability,
    ),
  );
  const [definitionFusion, setDefinitionFusion] = useState(
    normalizeDefinitionFusion(initialPreferences.current.definitionFusion),
  );
  const [blueprintComposition, setBlueprintComposition] = useState(
    normalizeBlueprintComposition(initialPreferences.current.blueprintComposition),
  );
  const [experimentalOcr, setExperimentalOcr] = useState(
    Boolean(initialPreferences.current.experimentalOcr),
  );
  // Issue #444: one boolean per external trusted service, kept in a single map so
  // the catalog stays the only place that enumerates the services. A missing
  // stored value defaults to enabled (opt-out model).
  const [externalServices, setExternalServices] = useState(() =>
    Object.fromEntries(
      EXTERNAL_TRUSTED_SERVICES.map((service) => [
        service.key,
        initialPreferences.current[service.key] !== false,
      ]),
    ),
  );
  const setExternalService = useCallback((key, value) => {
    setExternalServices((prev) => ({ ...prev, [key]: Boolean(value) }));
  }, []);
  const [associativeProjectPromotion, setAssociativeProjectPromotion] = useState(
    initialPreferences.current.associativeProjectPromotion !== false,
  );
  const [themePreference, setThemePreference] = useState(
    normalizeThemePreference(initialPreferences.current.theme),
  );
  const [uiSkin, setUiSkin] = useState(
    normalizeUiSkin(initialPreferences.current.uiSkin),
  );
  const [glassOpacity, setGlassOpacity] = useState(
    clampNumber(initialPreferences.current.glassOpacity, 0.35, 1, 0.78),
  );
  const [chatStyle, setChatStyle] = useState(
    normalizeChatStyle(initialPreferences.current.chatStyle),
  );
  const [composerStyle, setComposerStyle] = useState(
    normalizeComposerStyle(initialPreferences.current.composerStyle),
  );
  const [composerAction, setComposerAction] = useState(
    normalizeComposerAction(initialPreferences.current.composerAction),
  );
  const [toolbarIconPack, setToolbarIconPack] = useState(
    normalizeToolbarIconPack(initialPreferences.current.toolbarIconPack),
  );
  const sidebarExpandOnlyLabel = t("buttons.expandOnlySection");
  const sidebarExpandOnlyTitle = t("titles.expandOnlySection");
  const SidebarSection = useCallback((props) => (
    <CollapsibleSection
      {...props}
      expandOnlyLabel={sidebarExpandOnlyLabel}
      expandOnlyTitle={sidebarExpandOnlyTitle}
      iconPack={toolbarIconPack}
    />
  ), [sidebarExpandOnlyLabel, sidebarExpandOnlyTitle, toolbarIconPack]);
  const [locationPreference, setLocationPreference] = useState(
    String(initialPreferences.current.location || ""),
  );
  const [assistantName, setAssistantName] = useState(
    normalizeAssistantName(initialPreferences.current.assistantName),
  );
  const [desktopStatus, setDesktopStatus] = useState(null);
  // Issue #672 (F2): the outcome of the desktop profile migration, so the user
  // can see that their data was carried forward — and ask for another pass if
  // something looks missing. `dataMigrationBusy` disables the replay button
  // while a pass is in flight; `dataMigrationDismissed` hides the notice for the
  // rest of the session once the user has acknowledged it.
  const [dataMigration, setDataMigration] = useState(null);
  const [dataMigrationBusy, setDataMigrationBusy] = useState(false);
  const [dataMigrationDismissed, setDataMigrationDismissed] = useState(false);
  const [desktopAgentStream, setDesktopAgentStream] = useState([]);
  // Issue #438 (follow-up): one-click start/stop of the prepared Docker
  // containers (Telegram bot + OpenAI-compatible server). `serviceStatus` holds
  // the latest snapshot from the desktop bridge; `serviceBusy` names the service
  // currently starting/stopping so its buttons disable; `telegramToken` backs the
  // inline token field the bot needs before it can start.
  const [serviceStatus, setServiceStatus] = useState(null);
  const [serviceBusy, setServiceBusy] = useState("");
  const [serviceError, setServiceError] = useState("");
  const [telegramToken, setTelegramToken] = useState("");
  const [sidebarServicesCollapsed, setSidebarServicesCollapsed] = useState(
    initialPreferences.current.sidebarServicesCollapsed,
  );
  const [updateBusy, setUpdateBusy] = useState("");
  const isolateSidebarSection = useCallback((testId) => {
    const activeSection = String(testId || "");
    if (!SIDEBAR_SECTION_TEST_IDS.includes(activeSection)) return;
    setSidebarMenuCollapsed(activeSection !== "drawer-menu-actions");
    setSidebarDesktopCollapsed(activeSection !== "sidebar-desktop");
    setSidebarServicesCollapsed(activeSection !== "sidebar-services");
    setSidebarConversationsCollapsed(activeSection !== "sidebar-conversations");
    setSidebarSettingsCollapsed(activeSection !== "sidebar-settings");
    setSidebarPromptsCollapsed(activeSection !== "sidebar-prompts");
    setSidebarToolsCollapsed(activeSection !== "sidebar-tools");
    setSidebarTraceCollapsed(activeSection !== "sidebar-trace");
  }, []);
  const handleSidebarSectionClickCapture = useCallback((event) => {
    const target = event.target;
    if (!target || typeof target.closest !== "function") return;
    const isolateButton = target.closest("[data-sidebar-section-action='isolate']");
    const shiftedHeader = event.shiftKey
      ? target.closest(".sidebar-section-header")
      : null;
    if (!isolateButton && !shiftedHeader) return;
    const section = target.closest(".sidebar-section");
    if (!section) return;
    event.preventDefault();
    event.stopPropagation();
    isolateSidebarSection(section.getAttribute("data-testid"));
  }, [isolateSidebarSection]);
  // Issue #554 (R2): one-click install of the formal-ai VS Code extension from
  // the desktop app. `vscodeInstallBusy` gates the button; `vscodeInstallResult`
  // holds the last {ok,state,reason} the main process returned.
  const [vscodeInstallBusy, setVscodeInstallBusy] = useState(false);
  const [vscodeInstallResult, setVscodeInstallResult] = useState(null);
  // Issue #27 / #513: the operating mode runs the user's prompt as a single
  // Q&A ("chat"), a multi-step plan ("agent"), or an auto-executing agent
  // ("fullAuto"). Persisted across reloads via preferences. The legacy
  // `agentMode` boolean is derived so existing readers keep working.
  const [mode, setMode] = useState(
    normalizeMode(
      initialPreferences.current.mode,
      initialPreferences.current.agentMode,
    ),
  );
  const agentMode = mode !== "chat";
  const [agentOnboardingSeen, setAgentOnboardingSeen] = useState(
    Boolean(initialPreferences.current.agentOnboardingSeen),
  );
  const [desktopToolGrants, setDesktopToolGrants] = useState(() =>
    normalizeDesktopToolGrants(initialPreferences.current.desktopToolGrants),
  );
  const [commandApprovals, setCommandApprovals] = useState({});
  // Issue #27: a mobile-friendly slide-out menu that hosts the entire sidebar
  // plus the topbar action buttons. On wide screens the menu is hidden via CSS.
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [colorSchemeTick, setColorSchemeTick] = useState(0);
  // Issue #27: conversations. `currentConversationId` is the thread the user is
  // typing in right now; on first user message the demo lazily mints a new id
  // if none is set. `conversations` is the sidebar-visible list of all known
  // threads, derived from the append-only event log and refreshed after every
  // turn.
  const [currentConversationId, setCurrentConversationId] = useState(
    initialPreferences.current.currentConversationId || "",
  );
  const [conversations, setConversations] = useState([]);
  // Issue #386: id of the conversation whose "copy as Markdown" button last
  // succeeded, so the entry can flash a short confirmation label.
  const [copiedConversationId, setCopiedConversationId] = useState("");
  const currentConversationRef = useRef(currentConversationId);
  const conversationTitlesRef = useRef(new Map());
  const conversationEventsRef = useRef([]);
  // Issue #541 (R4): demo mode runs in its own conversation so user
  // conversations are never deleted or overwritten. The id lives for the
  // lifetime of the React app and is reused across cycles so the "last
  // example" persists even when the user toggles demo off and back on within
  // a session. It is intentionally NOT persisted as the "current" conversation
  // — `currentConversationRef` keeps pointing at the user's real thread, so
  // restoring the UI on demo-off is a single lookup against IndexedDB.
  const demoConversationIdRef = useRef("");

  useDocumentEnvironment({
    setI18nRuntimeTick, uiLanguage, setContextPanelWidth, themePreference, setColorSchemeTick,
  });

  useEffect(() => {
    currentConversationRef.current = currentConversationId;
  }, [currentConversationId]);

  const {
    handleReplayDataMigration, handleStartService, handleStopService, handleCheckForUpdates,
    handleInstallUpdate, handleInstallVsCodeExtension,
  } = useDesktopIntegration({
    desktopStatus, setDesktopStatus, setDataMigration, setDataMigrationBusy,
    setDesktopAgentStream, setServiceStatus, setServiceBusy, setServiceError, telegramToken,
    setUpdateBusy, setVscodeInstallBusy, setVscodeInstallResult, mode, desktopToolGrants,
  });

  useEffect(() => {
    showDeletedConversationsRef.current = showDeletedConversations;
  }, [showDeletedConversations]);

  const userContext = useMemo(
    () =>
      collectUserContext({
        uiLanguage,
        uiLanguagePreference,
        themePreference,
        uiSkin,
        glassOpacity,
        chatStyle,
        composerStyle,
        composerAction,
        toolbarIconPack,
        locationPreference,
        assistantName,
        guessProbability,
        temperature,
        followUpProbability,
        definitionFusion,
        thinkingDetailLevel,
        experimentalOcr,
      }),
    [
      uiLanguage,
      uiLanguagePreference,
      themePreference,
      uiSkin,
      glassOpacity,
      chatStyle,
      composerStyle,
      composerAction,
      toolbarIconPack,
      locationPreference,
      assistantName,
      guessProbability,
      temperature,
      followUpProbability,
      definitionFusion,
      thinkingDetailLevel,
      experimentalOcr,
      colorSchemeTick,
    ],
  );
  const userContextRef = useRef(userContext);
  useEffect(() => {
    userContextRef.current = userContext;
  }, [userContext]);

  useEffect(() => {
    if (!workerReady || typeof window === "undefined" || !window.FormalAiSeed) return;
    let cancelled = false;
    window.FormalAiSeed.loadAll().then((loaded) => {
      if (cancelled) return;
      setSeed(loaded);
    });
    return () => {
      cancelled = true;
    };
  }, [workerReady]);

  // Issue #27: on mount, hydrate the conversation list from the append-only
  // event log and restore the active thread's messages. Operates purely as a
  // projection — no events are mutated.
  const refreshConversations = useCallback(async (showDeletedOverride) => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      return [];
    }
    try {
      const shouldShowDeleted =
        typeof showDeletedOverride === "boolean"
          ? showDeletedOverride
          : showDeletedConversationsRef.current;
      const events = await window.FormalAiMemory.listEvents();
      conversationEventsRef.current = events;
      const list = groupConversations(events, {
        showDeleted: shouldShowDeleted,
      });
      list.forEach((entry) => {
        if (entry.title) {
          conversationTitlesRef.current.set(entry.id, entry.title);
        }
      });
      setConversations(list);
      return events;
    } catch (_error) {
      conversationEventsRef.current = [];
      return [];
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    refreshConversations().then((events) => {
      if (cancelled || !Array.isArray(events) || events.length === 0) return;
      const initialId = initialPreferences.current.currentConversationId;
      if (!initialId) return;
      const restored = messagesForConversation(events, initialId);
      if (restored.length > 0) {
        setMessages(restored);
        setDemoMode(false);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [refreshConversations]);

  const {
    handleExportMemory, handleImportMemory, triggerImportMemory, handleResetMemory,
    handlePurgeDeletedConversations, handlePurgeConversation,
  } = useMemoryActions({
    importInputRef, messages, setMessages, setPrompt, workerState, setMemoryStatus, seed, t,
    demoMode, setShowDeletedConversations, desktopStatus, setCurrentConversationId,
    currentConversationRef, userContext, refreshConversations,
  });

  const { triggerAttachFiles, handleAttachFiles, prepareAttachmentsForSend } = useAttachments({
    attachmentInputRef, setComposerMenuOpen, setAttachments, experimentalOcr,
  });

  const handleShowDeletedConversations = useCallback((event) => {
    const next = Boolean(event.target.checked);
    setShowDeletedConversations(next);
    refreshConversations(next);
  }, [refreshConversations]);

  const { handleContextResizePointerDown, handleContextResizeKeyDown } = useContextPanelResize({
    contextPanelWidth, setContextPanelWidth,
  });

  const handleDeleteConversation = useCallback(async (entry) => {
    if (!entry || !entry.id) return;
    await recordMemoryEvent({
      kind: "conversation_deleted",
      role: "system",
      content: `Conversation deleted: ${entry.title || entry.id}`,
      sentAt: new Date().toISOString(),
      conversationId: entry.id,
      conversationTitle: entry.title || "",
    });
    if (entry.id === currentConversationRef.current) {
      currentConversationRef.current = "";
      setCurrentConversationId("");
      setMessages([]);
      setPrompt("");
      setDemoMode(false);
    }
    setShowDeletedConversations(false);
    await refreshConversations(false);
  }, [refreshConversations]);

  // Issue #386: copy a whole conversation to the clipboard as Markdown. When
  // diagnostics mode is on, the persisted reasoning steps are folded in after
  // each AI message so the copy matches the on-screen diagnostics surface.
  const handleCopyConversation = useCallback(
    async (entry) => {
      if (!entry || !entry.id) return;
      const events = conversationEventsRef.current;
      const markdown = conversationToMarkdown(events, entry.id, {
        title: entry.title || "",
        userLabel: t("message.author.user"),
        assistantLabel:
          normalizeAssistantName(assistantNameRef.current) || "formal-ai",
        reasoningLabel: t("message.diagnosticsSteps"),
        includeReasoning: diagnosticsModeRef.current,
      });
      const ok = await copyTextToClipboard(markdown);
      if (ok) {
        setCopiedConversationId(entry.id);
        refreshConversations();
        setTimeout(() => {
          setCopiedConversationId((current) =>
            current === entry.id ? "" : current,
          );
        }, 1600);
      }
    },
    [refreshConversations, t],
  );

  const { loadBrowserRuntime } = useFormalAiWorker({
    workerRef, pendingResponses, setWorkerState, setWorkerReady, setEngineUnavailable,
    diagnosticJsFallback, browserRuntimeState, setBrowserRuntimeState, uiLanguagePreference,
    responseLanguage, preferredLanguage, demoMode, diagnosticsMode, thinkingDetailLevel,
    minMessageAnimationMs, contextPanelWidth, sidebarMenuCollapsed, sidebarDesktopCollapsed,
    sidebarPromptsCollapsed, sidebarToolsCollapsed, sidebarTraceCollapsed,
    sidebarConversationsCollapsed, sidebarSettingsCollapsed, sidebarCollapsed,
    showDeletedConversations, greetingVariations, guessProbability, temperature,
    followUpProbability, definitionFusion, blueprintComposition, experimentalOcr,
    externalServices, associativeProjectPromotion, themePreference, uiSkin, glassOpacity,
    chatStyle, composerStyle, composerAction, toolbarIconPack, locationPreference,
    assistantName, sidebarServicesCollapsed, mode, agentMode, agentOnboardingSeen,
    desktopToolGrants, currentConversationId,
  });

  useEffect(() => {
    transcriptEndRef.current?.scrollIntoView({ block: "end" });
  }, [messages]);

  useEffect(() => {
    resizeComposerInput(composerInputRef.current);
  }, [prompt, demoMode]);

  const {
    greetingVariationsRef, diagnosticsModeRef, demoModeRef, guessProbabilityRef, temperatureRef,
    followUpProbabilityRef, definitionFusionRef, blueprintCompositionRef, experimentalOcrRef,
    externalServicesRef, associativeProjectPromotionRef, agentModeRef, modeRef,
    agentOnboardingSeenRef, desktopToolGrantsRef, commandApprovalsRef, pendingAgentTaskRef,
    hasPendingAgentTask, setHasPendingAgentTask, themePreferenceRef, uiLanguagePreferenceRef,
    responseLanguageRef, preferredLanguageRef, uiSkinRef, chatStyleRef, composerStyleRef,
    composerActionRef, locationPreferenceRef, assistantNameRef, desktopStatusRef,
  } = useLatestValueRefs({
    uiLanguagePreference, responseLanguage, preferredLanguage, demoMode, diagnosticsMode,
    greetingVariations, guessProbability, temperature, followUpProbability, definitionFusion,
    blueprintComposition, experimentalOcr, externalServices, associativeProjectPromotion,
    themePreference, uiSkin, chatStyle, composerStyle, composerAction, locationPreference,
    assistantName, desktopStatus, mode, agentMode, agentOnboardingSeen, desktopToolGrants,
    commandApprovals,
  });

  const { requestAnswer } = useAnswerRequest({
    workerRef, pendingResponses, setDesktopAgentStream, currentConversationRef, userContextRef,
    greetingVariationsRef, diagnosticsModeRef, demoModeRef, guessProbabilityRef, temperatureRef,
    followUpProbabilityRef, definitionFusionRef, blueprintCompositionRef, experimentalOcrRef,
    externalServicesRef, associativeProjectPromotionRef, agentModeRef, modeRef,
    desktopToolGrantsRef, themePreferenceRef, uiLanguagePreferenceRef, responseLanguageRef,
    preferredLanguageRef, uiSkinRef, chatStyleRef, composerStyleRef, composerActionRef,
    locationPreferenceRef, assistantNameRef, desktopStatusRef,
  });

  const {
    ensureConversation, appendUserMessage, appendSystemMessage, showAgentOnboarding,
    setDesktopToolGrant, capturePendingAgentTask, clearPendingAgentTask, appendAssistantMessage,
  } = useConversationMessages({
    workerRef, setMessages, t, thinkingDetailLevel, agentMode, setAgentOnboardingSeen,
    setDesktopToolGrants, setCurrentConversationId, currentConversationRef,
    conversationTitlesRef, demoConversationIdRef, refreshConversations, demoModeRef,
    agentOnboardingSeenRef, pendingAgentTaskRef, setHasPendingAgentTask,
  });

  const { handleMemoryOperation } = useMemoryOperation({
    refreshConversations, ensureConversation,
  });

  const {
    grantAllAndRunPending, requestTerminalCommandExecution, approveDesktopCommand,
    denyDesktopCommand,
  } = useDesktopCommands({
    t, setMode, setDesktopToolGrants, setCommandApprovals, agentModeRef, modeRef,
    desktopToolGrantsRef, commandApprovalsRef, pendingAgentTaskRef, desktopStatusRef,
    appendSystemMessage, showAgentOnboarding, capturePendingAgentTask, clearPendingAgentTask,
    appendAssistantMessage,
  });

  const { conversationHistory, applyInterfaceCommand, runAgentPlan } = useInterfaceCommands({
    messages, setUiLanguagePreference, setResponseLanguage, setPreferredLanguage, setDemoMode,
    setDiagnosticsMode, setThinkingDetailLevel, setMinMessageAnimationMs, setSidebarCollapsed,
    setShowDeletedConversations, setGreetingVariations, setGuessProbability, setTemperature,
    setFollowUpProbability, setDefinitionFusion, setBlueprintComposition, setExperimentalOcr,
    setAssociativeProjectPromotion, setThemePreference, setUiSkin, setChatStyle,
    setComposerStyle, setComposerAction, setToolbarIconPack, setLocationPreference,
    setAssistantName, setMode, refreshConversations, triggerAttachFiles, requestAnswer,
    appendAssistantMessage,
  });

  async function sendText(text, extra = {}) {
    const trimmed = text.trim();
    const displayText = String(extra.displayText || trimmed).trim();
    const hasAttachments =
      Array.isArray(extra.attachments) && extra.attachments.length > 0;
    if ((!trimmed && !displayText) || pending) {
      return;
    }

    setPending(true);
    const history = conversationHistory();
    appendUserMessage(displayText || trimmed, extra);

    // Issue #27: short-circuit memory-action phrases to the corresponding
    // toolbar button before invoking the worker so the chat surface and the
    // sidebar stay in lock-step.
    const memoryAction = hasAttachments ? null : recognizeMemoryAction(displayText);
    if (memoryAction === "export") {
      await handleExportMemory();
      appendAssistantMessage({
        intent: "memory_export",
        content: t("memory.exportTriggered"),
        confidence: 1.0,
        evidence: ["rule:memory_export"],
        steps: [{ step: "trigger_button", detail: "memory-export" }],
        toolCalls: [
          {
            tool: "export_memory",
            inputs: { prompt: displayText },
            outputs: { intent: "memory_export" },
          },
        ],
      });
      setPending(false);
      return;
    }
    if (memoryAction === "import") {
      triggerImportMemory();
      appendAssistantMessage({
        intent: "memory_import",
        content: t("memory.importTriggered"),
        confidence: 1.0,
        evidence: ["rule:memory_import"],
        steps: [{ step: "trigger_button", detail: "memory-import" }],
        toolCalls: [
          {
            tool: "import_memory",
            inputs: { prompt: displayText },
            outputs: { intent: "memory_import" },
          },
        ],
      });
      setPending(false);
      return;
    }
    if (memoryAction === "reset") {
      const result = await handleResetMemory();
      if (!result.cancelled) {
        setPending(false);
        return;
      }
      appendAssistantMessage({
        intent: "memory_reset",
        content: t("memory.resetCancelled"),
        confidence: 1.0,
        evidence: ["rule:memory_reset"],
        steps: [{ step: "trigger_button", detail: "memory-reset" }],
        toolCalls: [
          {
            tool: "reset_memory",
            inputs: { prompt: displayText },
            outputs: { intent: "memory_reset", events: result.removed },
          },
        ],
      });
      setPending(false);
      return;
    }

    const interfaceCommand = hasAttachments
      ? null
      : recognizeInterfaceCommand(displayText, seed.interfaceCapabilities);
    if (interfaceCommand) {
      const valueLabel = commandValueLabel(interfaceCommand);
      if (interfaceCommand.kind !== "report_issue") {
        applyInterfaceCommand(interfaceCommand);
      }
      appendAssistantMessage({
        intent: interfaceCommand.intent,
        content: interfaceCommandResponse(interfaceCommand, currentReportUrl),
        confidence: 1.0,
        evidence: [
          `rule:${interfaceCommand.intent}`,
          `command:${interfaceCommand.kind}`,
          ...(interfaceCommand.key ? [`preference:${interfaceCommand.key}`] : []),
          `value:${valueLabel}`,
        ],
        steps: [
          {
            step:
              interfaceCommand.kind === "set_preference"
                ? "apply_message_command"
                : "trigger_message_action",
            detail: interfaceCommand.key
              ? `${interfaceCommand.key}=${valueLabel}`
              : interfaceCommand.label,
          },
        ],
        toolCalls: [
          {
            tool:
              interfaceCommand.kind === "set_preference"
                ? "configure_preference"
                : interfaceCommand.intent,
            inputs: { prompt: displayText },
            outputs: {
              kind: interfaceCommand.kind,
              key: interfaceCommand.key || interfaceCommand.action || "",
              value: interfaceCommand.value ?? interfaceCommand.label,
            },
          },
        ],
      });
      setPending(false);
      return;
    }

    // Issue #27 R11: cross-conversation recall. Phrases like "when did I ask
    // about Rust" / "find Donald Trump in another conversation" search the
    // append-only memory log on the main thread (where FormalAiMemory lives)
    // and emit a Markdown report grouped by conversation. The recognition
    // happens before the worker round-trip so we never have to ferry the full
    // event log across the worker boundary.
    const recallQuery = hasAttachments ? null : recognizeRecallQuery(displayText);
    if (recallQuery && typeof window !== "undefined" && window.FormalAiMemory) {
      let events = [];
      try {
        events = await window.FormalAiMemory.listEvents();
      } catch (_error) {
        events = [];
      }
      const report = buildRecallReport({
        events,
        term: recallQuery.term,
        scope: recallQuery.scope,
        currentConversationId: currentConversationRef.current,
        triggerText: displayText,
      });
      appendAssistantMessage({
        intent: "conversation_recall",
        content: report.content,
        confidence: 1.0,
        evidence: [
          "rule:conversation_recall",
          `scope:${recallQuery.scope}`,
          `matches:${report.matches.reduce((sum, g) => sum + g.events.length, 0)}`,
        ],
        steps: [
          { step: "extract_term", detail: recallQuery.term },
          { step: "scan_memory", detail: `${events.length} event(s)` },
          { step: "group_by_conversation", detail: `${report.matches.length} group(s)` },
        ],
        toolCalls: [
          {
            tool: "conversation_recall",
            inputs: { term: recallQuery.term, scope: recallQuery.scope },
            outputs: {
              conversations: report.matches.length,
              matches: report.matches.reduce((sum, g) => sum + g.events.length, 0),
            },
          },
        ],
      });
      setPending(false);
      return;
    }

    // Issue #27: agent mode decomposes the prompt into sub-tasks and executes
    // them sequentially, producing one consolidated assistant message with a
    // plan preamble and a per-step result list. Chat mode runs the single-step
    // path unchanged.
    if (agentModeRef.current && !hasAttachments) {
      const steps = decomposeAgentTask(displayText);
      if (steps.length > 1) {
        await runAgentPlan(steps, history);
        setPending(false);
        return;
      }
    }

    const answer = await requestAnswer(trimmed, history);
    const terminalCommand = terminalCommandFromAnswer(answer);
    if (terminalCommand) {
      await requestTerminalCommandExecution(terminalCommand, answer);
      setPending(false);
      return;
    }
    appendAssistantMessage(answer);
    // Issue #529: persist the user's memory write before releasing the composer;
    // the #1184 derivation record is bookkeeping, filed in the background.
    if (answer.memoryOperation) await handleMemoryOperation(answer.memoryOperation);
    setPending(false);
    if (answer.derivationRecord) handleMemoryOperation(answer.derivationRecord);
  }

  async function send() {
    const text = prompt.trim();
    if (!text && attachments.length === 0) {
      return;
    }

    // Issue #825: a sent prompt becomes autocomplete history for the next
    // one (newest first, capped; purely local).
    rememberInputValue("composerPrompts", text);
    composerHistoryRef.current = recallInputValues("composerPrompts");
    autocompleteControllerRef.current.close();
    setComposerAutocomplete({ open: false, items: [], activeIndex: -1 });
    setPrompt("");
    setComposerMenuOpen(false);
    const queuedAttachments = attachments;
    setAttachments([]);
    const preparedAttachments = await prepareAttachmentsForSend(queuedAttachments);
    const displayText = text || attachmentOnlyPrompt(preparedAttachments);
    const solverText = buildPromptWithAttachments(displayText, preparedAttachments);
    await sendText(solverText, {
      displayText,
      attachments: preparedAttachments,
    });
  }

  // Issue #825: re-rank suggestions on every keystroke against the word
  // being typed (see autocomplete.js for the trigger heuristics).
  function handleComposerAutocompleteInput(value) {
    const controller = autocompleteControllerRef.current;
    controller.handleInput(value);
    setComposerAutocomplete(controller.snapshot());
  }

  function handleKeyDown(event) {
    // Issue #825: while the suggestion list is open the controller owns the
    // keys — Tab (or Enter after arrowing) completes, Escape closes, arrows
    // move. Anything else, Enter on an untouched list included, falls through.
    if (autocompleteControllerRef.current.handleKeyDown(event) === "handled") {
      event.preventDefault();
      return;
    }
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      send();
    }
  }

  useDemoPlayback({
    setMessages, setPending, demoMode, setDemoPhase, setDemoCountdown, currentConversationRef,
    conversationEventsRef, requestAnswer, appendUserMessage, appendAssistantMessage,
  });

  const lastAssistant = useMemo(
    () => [...messages].reverse().find((message) => message.role === "assistant"),
    [messages],
  );

  const demoStatus = demoMode
    ? demoPhase === "waiting" && demoCountdown !== null
      ? t("status.nextDialogIn", { seconds: demoCountdown })
      : t("status.demoPlaying")
    : t("status.manual");
  // Issue #513: localized labels for the three-way operating-mode radio group
  // and the status indicator that reflects the active mode.
  const modeLabel = (option) => t(MODE_LABEL_KEYS[option]);
  const modeTitle = (option) => t(MODE_TITLE_KEYS[option]);
  const modeStatusText = t("status.mode", { mode: modeLabel(mode) });
  const reportContext = {
    messages,
    workerState,
    demoMode,
    demoStatus,
    diagnosticsMode,
    userContext,
    // Issue #839: the report document's own phrases come from the seed, so the
    // web and the CLI file issues that read identically.
    agentInfo: seed && seed.agentInfo ? seed.agentInfo : {},
  };
  const currentReportUrl = createIssueUrl(reportContext);

  // Issue #386: registry of user-facing settings so the panel can reset each
  // one (or all of them) to its shipped default. Each entry pairs a
  // PREFERENCE_DEFAULTS key with the live value, its setter, and the i18n key
  // used as its label in the reset list.
  const settingDescriptors = [
    { key: "guessProbability", value: guessProbability, set: setGuessProbability, label: "settings.ambiguity" },
    { key: "followUpProbability", value: followUpProbability, set: setFollowUpProbability, label: "settings.followUpInitiative" },
    { key: "temperature", value: temperature, set: setTemperature, label: "settings.temperature" },
    { key: "greetingVariations", value: greetingVariations, set: setGreetingVariations, label: "settings.variations" },
    { key: "definitionFusion", value: definitionFusion, set: setDefinitionFusion, label: "settings.definitionFusion" },
    { key: "blueprintComposition", value: blueprintComposition, set: setBlueprintComposition, label: "settings.blueprintComposition" },
    { key: "thinkingDetailLevel", value: thinkingDetailLevel, set: setThinkingDetailLevel, label: "settings.thinkingDetail" },
    { key: "minMessageAnimationMs", value: minMessageAnimationMs, set: setMinMessageAnimationMs, label: "settings.minMessageAnimation" },
    { key: "experimentalOcr", value: experimentalOcr, set: setExperimentalOcr, label: "settings.experimentalOcr" },
    // Issue #444: one reset descriptor per external trusted service so the
    // "modified settings" reset bar lists any service the user turned off.
    ...EXTERNAL_TRUSTED_SERVICES.map((service) => ({
      key: service.key,
      value: externalServices[service.key],
      set: (next) => setExternalService(service.key, next),
      label: service.label,
    })),
    { key: "uiLanguage", value: uiLanguagePreference, set: setUiLanguagePreference, label: "settings.language" },
    { key: "responseLanguage", value: responseLanguage, set: setResponseLanguage, label: "settings.responseLanguage" },
    { key: "preferredLanguage", value: preferredLanguage, set: setPreferredLanguage, label: "settings.preferredLanguage" },
    { key: "theme", value: themePreference, set: setThemePreference, label: "settings.theme" },
    { key: "uiSkin", value: uiSkin, set: setUiSkin, label: "settings.uiSkin" },
    { key: "chatStyle", value: chatStyle, set: setChatStyle, label: "settings.chatStyle" },
    { key: "composerStyle", value: composerStyle, set: setComposerStyle, label: "settings.composerStyle" },
    { key: "composerAction", value: composerAction, set: setComposerAction, label: "settings.composerAction" },
    { key: "toolbarIconPack", value: toolbarIconPack, set: setToolbarIconPack, label: "settings.toolbarIconPack" },
    { key: "assistantName", value: assistantName, set: setAssistantName, label: "settings.assistantName" },
    { key: "location", value: locationPreference, set: setLocationPreference, label: "settings.location" },
  ];
  const modifiedSettings = settingDescriptors.filter(
    (descriptor) => !settingIsDefault(descriptor.key, descriptor.value),
  );
  const resetSetting = (descriptor) => {
    descriptor.set(PREFERENCE_DEFAULTS[descriptor.key]);
  };
  const resetAllSettings = () => {
    for (const descriptor of modifiedSettings) {
      resetSetting(descriptor);
    }
  };

  const composerActionIcon =
    composerAction === "plus"
      ? "+"
      : <ToolbarIcon action="attachFiles" pack={toolbarIconPack} className="composer-action-icon" />;
  const attachmentStatus =
    attachments.length > 0
      ? t("composer.attachments", { count: attachments.length })
      : "";
  const desktopStatusText = desktopStatusLabel(desktopStatus, agentMode);
  const desktopAgentPermission = agentMode ? "Opted in" : "Off";
  const desktopGrantedToolCount = desktopToolGrantCount(desktopToolGrants);
  const desktopToolPermission = t("permissions.toolCount", {
    granted: desktopGrantedToolCount,
    total: DESKTOP_TOOL_OPTIONS.length,
  });
  const appVersionLabel = desktopAppVersionLabel(desktopStatus);
  const updater = desktopStatus && desktopStatus.updater;
  const updateInFlight = updateBusy || (updater && desktopUpdaterBusy(updater));
  const canCheckForUpdates = Boolean(
    updater
      && updater.supported
      && updater.enabled
      && !updateInFlight,
  );
  const canInstallUpdate = Boolean(
    updater
      && updater.supported
      && updater.enabled
      && (updater.updateAvailable || updater.downloaded)
      && !updateInFlight,
  );
  const renderDesktopPermissionPanel = (testId) =>
    <DesktopPermissionPanel grants={desktopToolGrants} mode={mode} onDecision={setDesktopToolGrant} onGrantAll={grantAllAndRunPending} hasPendingTask={hasPendingAgentTask} testId={testId} t={t} />;
  const handleDesktopEngineChange = async (event) => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.setEngine !== "function") return;
    try {
      setDesktopStatus(normalizeDesktopStatus(await bridge.setEngine(event.target.value)));
      setDesktopAgentStream([]);
    } catch (error) {
      setDesktopStatus((current) => normalizeDesktopStatus({
        ...(current || {}),
        apiError: error && error.message ? error.message : String(error),
      }));
    }
  };

  return <main style={{ "--fa-glass-opacity": glassOpacity }} className={["app", `ui-skin-${uiSkin}`, `chat-style-${chatStyle}`, `composer-style-${composerStyle}`, `toolbar-icon-pack-${toolbarIconPack}`, desktopStatus ? "desktop-shell" : ""].filter(Boolean).join(" ")}><chakra.header className="topbar">
      <ToolbarButton className="mobile-menu-toggle topbar-menu-toggle" testId="mobile-menu-toggle" ariaLabel={mobileMenuOpen ? t("buttons.closeMenu") : t("buttons.openMenu")} title={mobileMenuOpen ? t("titles.menuClose") : t("titles.menuOpen")} onClick={() => setMobileMenuOpen(value => !value)} extraProps={{
      "aria-pressed": mobileMenuOpen
    }}>
        <MenuGlyph open={mobileMenuOpen} />
      </ToolbarButton>
      <ToolbarButton className={`sidebar-toggle${sidebarCollapsed ? " is-collapsed" : ""}`} testId="sidebar-toggle" ariaLabel={sidebarCollapsed ? t("buttons.expandSidebar") : t("buttons.collapseSidebar")} title={sidebarCollapsed ? t("titles.expandSidebar") : t("titles.collapseSidebar")} onClick={() => setSidebarCollapsed(value => !value)} extraProps={{
      "aria-pressed": !sidebarCollapsed
    }}>
        <SidebarToggleGlyph collapsed={sidebarCollapsed} />
      </ToolbarButton>
      <chakra.div className="brand">
        <chakra.span className="mark">FA</chakra.span>
        <chakra.strong>formal-ai</chakra.strong>
        <chakra.span className="brand-version" data-testid="app-version">
          {appVersionLabel}
        </chakra.span>
      </chakra.div>
      <chakra.div className="topbar-actions">
        {engineUnavailable ? <chakra.span className="engine-unavailable" data-testid="engine-unavailable" role="alert" data-menu-priority="8" title={engineUnavailable}>
            {"engine unavailable — reload to retry"}
          </chakra.span> : null}
        {desktopStatus ? <chakra.span className="desktop-status" data-testid="desktop-shell-status" data-menu-priority="7" role="status" title={desktopStatus.apiError || desktopStatusText}>
            {desktopStatusText}
          </chakra.span> : null}
        <chakra.span className="demo-status" data-testid="demo-status" data-menu-priority="7" role="status">
          {demoStatus}
        </chakra.span>
        <chakra.span className={`mode-status mode-status-${mode}`} data-testid="mode-status" data-mode={mode} data-menu-priority="7" role="status">
          {modeStatusText}
        </chakra.span>
        {diagnosticsMode ? <chakra.span className="status" data-menu-priority="7">
            {workerState}
          </chakra.span> : null}
        <ToolbarButton className="source-code-button" testId="source-code" menuPriority="5" href={SOURCE_CODE_URL} target="_blank" rel="noopener noreferrer" title={t("titles.sourceCode")} ariaLabel={t("buttons.sourceCode")} icon="sourceCode" iconPack={toolbarIconPack} label={t("buttons.sourceCode")} />
        <ToolbarButton className="download-button" testId="download-link" menuPriority="5" href="download/" title={t("titles.download")} ariaLabel={t("buttons.download")} icon="download" iconPack={toolbarIconPack} label={t("buttons.download")} />
        <ToolbarButton className="report-button" testId="report-issue" menuPriority="1" href={currentReportUrl} target="_blank" rel="noopener noreferrer" title={t("titles.reportIssue")} ariaLabel={t("buttons.reportIssue")} icon="reportIssue" iconPack={toolbarIconPack} label={t("buttons.reportIssue")} />
        <ToolbarButton className="memory-button" testId="memory-export" menuPriority="6" onClick={handleExportMemory} title={t("titles.exportMemory")} ariaLabel={t("buttons.exportMemory")} icon="exportMemory" iconPack={toolbarIconPack} label={t("buttons.exportMemory")} />
        <ToolbarButton className="memory-button" testId="memory-import" menuPriority="6" onClick={triggerImportMemory} title={t("titles.importMemory")} ariaLabel={t("buttons.importMemory")} icon="importMemory" iconPack={toolbarIconPack} label={t("buttons.importMemory")} />
        <ToolbarButton className="memory-button memory-reset-button" testId="memory-reset" menuPriority="6" onClick={handleResetMemory} title={t("titles.resetMemory")} ariaLabel={t("buttons.resetMemory")} icon="resetMemory" iconPack={toolbarIconPack} label={t("buttons.resetMemory")} />
        <chakra.input ref={importInputRef} type="file" accept=".lino,text/plain" style={{
        display: "none"
      }} data-testid="memory-import-input" onChange={handleImportMemory} />
        {memoryStatus ? <chakra.span className="memory-status" role="status" data-testid="memory-status" data-menu-priority="7">
            {memoryStatus}
          </chakra.span> : null}
        <ToolbarButton className="diagnostics-toggle" menuPriority="2" onClick={() => setDiagnosticsMode(value => !value)} title={diagnosticsMode ? t("titles.diagnosticsHide") : t("titles.diagnosticsShow")} ariaLabel={diagnosticsMode ? t("buttons.diagnosticsOn") : t("buttons.diagnostics")} icon="diagnostics" iconPack={toolbarIconPack} label={diagnosticsMode ? t("buttons.diagnosticsOn") : t("buttons.diagnostics")} extraProps={{
        "aria-pressed": diagnosticsMode
      }} />
        <chakra.div className="mode-radio" data-testid="mode-radio" data-menu-priority="4" role="radiogroup" aria-label={t("titles.modeGroup")}>
          {MODE_OPTIONS.map(option => <ToolbarButton key={option} className={`mode-option mode-option-${option}${mode === option ? " is-active" : ""}`}
            testId={`mode-option-${option}`} title={modeTitle(option)} ariaLabel={modeLabel(option)} icon={option === "chat" ? "chat" : "agent"} iconPack={toolbarIconPack}
            label={modeLabel(option)} onClick={() => setMode(option)} extraProps={{
          "data-mode": option,
          role: "radio",
          "aria-checked": mode === option
        }} />)}
        </chakra.div>
        <ToolbarButton className="mode-toggle" menuPriority="3" onClick={() => setDemoMode(value => !value)} title={demoMode ? t("titles.demoOn") : t("titles.demoOff")} ariaLabel={demoMode ? t("buttons.demoOn") : t("buttons.demo")} icon="demo" iconPack={toolbarIconPack} label={demoMode ? t("buttons.demoOn") : t("buttons.demo")} extraProps={{
        "aria-pressed": demoMode
      }} />
      </chakra.div>
    </chakra.header>
      {dataMigrationDismissed ? null : <DataMigrationNotice migration={dataMigration} busy={dataMigrationBusy} onReplay={handleReplayDataMigration}
      onDismiss={() => setDataMigrationDismissed(true)} t={t} />}{mobileMenuOpen ? <div className="mobile-menu-backdrop" data-testid="mobile-menu-backdrop"
      onClick={() => setMobileMenuOpen(false)} /> : null}
      <section className={`workspace${sidebarCollapsed ? " sidebar-collapsed" : ""}`} style={{
    "--context-panel-width": `${contextPanelWidth}px`
  }}>
    <aside className={`context-panel${mobileMenuOpen ? " is-mobile-open" : ""}${sidebarCollapsed ? " is-desktop-collapsed" : ""}`} data-testid="context-panel"
    aria-hidden={sidebarCollapsed && !mobileMenuOpen ? "true" : "false"} onClickCapture={handleSidebarSectionClickCapture}>
    <div className="drawer-brand" data-testid="drawer-brand"><div className="drawer-brand-main"><span className="mark">{"FA"}</span><div className="drawer-brand-copy"><strong>
    {"formal-ai"}</strong><span className="brand-version">{appVersionLabel}</span></div></div>
    <button type="button" className="drawer-close" data-testid="drawer-close" aria-label={t("buttons.closeMenu")} title={t("titles.menuClose")}
    onClick={() => setMobileMenuOpen(false)}>
    <MenuGlyph open={true} /></button></div>
    <SidebarSection title={t("sidebar.menu")} testId="drawer-menu-actions" collapsed={sidebarMenuCollapsed} onToggle={() => setSidebarMenuCollapsed(value => !value)}
    className="drawer-menu-section" bodyClassName="drawer-menu-body" children={<div className="drawer-action-list">
    <a className="drawer-action" data-testid="drawer-source-code" href={SOURCE_CODE_URL} target="_blank" rel="noopener noreferrer">
    <ToolbarIcon action="sourceCode" pack={toolbarIconPack} /><span>{t("buttons.sourceCode")}</span></a>
    <a className="drawer-action" data-testid="drawer-report-issue" href={currentReportUrl} target="_blank" rel="noopener noreferrer">
    <ToolbarIcon action="reportIssue" pack={toolbarIconPack} /><span>{t("buttons.reportIssue")}</span></a>
    <button type="button" className="drawer-action" data-testid="drawer-memory-export" onClick={handleExportMemory}><ToolbarIcon action="exportMemory" pack={toolbarIconPack} />
    <span>{t("buttons.exportMemory")}</span></button><button type="button" className="drawer-action" data-testid="drawer-memory-import" onClick={triggerImportMemory}>
    <ToolbarIcon action="importMemory" pack={toolbarIconPack} /><span>{t("buttons.importMemory")}</span></button>
    <button type="button" className="drawer-action" data-testid="drawer-memory-reset" onClick={handleResetMemory}><ToolbarIcon action="resetMemory" pack={toolbarIconPack} /><span>
    {t("buttons.resetMemory")}</span></button><button type="button" className="drawer-action" aria-pressed={diagnosticsMode} onClick={() => setDiagnosticsMode(value => !value)}>
    <ToolbarIcon action="diagnostics" pack={toolbarIconPack} /><span>{diagnosticsMode ? t("buttons.diagnosticsOn") : t("buttons.diagnostics")}</span></button>
    <div className="drawer-action drawer-mode-radio" data-testid="drawer-mode-radio" role="radiogroup" aria-label={t("titles.modeGroup")}>
    {MODE_OPTIONS.map(option => <button key={option} type="button" className={`mode-option mode-option-${option}${mode === option ? " is-active" : ""}`}
    data-testid={`drawer-mode-option-${option}`} data-mode={option} role="radio" aria-checked={mode === option} title={modeTitle(option)} onClick={() => setMode(option)}>
    <ToolbarIcon action={option === "chat" ? "chat" : "agent"} pack={toolbarIconPack} /><span>{modeLabel(option)}</span></button>)}</div>
    <button type="button" className="drawer-action" aria-pressed={demoMode} onClick={() => setDemoMode(value => !value)}><ToolbarIcon action="demo" pack={toolbarIconPack} /><span>
    {demoMode ? t("buttons.demoOn") : t("buttons.demo")}</span></button></div>} />
    {desktopStatus ? <SidebarSection title={desktopSurfaceLabel(desktopStatus)} testId="sidebar-desktop" collapsed={sidebarDesktopCollapsed}
    onToggle={() => setSidebarDesktopCollapsed(value => !value)} className="desktop-shell-section" children={<dl className="desktop-shell-panel" data-testid="desktop-shell-panel">
    {desktopStatus.engineSelectionAvailable ? <div className="desktop-engine-row"><dt>{"Engine"}</dt><dd>
    <select data-testid="desktop-engine-selector" aria-label="Desktop engine" value={desktopStatus.activeEngine} onChange={handleDesktopEngineChange}>
    {desktopStatus.engines.map(engine => <option key={engine.id} value={engine.id}>{engine.label}</option>)}</select></dd></div> : null}<div><dt>{"Shell"}</dt><dd>
    {desktopStatus.shell}</dd></div><div><dt>{t("updates.currentVersion")}</dt><dd data-testid="desktop-app-version">{appVersionLabel}</dd></div><div><dt>{"API"}</dt>
    <dd data-testid="desktop-api-base">{compactUrl(desktopStatus.apiBase)}</dd></div><div><dt>{"Network"}</dt><dd>
    <a href={desktopStatus.graphUrl || "#"} target="_blank" rel="noopener noreferrer" data-testid="desktop-network-link">{compactUrl(desktopStatus.graphUrl)}</a></dd></div><div>
    <dt>{"Memory"}</dt><dd data-testid="desktop-memory-bundle">{desktopStatus.memory}</dd></div><div><dt>{"Agent"}</dt><dd data-testid="desktop-agent-permission">
    {desktopAgentPermission}</dd></div><div><dt>{"Tool calls"}</dt><dd data-testid="desktop-tool-permission">{desktopToolPermission}</dd></div>
    <div className="desktop-permission-row"><dt>{t("permissions.panel.rowLabel")}</dt><dd>{renderDesktopPermissionPanel("desktop-permission-panel-sidebar")}</dd></div>
    {updater ? <div className="desktop-update-row"><dt>{t("updates.title")}</dt><dd>
    <div className="desktop-update-panel" data-testid="desktop-update-panel" data-state={updater.state}>
    <span className="desktop-update-state" data-testid="desktop-update-state" role={updater.updateAvailable || updater.downloaded ? "status" : undefined}>
    {desktopUpdaterStateLabel(updater, t)}</span>
    {updater.state === "downloading" ? <progress className="desktop-update-progress" data-testid="desktop-update-progress" max="100"
    value={String(Math.round(updater.progressPercent || 0))} aria-label={t("updates.progress", {
                percent: Math.round(updater.progressPercent || 0)
              })} /> : null}<div className="desktop-update-actions">
                <button type="button" data-testid="desktop-update-check" disabled={!canCheckForUpdates} onClick={handleCheckForUpdates}>
                {updateBusy === "check" || updater && updater.state === "checking" ? t("updates.checking") : t("updates.check")}</button>
                <button type="button" className="desktop-update-install" data-testid="desktop-update-install" disabled={!canInstallUpdate} onClick={handleInstallUpdate}>
                {updateBusy === "install" || updater && updater.state === "installing" ? t("updates.updating") : t("updates.update")}</button></div></div></dd></div> : null}
                <div className="desktop-vscode-row" data-testid="desktop-vscode-install-row"><dt>{t("vscodeInstall.title")}</dt><dd>
                <div className="desktop-vscode-panel" data-testid="desktop-vscode-install-panel"><p className="desktop-vscode-summary">{t("vscodeInstall.summary")}</p>
                <div className="desktop-vscode-actions">
                <button type="button" className="desktop-vscode-install" data-testid="desktop-vscode-install" disabled={vscodeInstallBusy} onClick={handleInstallVsCodeExtension}>
                {vscodeInstallBusy ? t("vscodeInstall.installing") : t("vscodeInstall.install")}</button></div>
                {vscodeInstallResult ? <p className={`desktop-vscode-status${vscodeInstallResult.ok ? " is-ok" : " is-error"}`} data-testid="desktop-vscode-install-status"
                role="status">
                {vscodeInstallStateLabel(vscodeInstallResult, t)}{vscodeInstallResult.ok || !vscodeInstallResult.reason ? "" : ` — ${vscodeInstallResult.reason}`}</p> : null}</div>
                </dd></div>
                </dl>} /> : null}{serviceStatus ? <SidebarSection title={t("services.title")} testId="sidebar-services" collapsed={sidebarServicesCollapsed}
                onToggle={() => setSidebarServicesCollapsed(value => !value)} className="desktop-services-section" children={<div className="desktop-services-panel"
                data-testid="desktop-services-panel">
                {serviceStatus.dockerAvailable === false ? <p className="desktop-services-note" data-testid="desktop-services-docker-missing">{t("services.dockerMissing")}
                </p> : null}{(Array.isArray(serviceStatus.services) ? serviceStatus.services : []).map(service => {
        const running = Boolean(service.running);
        const busy = serviceBusy === service.key;
        const dockerReady = serviceStatus.dockerAvailable !== false;
        const isAgentEnvironment = service.key === "agent";
        const serviceLabel = service.labelKey ? t(service.labelKey) : service.label;
        return <div key={service.key} className="desktop-service" data-testid={`desktop-service-${service.key}`} data-state={service.state}><div className="desktop-service-head">
          <span className={`desktop-service-dot${running ? " is-running" : ""}`} data-testid={`desktop-service-dot-${service.key}`} /><span className="desktop-service-label">
          {serviceLabel}</span><span className="desktop-service-state" data-testid={`desktop-service-state-${service.key}`}>{serviceStateLabel(service.state, t)}</span></div>
          {service.key === "telegram" && !running ? <input type="password" className="desktop-service-token" data-testid="desktop-service-telegram-token"
          placeholder="TELEGRAM_BOT_TOKEN" value={telegramToken} autoComplete="off" spellCheck={false}
          onChange={event => setTelegramToken(event.target.value)} /> : null}{running && service.url ? <a className="desktop-service-url" href={service.url} target="_blank"
          rel="noopener noreferrer" data-testid={`desktop-service-url-${service.key}`}>
          {compactUrl(service.url)}</a> : null}<div className="desktop-service-actions">
          <button type="button" className="desktop-service-start" data-testid={`desktop-service-start-${service.key}`}
          disabled={!isAgentEnvironment && running || busy || !dockerReady} onClick={() => handleStartService(service.key)}>
          {isAgentEnvironment ? busy ? t("services.installing") : t("services.installAgent") : busy ? t("services.starting") : t("services.start")}</button>
          <button type="button" className="desktop-service-stop" data-testid={`desktop-service-stop-${service.key}`} disabled={!running || busy}
          onClick={() => handleStopService(service.key)}>
          {busy ? t("services.stopping") : t("services.stop")}</button></div></div>;
      })}{serviceError ? <p className="desktop-services-error" data-testid="desktop-services-error">{serviceError}</p> : null}</div>} /> : null}
        <SidebarSection title={t("sidebar.conversations")} testId="sidebar-conversations" collapsed={sidebarConversationsCollapsed}
        onToggle={() => setSidebarConversationsCollapsed(value => !value)} children={<div className="conversation-list" data-testid="conversation-list">
        <button type="button" className="conversation-new" data-testid="conversation-new" disabled={messages.length === 0 && !currentConversationId && prompt.trim().length === 0}
        onClick={() => {
          currentConversationRef.current = "";
          setCurrentConversationId("");
          setMessages([]);
          setDemoMode(false);
          setPrompt("");
        }}>{t("conversation.new")}</button><label className="conversation-deleted-toggle">
          <input type="checkbox" checked={showDeletedConversations} data-testid="conversation-show-deleted" onChange={handleShowDeletedConversations} /><span>
          {t("conversation.showDeleted")}</span></label>
          {showDeletedConversations ? <button type="button" className="conversation-purge-deleted" data-testid="conversation-purge-deleted" disabled={conversations.length === 0}
          onClick={handlePurgeDeletedConversations} title={t("conversation.purgeDeletedTitle")}>
          {t("conversation.purgeDeleted")}</button> : null}{conversations.length === 0 ? <p className="conversation-empty">
          {showDeletedConversations ? t("conversation.deletedEmpty") : t("conversation.empty")}</p> : <ul className="conversation-entries" data-testid="conversation-entries">
          {conversations.map(entry => {
            const active = entry.id === currentConversationId;
            return <li key={entry.id} className={["conversation-entry", active ? "is-active" : "", entry.deleted ? "is-deleted" : ""].filter(Boolean).join(" ")}><div className="conversation-entry-row"><button type="button" className="conversation-entry-button" data-conversation-id={entry.id} aria-pressed={active} onClick={async () => {
                  if (entry.id === currentConversationRef.current) {
                    return;
                  }
                  currentConversationRef.current = entry.id;
                  setCurrentConversationId(entry.id);
                  setDemoMode(false);
                  try {
                    const events = await window.FormalAiMemory.listEvents();
                    setMessages(messagesForConversation(events, entry.id));
                  } catch (_error) {
                    setMessages([]);
                  }
                }}><span className="conversation-entry-title">{entry.title || t("conversation.emptyTitle")}</span><span className="conversation-entry-meta">{t("conversation.messageCount", {
                      count: entry.messageCount
                    })}</span></button>
                      <button type="button" className={`conversation-copy${copiedConversationId === entry.id ? " is-copied" : ""}`} data-testid="conversation-copy"
                      data-conversation-id={entry.id} data-copied={copiedConversationId === entry.id ? "true" : null} aria-label={t("conversation.copyMarkdownTitle")}
                      title={t("conversation.copyMarkdownTitle")} onClick={() => handleCopyConversation(entry)}>
                      {copiedConversationId === entry.id ? t("conversation.copyMarkdownDone") : t("conversation.copyMarkdown")}</button>
                      {entry.deleted ? <button type="button" className="conversation-delete conversation-permanent-delete" data-testid="conversation-purge-one"
                      aria-label={t("conversation.deletePermanent")} title={t("conversation.deletePermanent")} onClick={() => handlePurgeConversation(entry)}>
                      {"!"}
                      </button> : <button type="button" className="conversation-delete" data-testid="conversation-delete" aria-label={t("conversation.delete")}
                      title={t("conversation.delete")} onClick={() => handleDeleteConversation(entry)}>
                      {"×"}</button>}</div></li>;
          })}</ul>}</div>} />
            <SidebarSection title={t("sidebar.settings")} testId="sidebar-settings" collapsed={sidebarSettingsCollapsed}
            onToggle={() => setSidebarSettingsCollapsed(value => !value)} children={<div className="settings-panel">
            <div className="settings-reset" data-testid="settings-reset"><div className="settings-reset-header"><span className="settings-reset-title">{t("settings.resetHeading")}
            </span>
            <button type="button" className="settings-reset-all" data-testid="settings-reset-all" disabled={modifiedSettings.length === 0} onClick={resetAllSettings}
            title={t("settings.resetAll")}>
            {t("settings.resetAll")}</button></div>{modifiedSettings.length === 0 ? <p className="settings-reset-empty" data-testid="settings-reset-empty">{t("settings.resetNone")}
            </p> : <ul className="settings-reset-list">{modifiedSettings.map(descriptor => <li key={descriptor.key} className="settings-reset-item">
            <span className="settings-reset-label">{t(descriptor.label)}</span>
            <button type="button" className="settings-reset-one" data-testid={`settings-reset-${descriptor.key}`} onClick={() => resetSetting(descriptor)}
            title={t("settings.resetOne")}>
            {t("settings.resetOne")}</button></li>)}</ul>}</div><div className="setting-row setting-row-slider"><label htmlFor="setting-guess-probability">{t("settings.ambiguity")}
            </label><div className="setting-poles"><span>{t("settings.moreQuestions")}</span><span>{t("settings.moreGuessing")}</span></div>
            <input id="setting-guess-probability" data-testid="setting-guess-probability" type="range" min="0" max="1" step="0.05" value={guessProbability}
            onChange={event => setGuessProbability(normalizeSliderPreference(event.target.value, 0.8))} />
            <output htmlFor="setting-guess-probability">{`${formatSliderValue(guessProbability)}%`}</output></div><div className="setting-row setting-row-slider">
            <label htmlFor="setting-follow-up-probability">{t("settings.followUpInitiative")}</label><div className="setting-poles"><span>{t("settings.userInitiative")}</span>
            <span>{t("settings.assistantInitiative")}</span></div>
            <input id="setting-follow-up-probability" data-testid="setting-follow-up-probability" type="range" min="0" max="1" step="0.05" value={followUpProbability}
            onChange={event => setFollowUpProbability(normalizeSliderPreference(event.target.value, PREFERENCE_DEFAULTS.followUpProbability))} />
            <output htmlFor="setting-follow-up-probability">{`${formatSliderValue(followUpProbability)}%`}</output></div><div className="setting-row setting-row-slider">
            <label htmlFor="setting-temperature">{t("settings.temperature")}</label><div className="setting-poles"><span>{t("settings.deterministic")}</span><span>
            {t("settings.varied")}</span></div>
            <input id="setting-temperature" data-testid="setting-temperature" type="range" min="0" max="1" step="0.05" value={temperature}
            onChange={event => setTemperature(normalizeSliderPreference(event.target.value, 0))} />
            <output htmlFor="setting-temperature">{normalizeSliderPreference(temperature, 0).toFixed(2)}</output></div><label className="setting-check">
            <input type="checkbox" checked={greetingVariations} onChange={event => setGreetingVariations(event.target.checked)} /><span>{t("settings.variations")}</span></label>
            <label className="setting-row"><span>{t("settings.definitionFusion")}</span>
            <select data-testid="setting-definition-fusion" value={definitionFusion} onChange={event => setDefinitionFusion(normalizeDefinitionFusion(event.target.value))}>
            <option value="explicit">{t("settings.definitionFusion.explicit")}</option><option value="auto">{t("settings.definitionFusion.auto")}</option></select></label>
            <label className="setting-row"><span>{t("settings.blueprintComposition")}</span>
            <select data-testid="setting-blueprint-composition" value={blueprintComposition}
            onChange={event => setBlueprintComposition(normalizeBlueprintComposition(event.target.value))}>
            <option value="composed">{t("settings.blueprintComposition.composed")}</option><option value="documented">{t("settings.blueprintComposition.documented")}</option>
            </select></label><label className="setting-row"><span>{t("settings.thinkingDetail")}</span>
            <select data-testid="setting-thinking-detail" value={thinkingDetailLevel} onChange={event => setThinkingDetailLevel(normalizeThinkingDetailLevel(event.target.value))}>
            <option value="brief">{t("settings.thinkingDetail.brief")}</option><option value="standard">{t("settings.thinkingDetail.standard")}</option><option value="detailed">
            {t("settings.thinkingDetail.detailed")}</option></select></label><div className="setting-row setting-row-slider"><label htmlFor="setting-min-message-animation">
            {t("settings.minMessageAnimation")}</label><div className="setting-poles"><span>{t("settings.animationImmediate")}</span><span>{t("settings.animationRelaxed")}</span>
            </div>
            <input id="setting-min-message-animation" data-testid="setting-min-message-animation" type="range" min="0" max="6000" step="250" value={minMessageAnimationMs}
            onChange={event => setMinMessageAnimationMs(normalizeAnimationBudgetMs(event.target.value))} />
            <output htmlFor="setting-min-message-animation">{minMessageAnimationMs === 0 ? t("settings.animationImmediate") : t("settings.animationSeconds", {
              seconds: (minMessageAnimationMs / 1000).toFixed(1)
            })}</output></div><div className="setting-row setting-row-ocr"><label className="setting-check">
              <input type="checkbox" checked={experimentalOcr} data-testid="setting-experimental-ocr" onChange={event => setExperimentalOcr(event.target.checked)} /><span>
              {t("settings.experimentalOcr")}</span></label><p className="setting-warning" data-testid="setting-experimental-ocr-warning" title={OCR_DOWNLOAD_WARNING}>
              {t("settings.experimentalOcr.warning")}</p></div><div className="setting-row setting-row-browser-runtime" data-testid="setting-browser-runtime">
              <p className="setting-section-title">{t("message.browserRuntime.title")}</p><p className="setting-section-note" role="status">
              {t(`message.browserRuntime.${browserRuntimeStatusKey(browserRuntimeState)}`, {
              error: browserRuntimeState.error
            })}</p>
              <button type="button" className="permission-button" data-testid="setting-browser-runtime-load"
              disabled={browserRuntimeState.status === "loading" || browserRuntimeState.status === "ready"} onClick={loadBrowserRuntime}>
              {t("message.browserRuntime.load")}</button></div><div className="setting-row setting-row-diagnostic-js-fallback" data-testid="setting-diagnostic-js-fallback">
              <label className="setting-check"><input type="checkbox" checked={diagnosticJsFallback} onChange={event => {
            // Issue #934: development-only override for the hard engine
            // failure. It takes effect on reload because the worker reads
            // the flag from its own URL before the engine initializes.
            try {
              window.localStorage.setItem(
                "formalAiDiagnosticJsFallback",
                event.target.checked ? "1" : "0",
              );
            } catch (_error) {
              // Storage may be unavailable; the toggle still applies for
              // this page load only.
            }
            setDiagnosticJsFallback(event.target.checked);
            window.location.reload();
          }} /><span>{"Diagnostic JavaScript fallback (development only; reloads the page)"}</span></label><p className="setting-warning">{"Answers produced under the override are marked engine: js (diagnostic override) in their trace."}</p></div>{
        // Issue #444: external trusted-services opt-in/opt-out section. The
        // checkbox list is generated from EXTERNAL_TRUSTED_SERVICES so the
        // catalog stays the single source of truth; each service is enabled
        // by default and the user can opt out of any one.
        <div className="setting-row setting-row-external-services" data-testid="settings-external-services"><p className="setting-section-title">{t("settings.externalServices")}
          </p><p className="setting-section-note">{t("settings.externalServices.note")}</p>
          {EXTERNAL_TRUSTED_SERVICES.map(service => <label className="setting-check" key={service.key}>
          <input type="checkbox" checked={externalServices[service.key] !== false} data-testid={`setting-${service.key}`}
          onChange={event => setExternalService(service.key, event.target.checked)} />
          <span>{t(service.label)}</span></label>)}</div>}<label className="setting-row"><span>{t("settings.language")}</span>
          <select data-testid="setting-ui-language" value={uiLanguagePreference} onChange={event => setUiLanguagePreference(normalizeUiLanguagePreference(event.target.value))}>
          <option value="auto">{t("settings.language.auto")}</option><option value="en">{"English"}</option><option value="ru">{"Русский"}</option><option value="zh">{"中文"}
          </option><option value="hi">{"हिन्दी"}</option></select></label><label className="setting-row"><span>{t("settings.responseLanguage")}</span>
          <select data-testid="setting-response-language" value={responseLanguage} onChange={event => setResponseLanguage(normalizeResponseLanguageMode(event.target.value))}>
          <option value="last_message">{t("settings.responseLanguage.lastMessage")}</option><option value="preferred">{t("settings.responseLanguage.preferred")}</option>
          <option value="ui">{t("settings.responseLanguage.ui")}</option></select></label>{responseLanguage === "preferred" ? <label className="setting-row"><span>
          {t("settings.preferredLanguage")}</span>
          <select data-testid="setting-preferred-language" value={preferredLanguage} onChange={event => setPreferredLanguage(normalizePreferredLanguage(event.target.value))}>
          <option value="en">{"English"}</option><option value="ru">{"Русский"}</option><option value="zh">{"中文"}</option><option value="hi">{"हिन्दी"}</option></select>
          </label> : null}<label className="setting-row"><span>{t("settings.theme")}</span>
          <select data-testid="setting-theme" value={themePreference} onChange={event => setThemePreference(normalizeThemePreference(event.target.value))}><option value="auto">
          {t("settings.theme.auto")}</option><option value="light">{t("settings.theme.light")}</option><option value="dark">{t("settings.theme.dark")}</option></select></label>
          <label className="setting-row"><span>{t("settings.uiSkin")}</span>
          <select data-testid="setting-ui-skin" value={uiSkin} onChange={event => setUiSkin(normalizeUiSkin(event.target.value))}><option value="flat">{t("settings.uiSkin.flat")}
          </option><option value="glass">{t("settings.uiSkin.glass")}</option><option value="material">{t("settings.uiSkin.material")}</option><option value="contrast">
          {t("settings.uiSkin.contrast")}</option></select></label>{uiSkin === "glass" ? <label className="setting-row"><span>{t("settings.glassOpacity")}</span>
          <input type="range" min="0.35" max="1" step="0.01" value={glassOpacity} data-testid="setting-glass-opacity"
          onChange={event => setGlassOpacity(clampNumber(event.target.value, 0.35, 1, 0.78))} />
          <output>{Math.round(glassOpacity * 100)}%</output></label> : null}<label className="setting-row"><span>{t("settings.toolbarIconPack")}</span>
          <select data-testid="setting-toolbar-icon-pack" value={toolbarIconPack} onChange={event => setToolbarIconPack(normalizeToolbarIconPack(event.target.value))}>
          <option value="fontawesome">{t("settings.toolbarIconPack.fontawesome")}</option><option value="material-symbols">{t("settings.toolbarIconPack.materialSymbols")}</option>
          <option value="bootstrap-icons">{t("settings.toolbarIconPack.bootstrapIcons")}</option><option value="ionicons">{t("settings.toolbarIconPack.ionicons")}</option>
          <option value="remix-icon">{t("settings.toolbarIconPack.remixIcon")}</option><option value="tabler-icons">{t("settings.toolbarIconPack.tablerIcons")}</option>
          <option value="names">{t("settings.toolbarIconPack.names")}</option></select></label><label className="setting-row"><span>{t("settings.chatStyle")}</span>
          <select data-testid="setting-chat-style" value={chatStyle} onChange={event => setChatStyle(normalizeChatStyle(event.target.value))}><option value="cards">
          {t("settings.chatStyle.cards")}</option><option value="compact">{t("settings.chatStyle.compact")}</option><option value="bubbles">{t("settings.chatStyle.bubbles")}
          </option></select></label><label className="setting-row"><span>{t("settings.composerStyle")}</span>
          <select data-testid="setting-composer-style" value={composerStyle} onChange={event => setComposerStyle(normalizeComposerStyle(event.target.value))}><option value="flat">
          {t("settings.composerStyle.flat")}</option><option value="glass-soft">{t("settings.composerStyle.glassSoft")}</option><option value="glass-clear">
          {t("settings.composerStyle.glassClear")}</option><option value="bubble">{t("settings.composerStyle.bubble")}</option></select></label><label className="setting-row">
          <span>{t("settings.composerAction")}</span>
          <select data-testid="setting-composer-action" value={composerAction} onChange={event => setComposerAction(normalizeComposerAction(event.target.value))}>
          <option value="attach">{t("settings.composerAction.attach")}</option><option value="plus">{t("settings.composerAction.plus")}</option></select></label>
          <label className="setting-row"><span>{t("settings.assistantName")}</span>
          <input data-testid="setting-assistant-name" type="text" value={assistantName} maxLength={64} placeholder={t("settings.assistantName.placeholder")}
          list="setting-assistant-name-options" aria-autocomplete="list" onChange={event => setAssistantName(sanitizeAssistantNameInput(event.target.value))} onBlur={event => {
            rememberInputValue("assistantName", event.target.value);
            setAssistantNameHistory(recallInputValues("assistantName"));
          }} /><datalist id="setting-assistant-name-options" data-testid="setting-assistant-name-options">{assistantNameHistory.map(value => <option key={value} value={value} />)}
            </datalist></label><label className="setting-row"><span>{t("settings.location")}</span>
            <input data-testid="setting-location" type="text" value={locationPreference} placeholder={t("settings.location.placeholder")} list="setting-location-options"
            aria-autocomplete="list" onChange={event => setLocationPreference(event.target.value.slice(0, 80))} onBlur={event => {
            rememberInputValue("location", event.target.value);
            setLocationHistory(recallInputValues("location"));
          }} /><datalist id="setting-location-options" data-testid="setting-location-options">{locationHistory.map(value => <option key={value} value={value} />)}</datalist>
            </label></div>} />
            <SidebarSection title={t("sidebar.examplePrompts")} testId="sidebar-prompts" collapsed={sidebarPromptsCollapsed}
            onToggle={() => setSidebarPromptsCollapsed(value => !value)} children={<div className="prompt-list" data-testid="example-prompts">
            {EXAMPLE_PROMPTS.map(entry => <button key={entry.text} type="button" data-prompt-label={entry.label} data-prompt-text={entry.text} onClick={() => {
          setDemoMode(false);
          setPrompt(entry.text);
        }} title={entry.label}>{entry.text}</button>)}</div>} />
          {seed.tools && seed.tools.length > 0 ? <SidebarSection title={t("sidebar.tools")} testId="sidebar-tools" collapsed={sidebarToolsCollapsed}
          onToggle={() => setSidebarToolsCollapsed(value => !value)} children={<div className="tool-registry" data-testid="tool-registry">
          <ul className="tool-list">{seed.tools.map(tool => {
            const displayTool = localizeTool(tool, uiLanguage);
            return <li key={tool.id} className={`tool tool-mode-${tool.mode || "thinking"}`} data-testid="tool-entry" data-tool-id={tool.id}
              data-tool-mode={tool.mode || "thinking"}>
              <div className="tool-head"><strong>{displayTool.name || tool.id}</strong><span className="tool-mode">
              {tool.mode === "agent" ? t("toolMode.agent") : t("toolMode.thinking")}</span></div>{displayTool.description ? <p className="tool-desc">{displayTool.description}
              </p> : null}</li>;
          })}</ul>
            </div>} /> : null}{diagnosticsMode ? <SidebarSection title={t("sidebar.trace")} testId="sidebar-trace" collapsed={sidebarTraceCollapsed}
            onToggle={() => setSidebarTraceCollapsed(value => !value)} children={<dl className="trace-list">
            <div><dt>{t("trace.model")}</dt><dd>{"formal-ai"}</dd></div><div><dt>{t("trace.mode")}</dt><dd>{demoStatus}</dd></div><div><dt>{t("trace.intent")}</dt><dd>
            {lastAssistant?.intent ?? "none"}</dd></div><div><dt>{t("trace.data")}</dt><dd>{"data/source-index.lino"}</dd></div><div><dt>{t("trace.seedFiles")}</dt><dd>
            {Object.keys(seed.raw || {}).join(", ") || "(loading)"}</dd></div><div><dt>{t("trace.toolsLoaded")}</dt><dd>{String((seed.tools || []).length)}</dd></div><div><dt>
            {t("trace.conceptsLoaded")}</dt><dd>{String((seed.concepts || []).length)}</dd></div></dl>} /> : null}</aside>
            <div className="context-resizer" data-testid="context-resizer" role="separator" aria-orientation="vertical" aria-label={t("titles.resizeSidebar")}
            aria-valuemin={CONTEXT_PANEL_MIN_WIDTH} aria-valuemax={contextPanelMaxWidth()} aria-valuenow={contextPanelWidth} tabIndex={0} title={t("titles.resizeSidebar")}
            onPointerDown={handleContextResizePointerDown} onKeyDown={handleContextResizeKeyDown} />
            <section className="chat-panel">
            {diagnosticsMode ? <DebuggerView messages={messages} apiBase={desktopStatus?.apiReady ? desktopStatus.apiBase : ""}
            debugToken={desktopStatus?.apiReady ? desktopStatus.debugToken || "" : ""} /> : null}
            <section className="messages" aria-live="polite" data-testid="message-list">
            {messages.map(message => <Message key={message.id} message={message} conversationMessages={messages} diagnosticsMode={diagnosticsMode} thinkingDetailLevel={thinkingDetailLevel}
            stepLevelOverrides={stepLevelOverrides} onEditStepLevel={editStepLevel} minMessageAnimationMs={minMessageAnimationMs}
            renderPermissionPanel={renderDesktopPermissionPanel} commandApprovals={commandApprovals} onApproveCommand={approveDesktopCommand} onDenyCommand={denyDesktopCommand}
            t={t} reportIssueUrl={shouldOfferMessageReport(message) ? createIssueUrl({
          ...reportContext,
          focusMessage: message
        }) : null} />)}{pending && desktopAgentStream.length > 0 ? <div className="desktop-agent-stream" data-testid="desktop-agent-stream" role="status"><strong>
          {`${desktopStatus && desktopStatus.activeEngine || "agent"}:`}</strong><span>{desktopAgentEventLabel(desktopAgentStream[desktopAgentStream.length - 1])}</span>
          </div> : null}{pending ? <PendingAssistantBubble t={t} /> : null}<div ref={transcriptEndRef} /></section><form className="composer" onSubmit={event => {
        event.preventDefault();
        send();
      }}><input ref={attachmentInputRef} type="file" multiple={true} style={{
          display: "none"
        }} data-testid="composer-attachment-input" onChange={handleAttachFiles} />{demoMode ? <p className="composer-demo-hint" data-testid="composer-demo-hint">
          {t("composer.demoHint.before")}<ToolbarIcon action="demo" pack={toolbarIconPack} className="composer-demo-hint-icon" />{t("composer.demoHint.after")}
          </p> : null}{composerMenuOpen ? <div className="composer-menu" data-testid="composer-menu">
          <button type="button" className="composer-menu-item" onClick={triggerAttachFiles}>{t("buttons.attachFiles")}</button>
          <button type="button" className="composer-menu-item" onClick={handleExportMemory}>{t("buttons.exportMemory")}</button>
          <button type="button" className="composer-menu-item" onClick={triggerImportMemory}>{t("buttons.importMemory")}</button>
          <a className="composer-menu-item" href={currentReportUrl} target="_blank" rel="noopener noreferrer">{t("buttons.reportIssue")}</a>
          </div> : null}{composerAutocomplete.open && composerAutocomplete.items.length ? <ul className="autocomplete-menu" role="listbox" aria-label={t("autocomplete.listLabel")}
          data-testid="composer-autocomplete" id="composer-autocomplete">
          {composerAutocomplete.items.map((item, index) => <li key={`${item.source}:${item.value}`} role="presentation">
          <button type="button" role="option" aria-selected={index === composerAutocomplete.activeIndex}
          className={`autocomplete-item${index === composerAutocomplete.activeIndex ? " is-active" : ""}`} data-testid="composer-autocomplete-item" data-source={item.source}
          onClick={() => {
              autocompleteControllerRef.current.commit(index);
              setComposerAutocomplete(autocompleteControllerRef.current.snapshot());
            }}>{item.value}<span className="autocomplete-item-hint">{t(`autocomplete.source.${item.source}`)}</span></button></li>)}</ul> : null}<div className="composer-grid">
              <button type="button" className="composer-action-button" data-testid="composer-menu-toggle" aria-expanded={composerMenuOpen} aria-label={t("buttons.composerMenu")}
              title={t("titles.composerMenu")} onClick={() => setComposerMenuOpen(value => !value)}>
              {composerActionIcon}</button>
              <textarea ref={composerInputRef} value={prompt} rows={1} placeholder={agentMode ? t("composer.placeholder.agent") : t("composer.placeholder.chat")} autoComplete="off"
              autoCorrect="off" autoCapitalize="sentences" enterKeyHint="send" inputMode="text" spellCheck={true} onChange={event => {
            setPrompt(event.target.value);
            handleComposerAutocompleteInput(event.target.value);
          }} onKeyDown={handleKeyDown} disabled={demoMode || !workerReady} data-testid="chat-composer-input" aria-controls="composer-autocomplete" aria-autocomplete="list" />
            <button className="send-button" type="submit" disabled={pending || demoMode || !workerReady || !prompt.trim() && attachments.length === 0}
            data-testid="chat-composer-submit">
            {pending ? <span className="send-spinner" aria-hidden="true" data-testid="send-spinner" /> : <span className="send-icon" aria-hidden="true">{"↑"}</span>}
            <span className="send-label">{pending ? t("composer.sending") : t("composer.send")}</span></button></div>
            {attachmentStatus ? <p className="composer-attachment-status" data-testid="composer-attachment-status">{attachmentStatus}</p> : null}</form></section></section></main>;
}
