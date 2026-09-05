<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Unit;

use LinkFoundation\Template\PackageInfo;
use PHPUnit\Framework\TestCase;

final class PackageInfoTest extends TestCase
{
    public function testReturnsSemanticVersionFromComposerJson(): void
    {
        $version = PackageInfo::version();

        self::assertMatchesRegularExpression('/^\d+\.\d+\.\d+/', $version);
    }
}
