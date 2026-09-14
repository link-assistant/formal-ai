<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Decides whether a release is needed, using the registry and GitHub Releases
 * API as the source of truth rather than git tags.
 *
 * This makes releases idempotent and self-healing: if a previous run bumped the
 * version and pushed a tag but failed before Packagist imported it (or before
 * the GitHub Release was created), the next push to main resumes the release
 * *without* re-bumping the version.
 */
final class ReleaseDecider
{
    /**
     * Pure decision logic — no I/O, fully unit-testable.
     */
    public static function decide(
        bool $hasChangesets,
        bool $publishedOnPackagist,
        bool $githubReleaseExists,
        bool $isSentinel,
    ): ReleaseDecision {
        if ($hasChangesets) {
            return new ReleaseDecision(true, false, 'Changesets present: a new version will be released.');
        }

        if ($isSentinel) {
            // Template default package: never auto-publish, even for self-heal.
            return new ReleaseDecision(false, false, 'Template sentinel package name: skipping release.');
        }

        if (!$publishedOnPackagist) {
            return new ReleaseDecision(true, true, 'Self-heal: current version is not on Packagist yet.');
        }

        if (!$githubReleaseExists) {
            return new ReleaseDecision(true, true, 'Self-heal: GitHub release is missing for the current version.');
        }

        return new ReleaseDecision(false, false, 'Up to date: no changesets and current version already released.');
    }

    public function __construct(
        private readonly Project $project,
        private readonly Packagist $packagist = new Packagist(),
        private readonly ?GitHub $github = null,
        private readonly string $tagPrefix = 'v',
    ) {
    }

    /**
     * Run the decision against the live registry + GitHub state.
     */
    public function evaluate(bool $hasChangesets): ReleaseDecision
    {
        $version = $this->project->version();
        $packageName = $this->project->packageName();
        $isSentinel = $this->project->isTemplateSentinel();

        $publishedOnPackagist = false;
        $githubReleaseExists = false;

        if (!$isSentinel) {
            $publishedOnPackagist = $this->packagist->hasVersion($packageName, $version);
            $github = $this->github ?? GitHub::fromEnv();
            $githubReleaseExists = $github->releaseExists($this->tagPrefix . $version);
        }

        return self::decide($hasChangesets, $publishedOnPackagist, $githubReleaseExists, $isSentinel);
    }
}
