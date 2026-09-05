<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Pure semantic-version math. No I/O, fully unit-testable.
 */
final class SemVer
{
    public const BUMP_TYPES = ['major', 'minor', 'patch'];

    private const PRECEDENCE = ['patch' => 1, 'minor' => 2, 'major' => 3];

    public function __construct(
        public readonly int $major,
        public readonly int $minor,
        public readonly int $patch,
    ) {
    }

    public static function parse(string $version): self
    {
        $clean = ltrim(trim($version), 'vV');

        if (!preg_match('/^(\d+)\.(\d+)\.(\d+)/', $clean, $m)) {
            throw new \InvalidArgumentException("Not a semantic version: {$version}");
        }

        return new self((int) $m[1], (int) $m[2], (int) $m[3]);
    }

    public function bump(string $type): self
    {
        return match ($type) {
            'major' => new self($this->major + 1, 0, 0),
            'minor' => new self($this->major, $this->minor + 1, 0),
            'patch' => new self($this->major, $this->minor, $this->patch + 1),
            default => throw new \InvalidArgumentException("Unknown bump type: {$type}"),
        };
    }

    public function __toString(): string
    {
        return "{$this->major}.{$this->minor}.{$this->patch}";
    }

    /**
     * Return the bump type with the highest precedence (major > minor > patch).
     *
     * @param list<string> $types
     */
    public static function highestBump(array $types, string $default = 'patch'): string
    {
        $winner = $default;

        foreach ($types as $type) {
            if (!isset(self::PRECEDENCE[$type])) {
                continue;
            }

            if (self::PRECEDENCE[$type] > self::PRECEDENCE[$winner]) {
                $winner = $type;
            }
        }

        return $winner;
    }

    /**
     * Compare two semantic versions. Returns -1, 0 or 1.
     */
    public static function compare(string $a, string $b): int
    {
        $va = self::parse($a);
        $vb = self::parse($b);

        return [$va->major, $va->minor, $va->patch] <=> [$vb->major, $vb->minor, $vb->patch];
    }
}
