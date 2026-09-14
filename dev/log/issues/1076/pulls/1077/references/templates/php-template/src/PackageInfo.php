<?php

declare(strict_types=1);

namespace LinkFoundation\Template;

/**
 * Exposes metadata about the installed package.
 *
 * The version is read from the package's own composer.json so that runtime
 * code and the release pipeline share a single source of truth.
 */
final class PackageInfo
{
    /**
     * Return the package version declared in composer.json.
     *
     * Falls back to "0.0.0" when the manifest cannot be located (for example
     * when the source tree is used without a composer install).
     */
    public static function version(): string
    {
        $manifest = self::locateComposerJson();

        if ($manifest === null) {
            return '0.0.0';
        }

        $contents = file_get_contents($manifest);

        if ($contents === false) {
            return '0.0.0';
        }

        /** @var array{version?: string} $data */
        $data = json_decode($contents, true) ?: [];

        return $data['version'] ?? '0.0.0';
    }

    private static function locateComposerJson(): ?string
    {
        $candidate = \dirname(__DIR__) . '/composer.json';

        return is_file($candidate) ? $candidate : null;
    }
}
