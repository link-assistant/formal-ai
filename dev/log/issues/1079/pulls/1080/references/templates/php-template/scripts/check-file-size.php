<?php

declare(strict_types=1);

/**
 * Fail when any PHP source file exceeds the line cap (default 1000 lines),
 * keeping files small enough to fit an AI context window.
 *
 * Usage: php scripts/check-file-size.php [--max=1000]
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\FileSizeChecker;
use LinkFoundation\Template\Pipeline\Project;

$options = getopt('', ['max::']);
$max = isset($options['max']) ? (int) $options['max'] : FileSizeChecker::MAX_LINES;

$project = Project::locate();
$checker = new FileSizeChecker($project->root(), $max);
$violations = $checker->violations();

if ($violations === []) {
    echo "All PHP files are within the {$max}-line limit.\n";
    exit(0);
}

fwrite(\STDERR, "The following files exceed the {$max}-line limit:\n");
foreach ($violations as $path => $lines) {
    fwrite(\STDERR, "  {$path}: {$lines} lines\n");
}
exit(1);
