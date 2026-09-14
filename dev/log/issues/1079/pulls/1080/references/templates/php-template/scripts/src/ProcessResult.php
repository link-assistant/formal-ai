<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Value object returned by {@see Process::run()}.
 */
final class ProcessResult
{
    public function __construct(
        public readonly int $exitCode,
        public readonly string $stdout,
        public readonly string $stderr,
    ) {
    }

    public function ok(): bool
    {
        return $this->exitCode === 0;
    }

    public function output(): string
    {
        return trim($this->stdout);
    }
}
