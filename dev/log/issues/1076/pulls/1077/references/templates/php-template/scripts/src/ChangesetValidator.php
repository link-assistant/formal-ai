<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Validates that a pull request adds exactly one well-formed changelog
 * fragment. Keeps every code change self-documenting and conflict-free.
 */
final class ChangesetValidator
{
    public function __construct(private readonly ChangelogFragments $fragments)
    {
    }

    /**
     * Find fragment files *added* by this PR (git status "A") under
     * changelog.d/. Falls back to "all fragments present" when run locally.
     *
     * @return list<string>
     */
    public static function addedFragments(?string $cwd = null): array
    {
        $baseSha = getenv('GITHUB_BASE_SHA') ?: '';
        $headSha = getenv('GITHUB_HEAD_SHA') ?: '';

        if ($baseSha !== '' && $headSha !== '') {
            $result = Process::run(
                ['git', 'diff', '--name-status', '--diff-filter=A', "{$baseSha}...{$headSha}"],
                $cwd,
            );

            if ($result->ok()) {
                return self::parseAdded($result->stdout);
            }
        }

        $baseRef = getenv('GITHUB_BASE_REF') ?: '';

        if ($baseRef !== '') {
            Process::run(['git', 'fetch', '--no-tags', '--depth', '1', 'origin', $baseRef], $cwd);
            $result = Process::run(
                ['git', 'diff', '--name-status', '--diff-filter=A', "origin/{$baseRef}...HEAD"],
                $cwd,
            );

            if ($result->ok()) {
                return self::parseAdded($result->stdout);
            }
        }

        return [];
    }

    /**
     * @return array{ok: bool, errors: list<string>, warnings: list<string>}
     */
    public function validate(?int $addedCount = null): array
    {
        $errors = [];
        $warnings = [];

        $count = $addedCount ?? $this->fragments->count();

        if ($count === 0) {
            $errors[] = 'No changelog fragment found. Add one with: composer changeset';

            return ['ok' => false, 'errors' => $errors, 'warnings' => $warnings];
        }

        if ($count > 1) {
            $warnings[] = "Found {$count} changelog fragments; usually one per PR is expected.";
        }

        foreach ($this->fragments->list() as $file) {
            $parsed = $this->fragments->parse($file);
            $name = basename($file);

            if ($parsed['bump'] === null) {
                $errors[] = "{$name}: missing or invalid 'bump:' frontmatter (expected major|minor|patch).";
            }

            if (trim($parsed['body']) === '') {
                $errors[] = "{$name}: fragment body is empty; describe the change.";
            }
        }

        return ['ok' => $errors === [], 'errors' => $errors, 'warnings' => $warnings];
    }

    /**
     * @return list<string>
     */
    private static function parseAdded(string $output): array
    {
        $added = [];

        foreach (preg_split('/\r?\n/', trim($output)) ?: [] as $line) {
            if ($line === '') {
                continue;
            }

            $parts = preg_split('/\s+/', $line, 2);
            $path = $parts[1] ?? '';

            if (!str_starts_with($path, 'changelog.d/') || !str_ends_with($path, '.md')) {
                continue;
            }

            if (\in_array(basename($path), ChangelogFragments::RESERVED, true)) {
                continue;
            }

            $added[] = $path;
        }

        return $added;
    }
}
