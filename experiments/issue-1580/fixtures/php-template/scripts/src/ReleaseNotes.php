<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Builds GitHub Release notes from a CHANGELOG.md section: adds a Packagist
 * badge and caps the body at GitHub's size limit. All methods are pure.
 */
final class ReleaseNotes
{
    /** GitHub rejects release bodies larger than ~125 000 bytes. */
    public const MAX_BYTES = 120_000;

    public static function packagistBadge(string $packageName): string
    {
        $slug = strtolower($packageName);

        return \sprintf(
            '[![Packagist Version](https://img.shields.io/packagist/v/%s.svg)](https://packagist.org/packages/%s)',
            $slug,
            $slug,
        );
    }

    public static function addBadge(string $body, string $packageName): string
    {
        if (str_contains($body, 'img.shields.io/packagist')) {
            return $body;
        }

        return self::packagistBadge($packageName) . "\n\n" . $body;
    }

    /**
     * Truncate to MAX_BYTES without splitting a multibyte character, appending
     * a notice that links to the full CHANGELOG at the release tag.
     */
    public static function capBytes(string $body, string $repository, string $tag, int $max = self::MAX_BYTES): string
    {
        if (\strlen($body) <= $max) {
            return $body;
        }

        $notice = "\n\n---\n\n_Release notes truncated. See the full "
            . "[CHANGELOG.md](https://github.com/{$repository}/blob/{$tag}/CHANGELOG.md)._\n";

        $budget = $max - \strlen($notice);
        $truncated = substr($body, 0, max(0, $budget));

        // Avoid cutting in the middle of a UTF-8 sequence.
        while ($truncated !== '' && (\ord($truncated[\strlen($truncated) - 1]) & 0xC0) === 0x80) {
            $truncated = substr($truncated, 0, -1);
        }

        return $truncated . $notice;
    }

    public static function build(
        string $version,
        ?string $changelogSection,
        string $packageName,
        string $repository,
        string $tag,
        bool $isSentinel,
    ): string {
        $body = $changelogSection !== null && trim($changelogSection) !== ''
            ? trim($changelogSection)
            : "Release {$version}.";

        if (!$isSentinel) {
            $body = self::addBadge($body, $packageName);
        }

        return self::capBytes($body, $repository, $tag);
    }
}
