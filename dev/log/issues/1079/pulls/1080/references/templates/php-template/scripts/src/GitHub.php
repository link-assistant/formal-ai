<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Thin wrapper over the `gh` CLI for GitHub Release operations. The CLI handles
 * authentication via GH_TOKEN / GITHUB_TOKEN, so no token plumbing is needed
 * here.
 */
final class GitHub
{
    public function __construct(private readonly string $repository)
    {
    }

    public static function fromEnv(?string $repository = null): self
    {
        $repo = $repository ?? getenv('GITHUB_REPOSITORY') ?: '';

        return new self($repo);
    }

    public function repository(): string
    {
        return $this->repository;
    }

    public function releaseExists(string $tag): bool
    {
        $result = Process::run([
            'gh', 'release', 'view', $tag, '--repo', $this->repository,
        ]);

        return $result->ok();
    }

    /**
     * Create a GitHub release. Returns true on success (or when it already
     * exists, which is treated as success for idempotency).
     */
    public function createRelease(string $tag, string $title, string $notesFile, bool $prerelease = false): bool
    {
        $command = [
            'gh', 'release', 'create', $tag,
            '--repo', $this->repository,
            '--title', $title,
            '--notes-file', $notesFile,
        ];

        if ($prerelease) {
            $command[] = '--prerelease';
        }

        $result = Process::run($command);

        if ($result->ok()) {
            return true;
        }

        if (str_contains($result->stderr, 'already exists')) {
            Actions::notice("Release {$tag} already exists; treating as success.");

            return true;
        }

        Actions::error('Failed to create release: ' . $result->stderr);

        return false;
    }
}
