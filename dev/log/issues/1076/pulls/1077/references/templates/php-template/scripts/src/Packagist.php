<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Queries the Packagist registry — the source of truth for "is this version
 * published?". We never trust git tags for that decision, because a tag can be
 * pushed while Packagist has not yet imported the release (mirroring the
 * registry-is-truth lesson from the sibling templates).
 */
final class Packagist
{
    public const DEFAULT_BASE_URL = 'https://repo.packagist.org';

    public function __construct(
        private readonly Http $http = new Http(),
        private readonly string $baseUrl = self::DEFAULT_BASE_URL,
    ) {
    }

    public function metadataUrl(string $packageName): string
    {
        return rtrim($this->baseUrl, '/') . '/p2/' . strtolower($packageName) . '.json';
    }

    /**
     * Return every published version for a package (lowercased "x.y.z", without
     * a leading "v"). Returns an empty array when the package is unknown (404).
     *
     * @return list<string>
     */
    public function publishedVersions(string $packageName): array
    {
        $response = $this->http->get($this->metadataUrl($packageName));

        if ($response['status'] !== 200 || $response['body'] === '') {
            return [];
        }

        /** @var array{packages?: array<string, list<array{version?: string}>>} $data */
        $data = json_decode($response['body'], true) ?: [];
        $releases = $data['packages'][strtolower($packageName)] ?? [];

        $versions = [];

        foreach ($releases as $release) {
            $version = $release['version'] ?? null;

            if (\is_string($version)) {
                $versions[] = strtolower(ltrim($version, 'vV'));
            }
        }

        return $versions;
    }

    public function hasVersion(string $packageName, string $version): bool
    {
        $needle = strtolower(ltrim($version, 'vV'));

        return \in_array($needle, $this->publishedVersions($packageName), true);
    }
}
