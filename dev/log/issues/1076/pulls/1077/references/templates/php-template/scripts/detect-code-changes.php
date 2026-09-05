<?php

declare(strict_types=1);

/**
 * Classify the files changed by the current push/PR and export the flags as
 * GitHub Actions step outputs so downstream jobs can skip irrelevant work.
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\Actions;
use LinkFoundation\Template\Pipeline\ChangeDetector;

$files = ChangeDetector::changedFiles();
$flags = ChangeDetector::classify($files);

echo 'Changed files (' . count($files) . "):\n";
foreach ($files as $file) {
    echo "  - {$file}\n";
}

echo "\nClassification:\n";
foreach ($flags as $name => $value) {
    Actions::setBoolOutput($name, $value);
    echo '  ' . str_pad($name, 20) . ' = ' . ($value ? 'true' : 'false') . "\n";
}
