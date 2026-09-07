<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Reads and updates CHANGELOG.md (Keep a Changelog format).
 *
 * New version entries are inserted at the `<!-- changelog-insert-here -->`
 * marker so concurrent releases never collide on a fixed line.
 */
final class Changelog
{
    public const INSERT_MARKER = '<!-- changelog-insert-here -->';

    public function __construct(private readonly string $path)
    {
    }

    public static function forProject(Project $project): self
    {
        return new self($project->path('CHANGELOG.md'));
    }

    public function path(): string
    {
        return $this->path;
    }

    public function read(): string
    {
        if (!is_file($this->path)) {
            return self::scaffold();
        }

        $contents = file_get_contents($this->path);

        return $contents === false ? self::scaffold() : str_replace("\r\n", "\n", $contents);
    }

    /**
     * Build a version entry block (heading + body).
     */
    public static function entry(string $version, string $body, string $date): string
    {
        $heading = "## [{$version}] - {$date}";
        $body = trim($body);

        return $body === '' ? $heading . "\n" : $heading . "\n\n" . $body . "\n";
    }

    /**
     * Insert a new entry at the marker, or just after the top matter if the
     * marker is absent.
     */
    public function insert(string $entry): void
    {
        $contents = $this->read();
        $entry = rtrim($entry) . "\n";

        if (str_contains($contents, self::INSERT_MARKER)) {
            $contents = str_replace(
                self::INSERT_MARKER,
                self::INSERT_MARKER . "\n\n" . $entry,
                $contents,
            );
        } else {
            $contents = rtrim($contents) . "\n\n" . $entry;
        }

        file_put_contents($this->path, $contents);
    }

    /**
     * Extract the body of a specific version's section (without the heading).
     * Returns null when the version is not present.
     */
    public function section(string $version): ?string
    {
        $contents = $this->read();
        $quoted = preg_quote($version, '/');
        $pattern = '/^##\s+\[?' . $quoted . '\]?.*$/m';

        if (!preg_match($pattern, $contents, $m, \PREG_OFFSET_CAPTURE)) {
            return null;
        }

        $start = $m[0][1] + \strlen($m[0][0]);
        $rest = substr($contents, $start);

        if (preg_match('/^##\s+\[?\d+\.\d+\.\d+/m', $rest, $next, \PREG_OFFSET_CAPTURE)) {
            $rest = substr($rest, 0, $next[0][1]);
        }

        return trim($rest);
    }

    public static function scaffold(): string
    {
        return "# Changelog\n\n"
            . "All notable changes to this project are documented in this file.\n\n"
            . "The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),\n"
            . "and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n"
            . self::INSERT_MARKER . "\n";
    }
}
