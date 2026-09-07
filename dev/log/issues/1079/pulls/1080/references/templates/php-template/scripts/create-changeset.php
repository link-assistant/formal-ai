<?php

declare(strict_types=1);

/**
 * Create a changelog fragment in changelog.d/ (the PHP equivalent of
 * `npx changeset`).
 *
 * Usage:
 *   php scripts/create-changeset.php --bump=minor --message="Add foo support"
 *   php scripts/create-changeset.php            # interactive prompts
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\ChangelogFragments;
use LinkFoundation\Template\Pipeline\Cli;
use LinkFoundation\Template\Pipeline\Process;
use LinkFoundation\Template\Pipeline\Project;
use LinkFoundation\Template\Pipeline\SemVer;

$options = getopt('', ['bump::', 'message::', 'category::']);

$bump = Cli::string($options, 'bump') ?? prompt('Bump type (major/minor/patch) [patch]: ', 'patch');
$bump = strtolower(trim($bump));

if (!in_array($bump, SemVer::BUMP_TYPES, true)) {
    fwrite(\STDERR, "Invalid bump type: {$bump}. Use major, minor or patch.\n");
    exit(1);
}

$message = Cli::string($options, 'message') ?? prompt('Describe the change: ', '');
$message = trim($message);

if ($message === '') {
    fwrite(\STDERR, "A description is required.\n");
    exit(1);
}

$category = Cli::string($options, 'category') ?? defaultCategory($bump);

$project = Project::locate();
$fragments = ChangelogFragments::forProject($project);
@mkdir($fragments->directory(), 0o775, true);

$timestamp = gmdate('Ymd_His');
$branch = currentBranchSlug($project->root());
$suffix = substr(bin2hex(random_bytes(4)), 0, 8);
$filename = "{$timestamp}_{$branch}_{$suffix}.md";
$path = $fragments->directory() . '/' . $filename;

$contents = "---\nbump: {$bump}\n---\n### {$category}\n- {$message}\n";
file_put_contents($path, $contents);

echo "Created changelog fragment: changelog.d/{$filename}\n";

/**
 * @return non-empty-string
 */
function defaultCategory(string $bump): string
{
    return match ($bump) {
        'major' => 'Changed',
        'minor' => 'Added',
        default => 'Fixed',
    };
}

function currentBranchSlug(string $cwd): string
{
    $branch = Process::run(['git', 'rev-parse', '--abbrev-ref', 'HEAD'], $cwd)->output();
    $slug = preg_replace('/[^a-z0-9]+/i', '_', $branch) ?? 'change';

    return trim(strtolower($slug), '_') ?: 'change';
}

function prompt(string $label, string $default): string
{
    if (!stream_isatty(\STDIN)) {
        return $default;
    }

    echo $label;
    $line = fgets(\STDIN);

    if ($line === false) {
        return $default;
    }

    $line = trim($line);

    return $line === '' ? $default : $line;
}
