// App hooks for user-initiated memory management and memory writes.

import React from "react";
import { APP_VERSION } from "./app-constants.jsx";
import { groupConversations } from "./conversations.jsx";
import { desktopBridge, syncDesktopMemory } from "./desktop-bridge.jsx";
import {
  MEMORY_EXPORT_FILENAME, downloadTextFile, recordMemoryEvent, waitForMemoryWrites,
} from "./memory-events.jsx";
import { loadPreferences } from "./preferences.jsx";

const { useCallback, useEffect } = React;

// Memory export/import/reset/purge actions and desktop memory sync.
export function useMemoryActions({
  importInputRef, messages, setMessages, setPrompt, workerState, setMemoryStatus, seed, t,
  demoMode, setShowDeletedConversations, desktopStatus, setCurrentConversationId,
  currentConversationRef, userContext, refreshConversations,
}) {
  const handleExportMemory = useCallback(async () => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      setMemoryStatus(t("status.memoryUnavailable"));
      return;
    }
    try {
      await waitForMemoryWrites();
      const events = await window.FormalAiMemory.listEvents();
      const preferences = loadPreferences();
      const text = window.FormalAiMemory.exportFullMemory({
        seed,
        events,
        preferences,
        info: {
          version: APP_VERSION,
          url: window.location.href,
          userAgent: navigator.userAgent,
          workerState,
          mode: demoMode ? "demo" : "manual",
          ...userContext,
        },
      });
      downloadTextFile(MEMORY_EXPORT_FILENAME, text);
      const seedFileCount = seed && seed.raw ? Object.keys(seed.raw).length : 0;
      setMemoryStatus(
        t("status.memoryExported", {
          events: events.length,
          seedFiles: seedFileCount,
        }),
      );
    } catch (_error) {
      setMemoryStatus(t("status.exportFailed"));
    }
  }, [seed, workerState, demoMode, userContext, t]);

  // R5c (D1): reconcile the browser (IndexedDB) memory log with the native store
  // via the desktop bridge. Pushes the current `demo_memory` event log to the
  // local server and folds any pulled delta back into IndexedDB. Best-effort.
  const syncDesktopMemoryNow = useCallback(async () => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.syncMemory !== "function") {
      return null;
    }
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      return null;
    }
    try {
      await waitForMemoryWrites();
      const events = await window.FormalAiMemory.listEvents();
      const lino = window.FormalAiMemory.exportLinksNotation(events);
      const result = await syncDesktopMemory(bridge, lino);
      const delta = result && result.pulled ? result.pulled.delta : "";
      if (delta && delta.trim()) {
        const imported = window.FormalAiMemory.importFullMemory(delta);
        if (imported && Array.isArray(imported.events) && imported.events.length > 0) {
          await window.FormalAiMemory.importEvents(imported.events);
        }
      }
      return result;
    } catch (_error) {
      return null;
    }
  }, []);

  useEffect(() => {
    // R5c: keep the native store in step with the browser log after each turn
    // while the local server is the active surface. Declared after
    // `syncDesktopMemoryNow` so the dependency reference is initialized.
    if (!desktopStatus || !desktopStatus.apiReady) {
      return;
    }
    syncDesktopMemoryNow();
  }, [messages, desktopStatus, syncDesktopMemoryNow]);

  const handleImportMemory = useCallback(async (event) => {
    const file = event.target.files && event.target.files[0];
    event.target.value = "";
    if (!file || typeof window === "undefined" || !window.FormalAiMemory) {
      return;
    }
    try {
      const text = await file.text();
      const imported = window.FormalAiMemory.importFullMemory(text);
      const inserted = await window.FormalAiMemory.importEvents(imported.events);
      const current = {
        agentInfo: seed && seed.agentInfo ? seed.agentInfo : {},
        info: { version: APP_VERSION },
      };
      const suggestions = window.FormalAiMemory.suggestMigrations({
        imported,
        current,
      });
      const headline =
        imported.kind === "bundle"
          ? t("status.memoryImportedBundle", { inserted })
          : t("status.memoryImportedEvents", { inserted });
      if (suggestions.length > 0) {
        setMemoryStatus(
          t("status.migration", {
            headline,
            suggestions: suggestions.join(" / "),
          }),
        );
      } else {
        setMemoryStatus(headline);
      }
    } catch (_error) {
      setMemoryStatus(t("status.importFailed"));
    }
  }, [seed, t]);

  const triggerImportMemory = useCallback(() => {
    if (importInputRef.current) {
      importInputRef.current.click();
    }
  }, []);

  const confirmDangerousMemoryAction = useCallback(
    async (exportPrompt, confirmPrompt) => {
      if (typeof window === "undefined" || typeof window.confirm !== "function") {
        return true;
      }
      if (window.confirm(exportPrompt)) {
        await handleExportMemory();
        return false;
      }
      return window.confirm(confirmPrompt);
    },
    [handleExportMemory],
  );

  const handleResetMemory = useCallback(async () => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      setMemoryStatus(t("status.memoryUnavailable"));
      return { cancelled: true, removed: 0 };
    }
    const proceed = await confirmDangerousMemoryAction(
      t("confirm.resetMemoryExportFirst"),
      t("confirm.resetMemory"),
    );
    if (!proceed) {
      return { cancelled: true, removed: 0 };
    }
    try {
      await waitForMemoryWrites();
      const removed = await window.FormalAiMemory.clearEvents();
      currentConversationRef.current = "";
      setCurrentConversationId("");
      setMessages([]);
      setPrompt("");
      setShowDeletedConversations(false);
      await refreshConversations(false);
      setMemoryStatus(t("status.memoryReset", { events: removed }));
      return { cancelled: false, removed };
    } catch (_error) {
      setMemoryStatus(t("status.memoryResetFailed"));
      return { cancelled: true, removed: 0 };
    }
  }, [confirmDangerousMemoryAction, refreshConversations, t]);

  const handlePurgeDeletedConversations = useCallback(async () => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      setMemoryStatus(t("status.memoryUnavailable"));
      return;
    }
    const proceed = await confirmDangerousMemoryAction(
      t("confirm.purgeDeletedExportFirst"),
      t("confirm.purgeDeleted"),
    );
    if (!proceed) {
      return;
    }
    try {
      await waitForMemoryWrites();
      const events = await window.FormalAiMemory.listEvents();
      const deletedIds = new Set(
        groupConversations(events, { showDeleted: true }).map((entry) => entry.id),
      );
      const removed = await window.FormalAiMemory.purgeDeletedConversations();
      if (deletedIds.has(currentConversationRef.current)) {
        currentConversationRef.current = "";
        setCurrentConversationId("");
        setMessages([]);
        setPrompt("");
      }
      setShowDeletedConversations(true);
      await refreshConversations(true);
      setMemoryStatus(t("status.deletedConversationsPurged", { events: removed }));
    } catch (_error) {
      setMemoryStatus(t("status.memoryResetFailed"));
    }
  }, [confirmDangerousMemoryAction, refreshConversations, t]);

  const handlePurgeConversation = useCallback(
    async (entry) => {
      if (!entry || !entry.id || typeof window === "undefined" || !window.FormalAiMemory) {
        return;
      }
      const proceed = await confirmDangerousMemoryAction(
        t("confirm.deleteConversationPermanentExportFirst"),
        t("confirm.deleteConversationPermanent"),
      );
      if (!proceed) {
        return;
      }
      try {
        await waitForMemoryWrites();
        const removed = await window.FormalAiMemory.deleteEventsByConversationId(entry.id);
        if (entry.id === currentConversationRef.current) {
          currentConversationRef.current = "";
          setCurrentConversationId("");
          setMessages([]);
          setPrompt("");
        }
        setShowDeletedConversations(true);
        await refreshConversations(true);
        setMemoryStatus(t("status.conversationPurged", { events: removed }));
      } catch (_error) {
        setMemoryStatus(t("status.memoryResetFailed"));
      }
    },
    [confirmDangerousMemoryAction, refreshConversations, t],
  );

  return {
    handleExportMemory, handleImportMemory, triggerImportMemory, handleResetMemory,
    handlePurgeDeletedConversations, handlePurgeConversation,
  };
}

// Applies natural-language memory writes returned by the worker.
export function useMemoryOperation({ refreshConversations, ensureConversation }) {
  // Issue #529: apply a natural-language memory write returned by the worker to
  // the persistent associative memory. This is the *write* half of the
  // Turing-complete memory primitive, the browser mirror of try_memory_write in
  // the Rust runtime: an append stores the bare statement as a new, queryable
  // memory event; a substitution rewrites every matching stored value in place
  // (an explicit, user-initiated departure from the passive append-only log) and
  // records an audit event. The user thereby has full read+write control over
  // the associative memory through ordinary chat messages.
  const handleMemoryOperation = useCallback(async (operation) => {
    if (
      !operation ||
      typeof window === "undefined" ||
      !window.FormalAiMemory
    ) {
      return;
    }
    const { conversationId, conversationTitle, isDemo } = ensureConversation("");
    const sentAt = new Date().toISOString();
    const demoFlag = isDemo ? true : undefined;
    if (operation.action === "append" && operation.statement) {
      const learnedAssociation = operation.kind === "associative_research";
      await recordMemoryEvent({
        kind: learnedAssociation ? "associative_research" : "message",
        role: learnedAssociation ? "system" : "user",
        intent: learnedAssociation ? "associative_research" : "memory_write",
        content: operation.statement,
        evidence: [
          learnedAssociation
            ? "memory_write:associative_research"
            : "memory_write:natural_language",
        ],
        sentAt,
        conversationId,
        conversationTitle,
        isDemo: demoFlag,
      });
      refreshConversations();
      return;
    }
    if (operation.action === "substitute" && operation.oldValue) {
      let applied = 0;
      try {
        applied = await window.FormalAiMemory.applySubstitution(
          operation.oldValue,
          operation.newValue,
        );
      } catch (_error) {
        applied = 0;
      }
      await recordMemoryEvent({
        kind: "memory_substitution",
        role: "user",
        intent: "memory_substitution",
        inputs: `replace:${operation.oldValue}`,
        outputs: `with:${operation.newValue}`,
        content: `replace ${operation.oldValue} with ${operation.newValue} in memory`,
        evidence: ["substitution_event:update", `substitution:applied=${applied}`],
        sentAt,
        conversationId,
        conversationTitle,
        isDemo: demoFlag,
      });
      refreshConversations();
      return;
    }
    // Issue #1184 R1184-9: a derivation record is filed with its path as `inputs` (formal_ai_worker_derivation.js reads it back).
    const { action, path, text } = operation;
    if (action === "derivation" && path && text) return recordMemoryEvent({ kind: action, role: "system", intent: action, inputs: path, content: text, evidence: [path], sentAt, conversationId, conversationTitle, isDemo: demoFlag });
    if (operation.action === "program") {
      try {
        await window.FormalAiMemory.applyProgramOperation(operation);
      } catch (_error) {
        return;
      }
      refreshConversations();
    }
  }, [ensureConversation, refreshConversations]);

  return { handleMemoryOperation };
}
