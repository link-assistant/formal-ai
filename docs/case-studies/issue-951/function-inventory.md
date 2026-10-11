# main.jsx function inventory

258 top-level function declarations at this snapshot. Every declaration is listed.
Classification is provisional; a Rust name match is a migration candidate, not proof of identical semantics.

| Function | Line | Classification | Candidate Rust home |
| --- | --- | --- | --- |
| `demoGreetings` | 161 | Port logic to Rust/WASM | unmapped |
| `demoFeaturePrompts` | 165 | Port logic to Rust/WASM | unmapped |
| `normalizeMemoryPrompt` | 234 | Port logic to Rust/WASM | unmapped |
| `recognizeMemoryAction` | 242 | Port logic to Rust/WASM | unmapped |
| `includesAnyText` | 257 | Port logic to Rust/WASM | unmapped |
| `matchesAnyPattern` | 261 | Port logic to Rust/WASM | unmapped |
| `containsThemeObject` | 265 | Port logic to Rust/WASM | unmapped |
| `detectToggleCommand` | 306 | Port logic to Rust/WASM | unmapped |
| `isExplicitUiLanguageCommand` | 369 | Port logic to Rust/WASM | unmapped |
| `commandNumberValue` | 375 | UI boundary candidate; audit mixed business logic | unmapped |
| `sanitizeAssistantNameInput` | 387 | Port logic to Rust/WASM | unmapped |
| `normalizeAssistantName` | 393 | Port logic to Rust/WASM | unmapped |
| `extractAssistantNameCommand` | 401 | UI boundary candidate; audit mixed business logic | unmapped |
| `commandValueLabel` | 449 | Port logic to Rust/WASM | unmapped |
| `interfaceCommandResponse` | 458 | Port logic to Rust/WASM | unmapped |
| `recognizeSeedInterfaceCommand` | 468 | UI boundary candidate; audit mixed business logic | unmapped |
| `recognizeInterfaceCommand` | 503 | UI boundary candidate; audit mixed business logic | unmapped |
| `stripRecallTerm` | 900 | Port logic to Rust/WASM | unmapped |
| `recoverOriginalRange` | 914 | Port logic to Rust/WASM | unmapped |
| `recognizeRecallQuery` | 945 | Rust-name candidate; verify duplication before migration | rust/src/solver_handlers/conversation_memory/mod.rs |
| `buildRecallReport` | 987 | UI boundary candidate; audit mixed business logic | unmapped |
| `settingIsDefault` | 1225 | Port logic to Rust/WASM | unmapped |
| `withAssetVersion` | 1322 | Port logic to Rust/WASM | unmapped |
| `recordMemoryEvent` | 1330 | UI boundary candidate; audit mixed business logic | unmapped |
| `waitForMemoryWrites` | 1345 | Port logic to Rust/WASM | unmapped |
| `downloadTextFile` | 1352 | UI boundary candidate; audit mixed business logic | unmapped |
| `isImageAttachment` | 1367 | UI boundary candidate; audit mixed business logic | unmapped |
| `isTextAttachment` | 1376 | UI boundary candidate; audit mixed business logic | unmapped |
| `formatFileSize` | 1398 | Port logic to Rust/WASM | unmapped |
| `readFileAsDataUrl` | 1406 | UI boundary candidate; audit mixed business logic | unmapped |
| `readFileAsText` | 1415 | UI boundary candidate; audit mixed business logic | unmapped |
| `sampleTextAttachmentContent` | 1427 | Port logic to Rust/WASM | unmapped |
| `loadOcrBundle` | 1446 | UI boundary candidate; audit mixed business logic | unmapped |
| `attachmentMemoryRecord` | 1480 | Port logic to Rust/WASM | unmapped |
| `attachmentOnlyPrompt` | 1499 | Port logic to Rust/WASM | unmapped |
| `attachmentContextText` | 1507 | UI boundary candidate; audit mixed business logic | unmapped |
| `buildPromptWithAttachments` | 1532 | Port logic to Rust/WASM | unmapped |
| `readStoredPreferencesRecord` | 1539 | UI boundary candidate; audit mixed business logic | unmapped |
| `migrateStoredSidebarCollapsePreferences` | 1557 | Port logic to Rust/WASM | unmapped |
| `loadPreferences` | 1571 | UI boundary candidate; audit mixed business logic | unmapped |
| `persistPreferences` | 1584 | UI boundary candidate; audit mixed business logic | unmapped |
| `clampNumber` | 1595 | Port logic to Rust/WASM | unmapped |
| `normalizeSliderPreference` | 1601 | Port logic to Rust/WASM | unmapped |
| `formatSliderValue` | 1605 | Port logic to Rust/WASM | unmapped |
| `contextPanelMaxWidth` | 1609 | UI boundary candidate; audit mixed business logic | unmapped |
| `normalizeContextPanelWidth` | 1626 | UI boundary candidate; audit mixed business logic | unmapped |
| `normalizeThemePreference` | 1637 | Port logic to Rust/WASM | unmapped |
| `normalizeUiSkin` | 1641 | Port logic to Rust/WASM | unmapped |
| `normalizeChatStyle` | 1645 | Port logic to Rust/WASM | unmapped |
| `normalizeComposerStyle` | 1649 | Port logic to Rust/WASM | unmapped |
| `normalizeComposerAction` | 1653 | Port logic to Rust/WASM | unmapped |
| `normalizeToolbarIconPack` | 1659 | Port logic to Rust/WASM | unmapped |
| `normalizeDefinitionFusion` | 1665 | Port logic to Rust/WASM | unmapped |
| `normalizeBlueprintComposition` | 1671 | Port logic to Rust/WASM | unmapped |
| `normalizeThinkingDetailLevel` | 1677 | Port logic to Rust/WASM | unmapped |
| `normalizeAnimationBudgetMs` | 1688 | Port logic to Rust/WASM | unmapped |
| `normalizeResponseLanguageMode` | 1697 | Port logic to Rust/WASM | unmapped |
| `normalizeMode` | 1706 | Port logic to Rust/WASM | unmapped |
| `normalizeDesktopToolGrants` | 1717 | UI boundary candidate; audit mixed business logic | unmapped |
| `serializeDesktopToolGrants` | 1752 | Port logic to Rust/WASM | unmapped |
| `desktopToolRouterGrants` | 1760 | UI boundary candidate; audit mixed business logic | unmapped |
| `desktopToolGrantCount` | 1770 | Port logic to Rust/WASM | unmapped |
| `desktopToolGrantState` | 1775 | Port logic to Rust/WASM | unmapped |
| `normalizePreferredLanguage` | 1782 | Port logic to Rust/WASM | unmapped |
| `toolbarIconFontName` | 1990 | Port logic to Rust/WASM | unmapped |
| `toolbarIconFontClass` | 1999 | Port logic to Rust/WASM | unmapped |
| `ToolbarIcon` | 2010 | React component | unmapped |
| `ToolbarButton` | 2042 | React component | unmapped |
| `i18nApi` | 2094 | UI boundary candidate; audit mixed business logic | unmapped |
| `normalizeUiLanguagePreference` | 2100 | Port logic to Rust/WASM | unmapped |
| `detectUiLanguage` | 2109 | Port logic to Rust/WASM | unmapped |
| `translateUi` | 2117 | Port logic to Rust/WASM | unmapped |
| `browserLanguagesList` | 2125 | UI boundary candidate; audit mixed business logic | unmapped |
| `currentColorScheme` | 2133 | UI boundary candidate; audit mixed business logic | unmapped |
| `resolvedLocale` | 2145 | Port logic to Rust/WASM | unmapped |
| `resolvedTimeZone` | 2153 | Port logic to Rust/WASM | unmapped |
| `collectUserContext` | 2161 | UI boundary candidate; audit mixed business logic | unmapped |
| `formatUiLanguagesField` | 2234 | Port logic to Rust/WASM | unmapped |
| `formatUiField` | 2258 | UI boundary candidate; audit mixed business logic | unmapped |
| `formatLocaleField` | 2267 | Port logic to Rust/WASM | unmapped |
| `formatThemeField` | 2276 | Port logic to Rust/WASM | unmapped |
| `userContextFields` | 2286 | UI boundary candidate; audit mixed business logic | unmapped |
| `randomItem` | 2333 | Port logic to Rust/WASM | unmapped |
| `generateConversationId` | 2341 | Port logic to Rust/WASM | unmapped |
| `deriveConversationTitle` | 2349 | Port logic to Rust/WASM | unmapped |
| `groupConversations` | 2364 | Port logic to Rust/WASM | unmapped |
| `resizeComposerInput` | 2441 | Port logic to Rust/WASM | unmapped |
| `localizeTool` | 2458 | Port logic to Rust/WASM | unmapped |
| `isAgentFormattingDirective` | 2499 | Port logic to Rust/WASM | unmapped |
| `extractAgentQuotedPhrases` | 2508 | UI boundary candidate; audit mixed business logic | unmapped |
| `agentResearchCommandPrefix` | 2517 | Port logic to Rust/WASM | unmapped |
| `agentComparisonFocus` | 2526 | UI boundary candidate; audit mixed business logic | unmapped |
| `expandAgentResearchStep` | 2543 | Port logic to Rust/WASM | unmapped |
| `decomposeAgentTask` | 2559 | UI boundary candidate; audit mixed business logic | unmapped |
| `messagesForConversation` | 2589 | UI boundary candidate; audit mixed business logic | unmapped |
| `conversationToMarkdown` | 2623 | UI boundary candidate; audit mixed business logic | unmapped |
| `randomInt` | 2661 | Port logic to Rust/WASM | unmapped |
| `timeLabel` | 2665 | Port logic to Rust/WASM | unmapped |
| `formatDiagnosticPayload` | 2677 | Port logic to Rust/WASM | unmapped |
| `truncateDiagnosticDetail` | 2687 | Port logic to Rust/WASM | unmapped |
| `summarizeToolCall` | 2693 | UI boundary candidate; audit mixed business logic | unmapped |
| `humanizeThinkingIdentifier` | 2710 | Port logic to Rust/WASM | unmapped |
| `thinkingLanguageLabel` | 2722 | Rust-name candidate; verify duplication before migration | rust/src/thinking.rs |
| `thinkingRouteLabel` | 2730 | Port logic to Rust/WASM | unmapped |
| `thinkingRuleLabel` | 2739 | Port logic to Rust/WASM | unmapped |
| `thinkingToolLabel` | 2748 | Port logic to Rust/WASM | unmapped |
| `truncateThinkingSummary` | 2753 | Port logic to Rust/WASM | unmapped |
| `summarizeThinkingDetail` | 2759 | Port logic to Rust/WASM | unmapped |
| `thinkingDetailText` | 2787 | Port logic to Rust/WASM | unmapped |
| `thinkingIndefiniteArticle` | 2799 | Port logic to Rust/WASM | unmapped |
| `formalizationOpLabel` | 2820 | Port logic to Rust/WASM | unmapped |
| `naturalizeThinkingStep` | 2838 | Rust-name candidate; verify duplication before migration | rust/src/thinking.rs |
| `thinkingStepKey` | 3045 | Port logic to Rust/WASM | unmapped |
| `normalizeStepLevel` | 3058 | Port logic to Rust/WASM | unmapped |
| `appendStepLevelEvent` | 3067 | Port logic to Rust/WASM | unmapped |
| `projectStepLevels` | 3077 | Port logic to Rust/WASM | unmapped |
| `filterThinkingEntriesForDetail` | 3089 | Port logic to Rust/WASM | unmapped |
| `filterThinkingSummariesForDetail` | 3150 | Port logic to Rust/WASM | unmapped |
| `buildThinkingPreviewSteps` | 3163 | Port logic to Rust/WASM | unmapped |
| `buildMessageThinkingPreviewSteps` | 3194 | Port logic to Rust/WASM | unmapped |
| `createMessage` | 3220 | Port logic to Rust/WASM | unmapped |
| `browserRuntimeStatusKey` | 3231 | Port logic to Rust/WASM | unmapped |
| `FormalizationView` | 3243 | React component | unmapped |
| `escapeHtml` | 3248 | Port logic to Rust/WASM | unmapped |
| `isHttpExternalLink` | 3257 | UI boundary candidate; audit mixed business logic | unmapped |
| `enhanceMarkdownLinks` | 3266 | UI boundary candidate; audit mixed business logic | unmapped |
| `markdownHtml` | 3287 | UI boundary candidate; audit mixed business logic | unmapped |
| `copyTextToClipboard` | 3304 | UI boundary candidate; audit mixed business logic | unmapped |
| `flashCopied` | 3337 | Port logic to Rust/WASM | unmapped |
| `enhanceCodeBlocks` | 3359 | UI boundary candidate; audit mixed business logic | unmapped |
| `normalizePrompt` | 3428 | Rust-name candidate; verify duplication before migration | rust/src/web_engine_core.rs, rust/src/engine.rs |
| `isIdentityPrompt` | 3435 | Port logic to Rust/WASM | unmapped |
| `isLocalAssistantFreeTimePrompt` | 3491 | Port logic to Rust/WASM | unmapped |
| `localPromptLanguage` | 3511 | Port logic to Rust/WASM | unmapped |
| `isAssistantNamePrompt` | 3519 | Port logic to Rust/WASM | unmapped |
| `localAssistantNameAnswer` | 3547 | Port logic to Rust/WASM | unmapped |
| `localBehaviorRuleId` | 3574 | Port logic to Rust/WASM | unmapped |
| `localCodeSpans` | 3584 | Port logic to Rust/WASM | unmapped |
| `localLooksLikeRuntimeRuleUpdate` | 3611 | Port logic to Rust/WASM | unmapped |
| `localRuntimeRuleFromText` | 3640 | Port logic to Rust/WASM | unmapped |
| `localBehaviorRuleRecords` | 3654 | Port logic to Rust/WASM | unmapped |
| `localLocalizedText` | 3721 | Port logic to Rust/WASM | unmapped |
| `localBehaviorRuleTopicLabel` | 3725 | Port logic to Rust/WASM | unmapped |
| `localBehaviorRuleListIntro` | 3756 | Port logic to Rust/WASM | unmapped |
| `localRuntimeRuleWhenThen` | 3765 | Port logic to Rust/WASM | unmapped |
| `localRuleResponse` | 3772 | Port logic to Rust/WASM | unmapped |
| `localRuleLabel` | 3797 | Port logic to Rust/WASM | unmapped |
| `localRuleMatches` | 3833 | Port logic to Rust/WASM | unmapped |
| `localRuleWhenThen` | 3869 | Port logic to Rust/WASM | unmapped |
| `localBehaviorRuleListFooter` | 3899 | Port logic to Rust/WASM | unmapped |
| `localBehaviorRulesList` | 3936 | UI boundary candidate; audit mixed business logic | unmapped |
| `localBehaviorRulesCount` | 3973 | Port logic to Rust/WASM | unmapped |
| `localBehaviorRuleDetail` | 4004 | Port logic to Rust/WASM | unmapped |
| `localAssistantNameStatus` | 4034 | Port logic to Rust/WASM | unmapped |
| `localLinoEscape` | 4039 | Port logic to Rust/WASM | unmapped |
| `localModeStatus` | 4052 | Port logic to Rust/WASM | unmapped |
| `localDefinitionFusionStatus` | 4056 | Port logic to Rust/WASM | unmapped |
| `localBlueprintCompositionStatus` | 4060 | Port logic to Rust/WASM | unmapped |
| `localSelfFacts` | 4064 | Port logic to Rust/WASM | unmapped |
| `localKnownFacts` | 4128 | Port logic to Rust/WASM | unmapped |
| `localContainsAny` | 4189 | Port logic to Rust/WASM | unmapped |
| `localIsSelfFactQuery` | 4193 | Port logic to Rust/WASM | unmapped |
| `localIsSelfIntroductionQuery` | 4202 | Port logic to Rust/WASM | unmapped |
| `localSelfAwarenessLanguage` | 4231 | Port logic to Rust/WASM | unmapped |
| `localSelfIntroductionContent` | 4241 | Port logic to Rust/WASM | unmapped |
| `localCleanConversationTopic` | 4251 | Port logic to Rust/WASM | unmapped |
| `localConversationTopic` | 4257 | UI boundary candidate; audit mixed business logic | unmapped |
| `localConversationTopicContent` | 4287 | Port logic to Rust/WASM | unmapped |
| `localIsKnownFactQuery` | 4300 | Port logic to Rust/WASM | unmapped |
| `localIsArchitectureQuestion` | 4347 | Port logic to Rust/WASM | unmapped |
| `localArchitectureExplanation` | 4390 | Port logic to Rust/WASM | unmapped |
| `localCleanRuleQuery` | 4398 | Port logic to Rust/WASM | unmapped |
| `localDetailQuery` | 4405 | UI boundary candidate; audit mixed business logic | unmapped |
| `localFindBehaviorRule` | 4416 | UI boundary candidate; audit mixed business logic | unmapped |
| `localRuntimeRuleForPrompt` | 4428 | Port logic to Rust/WASM | unmapped |
| `localCollectRuntimeRules` | 4442 | UI boundary candidate; audit mixed business logic | unmapped |
| `tryLocalBehaviorRules` | 4457 | Port logic to Rust/WASM | unmapped |
| `matchesLocalBehaviorRulesListPattern` | 4558 | Port logic to Rust/WASM | unmapped |
| `localIsBehaviorRulesList` | 4565 | Port logic to Rust/WASM | unmapped |
| `localIsBehaviorRulesCount` | 4576 | Port logic to Rust/WASM | unmapped |
| `localPriorBehaviorRulesListContext` | 4601 | Port logic to Rust/WASM | unmapped |
| `isSupportedLanguageBehaviorRulesListQuery` | 4615 | Rust-name candidate; verify duplication before migration | rust/src/solver_handlers/behavior_rule_matching.rs |
| `isEnglishBehaviorRulesListQuery` | 4624 | Port logic to Rust/WASM | unmapped |
| `isRussianBehaviorRulesListQuery` | 4644 | Port logic to Rust/WASM | unmapped |
| `isHindiBehaviorRulesListQuery` | 4663 | Port logic to Rust/WASM | unmapped |
| `isChineseBehaviorRulesListQuery` | 4683 | Port logic to Rust/WASM | unmapped |
| `chooseVariant` | 4704 | Port logic to Rust/WASM | unmapped |
| `shouldIncludeCourtesyFollowUp` | 4710 | Port logic to Rust/WASM | unmapped |
| `courtesyResponseContent` | 4721 | Port logic to Rust/WASM | unmapped |
| `desktopBridge` | 4735 | UI boundary candidate; audit mixed business logic | unmapped |
| `desktopServiceBridge` | 4745 | Port logic to Rust/WASM | unmapped |
| `normalizeAppVersion` | 4753 | UI boundary candidate; audit mixed business logic | unmapped |
| `desktopAppVersionLabel` | 4761 | Port logic to Rust/WASM | unmapped |
| `normalizeDesktopUpdaterStatus` | 4768 | Port logic to Rust/WASM | unmapped |
| `mergeDesktopUpdateStatus` | 4793 | Port logic to Rust/WASM | unmapped |
| `desktopUpdaterStateLabel` | 4810 | Port logic to Rust/WASM | unmapped |
| `desktopUpdaterBusy` | 4838 | Port logic to Rust/WASM | unmapped |
| `serviceStateLabel` | 4844 | Port logic to Rust/WASM | unmapped |
| `vscodeInstallStateLabel` | 4863 | Port logic to Rust/WASM | unmapped |
| `normalizeDataMigration` | 4886 | Port logic to Rust/WASM | unmapped |
| `shouldShowDataMigrationNotice` | 4908 | Port logic to Rust/WASM | unmapped |
| `DataMigrationNotice` | 4915 | React component | unmapped |
| `normalizeDesktopStatus` | 4967 | Port logic to Rust/WASM | unmapped |
| `compactUrl` | 5021 | Port logic to Rust/WASM | unmapped |
| `desktopSurfaceLabel` | 5038 | Port logic to Rust/WASM | unmapped |
| `desktopStatusLabel` | 5042 | Port logic to Rust/WASM | unmapped |
| `desktopAgentEventLabel` | 5058 | Port logic to Rust/WASM | unmapped |
| `desktopMessages` | 5069 | UI boundary candidate; audit mixed business logic | unmapped |
| `syncDesktopToolGrants` | 5087 | UI boundary candidate; audit mixed business logic | unmapped |
| `ensureDesktopAgentServer` | 5094 | Port logic to Rust/WASM | unmapped |
| `requestDesktopToolCall` | 5104 | Port logic to Rust/WASM | unmapped |
| `requestDesktopAgentProvider` | 5124 | Port logic to Rust/WASM | unmapped |
| `chatAnswerFromAgentProviderResult` | 5141 | Port logic to Rust/WASM | unmapped |
| `terminalCommandFromAnswer` | 5148 | UI boundary candidate; audit mixed business logic | unmapped |
| `desktopShellCommand` | 5162 | Port logic to Rust/WASM | unmapped |
| `shellOutputMarkdown` | 5170 | Port logic to Rust/WASM | unmapped |
| `desktopToolResultReason` | 5179 | Port logic to Rust/WASM | unmapped |
| `syncDesktopMemory` | 5202 | Port logic to Rust/WASM | unmapped |
| `normalizeApiThinkingStep` | 5213 | Port logic to Rust/WASM | unmapped |
| `requestDesktopAnswer` | 5233 | UI boundary candidate; audit mixed business logic | unmapped |
| `localFallbackAnswer` | 5305 | Port logic to Rust/WASM | unmapped |
| `localSelectUnknownOpener` | 5383 | Port logic to Rust/WASM | unmapped |
| `localUnknownAnswerWithVariation` | 5392 | UI boundary candidate; audit mixed business logic | unmapped |
| `createDemoTurns` | 5408 | UI boundary candidate; audit mixed business logic | unmapped |
| `reportTurns` | 5430 | Rust-name candidate; verify duplication before migration | rust/src/cli_report.rs |
| `truncateSingleLine` | 5451 | Port logic to Rust/WASM | unmapped |
| `truncateMessageContent` | 5469 | Port logic to Rust/WASM | unmapped |
| `compactReportTraceValue` | 5487 | Port logic to Rust/WASM | unmapped |
| `appendLimitedTraceItems` | 5496 | UI boundary candidate; audit mixed business logic | unmapped |
| `reasoningTraceLines` | 5518 | UI boundary candidate; audit mixed business logic | unmapped |
| `buildIssueUrl` | 5574 | Port logic to Rust/WASM | unmapped |
| `buildIssueUrlForMessages` | 5579 | Port logic to Rust/WASM | unmapped |
| `fitIssueUrl` | 5584 | Port logic to Rust/WASM | unmapped |
| `shortText` | 5691 | Port logic to Rust/WASM | unmapped |
| `promptBeforeMessage` | 5700 | Port logic to Rust/WASM | unmapped |
| `lastUnknownAssistantMessage` | 5713 | Port logic to Rust/WASM | unmapped |
| `createIssueTitle` | 5722 | Port logic to Rust/WASM | unmapped |
| `formatVersionWithWorker` | 5737 | Port logic to Rust/WASM | unmapped |
| `reportLabels` | 5761 | Port logic to Rust/WASM | unmapped |
| `environmentFields` | 5779 | UI boundary candidate; audit mixed business logic | unmapped |
| `createIssueReportBody` | 5799 | Port logic to Rust/WASM | unmapped |
| `createIssueUrl` | 5816 | Port logic to Rust/WASM | unmapped |
| `shouldOfferMessageReport` | 5820 | Port logic to Rust/WASM | unmapped |
| `formatHttpExchangeAsLinks` | 5827 | UI boundary candidate; audit mixed business logic | unmapped |
| `DiagnosticsHttpPanel` | 5855 | React component | unmapped |
| `usePrefersReducedMotion` | 5902 | UI boundary candidate; audit mixed business logic | unmapped |
| `useMessageReveal` | 5947 | UI boundary candidate; audit mixed business logic | unmapped |
| `usePendingThinkingPhases` | 6003 | Port logic to Rust/WASM | unmapped |
| `PendingAssistantBubble` | 6035 | React component | unmapped |
| `thinkingNarrative` | 6096 | Rust-name candidate; verify duplication before migration | rust/src/thinking.rs |
| `ThinkingPreview` | 6106 | React component | unmapped |
| `DesktopPermissionPanel` | 6130 | React component | unmapped |
| `CommandApprovalPanel` | 6168 | React component | unmapped |
| `StepHierarchyMenu` | 6192 | React component | unmapped |
| `Message` | 6213 | React component | rust/src/memory/upgrade.rs |
| `CollapsibleSection` | 6369 | React component | unmapped |
| `App` | 6411 | React component | unmapped |
| `wait` | 9418 | UI boundary candidate; audit mixed business logic | unmapped |
