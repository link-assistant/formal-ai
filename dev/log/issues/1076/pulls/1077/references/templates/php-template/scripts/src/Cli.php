<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Small helpers for reading {@see getopt()} results in a type-safe way.
 *
 * getopt() returns `string|false|array` per option, so callers need a guarded
 * accessor to keep static analysis happy and avoid surprises with repeated or
 * value-less flags.
 */
final class Cli
{
    /**
     * Read a string option, returning $default when it is absent or value-less.
     *
     * @param array<string, mixed> $options
     */
    public static function string(array $options, string $key, ?string $default = null): ?string
    {
        $value = $options[$key] ?? null;

        if (\is_string($value)) {
            return $value;
        }

        // Repeated option: take the last occurrence.
        if (\is_array($value)) {
            $last = end($value);

            return \is_string($last) ? $last : $default;
        }

        return $default;
    }

    /**
     * True when a value-less flag (e.g. --skip-bump) is present.
     *
     * @param array<string, mixed> $options
     */
    public static function flag(array $options, string $key): bool
    {
        return \array_key_exists($key, $options);
    }
}
