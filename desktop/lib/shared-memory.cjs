"use strict";

const os = require("node:os");
const path = require("node:path");
const fs = require("node:fs");

const CONTAINER_MEMORY_DIRECTORY = "/root/.formal-ai";
const CONTAINER_MEMORY_PATH = `${CONTAINER_MEMORY_DIRECTORY}/memory.lino`;

function resolveSharedMemoryPath(env = process.env, options = {}) {
  const override = String(env.FORMAL_AI_MEMORY_PATH || "").trim();
  if (override) {
    return override;
  }
  const platform = options.platform || process.platform;
  const pathImpl = options.pathImpl || path;
  if (platform === "win32") {
    const appData = String(options.appData || env.APPDATA || env.LOCALAPPDATA || "").trim();
    const base = appData || String(options.homeDir || env.USERPROFILE || os.homedir() || ".").trim() || ".";
    return pathImpl.join(base, "formal-ai", "memory.lino");
  }
  const home = String(options.homeDir || env.HOME || os.homedir() || ".").trim() || ".";
  return pathImpl.join(home, ".formal-ai", "memory.lino");
}

function sharedMemoryDirectory(env = process.env, options = {}) {
  return (options.pathImpl || path).dirname(resolveSharedMemoryPath(env, options));
}

function dockerMemoryArgs(env = process.env, options = {}) {
  return [
    "-v",
    `${sharedMemoryDirectory(env, options)}:${CONTAINER_MEMORY_DIRECTORY}`,
    "-e",
    `FORMAL_AI_MEMORY_PATH=${CONTAINER_MEMORY_PATH}`,
  ];
}

function ensureSharedMemoryDirectory(env = process.env, options = {}) {
  const directory = sharedMemoryDirectory(env, options);
  const existed = fs.existsSync(directory);
  fs.mkdirSync(directory, { recursive: true, mode: 0o700 });
  if (!existed && process.platform !== "win32") {
    fs.chmodSync(directory, 0o700);
  }
  return directory;
}

module.exports = {
  CONTAINER_MEMORY_DIRECTORY,
  CONTAINER_MEMORY_PATH,
  resolveSharedMemoryPath,
  sharedMemoryDirectory,
  dockerMemoryArgs,
  ensureSharedMemoryDirectory,
};
