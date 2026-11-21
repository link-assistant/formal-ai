<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Minimal wrapper around running external commands (mostly `git` and `gh`).
 *
 * Commands are passed as argument arrays and escaped, avoiding shell-injection
 * and quoting headaches.
 */
final class Process
{
    /**
     * @param list<string> $command
     */
    public static function run(array $command, ?string $cwd = null): ProcessResult
    {
        $descriptors = [
            0 => ['pipe', 'r'],
            1 => ['pipe', 'w'],
            2 => ['pipe', 'w'],
        ];

        $cmd = implode(' ', array_map('escapeshellarg', $command));
        $proc = proc_open($cmd, $descriptors, $pipes, $cwd);

        if (!\is_resource($proc)) {
            throw new \RuntimeException('Failed to start command: ' . $cmd);
        }

        fclose($pipes[0]);
        $stdout = stream_get_contents($pipes[1]) ?: '';
        $stderr = stream_get_contents($pipes[2]) ?: '';
        fclose($pipes[1]);
        fclose($pipes[2]);
        $exitCode = proc_close($proc);

        return new ProcessResult($exitCode, $stdout, $stderr);
    }

    /**
     * Run a command and throw if it exits non-zero.
     *
     * @param list<string> $command
     */
    public static function mustRun(array $command, ?string $cwd = null): ProcessResult
    {
        $result = self::run($command, $cwd);

        if (!$result->ok()) {
            throw new \RuntimeException(\sprintf(
                "Command failed (%d): %s\n%s",
                $result->exitCode,
                implode(' ', $command),
                $result->stderr,
            ));
        }

        return $result;
    }
}
