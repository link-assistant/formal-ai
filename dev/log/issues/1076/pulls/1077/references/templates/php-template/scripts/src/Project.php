<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Reads and writes the project's single source of truth: composer.json.
 *
 * The package version lives in composer.json `version`; the package name lives
 * in `name`. Scripts use this class instead of hardcoding either value, so the
 * template works after a rename without editing the pipeline.
 */
final class Project
{
    public function __construct(private readonly string $root)
    {
    }

    /**
     * Locate the project root by walking up from this file until a composer.json
     * is found. Allows callers to override for tests.
     */
    public static function locate(?string $start = null): self
    {
        $dir = $start ?? __DIR__;

        while (true) {
            if (is_file($dir . '/composer.json')) {
                return new self($dir);
            }

            $parent = \dirname($dir);

            if ($parent === $dir) {
                // Reached the filesystem root; fall back to two levels up
                // from scripts/src so the template still works in odd layouts.
                return new self(\dirname(__DIR__, 2));
            }

            $dir = $parent;
        }
    }

    public function root(): string
    {
        return $this->root;
    }

    public function path(string $relative): string
    {
        return $this->root . '/' . ltrim($relative, '/');
    }

    public function composerJsonPath(): string
    {
        return $this->path('composer.json');
    }

    /**
     * @return array<string, mixed>
     */
    public function composer(): array
    {
        $raw = file_get_contents($this->composerJsonPath());

        if ($raw === false) {
            throw new \RuntimeException('Unable to read composer.json at ' . $this->composerJsonPath());
        }

        /** @var array<string, mixed> $data */
        $data = json_decode($raw, true, 512, \JSON_THROW_ON_ERROR);

        return $data;
    }

    public function version(): string
    {
        $composer = $this->composer();
        $version = $composer['version'] ?? null;

        if (!\is_string($version) || $version === '') {
            throw new \RuntimeException('composer.json is missing a "version" field.');
        }

        return $version;
    }

    /**
     * Rewrite the `version` field while preserving the rest of the file
     * formatting as closely as possible (regex edit, not full re-encode).
     */
    public function setVersion(string $newVersion): void
    {
        $raw = file_get_contents($this->composerJsonPath());

        if ($raw === false) {
            throw new \RuntimeException('Unable to read composer.json.');
        }

        $updated = preg_replace(
            '/("version"\s*:\s*")[^"]*(")/',
            '${1}' . $newVersion . '${2}',
            $raw,
            1,
            $count,
        );

        if ($updated === null || $count === 0) {
            throw new \RuntimeException('Could not find a "version" field to update in composer.json.');
        }

        file_put_contents($this->composerJsonPath(), $updated);
    }

    public function packageName(): string
    {
        $composer = $this->composer();
        $name = $composer['name'] ?? null;

        if (!\is_string($name) || $name === '') {
            throw new \RuntimeException('composer.json is missing a "name" field.');
        }

        return $name;
    }

    /**
     * The default template package name. When the project still uses this
     * sentinel, the pipeline skips real publishing so the template runs CI
     * green without ever releasing to Packagist.
     */
    public const SENTINEL_PACKAGE_NAME = 'link-foundation/example-package-name';

    public function isTemplateSentinel(): bool
    {
        return $this->packageName() === self::SENTINEL_PACKAGE_NAME;
    }
}
