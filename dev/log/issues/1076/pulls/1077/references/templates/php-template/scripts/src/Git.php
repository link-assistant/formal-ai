<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Convenience wrapper over the git operations the release pipeline needs.
 */
final class Git
{
    public function __construct(private readonly ?string $cwd = null)
    {
    }

    public function configureBotIdentity(): void
    {
        Process::run(['git', 'config', 'user.name', 'github-actions[bot]'], $this->cwd);
        Process::run(['git', 'config', 'user.email', '41898282+github-actions[bot]@users.noreply.github.com'], $this->cwd);
    }

    public function tagExists(string $tag): bool
    {
        $result = Process::run(['git', 'rev-parse', '--verify', '--quiet', "refs/tags/{$tag}"], $this->cwd);

        return $result->ok() && $result->output() !== '';
    }

    /**
     * @param list<string> $paths
     */
    public function add(array $paths): void
    {
        Process::mustRun(array_merge(['git', 'add', '--'], $paths), $this->cwd);
    }

    public function commit(string $message): void
    {
        Process::mustRun(['git', 'commit', '-m', $message], $this->cwd);
    }

    public function tag(string $tag, string $message): void
    {
        Process::mustRun(['git', 'tag', '-a', $tag, '-m', $message], $this->cwd);
    }

    public function push(string $remote = 'origin', string $branch = 'HEAD'): ProcessResult
    {
        return Process::run(['git', 'push', $remote, $branch], $this->cwd);
    }

    public function pushTags(string $remote = 'origin'): ProcessResult
    {
        return Process::run(['git', 'push', $remote, '--tags'], $this->cwd);
    }

    public function currentBranch(): string
    {
        return Process::run(['git', 'rev-parse', '--abbrev-ref', 'HEAD'], $this->cwd)->output();
    }
}
