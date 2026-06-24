import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { createAutoUpdateController } = require("../lib/auto-update.cjs");

class FakeUpdater extends EventEmitter {
  constructor() {
    super();
    this.checks = 0;
    this.downloads = 0;
    this.installs = [];
  }

  async checkForUpdates() {
    this.checks += 1;
    this.emit("checking-for-update");
    this.emit("update-available", { version: "0.213.0" });
    return { updateInfo: { version: "0.213.0" } };
  }

  async downloadUpdate() {
    this.downloads += 1;
    this.emit("download-progress", { percent: 45.5 });
    this.emit("update-downloaded", { version: "0.213.0" });
    return ["formal-ai-desktop"];
  }

  quitAndInstall(isSilent, isForceRunAfter) {
    this.installs.push({ isSilent, isForceRunAfter });
  }
}

function packagedApp(version = "0.212.0") {
  return {
    isPackaged: true,
    getVersion: () => version,
  };
}

test("auto-update controller exposes packaged app version and disables autoDownload", () => {
  const updater = new FakeUpdater();
  const controller = createAutoUpdateController({
    app: packagedApp(),
    autoUpdater: updater,
    platform: "darwin",
  });

  assert.equal(updater.autoDownload, false);
  assert.equal(updater.autoInstallOnAppQuit, true);
  assert.deepEqual(controller.status(), {
    supported: true,
    enabled: true,
    platform: "darwin",
    currentVersion: "0.212.0",
    state: "idle",
    updateAvailable: false,
    downloaded: false,
    latestVersion: "",
    progressPercent: 0,
    checkedAt: "",
    error: "",
    message: "",
  });
});

test("checkForUpdates records an available update for the renderer notification", async () => {
  const updater = new FakeUpdater();
  const seen = [];
  const controller = createAutoUpdateController({
    app: packagedApp(),
    autoUpdater: updater,
    platform: "linux",
    clock: () => new Date("2026-06-20T00:00:00.000Z"),
  });
  controller.onStatusChange((status) => seen.push(status.state));

  const status = await controller.checkForUpdates();

  assert.equal(updater.checks, 1);
  assert.equal(status.state, "available");
  assert.equal(status.updateAvailable, true);
  assert.equal(status.latestVersion, "0.213.0");
  assert.equal(status.checkedAt, "2026-06-20T00:00:00.000Z");
  assert.ok(seen.includes("checking"));
  assert.ok(seen.includes("available"));
});

test("installUpdate downloads the available release and calls quitAndInstall", async () => {
  const updater = new FakeUpdater();
  const controller = createAutoUpdateController({
    app: packagedApp(),
    autoUpdater: updater,
    platform: "win32",
  });
  await controller.checkForUpdates();

  const status = await controller.installUpdate();

  assert.equal(updater.downloads, 1);
  assert.deepEqual(updater.installs, [{ isSilent: false, isForceRunAfter: true }]);
  assert.equal(status.state, "installing");
  assert.equal(status.downloaded, true);
  assert.equal(status.progressPercent, 100);
});

test("auto-update is disabled for unpackaged development runs", async () => {
  const updater = new FakeUpdater();
  const controller = createAutoUpdateController({
    app: { isPackaged: false, getVersion: () => "0.212.0" },
    autoUpdater: updater,
    platform: "linux",
  });

  const status = await controller.checkForUpdates();

  assert.equal(updater.checks, 0);
  assert.equal(status.supported, false);
  assert.equal(status.enabled, false);
  assert.equal(status.state, "disabled");
  assert.match(status.message, /packaged desktop builds/);
});
