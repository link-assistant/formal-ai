// App hooks for the desktop (Electron) shell: bridge wiring and terminal
// command execution with per-command approval.

import React from "react";
import {
  chatAnswerFromAgentProviderResult, desktopBridge, desktopServiceBridge, desktopShellCommand,
  desktopToolResultReason, ensureDesktopAgentServer, mergeDesktopUpdateStatus,
  normalizeDataMigration, normalizeDesktopStatus, requestDesktopAgentProvider,
  requestDesktopToolCall, shellOutputMarkdown, syncDesktopToolGrants,
} from "./desktop-bridge.jsx";
import { DESKTOP_TOOL_OPTIONS, desktopToolRouterGrants } from "./preferences.jsx";

const { useCallback, useEffect } = React;

// Desktop (Electron) bridge wiring: status, data migration, updates,
// agent events, prepared services and the VS Code extension installer.
export function useDesktopIntegration({
  desktopStatus, setDesktopStatus, setDataMigration, setDataMigrationBusy,
  setDesktopAgentStream, setServiceStatus, setServiceBusy, setServiceError, telegramToken,
  setUpdateBusy, setVscodeInstallBusy, setVscodeInstallResult, mode, desktopToolGrants,
}) {
  useEffect(() => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.getStatus !== "function") {
      return undefined;
    }
    let cancelled = false;
    bridge
      .getStatus()
      .then((status) => {
        if (!cancelled) {
          setDesktopStatus(normalizeDesktopStatus(status));
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setDesktopStatus(
            normalizeDesktopStatus({
              shell: "Electron",
              apiError: error && error.message ? error.message : String(error),
              apiReady: false,
            }),
          );
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Issue #672 (F2): read the startup migration result once. Web builds have no
  // bridge and older desktop builds have no channel, so both are treated the
  // same way — no notice at all.
  useEffect(() => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.dataMigrationStatus !== "function") {
      return undefined;
    }
    let cancelled = false;
    Promise.resolve()
      .then(() => bridge.dataMigrationStatus())
      .then((result) => {
        if (!cancelled) setDataMigration(normalizeDataMigration(result));
      })
      .catch(() => {
        // A migration we cannot report on is not worth interrupting boot for.
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const handleReplayDataMigration = useCallback(() => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.replayDataMigration !== "function") return;
    setDataMigrationBusy(true);
    Promise.resolve()
      .then(() => bridge.replayDataMigration())
      .then((result) => {
        setDataMigration(normalizeDataMigration(result, { replayed: true }));
      })
      .catch((error) => {
        setDataMigration(
          normalizeDataMigration(
            {
              known: true,
              reason: "failed",
              error: error && error.message ? error.message : String(error),
            },
            { replayed: true },
          ),
        );
      })
      .finally(() => setDataMigrationBusy(false));
  }, []);

  useEffect(() => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.onUpdateStatus !== "function") {
      return undefined;
    }
    const unsubscribe = bridge.onUpdateStatus((status) => {
      setDesktopStatus((current) => mergeDesktopUpdateStatus(current, status));
    });
    return typeof unsubscribe === "function" ? unsubscribe : undefined;
  }, []);

  useEffect(() => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.onAgentEvent !== "function") return undefined;
    const unsubscribe = bridge.onAgentEvent((payload) => {
      setDesktopAgentStream((current) => [...current.slice(-19), payload]);
    });
    return typeof unsubscribe === "function" ? unsubscribe : undefined;
  }, []);

  useEffect(() => {
    // Push the explicit per-tool grant map to the local router whenever either
    // the operating mode or a grant decision changes.
    syncDesktopToolGrants(desktopBridge(), mode, desktopToolGrants);
  }, [mode, desktopToolGrants, desktopStatus]);

  useEffect(() => {
    if (mode === "chat") {
      return undefined;
    }
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.ensureAgentServer !== "function") {
      return undefined;
    }
    let cancelled = false;
    ensureDesktopAgentServer(bridge)
      .then((status) => {
        if (!cancelled && status) {
          setDesktopStatus(status);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setDesktopStatus((current) =>
            normalizeDesktopStatus({
              ...(current || {}),
              shell: (current && current.shell) || "Electron",
              apiReady: false,
              apiError: error && error.message ? error.message : String(error),
            }),
          );
        }
      });
    return () => {
      cancelled = true;
    };
  }, [mode]);

  // Issue #438 (follow-up): poll the desktop bridge for the prepared-container
  // status so the Services panel reflects running/stopped without a manual
  // refresh, and expose it for the one-click buttons.
  const refreshServiceStatus = useCallback(async () => {
    const bridge = desktopServiceBridge();
    if (!bridge) {
      return null;
    }
    try {
      const snapshot = await bridge.serviceStatus();
      setServiceStatus(snapshot && typeof snapshot === "object" ? snapshot : null);
      return snapshot;
    } catch (error) {
      setServiceError(error && error.message ? error.message : String(error));
      return null;
    }
  }, []);

  useEffect(() => {
    const bridge = desktopServiceBridge();
    if (!bridge) {
      return undefined;
    }
    let active = true;
    const tick = () => {
      if (active) {
        refreshServiceStatus();
      }
    };
    tick();
    const timer = setInterval(tick, 5000);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [refreshServiceStatus]);

  const handleStartService = useCallback(
    async (key) => {
      const bridge = desktopServiceBridge();
      if (!bridge || typeof bridge.startService !== "function") {
        return;
      }
      setServiceBusy(key);
      setServiceError("");
      try {
        const request = { service: key };
        if (key === "telegram") {
          request.token = telegramToken.trim();
        }
        const result =
          key === "agent" && typeof bridge.installAgentEnvironment === "function"
            ? await bridge.installAgentEnvironment()
            : await bridge.startService(request);
        if (result && result.ok === false && result.reason) {
          setServiceError(result.reason);
        }
      } catch (error) {
        setServiceError(error && error.message ? error.message : String(error));
      } finally {
        setServiceBusy("");
        await refreshServiceStatus();
      }
    },
    [telegramToken, refreshServiceStatus],
  );

  const handleStopService = useCallback(
    async (key) => {
      const bridge = desktopServiceBridge();
      if (!bridge || typeof bridge.stopService !== "function") {
        return;
      }
      setServiceBusy(key);
      setServiceError("");
      try {
        const result = await bridge.stopService({ service: key });
        if (result && result.ok === false && result.reason) {
          setServiceError(result.reason);
        }
      } catch (error) {
        setServiceError(error && error.message ? error.message : String(error));
      } finally {
        setServiceBusy("");
        await refreshServiceStatus();
      }
    },
    [refreshServiceStatus],
  );

  const handleCheckForUpdates = useCallback(async () => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.checkForUpdates !== "function") {
      return;
    }
    setUpdateBusy("check");
    try {
      const status = await bridge.checkForUpdates();
      setDesktopStatus((current) => mergeDesktopUpdateStatus(current, status));
    } finally {
      setUpdateBusy("");
    }
  }, []);

  const handleInstallUpdate = useCallback(async () => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.installUpdate !== "function") {
      return;
    }
    setUpdateBusy("install");
    try {
      const status = await bridge.installUpdate();
      setDesktopStatus((current) => mergeDesktopUpdateStatus(current, status));
    } finally {
      setUpdateBusy("");
    }
  }, []);

  // Issue #554 (R2): one-click VS Code extension install. The main process
  // detects a VS Code CLI, downloads the latest release `.vsix`, and runs
  // `code --install-extension`; we surface the structured result inline.
  const handleInstallVsCodeExtension = useCallback(async () => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.installVsCodeExtension !== "function") {
      return;
    }
    setVscodeInstallBusy(true);
    setVscodeInstallResult(null);
    try {
      const result = await bridge.installVsCodeExtension();
      setVscodeInstallResult(result || { ok: false, state: "error" });
    } catch (error) {
      setVscodeInstallResult({
        ok: false,
        state: "error",
        reason: error && error.message ? error.message : String(error),
      });
    } finally {
      setVscodeInstallBusy(false);
    }
  }, []);

  useEffect(() => {
    // R5d: expose a thin hook so the local tool router (and the desktop e2e
    // suite) can route a tool call through the bridge to the local process /
    // Docker sandbox. The gate still applies — denied calls return a refusal.
    if (typeof window === "undefined") {
      return undefined;
    }
    window.formalAiDesktopToolCall = (tool, input) =>
      requestDesktopToolCall(desktopBridge(), tool, input);
    return () => {
      delete window.formalAiDesktopToolCall;
    };
  }, []);

  return {
    handleReplayDataMigration, handleStartService, handleStopService, handleCheckForUpdates,
    handleInstallUpdate, handleInstallVsCodeExtension,
  };
}

// Terminal command execution, approval and permission escalation.
export function useDesktopCommands({
  t, setMode, setDesktopToolGrants, setCommandApprovals, agentModeRef, modeRef,
  desktopToolGrantsRef, commandApprovalsRef, pendingAgentTaskRef, desktopStatusRef,
  appendSystemMessage, showAgentOnboarding, capturePendingAgentTask, clearPendingAgentTask,
  appendAssistantMessage,
}) {
  const executeTerminalCommand = useCallback(async (command, executionMode = "agent") => {
    const bridge = desktopBridge();
    const providerResult = await requestDesktopAgentProvider(bridge, {
      mode: executionMode === "fullAuto" ? "fullAuto" : "agent",
      tool: "shell",
      command,
      grants: desktopToolRouterGrants(modeRef.current, desktopToolGrantsRef.current),
      transcript: true,
    });
    const providerAnswer = chatAnswerFromAgentProviderResult(providerResult);
    if (providerAnswer) {
      appendAssistantMessage({
        ...providerAnswer,
        content: String(providerAnswer.content || desktopToolResultReason(providerResult, t)),
        evidence: [
          ...(Array.isArray(providerAnswer.evidence) ? providerAnswer.evidence : []),
          "desktop_agent_provider",
          `mode:${executionMode}`,
        ],
      });
      return providerResult;
    }

    const result = providerResult || (await requestDesktopToolCall(bridge, "shell", { command }));
    const ok = result && result.ok === true && result.executed === true;
    const content = ok
      ? [
          t("permissions.message.shellRan", {
            mode:
              executionMode === "fullAuto"
                ? t("buttons.fullAuto")
                : t("buttons.agent"),
            command,
          }),
          "",
          shellOutputMarkdown(result.body, t),
        ].join("\n")
      : [
          t("permissions.message.shellNotRun", { command }),
          "",
          desktopToolResultReason(result, t),
        ].join("\n");
    appendAssistantMessage({
      intent: ok ? "desktop_shell_result" : "desktop_shell_refused",
      content,
      confidence: ok ? 0.9 : 1.0,
      evidence: [
        "desktop_tool:shell",
        `mode:${executionMode}`,
        ok ? "desktop_tool:executed" : "desktop_tool:refused",
      ],
      steps: [
        {
          step: ok ? "execute_shell" : "refuse_shell",
          detail: command,
        },
      ],
      toolCalls: [
        {
          tool: "shell",
          inputs: { command, mode: executionMode },
          outputs: result || { ok: false, executed: false, status: "unavailable" },
        },
      ],
    });
    return result;
  }, [appendAssistantMessage, t]);

  // Issue #541 (R9): single-click escalation from the permission panel.
  // 1. Mode flips to "agent" so `requestTerminalCommandExecution` will route
  //    the shell command instead of bouncing it back to chat.
  // 2. Every desktop tool grant flips to true so the router approves the call.
  // 3. Refs are mirrored synchronously (React state lands on the next render,
  //    but `executeTerminalCommand` reads `modeRef.current` and
  //    `desktopToolGrantsRef.current` synchronously inside this callback —
  //    without the manual mirror the replay would race the React update).
  // 4. The replayed command runs in "agent" mode (per-command prompt) so the
  //    user still sees the approve/deny step for the deferred shell command
  //    rather than skipping straight to autorun. If they wanted skip-prompt
  //    behaviour they would choose Full Auto mode instead.
  const grantAllAndRunPending = useCallback(async () => {
    const allGranted = {};
    DESKTOP_TOOL_OPTIONS.forEach((tool) => { allGranted[tool] = true; });
    desktopToolGrantsRef.current = allGranted;
    setDesktopToolGrants(allGranted);
    modeRef.current = "agent";
    agentModeRef.current = true;
    setMode("agent");
    const task = pendingAgentTaskRef.current;
    clearPendingAgentTask();
    if (task && task.kind === "shell" && task.command) {
      await executeTerminalCommand(task.command, "agent");
    }
  }, [clearPendingAgentTask, executeTerminalCommand]);

  const requestTerminalCommandExecution = useCallback(
    async (command, answer) => {
      const safeCommand = desktopShellCommand(command, desktopStatusRef.current);
      if (!safeCommand) {
        appendAssistantMessage(answer);
        return;
      }
      if (!agentModeRef.current) {
        // Issue #541 (R9): stash the command so the panel's "Grant all" button
        // can replay it without the user having to retype the prompt.
        capturePendingAgentTask(safeCommand);
        appendAssistantMessage(answer);
        showAgentOnboarding();
        return;
      }
      showAgentOnboarding();
      if (desktopToolGrantsRef.current.shell !== true) {
        // Same as above — user opted in to Agent mode but hasn't granted
        // `shell` yet. Stash the command for one-click recovery.
        capturePendingAgentTask(safeCommand);
        appendAssistantMessage({
          intent: "desktop_shell_not_granted",
          content: t("permissions.message.shellNotGranted"),
          confidence: 1.0,
          evidence: ["desktop_tool:shell", "desktop_tool:not_granted"],
          steps: [{ step: "check_tool_grant", detail: "shell=false" }],
          toolCalls: [
            {
              tool: "shell",
              inputs: { command: safeCommand },
              outputs: {
                ok: false,
                executed: false,
                status: "refused",
                reason: "shell tool is not granted",
              },
            },
          ],
        });
        return;
      }
      if (modeRef.current === "fullAuto") {
        await executeTerminalCommand(safeCommand, "fullAuto");
        return;
      }
      const approval = {
        id: `command-${Date.now()}-${Math.random().toString(16).slice(2)}`,
        tool: "shell",
        command: safeCommand,
        status: "pending",
      };
      setCommandApprovals((current) => ({
        ...current,
        [approval.id]: approval,
      }));
      appendSystemMessage(
        `${t("permissions.message.approvalPrompt")}\n\n\`${safeCommand}\``,
        {
          intent: "desktop_command_approval",
          commandApproval: approval,
        },
      );
    },
    [appendAssistantMessage, appendSystemMessage, capturePendingAgentTask, executeTerminalCommand, showAgentOnboarding, t],
  );

  const approveDesktopCommand = useCallback(
    async (approval) => {
      if (!approval || !approval.id) {
        return;
      }
      const existing = commandApprovalsRef.current[approval.id] || approval;
      if (existing.status !== "pending") {
        return;
      }
      const running = { ...existing, status: "running" };
      commandApprovalsRef.current = {
        ...commandApprovalsRef.current,
        [approval.id]: running,
      };
      setCommandApprovals(commandApprovalsRef.current);
      await executeTerminalCommand(approval.command, "agent");
      const approved = { ...running, status: "approved" };
      commandApprovalsRef.current = {
        ...commandApprovalsRef.current,
        [approval.id]: approved,
      };
      setCommandApprovals(commandApprovalsRef.current);
    },
    [executeTerminalCommand],
  );

  const denyDesktopCommand = useCallback(
    (approval) => {
      if (!approval || !approval.id) {
        return;
      }
      const existing = commandApprovalsRef.current[approval.id] || approval;
      if (existing.status !== "pending") {
        return;
      }
      const denied = { ...existing, status: "denied" };
      commandApprovalsRef.current = {
        ...commandApprovalsRef.current,
        [approval.id]: denied,
      };
      setCommandApprovals(commandApprovalsRef.current);
      appendAssistantMessage({
        intent: "desktop_shell_denied",
        content: t("permissions.message.commandDeclined", {
          command: approval.command,
        }),
        confidence: 1.0,
        evidence: ["desktop_tool:shell", "desktop_tool:user_denied"],
        steps: [{ step: "user_denied_shell", detail: approval.command }],
        toolCalls: [
          {
            tool: "shell",
            inputs: { command: approval.command, mode: "agent" },
            outputs: { ok: false, executed: false, status: "denied" },
          },
        ],
      });
    },
    [appendAssistantMessage, t],
  );

  return {
    grantAllAndRunPending, requestTerminalCommandExecution, approveDesktopCommand,
    denyDesktopCommand,
  };
}
