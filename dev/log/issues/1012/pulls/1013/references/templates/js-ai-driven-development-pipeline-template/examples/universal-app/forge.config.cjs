'use strict';

module.exports = {
  packagerConfig: {
    asar: true,
    executableName: 'universal-example-app',
    name: 'Universal Example App',
  },
  rebuildConfig: {},
  makers: [
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin', 'linux', 'win32'],
    },
    {
      name: '@electron-forge/maker-squirrel',
      config: {
        name: 'universal_example_app',
      },
    },
    {
      name: '@electron-forge/maker-deb',
      config: {
        options: {
          maintainer: 'Template Maintainers',
          homepage:
            'https://github.com/link-foundation/js-ai-driven-development-pipeline-template',
        },
      },
    },
    {
      name: '@electron-forge/maker-rpm',
      config: {},
    },
  ],
};
