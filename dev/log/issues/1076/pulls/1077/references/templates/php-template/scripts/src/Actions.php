<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Helpers for talking to the GitHub Actions runner: step outputs, the job
 * summary, and log annotations. All methods degrade gracefully when run
 * locally (no GITHUB_OUTPUT / GITHUB_STEP_SUMMARY set), so the scripts can be
 * executed and tested outside CI.
 */
final class Actions
{
    /**
     * Write a `name=value` pair to $GITHUB_OUTPUT (multiline-safe).
     */
    public static function setOutput(string $name, string $value): void
    {
        $file = getenv('GITHUB_OUTPUT');

        if ($file === false || $file === '') {
            return;
        }

        if (str_contains($value, "\n")) {
            $delimiter = 'ghadelimiter_' . bin2hex(random_bytes(8));
            $line = "{$name}<<{$delimiter}\n{$value}\n{$delimiter}\n";
        } else {
            $line = "{$name}={$value}\n";
        }

        file_put_contents($file, $line, \FILE_APPEND);
    }

    public static function setBoolOutput(string $name, bool $value): void
    {
        self::setOutput($name, $value ? 'true' : 'false');
    }

    public static function summary(string $markdown): void
    {
        $file = getenv('GITHUB_STEP_SUMMARY');

        if ($file === false || $file === '') {
            echo $markdown . "\n";

            return;
        }

        file_put_contents($file, $markdown . "\n", \FILE_APPEND);
    }

    public static function notice(string $message): void
    {
        echo '::notice::' . self::escape($message) . "\n";
    }

    public static function warning(string $message): void
    {
        echo '::warning::' . self::escape($message) . "\n";
    }

    public static function error(string $message): void
    {
        echo '::error::' . self::escape($message) . "\n";
    }

    private static function escape(string $message): string
    {
        return str_replace(["\r", "\n"], ['%0D', '%0A'], $message);
    }
}
