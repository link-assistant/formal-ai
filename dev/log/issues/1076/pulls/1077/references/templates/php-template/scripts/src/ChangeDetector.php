<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Classifies the files changed by a push or pull request so the workflow can
 * skip irrelevant jobs (e.g. docs-only PRs skip lint/test/changeset gates).
 */
final class ChangeDetector
{
    /** Path prefixes whose changes never count as "code". */
    private const EXCLUDED_PREFIXES = ['changelog.d/', 'docs/', 'examples/', 'experiments/'];

    /**
     * Compute the changed file list from the git history, based on the GitHub
     * event environment.
     *
     * @return list<string>
     */
    public static function changedFiles(?string $cwd = null): array
    {
        $event = getenv('GITHUB_EVENT_NAME') ?: '';
        $baseSha = getenv('GITHUB_BASE_SHA') ?: '';
        $headSha = getenv('GITHUB_HEAD_SHA') ?: '';

        if ($event === 'pull_request' && $baseSha !== '' && $headSha !== '') {
            // Ensure the base commit is present locally.
            Process::run(['git', 'fetch', '--no-tags', '--depth', '1', 'origin', $baseSha], $cwd);
            $result = Process::run(['git', 'diff', '--name-only', "{$baseSha}...{$headSha}"], $cwd);

            if ($result->ok()) {
                return self::splitLines($result->stdout);
            }
        }

        // Push (or fallback): diff the last commit; first commit lists the tree.
        $result = Process::run(['git', 'diff', '--name-only', 'HEAD^', 'HEAD'], $cwd);

        if ($result->ok()) {
            return self::splitLines($result->stdout);
        }

        $tree = Process::run(['git', 'ls-tree', '-r', '--name-only', 'HEAD'], $cwd);

        return $tree->ok() ? self::splitLines($tree->stdout) : [];
    }

    /**
     * @param list<string> $files
     * @return array<string, bool>
     */
    public static function classify(array $files): array
    {
        $flags = [
            'php-changed' => false,
            'tests-changed' => false,
            'package-changed' => false,
            'docs-changed' => false,
            'workflow-changed' => false,
            'any-code-changed' => false,
        ];

        foreach ($files as $file) {
            if ($file === '') {
                continue;
            }

            if (str_ends_with($file, '.php')) {
                $flags['php-changed'] = true;
            }

            if (str_starts_with($file, 'tests/')) {
                $flags['tests-changed'] = true;
            }

            if (\in_array($file, ['composer.json', 'composer.lock'], true)) {
                $flags['package-changed'] = true;
            }

            if (str_starts_with($file, 'docs/') || str_ends_with($file, '.md')) {
                $flags['docs-changed'] = true;
            }

            if (str_starts_with($file, '.github/workflows/')) {
                $flags['workflow-changed'] = true;
            }

            if (self::isCode($file)) {
                $flags['any-code-changed'] = true;
            }
        }

        return $flags;
    }

    private static function isCode(string $file): bool
    {
        if (str_ends_with($file, '.md')) {
            return false;
        }

        foreach (self::EXCLUDED_PREFIXES as $prefix) {
            if (str_starts_with($file, $prefix)) {
                return false;
            }
        }

        if (str_starts_with($file, '.github/workflows/')) {
            return true;
        }

        foreach (['.php', '.json', '.yml', '.yaml', '.neon', '.dist'] as $ext) {
            if (str_ends_with($file, $ext)) {
                return true;
            }
        }

        return false;
    }

    /**
     * @return list<string>
     */
    private static function splitLines(string $text): array
    {
        $lines = preg_split('/\r?\n/', trim($text)) ?: [];

        return array_values(array_filter($lines, static fn (string $l): bool => $l !== ''));
    }
}
