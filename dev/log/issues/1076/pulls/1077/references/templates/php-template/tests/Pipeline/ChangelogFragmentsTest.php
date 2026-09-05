<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\ChangelogFragments;
use PHPUnit\Framework\TestCase;

final class ChangelogFragmentsTest extends TestCase
{
    private string $dir;

    protected function setUp(): void
    {
        $this->dir = sys_get_temp_dir() . '/cf-' . bin2hex(random_bytes(6));
        mkdir($this->dir);
    }

    protected function tearDown(): void
    {
        foreach (glob($this->dir . '/*') ?: [] as $file) {
            @unlink($file);
        }

        @rmdir($this->dir);
    }

    private function write(string $name, string $contents): void
    {
        file_put_contents($this->dir . '/' . $name, $contents);
    }

    public function testParsesFrontmatterBump(): void
    {
        $parsed = ChangelogFragments::parseContents("---\nbump: minor\n---\n### Added\n- Thing.");

        self::assertSame('minor', $parsed['bump']);
        self::assertStringContainsString('- Thing.', $parsed['body']);
    }

    public function testInvalidBumpBecomesNull(): void
    {
        $parsed = ChangelogFragments::parseContents("---\nbump: huge\n---\n- Thing.");

        self::assertNull($parsed['bump']);
    }

    public function testDeterminesHighestBumpAcrossFragments(): void
    {
        $this->write('a.md', "---\nbump: patch\n---\n### Fixed\n- A.");
        $this->write('b.md', "---\nbump: minor\n---\n### Added\n- B.");

        $fragments = new ChangelogFragments($this->dir);

        self::assertSame(2, $fragments->count());
        self::assertSame('minor', $fragments->determineBump());
    }

    public function testReservedFilesAreNotFragments(): void
    {
        $this->write('README.md', '# docs');
        $this->write('fragment_template.md', 'template');
        $this->write('real.md', "---\nbump: patch\n---\n- Real.");

        $fragments = new ChangelogFragments($this->dir);

        self::assertSame(1, $fragments->count());
    }

    public function testCollectBodyGroupsByCategory(): void
    {
        $this->write('a.md', "---\nbump: minor\n---\n### Added\n- New A.");
        $this->write('b.md', "---\nbump: patch\n---\n### Fixed\n- Bug B.");
        $this->write('c.md', "---\nbump: minor\n---\n### Added\n- New C.");

        $body = (new ChangelogFragments($this->dir))->collectBody();

        self::assertStringContainsString("### Added\n- New A.\n- New C.", $body);
        self::assertStringContainsString("### Fixed\n- Bug B.", $body);
        // Added precedes Fixed in Keep a Changelog order.
        self::assertLessThan(strpos($body, '### Fixed'), strpos($body, '### Added'));
    }

    public function testClearDeletesFragments(): void
    {
        $this->write('a.md', "---\nbump: patch\n---\n- A.");
        $fragments = new ChangelogFragments($this->dir);
        $fragments->clear();

        self::assertSame(0, $fragments->count());
    }
}
