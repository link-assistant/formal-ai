'use strict';

const { signAsync } = require('@electron/osx-sign');
const { spawnSync } = require('node:child_process');
const path = require('node:path');

function isDebugEnabled() {
  return process.env.FORMAL_AI_MACOS_SIGN_DEBUG === '1';
}

// Diagnostics go to stderr, never stdout. electron-builder's own logger and
// electron-osx-sign's `debug` output both use stderr; interleaving there keeps
// the sign trace readable in GitHub Actions logs, and stderr is the stream that
// survives when electron-builder aborts the CLI on a signing error.
function log(message) {
  process.stderr.write(`[adhoc-sign-mac] ${message}\n`);
}

function debugLog(message) {
  if (isDebugEnabled()) {
    log(message);
  }
}

function findAppPath(signOptions) {
  debugLog(`signOptions keys: ${Object.keys(signOptions).sort().join(', ')}`);
  debugLog(`cwd: ${process.cwd()}`);
  for (const field of ['app', 'appPath', 'path']) {
    debugLog(`candidate ${field}: ${JSON.stringify(signOptions[field] ?? null)}`);
  }
  for (const field of ['app', 'appPath', 'path']) {
    if (typeof signOptions[field] === 'string' && signOptions[field].endsWith('.app')) {
      return signOptions[field];
    }
  }

  throw new Error(
    `Could not determine macOS app bundle path from signing options: ${Object.keys(signOptions)
      .sort()
      .join(', ')}`,
  );
}

function isBundledBrowserRuntime(filePath, appPath) {
  const browserRuntime = path.join(
    path.resolve(appPath),
    'Contents',
    'Resources',
    'browser-runtime',
  );
  const relative = path.relative(browserRuntime, path.resolve(filePath));
  const bundled =
    relative === '' ||
    (!relative.startsWith(`..${path.sep}`) && relative !== '..' && !path.isAbsolute(relative));
  debugLog(
    `ignore ${bundled ? 'SKIP' : 'sign'} root=${browserRuntime} relative=${relative} path=${filePath}`,
  );
  return bundled;
}

// Issue #808: @electron/osx-sign's `validateOptsIgnore()` is
//
//   function validateOptsIgnore (ignore) {
//     if (ignore && !(ignore instanceof Array)) { return [ignore] }
//   }
//
// -- it has no `return ignore` for the array case, so passing an **array**
// silently yields `undefined` and every ignore rule is discarded. That is why
// run 29731405782 signed `Contents/Resources/browser-runtime/...` anyway and
// died with "unsealed contents present in the root directory of an embedded
// framework", and why electron-builder's own `mac.signIgnore` (which it forwards
// as an array) never took effect either. We therefore hand the library a single
// predicate function, which it wraps into `[fn]` itself.
function signingIgnoreRules(signOptions) {
  const upstreamIgnore = signOptions.ignore
    ? Array.isArray(signOptions.ignore)
      ? signOptions.ignore
      : [signOptions.ignore]
    : [];
  const appPath = findAppPath(signOptions);

  const matchers = [
    ...upstreamIgnore,
    (filePath) => isBundledBrowserRuntime(filePath, appPath),
  ];

  return (filePath) =>
    matchers.some((matcher) =>
      typeof matcher === 'function' ? matcher(filePath) : Boolean(filePath.match(matcher)),
    );
}

function resolvePath(value) {
  return typeof value === 'string' && value.length > 0 ? path.resolve(value) : undefined;
}

function normalizeSignatureFlags(signatureFlags) {
  if (Array.isArray(signatureFlags)) {
    return signatureFlags;
  }
  if (typeof signatureFlags === 'string') {
    return signatureFlags
      .split(',')
      .map((flag) => flag.trim())
      .filter(Boolean);
  }
  return [];
}

function appendSignOptions(args, appFileOptions) {
  const requirements = appFileOptions.requirements;
  if (typeof requirements === 'string' && requirements.length > 0) {
    if (requirements.startsWith('=')) {
      args.push(`-r${requirements}`);
    } else {
      args.push('--requirements', requirements);
    }
  }

  const optionFlags = normalizeSignatureFlags(appFileOptions.signatureFlags);
  if (appFileOptions.hardenedRuntime !== false) {
    optionFlags.push('runtime');
  }
  if (optionFlags.length > 0) {
    args.push('--options', [...new Set(optionFlags)].join(','));
  }

  if (Array.isArray(appFileOptions.additionalArguments)) {
    args.push(...appFileOptions.additionalArguments);
  }

  const entitlements = resolvePath(appFileOptions.entitlements);
  if (entitlements) {
    args.push('--entitlements', entitlements);
  }
}

function runCodesign(args, action) {
  const result = spawnSync('/usr/bin/codesign', args, { encoding: 'utf8' });

  if (result.error) {
    throw result.error;
  }

  if (isDebugEnabled() || result.status !== 0) {
    if (result.stdout) {
      process.stdout.write(result.stdout);
    }
    if (result.stderr) {
      process.stderr.write(result.stderr);
    }
  }

  if (result.status !== 0) {
    throw new Error(`codesign failed to ${action} (exit ${result.status})`);
  }
}

module.exports = async function adhocSignMac(signOptions) {
  if (process.env.MACOS_ADHOC_SIGN !== '1') {
    throw new Error('Ad-hoc macOS signing must be enabled explicitly.');
  }

  // One unconditional line. Run 29724500254 failed inside electron-osx-sign
  // while FORMAL_AI_MACOS_SIGN_DEBUG=1 was set, yet the log contained no
  // `[adhoc-sign-mac]` output at all, so we could not tell whether this hook
  // ever ran. This banner settles that question on every future run; the
  // per-file trace stays behind the env var and off by default.
  log(`hook entered (debug=${isDebugEnabled() ? 'on' : 'off'})`);

  const upstreamOptionsForFile = signOptions.optionsForFile;

  await signAsync({
    ...signOptions,
    // Playwright's Chrome for Testing bundle is separately signed upstream.
    // Treat it as an opaque resource: signing its framework aliases and files
    // as Electron children breaks the framework seal. The final app signing
    // below still includes the whole runtime in the app's resource envelope.
    ignore: signingIgnoreRules(signOptions),
    identity: '-',
    identityValidation: false,
    provisioningProfile: undefined,
    timestamp: 'none',
    optionsForFile(filePath) {
      const fileOptions = upstreamOptionsForFile
        ? upstreamOptionsForFile(filePath)
        : {};

      return {
        ...fileOptions,
        timestamp: 'none',
      };
    },
  });

  const appPath = findAppPath(signOptions);
  const appFileOptions = upstreamOptionsForFile ? upstreamOptionsForFile(appPath) || {} : {};
  const sealArgs = ['--force', '--timestamp=none', '--sign', '-'];
  appendSignOptions(sealArgs, appFileOptions);
  sealArgs.push(appPath);

  // electron-builder 26 can leave ad-hoc bundles without a resource seal.
  // Re-sign the final bundle so CI verifies the same CodeResources envelope
  // that Gatekeeper checks after users copy the app out of the DMG.
  runCodesign(sealArgs, 'seal the ad-hoc macOS app resource envelope');
  runCodesign(
    ['--verify', '--deep', '--strict', '--verbose=2', appPath],
    'verify the ad-hoc macOS app signature',
  );
};

module.exports.isBundledBrowserRuntime = isBundledBrowserRuntime;
module.exports.signingIgnoreRules = signingIgnoreRules;
