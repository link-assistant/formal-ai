<?php

declare(strict_types=1);

/**
 * Decide whether the release job should run.
 *
 * Uses Packagist + GitHub Releases as the source of truth (never git tags), so
 * the decision is idempotent and self-healing. Exposes the decision as step
 * outputs consumed by the release job in the workflow.
 *
 * Outputs:
 *   should_release  true|false
 *   skip_bump       true|false  (self-heal: release the current version as-is)
 *   reason          human-readable explanation
 *   current_version the version currently in composer.json
 */

require_once __DIR__ . '/bootstrap.php';

use LinkFoundation\Template\Pipeline\Actions;
use LinkFoundation\Template\Pipeline\ChangelogFragments;
use LinkFoundation\Template\Pipeline\Project;
use LinkFoundation\Template\Pipeline\ReleaseDecider;

$project = Project::locate();
$fragments = ChangelogFragments::forProject($project);
$hasChangesets = $fragments->count() > 0;

$decider = new ReleaseDecider($project);
$decision = $decider->evaluate($hasChangesets);

Actions::setBoolOutput('should_release', $decision->shouldRelease);
Actions::setBoolOutput('skip_bump', $decision->skipBump);
Actions::setOutput('reason', $decision->reason);
Actions::setOutput('current_version', $project->version());

echo 'Changesets present : ' . ($hasChangesets ? 'yes' : 'no') . "\n";
echo 'Should release     : ' . ($decision->shouldRelease ? 'yes' : 'no') . "\n";
echo 'Skip bump          : ' . ($decision->skipBump ? 'yes' : 'no') . "\n";
echo 'Reason             : ' . $decision->reason . "\n";

Actions::summary("### Release decision\n\n- **Should release:** "
    . ($decision->shouldRelease ? 'yes' : 'no')
    . "\n- **Reason:** {$decision->reason}");
