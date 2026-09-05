<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\Process;
use LinkFoundation\Template\Pipeline\Project;
use LinkFoundation\Template\Pipeline\VersionReleaser;
use PHPUnit\Framework\TestCase;

/**
 * End-to-end test of the release orchestration in a throwaway git repository.
 * Pushes fail silently (no remote) by design, so the commit + tag are still
 * exercised without network access.
 */
final class VersionReleaserTest extends TestCase
{
    private string $root;

    protected function setUp(): void
    {
        $this->root = sys_get_temp_dir() . '/vr-' . bin2hex(random_bytes(6));
        mkdir($this->root . '/changelog.d', 0o775, true);

        $composer = json_encode(['name' => 'vendor/pkg', 'version' => '1.0.0'], \JSON_PRETTY_PRINT);
        self::assertIsString($composer);
        file_put_contents($this->root . '/composer.json', $composer . "\n");
        file_put_contents(
            $this->root . '/CHANGELOG.md',
            "# Changelog\n\n<!-- changelog-insert-here -->\n",
        );

        Process::mustRun(['git', 'init', '-q'], $this->root);
        Process::mustRun(['git', 'config', 'user.email', 'test@example.com'], $this->root);
        Process::mustRun(['git', 'config', 'user.name', 'Test'], $this->root);
        Process::mustRun(['git', 'add', '.'], $this->root);
        Process::mustRun(['git', 'commit', '-q', '-m', 'init'], $this->root);
    }

    protected function tearDown(): void
    {
        $this->rrmdir($this->root);
    }

    private function rrmdir(string $dir): void
    {
        foreach (scandir($dir) ?: [] as $entry) {
            if ($entry === '.' || $entry === '..') {
                continue;
            }

            $path = $dir . '/' . $entry;
            is_dir($path) ? $this->rrmdir($path) : @unlink($path);
        }

        @rmdir($dir);
    }

    public function testChangesetReleaseBumpsCommitsAndTags(): void
    {
        file_put_contents(
            $this->root . '/changelog.d/a.md',
            "---\nbump: minor\n---\n### Added\n- A new feature.",
        );

        $project = new Project($this->root);
        $result = VersionReleaser::forProject($project)->release('changeset', null, null, '2026-06-09');

        self::assertSame('1.1.0', $result['new_version']);
        self::assertTrue($result['version_committed']);
        self::assertFalse($result['already_released']);

        // composer.json was bumped.
        self::assertSame('1.1.0', $project->version());

        // CHANGELOG.md gained the entry.
        $changelog = file_get_contents($this->root . '/CHANGELOG.md');
        self::assertIsString($changelog);
        self::assertStringContainsString('## [1.1.0] - 2026-06-09', $changelog);
        self::assertStringContainsString('- A new feature.', $changelog);

        // Fragments were consumed.
        self::assertSame([], glob($this->root . '/changelog.d/*.md'));

        // Tag was created.
        $tags = Process::run(['git', 'tag'], $this->root)->output();
        self::assertStringContainsString('v1.1.0', $tags);
    }

    public function testInstantReleaseUsesExplicitBump(): void
    {
        $project = new Project($this->root);
        $result = VersionReleaser::forProject($project)
            ->release('instant', 'patch', 'Hotfix the thing.', '2026-06-09');

        self::assertSame('1.0.1', $result['new_version']);
        $changelog = file_get_contents($this->root . '/CHANGELOG.md');
        self::assertIsString($changelog);
        self::assertStringContainsString('- Hotfix the thing.', $changelog);
    }

    public function testExistingTagIsTreatedAsAlreadyReleased(): void
    {
        // VersionReleaser emits a GitHub Actions ::notice:: when the tag exists.
        $this->expectOutputRegex('/already (exists|released)/');

        Process::mustRun(['git', 'tag', 'v1.0.1'], $this->root);

        $project = new Project($this->root);
        $result = VersionReleaser::forProject($project)
            ->release('instant', 'patch', 'Anything.', '2026-06-09');

        self::assertTrue($result['already_released']);
        self::assertFalse($result['version_committed']);
        // Version unchanged because the release was skipped.
        self::assertSame('1.0.0', $project->version());
    }
}
