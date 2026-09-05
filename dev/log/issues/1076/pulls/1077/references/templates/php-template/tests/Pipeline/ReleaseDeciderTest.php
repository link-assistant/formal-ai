<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\ReleaseDecider;
use PHPUnit\Framework\TestCase;

final class ReleaseDeciderTest extends TestCase
{
    public function testChangesetsAlwaysTriggerAReleaseWithBump(): void
    {
        $decision = ReleaseDecider::decide(
            hasChangesets: true,
            publishedOnPackagist: true,
            githubReleaseExists: true,
            isSentinel: false,
        );

        self::assertTrue($decision->shouldRelease);
        self::assertFalse($decision->skipBump);
    }

    public function testSentinelNeverReleasesWithoutChangesets(): void
    {
        $decision = ReleaseDecider::decide(
            hasChangesets: false,
            publishedOnPackagist: false,
            githubReleaseExists: false,
            isSentinel: true,
        );

        self::assertFalse($decision->shouldRelease);
    }

    public function testSelfHealsWhenNotOnPackagist(): void
    {
        $decision = ReleaseDecider::decide(
            hasChangesets: false,
            publishedOnPackagist: false,
            githubReleaseExists: true,
            isSentinel: false,
        );

        self::assertTrue($decision->shouldRelease);
        self::assertTrue($decision->skipBump, 'self-heal must not bump the version');
    }

    public function testSelfHealsWhenGithubReleaseMissing(): void
    {
        $decision = ReleaseDecider::decide(
            hasChangesets: false,
            publishedOnPackagist: true,
            githubReleaseExists: false,
            isSentinel: false,
        );

        self::assertTrue($decision->shouldRelease);
        self::assertTrue($decision->skipBump);
    }

    public function testNoOpWhenEverythingIsUpToDate(): void
    {
        $decision = ReleaseDecider::decide(
            hasChangesets: false,
            publishedOnPackagist: true,
            githubReleaseExists: true,
            isSentinel: false,
        );

        self::assertFalse($decision->shouldRelease);
        self::assertFalse($decision->skipBump);
    }
}
