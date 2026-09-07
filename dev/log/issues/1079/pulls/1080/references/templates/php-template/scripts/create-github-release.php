<?php

declare(strict_types=1);

/**
 * Create (or self-heal) the GitHub Release for a version, with notes taken from
 * the matching CHANGELOG.md section and a Packagist badge prepended.
 *
 * Creation is idempotent: if the release already exists it is treated as a
 * success, so re-running the pipeline never errors.
 *
 * Usage:
 *   php scripts/create-github-release.php --version=1.2.3 [--repo=owner/name]
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\Actions;
use LinkFoundation\Template\Pipeline\Changelog;
use LinkFoundation\Template\Pipeline\Cli;
use LinkFoundation\Template\Pipeline\GitHub;
use LinkFoundation\Template\Pipeline\Project;
use LinkFoundation\Template\Pipeline\ReleaseNotes;

$options = getopt('', ['version::', 'repo::', 'tag-prefix::']);

$project = Project::locate();
$version = Cli::string($options, 'version') ?? $project->version();
$tagPrefix = Cli::string($options, 'tag-prefix', 'v') ?? 'v';
$tag = $tagPrefix . $version;

$repository = Cli::string($options, 'repo')
    ?? (getenv('GITHUB_REPOSITORY') ?: 'link-foundation/php-ai-driven-development-pipeline-template');

$isSentinel = $project->isTemplateSentinel();
$packageName = $project->packageName();

$changelog = Changelog::forProject($project);
$section = $changelog->section($version);

$notes = ReleaseNotes::build($version, $section, $packageName, $repository, $tag, $isSentinel);

$notesFile = tempnam(sys_get_temp_dir(), 'release-notes-') ?: (sys_get_temp_dir() . '/release-notes.md');
file_put_contents($notesFile, $notes);

$github = GitHub::fromEnv($repository);

echo "Creating GitHub release {$tag} for {$repository}...\n";
$ok = $github->createRelease($tag, $version, $notesFile);

@unlink($notesFile);

if (!$ok) {
    fwrite(\STDERR, "Failed to create GitHub release {$tag}.\n");
    exit(1);
}

Actions::summary("### Released {$tag}\n\nGitHub release published for `{$repository}`.");
echo "GitHub release {$tag} is published.\n";
