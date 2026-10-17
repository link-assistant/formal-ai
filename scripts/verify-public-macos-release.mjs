// Execute only in hosted macOS CI, using already authenticated public assets.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawn, spawnSync} from 'node:child_process';
import {pathToFileURL, fileURLToPath} from 'node:url';
import {setTimeout as delay} from 'node:timers/promises';
import {validateSigningEvidence, MAC_PACKAGE_TARGETS} from './native-release-evidence.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
export function inputs(env, platform = process.platform, architecture = process.arch) {
  assert.equal(env.CI, 'true', 'hosted CI required');
  assert.equal(env.GITHUB_ACTIONS, 'true', 'GitHub hosted execution required');
  assert.equal(platform, 'darwin', 'macOS required');
  assert.ok(['arm64', 'x64'].includes(env.MACOS_ARCH));
  assert.equal(architecture, env.MACOS_ARCH, 'runner architecture mismatch');
  assert.match(env.RELEASE_VERSION, /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/u);
  assert.equal(env.RELEASE_TAG, 'v' + env.RELEASE_VERSION);
  for (const name of ['RELEASE_SOURCE_SHA', 'RELEASE_SOURCE_TREE']) {
    assert.match(env[name], /^[a-f0-9]{40}$/u);
  }
  assert.ok(path.isAbsolute(env.RELEASE_DOWNLOAD_DIRECTORY));
  return {directory: env.RELEASE_DOWNLOAD_DIRECTORY, version: env.RELEASE_VERSION,
    source: env.RELEASE_SOURCE_SHA, tree: env.RELEASE_SOURCE_TREE,
    label: 'macos-' + env.MACOS_ARCH};
}

export function digestFile(file) {
  const fd = fs.openSync(file, 'r'), buffer = Buffer.alloc(1024 * 1024);
  const hash = createHash('sha256');
  try {
    let count;
    while ((count = fs.readSync(fd, buffer)) > 0) hash.update(buffer.subarray(0, count));
    return hash.digest('hex');
  } finally { fs.closeSync(fd); }
}

export function validateAppObservation(actual, expected) {
  assert.equal(actual.bundleId, 'ai.formal.desktop');
  assert.equal(actual.version, expected.version);
  assert.ok(actual.envelope, 'signed resource envelope missing');
  assert.equal(actual.signatureStatus, 0, 'deep strict signature verification failed');
  const adhoc = /^Signature=adhoc$/mu.test(actual.display);
  const signed = /^Authority=Developer ID Application:/mu.test(actual.display);
  assert.notEqual(adhoc, signed, 'ambiguous signing mode');
  assert.equal(adhoc ? 'adhoc' : 'signed', expected.mode);
  assert.equal(actual.nativeSha256, expected.nativeSha256);
  assert.equal(actual.nativeBytes, expected.nativeBytes);
  assert.equal(actual.nativeVersion.trim(), 'formal-ai ' + expected.version);
}

export const rendererReadinessExpression = `(async()=>{
  const root = document.getElementById('root');
  const composer = root?.querySelector('[data-testid="chat-composer-input"]');
  const submit = root?.querySelector('[data-testid="chat-composer-submit"]');
  return {readyState:document.readyState, pathname:location.pathname,
    desktop:new URLSearchParams(location.search).get('desktop'),
    ui:{rootChildren:root?.childElementCount ?? 0,
      rootTextLength:root?.innerText?.trim().length ?? 0,
      composerCount:root?.querySelectorAll('textarea[data-testid="chat-composer-input"]').length ?? 0,
      composerPlaceholderLength:composer?.getAttribute('placeholder')?.trim().length ?? 0,
      submitCount:root?.querySelectorAll('button[data-testid="chat-composer-submit"]').length ?? 0,
      submitTextLength:submit?.innerText?.trim().length ?? 0,
      messageListCount:root?.querySelectorAll('[data-testid="message-list"]').length ?? 0},
    status:await window.FormalAiDesktop.getStatus()};
})()`;

export function validateRendererProbe(value, version) {
  assert.equal(value.readyState, 'complete');
  assert.equal(value.pathname, '/app/index.html');
  assert.equal(value.desktop, '1');
  assert.equal(value.status?.appVersion, version);
  const diagnostic = 'actual rendered desktop UI: ' + JSON.stringify(value.ui).slice(0, 1024);
  assert.ok(value.ui?.rootChildren > 0 && value.ui.rootTextLength > 0, diagnostic);
  assert.equal(value.ui.composerCount, 1, diagnostic);
  assert.ok(value.ui.composerPlaceholderLength > 0, diagnostic);
  assert.equal(value.ui.submitCount, 1, diagnostic);
  assert.ok(value.ui.submitTextLength > 0, diagnostic);
  assert.equal(value.ui.messageListCount, 1, diagnostic);
}

function command(file, args, options = {}) {
  const result = spawnSync(file, args, {encoding: 'utf8', timeout: 30000,
    maxBuffer: 1024 * 1024, ...options});
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${path.basename(file)} failed: ${result.stderr}`);
  return result.stdout;
}

async function rendererProbe(socketUrl, version, deadline) {
  assert.equal(typeof WebSocket, 'function', 'Node 22+ built-in WebSocket required');
  const socket = new WebSocket(socketUrl);
  try {
    const response = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(Error('renderer probe deadline')),
        Math.max(1, deadline - Date.now()));
      socket.addEventListener('error', () => {
        clearTimeout(timer); reject(Error('renderer debugger connection failed'));
      }, {once: true});
      socket.addEventListener('open', () => socket.send(JSON.stringify({id: 1,
        method: 'Runtime.evaluate', params: {awaitPromise: true, returnByValue: true,
          expression: rendererReadinessExpression}})), {once: true});
      socket.addEventListener('message', event => {
        const message = JSON.parse(String(event.data));
        if (message.id !== 1) return;
        clearTimeout(timer);
        if (message.error || message.result?.exceptionDetails) {
          const detail = message.error?.message ??
            message.result?.exceptionDetails?.exception?.description ??
            message.result?.exceptionDetails?.text ?? 'unknown CDP exception';
          reject(Error('actual renderer bridge failed: ' + String(detail).slice(0, 1024))); return;
        }
        resolve(message.result?.result?.value);
      });
    });
    validateRendererProbe(response, version);
    return {readyState: response.readyState, pathname: response.pathname,
      desktop: response.desktop, appVersion: response.status.appVersion, ui: response.ui};
  } finally { socket.close(); }
}

async function startup(binary, directory, version) {
  const output = fs.openSync(path.join(directory, 'electron.stdout.log'), 'wx');
  const errors = fs.openSync(path.join(directory, 'electron.stderr.log'), 'wx');
  const profile = path.join(directory, 'profile');
  fs.mkdirSync(profile);
  const child = spawn(binary, ['--remote-debugging-port=0',
    '--remote-debugging-address=127.0.0.1', '--user-data-dir=' + profile],
  {stdio: ['ignore', output, errors], env: process.env, detached: true});
  fs.closeSync(output); fs.closeSync(errors);
  let exited = false, spawnError = null;
  const reaped = new Promise(resolve => {
    child.once('error', error => { spawnError = error; exited = true; resolve(); });
    child.once('exit', () => { exited = true; resolve(); });
  });
  const deadline = Date.now() + 45000;
  let lastError = null;
  const stop = signal => {
    if (!exited && child.pid) {
      try { process.kill(-child.pid, signal); } catch (error) {
        if (error.code !== 'ESRCH') throw error;
      }
    }
  };
  try {
    while (Date.now() < deadline) {
      if (spawnError) throw spawnError;
      assert.ok(!exited, 'Electron exited before renderer readiness');
      const active = path.join(profile, 'DevToolsActivePort');
      if (fs.existsSync(active)) {
        const port = Number(fs.readFileSync(active, 'utf8').split('\n')[0]);
        assert.ok(Number.isInteger(port) && port > 0 && port < 65536);
        try {
        const response = await fetch(`http://127.0.0.1:${port}/json/list`,
          {signal: AbortSignal.timeout(Math.min(3000, deadline - Date.now()))});
        assert.ok(response.ok);
        const pages = await response.json();
        const page = pages.find(value => value.type === 'page' &&
          new URL(value.url).pathname === '/app/index.html');
        if (page) return await rendererProbe(page.webSocketDebuggerUrl, version, deadline);
        } catch (error) { lastError = String(error); }
      }
      await delay(200);
    }
    throw Error('Electron renderer deadline: ' + (lastError ?? 'no loaded desktop page'));
  } finally {
    stop('SIGTERM');
    await Promise.race([reaped, delay(2000)]);
    if (!exited) { stop('SIGKILL'); await Promise.race([reaped, delay(2000)]); }
    assert.ok(exited, 'Electron process did not terminate after bounded cleanup');
  }
}

function checksumMap(bytes) {
  const result = new Map();
  for (const line of bytes.trim().split(/\r?\n/u)) {
    const match = /^([a-f0-9]{64})  ([A-Za-z0-9_.-]+)$/u.exec(line);
    assert.ok(match && !result.has(match[2]), 'invalid or duplicate checksum');
    result.set(match[2], match[1]);
  }
  return result;
}

async function inspectApp(application, directory, config, signing) {
  const plist = path.join(application, 'Contents/Info.plist');
  const get = key => command('/usr/libexec/PlistBuddy', ['-c', 'Print :' + key, plist]).trim();
  const executable = get('CFBundleExecutable');
  assert.ok(executable && path.basename(executable) === executable);
  const native = path.join(application, 'Contents/Resources/bin/formal-ai');
  const display = spawnSync('/usr/bin/codesign', ['--display', '--verbose=2', application],
    {encoding: 'utf8', timeout: 30000, maxBuffer: 1024 * 1024});
  assert.equal(display.status, 0);
  const signature = command('/usr/bin/codesign',
    ['--verify', '--deep', '--strict', '--verbose=2', application]);
  const actual = {bundleId: get('CFBundleIdentifier'), version: get('CFBundleShortVersionString'),
    envelope: fs.existsSync(path.join(application, 'Contents/_CodeSignature/CodeResources')),
    signatureStatus: 0, signature, display: display.stdout + '\n' + display.stderr,
    nativeSha256: digestFile(native), nativeBytes: fs.statSync(native).size,
    nativeVersion: command(native, ['--version'])};
  validateAppObservation(actual, {version: config.version, mode: signing.mode,
    nativeSha256: signing.native_component.sha256, nativeBytes: signing.native_component.bytes});
  const gatekeeper = spawnSync('/usr/sbin/spctl', ['--assess', '--type', 'execute', '--verbose=4', application],
    {encoding: 'utf8', timeout: 30000, maxBuffer: 1024 * 1024});
  let stapler = null;
  if (signing.mode === 'signed') {
    assert.equal(gatekeeper.status, 0);
    stapler = command('/usr/bin/xcrun', ['stapler', 'validate', application]);
  }
  const quarantine = spawnSync('/usr/bin/xattr', ['-p', 'com.apple.quarantine', application],
    {encoding: 'utf8', timeout: 5000, maxBuffer: 16384});
  return {signature: actual, stapler, quarantine: {status: quarantine.status,
    value: quarantine.stdout?.trim() ?? '', modified: false}, gatekeeper: {status: gatekeeper.status, signal: gatekeeper.signal,
    stdout: gatekeeper.stdout, stderr: gatekeeper.stderr},
  startup: await startup(path.join(application, 'Contents/MacOS', executable), directory, config.version)};
}

export async function main() {
  const config = inputs(process.env), outcome = {schema: 'public-macos-download-check/v1',
    ...config, status: 'failed', packages: [],
    quarantineModified: false, signaturesModified: false,
    gatekeeperPolicy: 'Ad-hoc self-signing does not imply Developer ID trust or notarization; user approval may be required.',
    publicationAuthority: false};
  const evidence = fs.mkdtempSync(path.join(config.directory, 'macos-public-evidence-'));
  let mount = null;
  try {
    const sourceBytes = fs.readFileSync(path.join(config.directory,
      `formal-ai-native-source-${config.version}.json`));
    const source = JSON.parse(sourceBytes);
    assert.equal(source.source_commit, config.source); assert.equal(source.source_tree, config.tree);
    const signing = JSON.parse(fs.readFileSync(path.join(config.directory,
      `formal-ai-signing-${config.label}-${config.version}.json`)));
    const parent = fs.readFileSync(path.join(config.directory,
      `formal-ai-native-${MAC_PACKAGE_TARGETS[config.label]}-${config.version}.json`));
    validateSigningEvidence(signing, source, config.label,
      createHash('sha256').update(sourceBytes).digest('hex'), parent);
    const sums = checksumMap(fs.readFileSync(path.join(config.directory, 'SHA256SUMS.txt'), 'utf8'));
    for (const format of ['dmg', 'zip']) {
      const name = `formal-ai-desktop-${config.label}-${config.version}.${format}`;
      const archive = path.join(config.directory, name);
      assert.equal(digestFile(archive), sums.get(name), 'public archive checksum mismatch');
      if (format === 'dmg') assert.equal(sums.get(name), signing.package.sha256);
      const directory = path.join(evidence, format); fs.mkdirSync(directory);
      const application = path.join(directory, 'formal-ai Desktop.app');
      if (format === 'dmg') {
        mount = path.join(directory, 'mount'); fs.mkdirSync(mount);
        command('/usr/bin/hdiutil', ['attach', archive, '-mountpoint', mount, '-nobrowse', '-readonly']);
        command('/usr/bin/ditto', [path.join(mount, 'formal-ai Desktop.app'), application]);
        command('/usr/bin/hdiutil', ['detach', mount]); mount = null;
      } else {
        command('/usr/bin/python3', [path.join(here, 'verify-public-macos-zip.py'), archive]);
        command('/usr/bin/ditto', ['-x', '-k', archive, directory]);
      }
      outcome.packages.push({name, sha256: sums.get(name),
        observations: await inspectApp(application, directory, config, signing)});
    }
    outcome.status = 'passed';
  } catch (error) { outcome.error = String(error.stack ?? error); throw error; }
  finally {
    if (mount) spawnSync('/usr/bin/hdiutil', ['detach', mount], {timeout: 30000});
    fs.writeFileSync(path.join(evidence, 'outcome.json'), JSON.stringify(outcome, null, 2) + '\n');
    console.log('Public macOS download evidence: ' + evidence);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => { console.error(error); process.exitCode = 1; });
}
