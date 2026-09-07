<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Outcome of {@see ReleaseDecider::decide()}.
 */
final class ReleaseDecision
{
    public function __construct(
        public readonly bool $shouldRelease,
        public readonly bool $skipBump,
        public readonly string $reason,
    ) {
    }
}
