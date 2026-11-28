<?php

declare(strict_types=1);

/**
 * Compute the next version, update composer.json + CHANGELOG.md, then commit,
 * tag and push.
 *
 * Modes:
 *   --mode=changeset                 derive bump + notes from changelog.d/
 *   --mode=instant --bump=minor      explicit bump with an optional --description
 *   --skip-bump                      self-heal: release the current version as-is
 *                                    (no new commit/tag — re-publish what exists)
 *
 * Outputs:
 *   new_version        the version that should be released
 *   version_committed  true when a release commit/tag was created
 *   already_released   true when the tag already existed
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\Actions;
use LinkFoundation\Template\Pipeline\Cli;
use LinkFoundation\Template\Pipeline\Project;
use LinkFoundation\Template\Pipeline\VersionReleaser;

$options = getopt('', ['mode::', 'bump::', 'description::', 'skip-bump', 'date::']);

$project = Project::locate();

// Self-heal path: do not bump, just surface the current version so the
// downstream release steps can re-publish it.
if (Cli::flag($options, 'skip-bump')) {
    $version = $project->version();
    Actions::setOutput('new_version', $version);
    Actions::setBoolOutput('version_committed', false);
    Actions::setBoolOutput('already_released', false);
    echo "Skip-bump: releasing current version {$version} without a new commit.\n";
    exit(0);
}

$mode = Cli::string($options, 'mode', 'changeset') ?? 'changeset';

if (!in_array($mode, ['changeset', 'instant'], true)) {
    fwrite(\STDERR, "Invalid --mode: {$mode}. Use 'changeset' or 'instant'.\n");
    exit(1);
}

$bump = Cli::string($options, 'bump');
$description = Cli::string($options, 'description');
$date = Cli::string($options, 'date', gmdate('Y-m-d')) ?? gmdate('Y-m-d');

$releaser = VersionReleaser::forProject($project);
$result = $releaser->release($mode, $bump, $description, $date);

Actions::setOutput('new_version', $result['new_version']);
Actions::setBoolOutput('version_committed', $result['version_committed']);
Actions::setBoolOutput('already_released', $result['already_released']);

echo "New version       : {$result['new_version']}\n";
echo 'Version committed : ' . ($result['version_committed'] ? 'yes' : 'no') . "\n";
echo 'Already released  : ' . ($result['already_released'] ? 'yes' : 'no') . "\n";
