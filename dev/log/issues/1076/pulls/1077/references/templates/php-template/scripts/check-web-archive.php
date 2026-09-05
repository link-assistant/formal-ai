<?php

declare(strict_types=1);

/**
 * Read a lychee broken-link report and check each URL against the Wayback
 * Machine. Sets the `all_archived` step output to true only when every broken
 * link has an archived snapshot, in which case the workflow does not fail.
 *
 * Input: LYCHEE_OUTPUT env var (path to the lychee markdown report).
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\Actions;
use LinkFoundation\Template\Pipeline\WebArchive;

$reportPath = getenv('LYCHEE_OUTPUT') ?: 'lychee/out.md';

if (!is_file($reportPath)) {
    Actions::warning("Lychee report not found at {$reportPath}; cannot check the Web Archive.");
    Actions::setBoolOutput('all_archived', false);
    exit(0);
}

$report = file_get_contents($reportPath) ?: '';
$urls = WebArchive::extractUrls($report);

if ($urls === []) {
    echo "No URLs found in the lychee report.\n";
    Actions::setBoolOutput('all_archived', true);
    exit(0);
}

echo 'Checking ' . count($urls) . " link(s) against the Web Archive...\n";

$archive = new WebArchive();
$result = $archive->review($urls);

foreach ($result['archived'] as $url => $snapshot) {
    Actions::notice("Archived copy available for {$url} -> {$snapshot}");
    echo "  ✓ {$url}\n      {$snapshot}\n";
}

foreach ($result['missing'] as $url) {
    Actions::warning("No Web Archive snapshot for {$url}");
    echo "  ✗ {$url}\n";
}

$allArchived = $result['missing'] === [];
Actions::setBoolOutput('all_archived', $allArchived);

echo $allArchived
    ? "All broken links have an archived fallback.\n"
    : 'Some broken links have no archived fallback.' . "\n";
