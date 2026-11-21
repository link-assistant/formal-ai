export const EXTERNAL_TRUSTED_SERVICES = [
  { key: "externalServiceWikihow", label: "settings.externalServiceWikihow" },
  { key: "externalServiceStackExchange", label: "settings.externalServiceStackExchange" },
  { key: "externalServiceMediawikiFamily", label: "settings.externalServiceMediawikiFamily" },
  { key: "externalServiceGithub", label: "settings.externalServiceGithub" },
  // Issue #1138 plan 01 L4: the lexical tier joined the live, opt-out-able
  // group when it gained a `need_kinds`, so the two settings keys that silence
  // a dictionary belong here too. Without them a user could opt out of every
  // procedural source and still have no way to opt out of the ones that answer
  // "what does this word mean".
  { key: "externalServiceWiktionary", label: "settings.externalServiceWiktionary" },
  { key: "externalServiceWordnet", label: "settings.externalServiceWordnet" },
];

// Shared persisted defaults, independent of UI rendering.
export const PREFERENCE_DEFAULTS = {
  demoMode: true,
  diagnosticsMode: false,
  contextPanelWidth: 300,
  // Issue #27: each sidebar section is a VS Code-style collapsible region; the
  // last expand/collapse state is persisted via FormalAiPreferences so opening
  // the demo never reshuffles the user's layout.
  sidebarMenuCollapsed: true,
  sidebarPromptsCollapsed: false,
  sidebarToolsCollapsed: true,
  sidebarTraceCollapsed: true,
  sidebarConversationsCollapsed: false,
  sidebarSettingsCollapsed: true,
  sidebarDesktopCollapsed: true,
  sidebarServicesCollapsed: true,
  // Issue #153: the side panel is collapsible to give the chat full viewport
  // width on desktop. The drawer view on mobile stays controlled by the
  // separate `mobileMenuOpen` toggle so phones can still slide it in.
  sidebarCollapsed: false,
  showDeletedConversations: false,
  // Issue #27: random greeting variations are opt-in but default to on so
  // newcomers see the multilingual surface immediately.
  greetingVariations: true,
  // Issue #488: user-facing thinking can be compact or detailed without
  // changing the raw diagnostics available to maintainers.
  // Issue #541 (R8): default to the 50% midpoint ("standard"), which surfaces
  // only the high-level human-readable steps (not the mechanical sub-steps), so
  // newcomers are not overwhelmed; "detailed" remains one notch away for power
  // users and still renders fully human-readable prose (no symbolic syntax).
  thinkingDetailLevel: "standard",
  // Issue #541 (R5): minimum wall-clock time, in milliseconds, that a freshly
  // produced assistant answer spends animating its reasoning + reveal so the
  // user *feels* the thinking happen even when the deterministic engine answers
  // instantly. 0 = immediate display; the shipped default is a relaxed 2s.
  minMessageAnimationMs: 2000,
  // Issue #82: user-tunable assistant behavior. The default still guesses
  // likely typo matches, while the sliders let cautious users ask first and
  // deterministic users turn random response variation off with temperature=0.
  guessProbability: 0.8,
  temperature: 0.7,
  // Issue #160 follow-up: polite courtesy responses can either leave the
  // initiative with the user or ask/propose the next action. This probability
  // controls whether the next-action sentence is appended.
  followUpProbability: 0.75,
  // Issue #63: definition fusion remains explicit-only by default, with an
  // opt-in mode that treats plain "What is X?" prompts as merge requests.
  definitionFusion: "explicit",
  // Issue #340: how composite-program blueprints project their annotated recipe
  // template into the program shown to the user.
  //   "composed" (default) — emit only the regions the request actually named,
  //                          so the program is a projection of the decomposition;
  //   "documented"         — always emit the fully documented program with every
  //                          optional region (error handling, comments) present.
  blueprintComposition: "composed",
  experimentalOcr: false,
  // Issue #444: external trusted-service opt-outs. Defined data-driven from
  // EXTERNAL_TRUSTED_SERVICES; every service ships enabled (opt-out model) so the
  // assistant keeps consulting them unless the user disables one in settings.
  ...Object.fromEntries(EXTERNAL_TRUSTED_SERVICES.map((service) => [service.key, true])),
  associativeProjectPromotion: true,
  theme: "auto",
  location: "",
  assistantName: "",
  // Issue #27: id of the conversation the user last typed in; on reload the
  // demo restores its event log into the main transcript. Empty string means
  // "no conversation yet — start a fresh one on first user input".
  currentConversationId: "",
  // Issue #27: Chat (single-turn Q&A) vs Agent (multi-step plan + execute) mode.
  // Persisted so the user keeps their preferred operating surface across
  // reloads. Agent mode in the browser sandbox decomposes the prompt into
  // sequential sub-tasks and runs each through the existing solver; a future
  // iteration will wire it to docker / WebVM execution.
  agentMode: false,
  // Issue #513: three-way operating mode replacing the binary agent toggle.
  //   "chat"     — single-turn reasoning, no command execution;
  //   "agent"    — multi-step plan + execute, capabilities gated on grant;
  //   "fullAuto" — agent mode that runs permitted commands automatically.
  // The legacy `agentMode` boolean is derived as `mode !== "chat"` so existing
  // readers (worker prefs, desktop tool grants) keep working unchanged.
  mode: "chat",
  // Issue #514: first-run Agent/Full Auto onboarding and per-tool grant
  // decisions are persisted independently. The grant string is a compact
  // Links-friendly map such as `shell:on,http_fetch:off`.
  agentOnboardingSeen: false,
  desktopToolGrants: "",
  // Issue #94: "auto" follows navigator.languages; explicit values use the
  // supported UI language catalog.
  uiLanguage: "auto",
  // Issue #324: which language drives the assistant's responses.
  //   "last_message" (default) — answer in the detected language of the prompt;
  //   "preferred"             — pin responses to `preferredLanguage`;
  //   "ui"                    — follow the UI-language preference.
  // The default reproduces the deterministic "reply in the message's language"
  // behavior, so a Russian prompt is answered in Russian.
  outputStandard: "cnl-default",
  lexicalDensity: "standard",
  responseLanguage: "last_message",
  // Issue #324: the explicit language used when `responseLanguage` is
  // "preferred". One of the supported response languages (en/ru/hi/zh).
  preferredLanguage: "en",
  // Issues #108/#110: UI, chat, and input surfaces are configurable while the
  // defaults stay flat and cheap to render.
  uiSkin: "flat",
  glassOpacity: 0.78,
  chatStyle: "cards",
  composerStyle: "flat",
  composerAction: "attach",
  toolbarIconPack: "fontawesome",
};
