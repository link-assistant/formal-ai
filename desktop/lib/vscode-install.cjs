"use strict";

// Desktop one-click VS Code extension install (issue #554, R2).
//
// The desktop app can install the formal-ai VS Code extension with a single
// click. Because the extension is NOT published to the Marketplace yet (issue
// #554, R3), the only install path is the signed `.vsix` attached to each GitHub
// release: this module (1) detects an available VS Code-family CLI, (2) resolves
// the latest release's `formal-ai-vscode-*.vsix` asset, (3) downloads it, and
// (4) runs `<cli> --install-extension <vsix> --force`. It mirrors the manual
// `scripts/install.sh vscode` flow so a one-click install and a curl|sh install
// land the exact same artifact.
//
// Every side-effecting dependency (process spawning, network fetch, file
// download, temp dir, clock) is injected so the whole contract is unit-testable
// without VS Code, a network, or the filesystem — matching the rest of
// desktop/lib (service-control.cjs, docker-detect.cjs, auto-update.cjs).

const REPO = "link-assistant/formal-ai";
const LATEST_RELEASE_API = `https://api.github.com/repos/${REPO}/releases/latest`;
const RELEASES_URL = `https://github.com/${REPO}/releases`;

// The release asset the desktop installer pulls. Matches the filename
// `vsce package` produces (formal-ai-vscode-<version>.vsix) and the identical
// pattern used by scripts/install.sh / scripts/install.ps1 / the /download
// verifier, so all install paths agree on one artifact.
const VSIX_ASSET_PATTERN = /^formal-ai-vscode-.*\.vsix$/;

// VS Code-family CLIs we know how to drive, in priority order. They all accept
// the same `--version` probe and `--install-extension <path> --force` contract,
// so a user on VS Code, the Insiders build, open-source VSCodium, or a fork
// (Cursor/Windsurf) gets the same one-click install. A FORMAL_AI_VSCODE_BIN
// override is honored first for unusual install locations.
const CLI_CANDIDATES = ["code", "code-insiders", "codium", "vscodium", "cursor", "windsurf"];

function resolveCliCandidates(env = {}) {
  const candidates = [];
  const override = String(env.FORMAL_AI_VSCODE_BIN || "").trim();
  if (override) {
    candidates.push(override);
  }
  for (const candidate of CLI_CANDIDATES) {
    if (!candidates.includes(candidate)) {
      candidates.push(candidate);
    }
  }
  return candidates;
}

// Pull the matching `.vsix` asset out of a GitHub release JSON blob. Returns the
// browser download URL, the asset filename, and the release tag, or null when the
// release carries no built extension yet.
function selectVsixAsset(release) {
  if (!release || typeof release !== "object" || !Array.isArray(release.assets)) {
    return null;
  }
  const asset = release.assets.find(
    (entry) => entry && typeof entry.name === "string" && VSIX_ASSET_PATTERN.test(entry.name),
  );
  if (!asset) {
    return null;
  }
  const url = String(asset.browser_download_url || "").trim();
  if (!url) {
    return null;
  }
  return {
    name: asset.name,
    url,
    tag: typeof release.tag_name === "string" ? release.tag_name : "",
  };
}

function normalizeResult(result) {
  if (!result || typeof result !== "object") {
    return { code: 1, stdout: "", stderr: "" };
  }
  return {
    code: typeof result.code === "number" ? result.code : result.code ? 1 : 0,
    stdout: String(result.stdout || ""),
    stderr: String(result.stderr || ""),
  };
}

function createVsCodeInstaller(options = {}) {
  const env = options.env || {};
  const runCommand = options.runCommand;
  const fetchJson = options.fetchJson;
  const downloadFile = options.downloadFile;
  const joinPath =
    typeof options.joinPath === "function" ? options.joinPath : (dir, file) => `${dir}/${file}`;
  const tmpDir = typeof options.tmpDir === "function" ? options.tmpDir : () => "/tmp";
  const log = typeof options.log === "function" ? options.log : () => {};

  if (typeof runCommand !== "function") {
    throw new Error("createVsCodeInstaller requires a runCommand(cmd, args) function");
  }
  if (typeof fetchJson !== "function") {
    throw new Error("createVsCodeInstaller requires a fetchJson(url) function");
  }
  if (typeof downloadFile !== "function") {
    throw new Error("createVsCodeInstaller requires a downloadFile(url, dest) function");
  }

  // Probe each candidate with `--version`; the first that exits 0 with a
  // non-empty stdout is the CLI we drive. The first stdout line is VS Code's
  // semver, surfaced to the UI so the user sees which editor was targeted.
  async function detectCli() {
    for (const candidate of resolveCliCandidates(env)) {
      let result;
      try {
        result = normalizeResult(await runCommand(candidate, ["--version"]));
      } catch (error) {
        log(`vscode cli probe failed for ${candidate}:`, error && error.message ? error.message : error);
        continue;
      }
      const version = result.stdout.split(/\r?\n/)[0].trim();
      if (result.code === 0 && version) {
        log(`vscode cli detected: ${candidate} (${version})`);
        return { cli: candidate, version };
      }
    }
    return null;
  }

  async function resolveAsset() {
    const release = await fetchJson(LATEST_RELEASE_API);
    return selectVsixAsset(release);
  }

  // The one-click flow: detect → resolve asset → download → install. `options`
  // may carry a `vsixPath` to install a pre-downloaded/bundled artifact without
  // touching the network (used by dev builds and tests).
  async function install(installOptions = {}) {
    const detected = await detectCli();
    if (!detected) {
      return {
        ok: false,
        state: "no-vscode-cli",
        reason:
          "No VS Code command-line tool was found. Install VS Code and enable the 'code' command in PATH, then try again.",
        releasesUrl: RELEASES_URL,
      };
    }

    let vsixPath = String(installOptions.vsixPath || "").trim();
    let assetName = "";
    let tag = "";
    if (!vsixPath) {
      let asset;
      try {
        asset = await resolveAsset();
      } catch (error) {
        return {
          ok: false,
          state: "release-lookup-failed",
          cli: detected.cli,
          cliVersion: detected.version,
          reason: error && error.message ? error.message : String(error),
          releasesUrl: RELEASES_URL,
        };
      }
      if (!asset) {
        return {
          ok: false,
          state: "no-release-asset",
          cli: detected.cli,
          cliVersion: detected.version,
          reason: `No formal-ai .vsix has been published to the latest release yet. See ${RELEASES_URL}/latest`,
          releasesUrl: RELEASES_URL,
        };
      }
      assetName = asset.name;
      tag = asset.tag;
      vsixPath = joinPath(tmpDir(), asset.name);
      try {
        await downloadFile(asset.url, vsixPath);
      } catch (error) {
        return {
          ok: false,
          state: "download-failed",
          cli: detected.cli,
          cliVersion: detected.version,
          asset: asset.name,
          tag,
          reason: error && error.message ? error.message : String(error),
          releasesUrl: RELEASES_URL,
        };
      }
    }

    let installResult;
    try {
      installResult = normalizeResult(
        await runCommand(detected.cli, ["--install-extension", vsixPath, "--force"]),
      );
    } catch (error) {
      return {
        ok: false,
        state: "install-failed",
        cli: detected.cli,
        cliVersion: detected.version,
        asset: assetName,
        tag,
        vsix: vsixPath,
        reason: error && error.message ? error.message : String(error),
        releasesUrl: RELEASES_URL,
      };
    }

    if (installResult.code !== 0) {
      return {
        ok: false,
        state: "install-failed",
        cli: detected.cli,
        cliVersion: detected.version,
        asset: assetName,
        tag,
        vsix: vsixPath,
        reason: (installResult.stderr || installResult.stdout || "code --install-extension failed").trim(),
        releasesUrl: RELEASES_URL,
      };
    }

    return {
      ok: true,
      state: "installed",
      cli: detected.cli,
      cliVersion: detected.version,
      asset: assetName,
      tag,
      vsix: vsixPath,
    };
  }

  return {
    detectCli,
    resolveAsset,
    install,
    candidates: () => resolveCliCandidates(env),
  };
}

module.exports = {
  REPO,
  LATEST_RELEASE_API,
  RELEASES_URL,
  VSIX_ASSET_PATTERN,
  CLI_CANDIDATES,
  resolveCliCandidates,
  selectVsixAsset,
  createVsCodeInstaller,
};
