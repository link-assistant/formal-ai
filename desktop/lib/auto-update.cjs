"use strict";

const SUPPORTED_PLATFORMS = new Set(["darwin", "linux", "win32"]);

function safeMessage(error) {
  return error && error.message ? error.message : String(error || "");
}

function readAppVersion(app) {
  try {
    if (app && typeof app.getVersion === "function") {
      const version = String(app.getVersion() || "").trim();
      if (version) {
        return version;
      }
    }
  } catch (_error) {
    /* fall through to dev */
  }
  return "dev";
}

function nowIso(clock) {
  const value = typeof clock === "function" ? clock() : new Date();
  return value instanceof Date ? value.toISOString() : new Date(value).toISOString();
}

function normalizePercent(value) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    return 0;
  }
  return Math.max(0, Math.min(100, parsed));
}

function updateVersionFromInfo(info, fallback = "") {
  if (!info || typeof info !== "object") {
    return fallback;
  }
  return String(info.version || fallback || "").trim();
}

function canUseUpdater({ app, autoUpdater, platform }) {
  if (!SUPPORTED_PLATFORMS.has(platform)) {
    return {
      supported: false,
      reason: `Auto update is not supported on ${platform || "this platform"}.`,
    };
  }
  if (!autoUpdater || typeof autoUpdater.checkForUpdates !== "function") {
    return {
      supported: false,
      reason: "Auto update runtime is unavailable.",
    };
  }
  if (app && app.isPackaged === false) {
    return {
      supported: false,
      reason: "Auto update is available in packaged desktop builds.",
    };
  }
  return { supported: true, reason: "" };
}

function attachUpdaterListener(autoUpdater, eventName, listener) {
  if (!autoUpdater || typeof autoUpdater.on !== "function") {
    return;
  }
  autoUpdater.on(eventName, listener);
}

function createAutoUpdateController(options = {}) {
  const app = options.app || null;
  const autoUpdater = options.autoUpdater || null;
  const platform = options.platform || process.platform;
  const log = typeof options.log === "function" ? options.log : () => {};
  const clock = typeof options.clock === "function" ? options.clock : () => new Date();
  const changeListeners = new Set();
  const availability = canUseUpdater({ app, autoUpdater, platform });
  const currentVersion = readAppVersion(app);
  let activeCheck = null;
  let activeInstall = null;

  const state = {
    supported: availability.supported,
    enabled: availability.supported,
    platform,
    currentVersion,
    state: availability.supported ? "idle" : "disabled",
    updateAvailable: false,
    downloaded: false,
    latestVersion: "",
    progressPercent: 0,
    checkedAt: "",
    error: "",
    message: availability.reason,
  };

  function snapshot() {
    return { ...state };
  }

  function emit() {
    const current = snapshot();
    if (typeof options.onStatusChange === "function") {
      try {
        options.onStatusChange(current);
      } catch (error) {
        log("auto-update status listener failed:", safeMessage(error));
      }
    }
    for (const listener of changeListeners) {
      try {
        listener(current);
      } catch (error) {
        log("auto-update subscriber failed:", safeMessage(error));
      }
    }
  }

  function assign(patch) {
    Object.assign(state, patch);
    emit();
    return snapshot();
  }

  function disabledStatus() {
    return assign({
      state: "disabled",
      enabled: false,
      message: availability.reason,
    });
  }

  async function checkForUpdates() {
    if (!state.supported) {
      return disabledStatus();
    }
    if (activeCheck) {
      return activeCheck;
    }
    activeCheck = (async () => {
      assign({
        state: "checking",
        error: "",
        message: "",
        checkedAt: nowIso(clock),
      });
      try {
        const result = await autoUpdater.checkForUpdates();
        if (state.state === "checking") {
          assign({
            state: "not-available",
            updateAvailable: false,
            downloaded: false,
            latestVersion: updateVersionFromInfo(
              result && result.updateInfo,
              state.latestVersion || state.currentVersion,
            ),
            progressPercent: 0,
          });
        }
      } catch (error) {
        assign({
          state: "error",
          error: safeMessage(error),
          message: safeMessage(error),
        });
      } finally {
        activeCheck = null;
      }
      return snapshot();
    })();
    return activeCheck;
  }

  async function installUpdate() {
    if (!state.supported) {
      return disabledStatus();
    }
    if (activeInstall) {
      return activeInstall;
    }
    activeInstall = (async () => {
      try {
        if (!state.updateAvailable && !state.downloaded) {
          await checkForUpdates();
        }
        if (!state.updateAvailable && !state.downloaded) {
          return snapshot();
        }

        if (!state.downloaded) {
          assign({
            state: "downloading",
            error: "",
            message: "",
          });
          if (typeof autoUpdater.downloadUpdate === "function") {
            await autoUpdater.downloadUpdate();
          }
          if (!state.downloaded) {
            assign({
              state: "downloaded",
              updateAvailable: true,
              downloaded: true,
              progressPercent: 100,
            });
          }
        }

        if (typeof autoUpdater.quitAndInstall === "function") {
          assign({ state: "installing" });
          autoUpdater.quitAndInstall(false, true);
        }
      } catch (error) {
        assign({
          state: "error",
          error: safeMessage(error),
          message: safeMessage(error),
        });
      } finally {
        activeInstall = null;
      }
      return snapshot();
    })();
    return activeInstall;
  }

  if (availability.supported) {
    try {
      autoUpdater.autoDownload = false;
      autoUpdater.autoInstallOnAppQuit = true;
    } catch (error) {
      log("auto-update configuration failed:", safeMessage(error));
    }

    attachUpdaterListener(autoUpdater, "checking-for-update", () => {
      assign({
        state: "checking",
        error: "",
        message: "",
        checkedAt: nowIso(clock),
      });
    });
    attachUpdaterListener(autoUpdater, "update-available", (info) => {
      assign({
        state: "available",
        updateAvailable: true,
        downloaded: false,
        latestVersion: updateVersionFromInfo(info, state.latestVersion),
        progressPercent: 0,
        error: "",
        message: "",
      });
    });
    attachUpdaterListener(autoUpdater, "update-not-available", (info) => {
      assign({
        state: "not-available",
        updateAvailable: false,
        downloaded: false,
        latestVersion: updateVersionFromInfo(info, state.currentVersion),
        progressPercent: 0,
        error: "",
        message: "",
      });
    });
    attachUpdaterListener(autoUpdater, "download-progress", (progress) => {
      assign({
        state: "downloading",
        progressPercent: normalizePercent(progress && progress.percent),
      });
    });
    attachUpdaterListener(autoUpdater, "update-downloaded", (info) => {
      assign({
        state: "downloaded",
        updateAvailable: true,
        downloaded: true,
        latestVersion: updateVersionFromInfo(info, state.latestVersion),
        progressPercent: 100,
        error: "",
        message: "",
      });
    });
    attachUpdaterListener(autoUpdater, "error", (error) => {
      assign({
        state: "error",
        error: safeMessage(error),
        message: safeMessage(error),
      });
    });
  }

  return {
    status: snapshot,
    checkForUpdates,
    installUpdate,
    onStatusChange(listener) {
      if (typeof listener !== "function") {
        return () => {};
      }
      changeListeners.add(listener);
      return () => changeListeners.delete(listener);
    },
  };
}

module.exports = {
  SUPPORTED_PLATFORMS,
  createAutoUpdateController,
};
