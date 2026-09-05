<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Reads and merges "changeset" fragments from changelog.d/.
 *
 * Each fragment is a markdown file with optional YAML-ish frontmatter declaring
 * the semver bump, followed by Keep-a-Changelog category sections:
 *
 *     ---
 *     bump: minor
 *     ---
 *     ### Added
 *     - New thing.
 *
 * This decouples "what changed" (recorded per PR, conflict-free) from "when we
 * release" (fragments are collected into CHANGELOG.md at release time).
 */
final class ChangelogFragments
{
    /** Keep a Changelog category order. */
    public const CATEGORIES = ['Added', 'Changed', 'Deprecated', 'Removed', 'Fixed', 'Security'];

    /** Files in changelog.d/ that are never treated as fragments. */
    public const RESERVED = ['README.md', '.gitkeep', 'fragment_template.md'];

    public function __construct(private readonly string $directory)
    {
    }

    public static function forProject(Project $project): self
    {
        return new self($project->path('changelog.d'));
    }

    public function directory(): string
    {
        return $this->directory;
    }

    /**
     * @return list<string> Absolute paths to fragment files.
     */
    public function list(): array
    {
        if (!is_dir($this->directory)) {
            return [];
        }

        $fragments = [];

        foreach (scandir($this->directory) ?: [] as $entry) {
            if ($entry === '.' || $entry === '..') {
                continue;
            }

            if (\in_array($entry, self::RESERVED, true)) {
                continue;
            }

            if (!str_ends_with($entry, '.md')) {
                continue;
            }

            $fragments[] = $this->directory . '/' . $entry;
        }

        sort($fragments);

        return $fragments;
    }

    public function count(): int
    {
        return \count($this->list());
    }

    /**
     * Parse a fragment file into its declared bump type and body.
     *
     * @return array{bump: ?string, body: string}
     */
    public function parse(string $file): array
    {
        $contents = file_get_contents($file);

        if ($contents === false) {
            throw new \RuntimeException("Unable to read fragment: {$file}");
        }

        return self::parseContents($contents);
    }

    /**
     * @return array{bump: ?string, body: string}
     */
    public static function parseContents(string $contents): array
    {
        $contents = str_replace("\r\n", "\n", $contents);
        $bump = null;

        if (preg_match('/^---\n(.*?)\n---\n?(.*)$/s', $contents, $m)) {
            $frontmatter = $m[1];
            $contents = $m[2];

            if (preg_match('/bump\s*:\s*([a-z]+)/i', $frontmatter, $bm)) {
                $candidate = strtolower($bm[1]);

                if (\in_array($candidate, SemVer::BUMP_TYPES, true)) {
                    $bump = $candidate;
                }
            }
        }

        return ['bump' => $bump, 'body' => trim($contents)];
    }

    /**
     * Determine the release bump type from all fragments (highest wins).
     */
    public function determineBump(string $default = 'patch'): string
    {
        $types = [];

        foreach ($this->list() as $file) {
            $parsed = $this->parse($file);

            if ($parsed['bump'] !== null) {
                $types[] = $parsed['bump'];
            }
        }

        return SemVer::highestBump($types, $default);
    }

    /**
     * Merge all fragment bodies into a single changelog entry body, grouped by
     * Keep-a-Changelog category. Returns the markdown that goes *under* the
     * version heading (no heading itself).
     */
    public function collectBody(): string
    {
        /** @var array<string, list<string>> $byCategory */
        $byCategory = [];
        $uncategorised = [];

        foreach ($this->list() as $file) {
            $body = $this->parse($file)['body'];
            $this->splitIntoCategories($body, $byCategory, $uncategorised);
        }

        $sections = [];

        foreach (self::CATEGORIES as $category) {
            if (empty($byCategory[$category])) {
                continue;
            }

            $sections[] = "### {$category}\n" . implode("\n", $byCategory[$category]);
        }

        if (!empty($uncategorised)) {
            $sections[] = "### Changed\n" . implode("\n", $uncategorised);
        }

        return implode("\n\n", $sections);
    }

    /**
     * @param array<string, list<string>> $byCategory
     * @param list<string>                 $uncategorised
     */
    private function splitIntoCategories(string $body, array &$byCategory, array &$uncategorised): void
    {
        $current = null;

        foreach (explode("\n", $body) as $line) {
            if (preg_match('/^###\s+(.+?)\s*$/', $line, $m)) {
                $heading = ucfirst(strtolower(trim($m[1])));
                $current = \in_array($heading, self::CATEGORIES, true) ? $heading : 'Changed';

                continue;
            }

            if (trim($line) === '') {
                continue;
            }

            $bullet = preg_match('/^\s*[-*]\s+/', $line) ? rtrim($line) : '- ' . trim($line);

            if ($current === null) {
                $uncategorised[] = $bullet;
            } else {
                $byCategory[$current][] = $bullet;
            }
        }
    }

    /**
     * Delete all fragment files (called after a successful collect).
     */
    public function clear(): void
    {
        foreach ($this->list() as $file) {
            @unlink($file);
        }
    }
}
