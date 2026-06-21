<#
.SYNOPSIS
  formal-ai universal installer for Windows (issue #554).

.DESCRIPTION
  One script installs every formal-ai interface from the GitHub Releases the
  project already publishes:

    desktop   the Electron desktop app (downloads the matching release asset)
    vscode    the VS Code extension (downloads the .vsix, runs `code --install-extension`)
    cli       the `formal-ai` command-line tool (via `cargo install formal-ai`)
    all       desktop + vscode + cli (best effort; skips what the host can't do)

  Run directly:
    powershell -ExecutionPolicy Bypass -File scripts\install.ps1 -Target vscode

  Run from the web (the only supported VS Code install method until the
  extension is on the Marketplace, issue #554 R3). Because piping into iex
  cannot forward -Target, configure the target with an environment variable:
    $env:FORMAL_AI_INSTALL_TARGET = 'vscode'
    irm https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.ps1 | iex

.PARAMETER Target
  desktop | vscode | cli | all. Defaults to $env:FORMAL_AI_INSTALL_TARGET, then 'desktop'.

.NOTES
  Environment variables (so the irm|iex form needs no parameters):
    FORMAL_AI_INSTALL_TARGET    desktop | vscode | cli | all (default: desktop)
    FORMAL_AI_INSTALL_VERSION   pin a release tag, e.g. v0.215.0 (default: latest)
    FORMAL_AI_INSTALL_DIR       directory for downloaded desktop assets
    FORMAL_AI_SKIP_VERIFY=1     skip the SHA-256 checksum verification
#>
[CmdletBinding()]
param(
  [ValidateSet('desktop', 'vscode', 'cli', 'all', 'help')]
  [string] $Target = $(if ($env:FORMAL_AI_INSTALL_TARGET) { $env:FORMAL_AI_INSTALL_TARGET } else { 'desktop' })
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# Negotiate TLS 1.2+; older Windows PowerShell defaults can fail the GitHub TLS handshake.
try { [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 } catch {}

$Repo = 'link-assistant/formal-ai'
$ReleasesUrl = "https://github.com/$Repo/releases"

function Write-Log { param([string] $Message) Write-Host "formal-ai: $Message" }
function Write-Err { param([string] $Message) Write-Host "formal-ai: error: $Message" -ForegroundColor Red }

function Show-Usage {
  Write-Host @'
formal-ai universal installer (Windows)

Usage: install.ps1 -Target <desktop|vscode|cli|all>

Targets:
  desktop   Download the desktop app installer for this architecture.
  vscode    Download the .vsix and install it with `code --install-extension`.
  cli       Install the `formal-ai` CLI with `cargo install formal-ai`.
  all       Install everything this machine can support (best effort).

Environment:
  FORMAL_AI_INSTALL_TARGET    target when none is passed
  FORMAL_AI_INSTALL_VERSION   pin a release tag (default: latest)
  FORMAL_AI_INSTALL_DIR       directory for downloaded desktop assets
  FORMAL_AI_SKIP_VERIFY=1     skip the SHA-256 checksum verification
'@
}

function Get-Arch {
  # PROCESSOR_ARCHITECTURE is the most reliable signal in both PS editions.
  $a = $env:PROCESSOR_ARCHITECTURE
  if (-not $a) { $a = '' }
  switch -Wildcard ($a.ToUpperInvariant()) {
    'ARM64' { return 'arm64' }
    'AMD64' { return 'x64' }
    'X86'   { return 'x64' }  # 32-bit shell on 64-bit Windows: still ship the x64 build.
    default { return 'x64' }
  }
}

function Get-ReleaseJson {
  if ($env:FORMAL_AI_INSTALL_VERSION) {
    $url = "https://api.github.com/repos/$Repo/releases/tags/$($env:FORMAL_AI_INSTALL_VERSION)"
  }
  else {
    $url = "https://api.github.com/repos/$Repo/releases/latest"
  }
  Write-Log "resolving $(if ($env:FORMAL_AI_INSTALL_VERSION) { $env:FORMAL_AI_INSTALL_VERSION } else { 'latest' }) release of $Repo"
  return Invoke-RestMethod -Uri $url -Headers @{ 'User-Agent' = 'formal-ai-installer' }
}

function Get-AssetUrl {
  param($Release, [string] $Pattern)
  $asset = $Release.assets | Where-Object { $_.name -match $Pattern } | Select-Object -First 1
  if ($asset) { return $asset.browser_download_url }
  return $null
}

function Get-ReleaseVersion {
  param($Release)
  if ($Release.tag_name -match '([0-9][0-9.]*([-+][0-9A-Za-z.-]+)?)') { return $Matches[1] }
  return $null
}

function Resolve-InstallDir {
  if ($env:FORMAL_AI_INSTALL_DIR) { return $env:FORMAL_AI_INSTALL_DIR }
  $downloads = Join-Path $env:USERPROFILE 'Downloads'
  if (Test-Path $downloads) { return $downloads }
  return (Get-Location).Path
}

function Test-Checksum {
  param([string] $File, $Release)
  if ($env:FORMAL_AI_SKIP_VERIFY -eq '1') { Write-Log 'skipping checksum verification'; return }

  $sumsUrl = Get-AssetUrl -Release $Release -Pattern 'SHA256SUMS\.txt$'
  if (-not $sumsUrl) { Write-Log 'no SHA256SUMS.txt in release; skipping verification'; return }

  try { $sums = Invoke-WebRequest -Uri $sumsUrl -UseBasicParsing -Headers @{ 'User-Agent' = 'formal-ai-installer' } }
  catch { Write-Log 'could not download SHA256SUMS.txt; skipping verification'; return }

  $base = Split-Path $File -Leaf
  $expected = $null
  foreach ($line in ($sums.Content -split "`n")) {
    if ($line -match "^([a-fA-F0-9]{64})\s+\*?$([regex]::Escape($base))\s*$") { $expected = $Matches[1]; break }
  }
  if (-not $expected) { Write-Log "no checksum line for $base; skipping verification"; return }

  $actual = (Get-FileHash -Path $File -Algorithm SHA256).Hash
  if ($actual -ieq $expected) { Write-Log "checksum OK for $base" }
  else { throw "checksum MISMATCH for $base (expected $expected, got $actual)" }
}

function Install-Desktop {
  param($Release)
  $arch = Get-Arch
  # Prefer the installer EXE; the project also ships a portable build.
  $pattern = "formal-ai-desktop-windows-installer-$arch-[0-9].*\.exe$"
  $url = Get-AssetUrl -Release $Release -Pattern $pattern
  if (-not $url) {
    $url = Get-AssetUrl -Release $Release -Pattern "formal-ai-desktop-windows-portable-$arch-[0-9].*\.exe$"
  }
  if (-not $url) { throw "no Windows desktop asset for $arch in the release. See $ReleasesUrl/latest" }

  $dir = Resolve-InstallDir
  if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
  $name = Split-Path $url -Leaf
  $dest = Join-Path $dir $name
  Write-Log "downloading $name -> $dir"
  Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing -Headers @{ 'User-Agent' = 'formal-ai-installer' }
  Test-Checksum -File $dest -Release $Release
  Write-Log "desktop installer saved to $dest"
  Write-Log 'run it to complete setup.'
}

function Install-VsCode {
  param($Release)
  $version = Get-ReleaseVersion -Release $Release
  $url = Get-AssetUrl -Release $Release -Pattern 'formal-ai-vscode-.*\.vsix$'
  if (-not $url) { throw "no .vsix in the release yet. Build one with 'npm run vscode:package' or see $ReleasesUrl/latest" }

  $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("formal-ai-vsix-" + [System.Guid]::NewGuid().ToString('N'))
  New-Item -ItemType Directory -Path $tmp -Force | Out-Null
  $name = Split-Path $url -Leaf
  $dest = Join-Path $tmp $name
  Write-Log "downloading $name"
  Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing -Headers @{ 'User-Agent' = 'formal-ai-installer' }
  Test-Checksum -File $dest -Release $Release

  $codeCli = $null
  foreach ($candidate in @('code.cmd', 'code', 'code-insiders.cmd', 'code-insiders', 'codium.cmd', 'codium')) {
    if (Get-Command $candidate -ErrorAction SilentlyContinue) { $codeCli = $candidate; break }
  }

  if ($codeCli) {
    Write-Log "installing the extension with '$codeCli --install-extension'"
    & $codeCli --install-extension $dest
    Write-Log "VS Code extension installed$(if ($version) { " (v$version)" }). Reload VS Code to activate it."
  }
  else {
    Write-Log "the 'code' CLI was not found on PATH."
    Write-Log "the .vsix is saved at: $dest"
    Write-Log "install it from VS Code: Extensions view -> ... menu -> 'Install from VSIX...'"
    Write-Log "or enable the CLI: VS Code Command Palette -> 'Shell Command: Install code command in PATH'."
  }
}

function Install-Cli {
  if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Log "installing the formal-ai CLI with 'cargo install formal-ai'"
    if ($env:FORMAL_AI_INSTALL_VERSION) {
      $ver = $env:FORMAL_AI_INSTALL_VERSION -replace '^v', ''
      & cargo install formal-ai --version $ver
    }
    else {
      & cargo install formal-ai
    }
    if ($LASTEXITCODE -ne 0) { throw 'cargo install failed' }
    Write-Log 'CLI installed. Try: formal-ai --help'
  }
  else {
    throw 'cargo is required to install the CLI. Install Rust from https://rustup.rs then re-run.'
  }
}

function Invoke-Main {
  if ($Target -eq 'help') { Show-Usage; return }

  $release = Get-ReleaseJson

  switch ($Target) {
    'desktop' { Install-Desktop -Release $release }
    'vscode'  { Install-VsCode -Release $release }
    'cli'     { Install-Cli }
    'all' {
      # Best effort: never abort the whole run because one optional interface is
      # missing its toolchain.
      try { Install-Desktop -Release $release } catch { Write-Err "desktop step did not complete: $_" }
      try { Install-VsCode -Release $release } catch { Write-Err "vscode step did not complete: $_" }
      try { Install-Cli } catch { Write-Err "cli step did not complete: $_" }
    }
  }

  Write-Log 'done.'
}

Invoke-Main
