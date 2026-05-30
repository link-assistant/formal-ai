export const RELEASE_API =
  'https://api.github.com/repos/konard/vk-bot-desktop/releases/latest';
export const RELEASES_URL =
  'https://github.com/konard/vk-bot-desktop/releases/latest';
export const CHECKSUM_ASSET_NAME = 'SHA256SUMS.txt';
export const PROVENANCE_ASSET_NAME = 'BUILD-PROVENANCE.txt';

export const downloadOptions = [
  {
    id: 'macos-arm64',
    os: 'macos',
    labelKey: 'macArm',
    assetPrefix: 'vk-bot-desktop-macos-arm64',
    extension: 'dmg',
  },
  {
    id: 'macos-arm64-zip',
    os: 'macos',
    labelKey: 'macArmZip',
    assetPrefix: 'vk-bot-desktop-macos-arm64',
    extension: 'zip',
  },
  {
    id: 'macos-x64',
    os: 'macos',
    labelKey: 'macIntel',
    assetPrefix: 'vk-bot-desktop-macos-x64',
    extension: 'dmg',
  },
  {
    id: 'macos-x64-zip',
    os: 'macos',
    labelKey: 'macIntelZip',
    assetPrefix: 'vk-bot-desktop-macos-x64',
    extension: 'zip',
  },
  {
    id: 'windows-x64',
    os: 'windows',
    labelKey: 'winInstaller',
    assetPrefix: 'vk-bot-desktop-windows-installer-x64',
    extension: 'exe',
  },
  {
    id: 'windows-arm64',
    os: 'windows',
    labelKey: 'winInstallerArm',
    assetPrefix: 'vk-bot-desktop-windows-installer-arm64',
    extension: 'exe',
  },
  {
    id: 'windows-portable-x64',
    os: 'windows',
    labelKey: 'winPortable',
    assetPrefix: 'vk-bot-desktop-windows-portable-x64',
    extension: 'exe',
  },
  {
    id: 'windows-portable-arm64',
    os: 'windows',
    labelKey: 'winPortableArm',
    assetPrefix: 'vk-bot-desktop-windows-portable-arm64',
    extension: 'exe',
  },
  {
    id: 'linux-appimage-x64',
    os: 'linux',
    labelKey: 'linuxAppImage',
    assetPrefix: 'vk-bot-desktop-linux-x64',
    extension: 'AppImage',
  },
  {
    id: 'linux-appimage-arm64',
    os: 'linux',
    labelKey: 'linuxAppImageArm',
    assetPrefix: 'vk-bot-desktop-linux-arm64',
    extension: 'AppImage',
  },
  {
    id: 'linux-deb-x64',
    os: 'linux',
    labelKey: 'linuxDeb',
    assetPrefix: 'vk-bot-desktop-linux-x64',
    extension: 'deb',
  },
  {
    id: 'linux-deb-arm64',
    os: 'linux',
    labelKey: 'linuxDebArm',
    assetPrefix: 'vk-bot-desktop-linux-arm64',
    extension: 'deb',
  },
  {
    id: 'linux-tar-x64',
    os: 'linux',
    labelKey: 'linuxTar',
    assetPrefix: 'vk-bot-desktop-linux-x64',
    extension: 'tar.gz',
  },
  {
    id: 'linux-tar-arm64',
    os: 'linux',
    labelKey: 'linuxTarArm',
    assetPrefix: 'vk-bot-desktop-linux-arm64',
    extension: 'tar.gz',
  },
];

export function primaryOptionFor(os) {
  if (os === 'macos') {
    return downloadOptions.find((option) => option.id === 'macos-arm64');
  }

  if (os === 'windows') {
    return downloadOptions.find((option) => option.id === 'windows-x64');
  }

  if (os === 'linux') {
    return downloadOptions.find((option) => option.id === 'linux-appimage-x64');
  }

  return undefined;
}

export function assetsByName(release) {
  return Object.fromEntries(
    (release?.assets || []).map((asset) => [asset.name, asset])
  );
}

export function releaseVersion(release) {
  const tag = String(
    release?.tag_name || release?.tagName || release?.name || ''
  );
  const match = tag.match(/(?:^|-)v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)/);

  return match?.[1];
}

export function assetNameFor(option, release) {
  if (!option) {
    return undefined;
  }

  const version = releaseVersion(release) || 'version';

  return `${option.assetPrefix}-${version}.${option.extension}`;
}

function legacyAssetNameFor(option) {
  return `${option.assetPrefix}.${option.extension}`;
}

export function candidateAssetNames(option, release) {
  if (!option) {
    return [];
  }

  return [assetNameFor(option, release), legacyAssetNameFor(option)].filter(
    (name, index, names) => name && names.indexOf(name) === index
  );
}

export function resolveDownloadAsset(option, releaseAssets, release) {
  for (const name of candidateAssetNames(option, release)) {
    const asset = releaseAssets[name];

    if (asset) {
      return asset;
    }
  }

  return undefined;
}

export function resolveDownloadHref(option, releaseAssets, release) {
  return resolveDownloadAsset(option, releaseAssets, release)
    ?.browser_download_url;
}

export function resolveChecksumHref(releaseAssets) {
  return (
    releaseAssets[CHECKSUM_ASSET_NAME]?.browser_download_url || RELEASES_URL
  );
}

export function resolveProvenanceHref(releaseAssets) {
  return (
    releaseAssets[PROVENANCE_ASSET_NAME]?.browser_download_url || RELEASES_URL
  );
}

export function groupedOptions() {
  return ['macos', 'windows', 'linux'].map((os) => ({
    os,
    options: downloadOptions.filter((option) => option.os === os),
  }));
}

export function optionById(id) {
  return downloadOptions.find((option) => option.id === id);
}

export function downloadFamilies() {
  return [
    {
      os: 'macos',
      families: [
        {
          id: 'macos-arm64',
          primary: optionById('macos-arm64'),
          secondary: [optionById('macos-arm64-zip')],
        },
        {
          id: 'macos-x64',
          primary: optionById('macos-x64'),
          secondary: [optionById('macos-x64-zip')],
        },
      ],
    },
    {
      os: 'windows',
      families: [
        {
          id: 'windows-installer',
          primary: optionById('windows-x64'),
          secondary: [optionById('windows-arm64')],
        },
        {
          id: 'windows-portable',
          primary: optionById('windows-portable-x64'),
          secondary: [optionById('windows-portable-arm64')],
        },
      ],
    },
    {
      os: 'linux',
      families: [
        {
          id: 'linux-x64',
          primary: optionById('linux-appimage-x64'),
          secondary: [optionById('linux-deb-x64'), optionById('linux-tar-x64')],
        },
        {
          id: 'linux-arm64',
          primary: optionById('linux-appimage-arm64'),
          secondary: [
            optionById('linux-deb-arm64'),
            optionById('linux-tar-arm64'),
          ],
        },
      ],
    },
  ];
}
