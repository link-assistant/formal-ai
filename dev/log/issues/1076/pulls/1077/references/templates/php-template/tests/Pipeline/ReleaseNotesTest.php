<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\ReleaseNotes;
use PHPUnit\Framework\TestCase;

final class ReleaseNotesTest extends TestCase
{
    public function testBadgeIsLowercasedAndLinksToPackagist(): void
    {
        $badge = ReleaseNotes::packagistBadge('Vendor/Pkg');

        self::assertStringContainsString('packagist/v/vendor/pkg.svg', $badge);
        self::assertStringContainsString('packagist.org/packages/vendor/pkg', $badge);
    }

    public function testAddBadgeIsIdempotent(): void
    {
        $once = ReleaseNotes::addBadge('Body.', 'vendor/pkg');
        $twice = ReleaseNotes::addBadge($once, 'vendor/pkg');

        self::assertSame(substr_count($once, 'img.shields.io'), substr_count($twice, 'img.shields.io'));
    }

    public function testBuildSkipsBadgeForSentinel(): void
    {
        $notes = ReleaseNotes::build('1.0.0', "### Added\n- Thing.", 'vendor/pkg', 'owner/repo', 'v1.0.0', true);

        self::assertStringNotContainsString('img.shields.io', $notes);
        self::assertStringContainsString('- Thing.', $notes);
    }

    public function testBuildFallsBackWhenSectionEmpty(): void
    {
        $notes = ReleaseNotes::build('1.0.0', null, 'vendor/pkg', 'owner/repo', 'v1.0.0', true);

        self::assertStringContainsString('Release 1.0.0.', $notes);
    }

    public function testCapBytesTruncatesAndLinksToChangelog(): void
    {
        $body = str_repeat('a', 5000);
        $capped = ReleaseNotes::capBytes($body, 'owner/repo', 'v1.0.0', 4000);

        self::assertLessThanOrEqual(4000, \strlen($capped));
        self::assertStringContainsString('CHANGELOG.md', $capped);
        self::assertStringContainsString('owner/repo', $capped);
    }

    public function testCapBytesLeavesShortBodyUntouched(): void
    {
        $body = 'short';

        self::assertSame($body, ReleaseNotes::capBytes($body, 'owner/repo', 'v1.0.0', 100));
    }
}
