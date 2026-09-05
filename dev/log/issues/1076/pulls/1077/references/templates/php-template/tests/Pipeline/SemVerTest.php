<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\SemVer;
use PHPUnit\Framework\TestCase;

final class SemVerTest extends TestCase
{
    public function testParsesPlainAndPrefixedVersions(): void
    {
        self::assertSame('1.2.3', (string) SemVer::parse('1.2.3'));
        self::assertSame('1.2.3', (string) SemVer::parse('v1.2.3'));
        self::assertSame('1.2.3', (string) SemVer::parse(' V1.2.3 '));
    }

    public function testRejectsNonSemanticVersion(): void
    {
        $this->expectException(\InvalidArgumentException::class);

        SemVer::parse('not-a-version');
    }

    public function testBumpsEachComponent(): void
    {
        $version = SemVer::parse('1.2.3');

        self::assertSame('2.0.0', (string) $version->bump('major'));
        self::assertSame('1.3.0', (string) $version->bump('minor'));
        self::assertSame('1.2.4', (string) $version->bump('patch'));
    }

    public function testHighestBumpWins(): void
    {
        self::assertSame('major', SemVer::highestBump(['patch', 'major', 'minor']));
        self::assertSame('minor', SemVer::highestBump(['patch', 'minor']));
        self::assertSame('patch', SemVer::highestBump([]));
        self::assertSame('patch', SemVer::highestBump(['unknown']));
    }

    public function testCompare(): void
    {
        self::assertSame(-1, SemVer::compare('1.0.0', '1.0.1'));
        self::assertSame(0, SemVer::compare('1.2.3', 'v1.2.3'));
        self::assertSame(1, SemVer::compare('2.0.0', '1.9.9'));
    }
}
