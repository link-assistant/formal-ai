<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Orchestrates a version bump: compute the new version, update composer.json
 * and CHANGELOG.md, then commit, tag and push.
 *
 * Two modes:
 *  - changeset: derive the bump and changelog body from changelog.d/ fragments.
 *  - instant:   use an explicit bump type and a one-line description.
 *
 * The git tag is the idempotency guard: if `v<new>` already exists, the run is
 * treated as already released and nothing is re-committed.
 */
final class VersionReleaser
{
    public function __construct(
        private readonly Project $project,
        private readonly ChangelogFragments $fragments,
        private readonly Changelog $changelog,
        private readonly Git $git,
        private readonly string $tagPrefix = 'v',
    ) {
    }

    public static function forProject(Project $project): self
    {
        return new self(
            $project,
            ChangelogFragments::forProject($project),
            Changelog::forProject($project),
            new Git($project->root()),
        );
    }

    /**
     * @return array{version_committed: bool, already_released: bool, new_version: string}
     */
    public function release(string $mode, ?string $bumpType, ?string $description, string $date): array
    {
        $current = $this->project->version();

        if ($mode === 'changeset') {
            $bump = $this->fragments->determineBump();
            $body = $this->fragments->collectBody();
        } else {
            $bump = $bumpType ?? 'patch';
            $body = $this->describeInstant($bump, $description);
        }

        $newVersion = (string) SemVer::parse($current)->bump($bump);
        $tag = $this->tagPrefix . $newVersion;

        if ($this->git->tagExists($tag)) {
            Actions::notice("Tag {$tag} already exists; treating as already released.");

            return [
                'version_committed' => false,
                'already_released' => true,
                'new_version' => $newVersion,
            ];
        }

        $this->project->setVersion($newVersion);
        $this->changelog->insert(Changelog::entry($newVersion, $body, $date));

        $paths = ['composer.json', 'CHANGELOG.md'];

        if ($mode === 'changeset') {
            $this->fragments->clear();
            $paths[] = 'changelog.d';
        }

        $this->git->configureBotIdentity();
        $this->git->add($paths);
        $this->git->commit("chore: release {$tag}");
        $this->git->tag($tag, "Release {$newVersion}");
        $this->git->push();
        $this->git->pushTags();

        return [
            'version_committed' => true,
            'already_released' => false,
            'new_version' => $newVersion,
        ];
    }

    private function describeInstant(string $bump, ?string $description): string
    {
        $category = match ($bump) {
            'major' => 'Changed',
            'minor' => 'Added',
            default => 'Fixed',
        };

        $text = $description !== null && trim($description) !== ''
            ? trim($description)
            : 'Maintenance release.';

        return "### {$category}\n- {$text}";
    }
}
