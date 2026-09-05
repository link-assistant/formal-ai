<?php

declare(strict_types=1);

/**
 * Poll Packagist until the released version is importable, so the GitHub Release
 * step only runs once the package is actually live.
 *
 * Packagist imports happen via webhook and can lag a few seconds behind the
 * pushed tag, so we poll rather than assume immediate availability.
 *
 * Usage:
 *   php scripts/wait-for-packagist.php --version=1.2.3 [--timeout=300] [--interval=10]
 *
 * For the template sentinel package nothing is published, so this is a no-op.
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\Actions;
use LinkFoundation\Template\Pipeline\Cli;
use LinkFoundation\Template\Pipeline\Packagist;
use LinkFoundation\Template\Pipeline\Project;

$options = getopt('', ['version::', 'timeout::', 'interval::']);

$project = Project::locate();
$version = Cli::string($options, 'version') ?? $project->version();
$timeout = (int) (Cli::string($options, 'timeout') ?? '300');
$interval = max(1, (int) (Cli::string($options, 'interval') ?? '10'));

if ($project->isTemplateSentinel()) {
    echo "Template sentinel package: skipping Packagist availability check.\n";
    exit(0);
}

$packageName = $project->packageName();
$packagist = new Packagist();

echo "Waiting for {$packageName} {$version} to appear on Packagist (timeout {$timeout}s)...\n";

$deadline = time() + $timeout;
$attempt = 0;

while (true) {
    ++$attempt;

    if ($packagist->hasVersion($packageName, $version)) {
        echo "Packagist has {$packageName} {$version} (after {$attempt} attempt(s)).\n";
        exit(0);
    }

    if (time() >= $deadline) {
        Actions::warning("Packagist did not import {$packageName} {$version} within {$timeout}s. "
            . 'Continuing anyway — Packagist may still be processing the webhook.');
        // Do not fail the pipeline: the package may simply be slow to import,
        // and the GitHub Release is still worth publishing.
        exit(0);
    }

    echo "  attempt {$attempt}: not yet available, retrying in {$interval}s...\n";
    sleep($interval);
}
