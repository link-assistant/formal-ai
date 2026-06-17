'use strict';

const { signAsync } = require('@electron/osx-sign');

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
};
