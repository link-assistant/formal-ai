<?php

declare(strict_types=1);

/**
 * Validate that the current PR adds exactly one well-formed changelog fragment.
 *
 * Exit code 1 fails the job. The workflow only runs this for code PRs, so
 * docs-only changes are exempt.
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\Actions;
use LinkFoundation\Template\Pipeline\ChangelogFragments;
use LinkFoundation\Template\Pipeline\ChangesetValidator;
use LinkFoundation\Template\Pipeline\Project;

$project = Project::locate();
$fragments = ChangelogFragments::forProject($project);
$validator = new ChangesetValidator($fragments);

$added = ChangesetValidator::addedFragments($project->root());
$addedCount = $added === [] ? null : count($added);

$result = $validator->validate($addedCount);

foreach ($result['warnings'] as $warning) {
    Actions::warning($warning);
}

foreach ($result['errors'] as $error) {
    Actions::error($error);
}

if (!$result['ok']) {
    fwrite(\STDERR, "Changeset validation failed.\n");
    fwrite(\STDERR, "Add a fragment with: composer changeset\n");
    exit(1);
}

echo "Changeset validation passed.\n";
