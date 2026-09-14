<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\Changelog;
use PHPUnit\Framework\TestCase;

final class ChangelogTest extends TestCase
{
    private string $path;

    protected function setUp(): void
    {
        $this->path = sys_get_temp_dir() . '/CHANGELOG-' . bin2hex(random_bytes(6)) . '.md';
    }

    protected function tearDown(): void
    {
        @unlink($this->path);
    }

    public function testScaffoldContainsInsertMarker(): void
    {
        self::assertStringContainsString(Changelog::INSERT_MARKER, Changelog::scaffold());
    }

    public function testInsertAddsEntryBelowMarker(): void
    {
        file_put_contents($this->path, Changelog::scaffold());
        $changelog = new Changelog($this->path);

        $changelog->insert(Changelog::entry('1.0.0', "### Added\n- First.", '2026-06-09'));

        $contents = file_get_contents($this->path);
        self::assertIsString($contents);
        self::assertStringContainsString('## [1.0.0] - 2026-06-09', $contents);
        self::assertStringContainsString('- First.', $contents);
        // Marker is preserved for the next release.
        self::assertStringContainsString(Changelog::INSERT_MARKER, $contents);
    }

    public function testNewestEntryComesFirst(): void
    {
        file_put_contents($this->path, Changelog::scaffold());
        $changelog = new Changelog($this->path);

        $changelog->insert(Changelog::entry('1.0.0', "### Added\n- First.", '2026-06-01'));
        $changelog->insert(Changelog::entry('1.1.0', "### Added\n- Second.", '2026-06-09'));

        $contents = file_get_contents($this->path);
        self::assertIsString($contents);
        self::assertLessThan(strpos($contents, '## [1.0.0]'), strpos($contents, '## [1.1.0]'));
    }

    public function testSectionExtractsBodyForVersion(): void
    {
        file_put_contents($this->path, Changelog::scaffold());
        $changelog = new Changelog($this->path);

        $changelog->insert(Changelog::entry('1.0.0', "### Added\n- First.", '2026-06-01'));
        $changelog->insert(Changelog::entry('1.1.0', "### Fixed\n- Second.", '2026-06-09'));

        $section = $changelog->section('1.1.0');
        self::assertIsString($section);
        self::assertStringContainsString('- Second.', $section);
        self::assertStringNotContainsString('- First.', $section);
    }

    public function testSectionReturnsNullForUnknownVersion(): void
    {
        file_put_contents($this->path, Changelog::scaffold());

        self::assertNull((new Changelog($this->path))->section('9.9.9'));
    }
}
