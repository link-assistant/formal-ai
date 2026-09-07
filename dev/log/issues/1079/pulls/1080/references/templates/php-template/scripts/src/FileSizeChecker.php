<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Enforces a maximum line count per source file so every file fits in an AI
 * context window and stays human-reviewable.
 */
final class FileSizeChecker
{
    public const MAX_LINES = 1000;

    private const EXCLUDED_DIRS = ['vendor', 'node_modules', '.git', 'build', 'var', 'docs'];

    /**
     * @param list<string> $extensions
     */
    public function __construct(
        private readonly string $root,
        private readonly int $maxLines = self::MAX_LINES,
        private readonly array $extensions = ['php'],
    ) {
    }

    /**
     * Scan the project and return files exceeding the line cap.
     *
     * @return array<string, int> Map of relative path => line count.
     */
    public function violations(): array
    {
        $violations = [];

        $iterator = new \RecursiveIteratorIterator(
            new \RecursiveCallbackFilterIterator(
                new \RecursiveDirectoryIterator($this->root, \FilesystemIterator::SKIP_DOTS),
                function (\SplFileInfo $current): bool {
                    if ($current->isDir()) {
                        return !\in_array($current->getFilename(), self::EXCLUDED_DIRS, true);
                    }

                    return true;
                },
            ),
        );

        foreach ($iterator as $file) {
            if (!$file instanceof \SplFileInfo || !$file->isFile()) {
                continue;
            }

            if (!\in_array($file->getExtension(), $this->extensions, true)) {
                continue;
            }

            $lines = $this->countLines($file->getPathname());

            if ($lines > $this->maxLines) {
                $relative = ltrim(str_replace($this->root, '', $file->getPathname()), '/');
                $violations[$relative] = $lines;
            }
        }

        ksort($violations);

        return $violations;
    }

    private function countLines(string $path): int
    {
        $handle = fopen($path, 'rb');

        if ($handle === false) {
            return 0;
        }

        $lines = 0;

        while (!feof($handle)) {
            $chunk = fread($handle, 1 << 16);

            if ($chunk === false) {
                break;
            }

            $lines += substr_count($chunk, "\n");
        }

        fclose($handle);

        return $lines + 1;
    }
}
