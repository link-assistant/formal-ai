<?php

declare(strict_types=1);

/**
 * Guard against manual version bumps in a pull request.
 *
 * The version in composer.json is owned by the release pipeline (driven by
 * changelog fragments), never by hand. If a PR edits the `version` field we
 * fail fast with a clear explanation, mirroring the "do not edit the version
 * manually" rule from the sibling templates.
 *
 * Release commits are made by the bot on the default branch, so this check only
 * runs for pull requests.
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\Actions;
use LinkFoundation\Template\Pipeline\Process;
use LinkFoundation\Template\Pipeline\Project;

/**
 * Extract the `version` field from raw composer.json text, or null when absent
 * or unparseable.
 */
function versionField(string $json): ?string
{
    $data = json_decode($json, true);

    if (is_array($data) && isset($data['version']) && is_string($data['version'])) {
        return $data['version'];
    }

    return null;
}

$project = Project::locate();
$root = $project->root();

$baseRef = getenv('GITHUB_BASE_REF') ?: '';

if ($baseRef === '') {
    echo "Not a pull request (no GITHUB_BASE_REF); skipping version-modification check.\n";
    exit(0);
}

// Make sure the base branch is available so we can read its composer.json.
Process::run(['git', 'fetch', '--no-tags', '--depth=1', 'origin', $baseRef], $root);

// Read composer.json as it exists on the base branch. A non-zero exit means the
// file does not exist there yet (it is being added in this PR), so there is no
// prior version to protect — allow it. This is what lets the template's own
// bootstrap PR, which creates composer.json for the first time, pass the guard.
$baseFile = Process::run(['git', 'show', "origin/{$baseRef}:composer.json"], $root);

if (!$baseFile->ok()) {
    echo "composer.json does not exist on the base branch (new file); skipping version-modification check.\n";
    exit(0);
}

$baseVersion = versionField($baseFile->stdout);
$headVersion = versionField((string) file_get_contents($project->composerJsonPath()));

// Compare the actual version *value*, not the textual diff: only a genuine
// change to an existing version counts as a manual bump. Reformatting,
// reordering, or leaving the version untouched is fine.
if ($baseVersion !== null && $headVersion !== null && $baseVersion !== $headVersion) {
    Actions::error(sprintf(
        'This pull request changes the "version" field in composer.json '
        . '(%s -> %s). Versions are managed automatically by the release '
        . 'pipeline from changelog fragments — add one with `composer changeset` '
        . 'instead of editing the version by hand.',
        $baseVersion,
        $headVersion,
    ));
    exit(1);
}

echo "composer.json version field is unchanged in this PR.\n";
