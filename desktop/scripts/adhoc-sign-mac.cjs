'use strict';

const { signAsync } = require('@electron/osx-sign');
const { spawnSync } = require('node:child_process');
const path = require('node:path');

function isDebugEnabled() {
  return process.env.FORMAL_AI_MACOS_SIGN_DEBUG === '1';
}

function findAppPath(signOptions) {
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

  const upstreamOptionsForFile = signOptions.optionsForFile;

  await signAsync({
    ...signOptions,
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
