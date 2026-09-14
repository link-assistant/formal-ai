<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\ChangeDetector;
use PHPUnit\Framework\TestCase;

final class ChangeDetectorTest extends TestCase
{
    public function testClassifiesPhpSourceAsCode(): void
    {
        $flags = ChangeDetector::classify(['src/Calculator.php']);

        self::assertTrue($flags['php-changed']);
        self::assertTrue($flags['any-code-changed']);
        self::assertFalse($flags['docs-changed']);
    }

    public function testDocsOnlyChangeIsNotCode(): void
    {
        $flags = ChangeDetector::classify(['docs/index.md', 'README.md']);

        self::assertTrue($flags['docs-changed']);
        self::assertFalse($flags['any-code-changed']);
        self::assertFalse($flags['php-changed']);
    }

    public function testChangelogFragmentIsNotCode(): void
    {
        $flags = ChangeDetector::classify(['changelog.d/20260101_x.md']);

        self::assertFalse($flags['any-code-changed']);
    }

    public function testWorkflowChangeCountsAsCode(): void
    {
        $flags = ChangeDetector::classify(['.github/workflows/release.yml']);

        self::assertTrue($flags['workflow-changed']);
        self::assertTrue($flags['any-code-changed']);
    }

    public function testComposerManifestIsPackageChange(): void
    {
        $flags = ChangeDetector::classify(['composer.json']);

        self::assertTrue($flags['package-changed']);
        self::assertTrue($flags['any-code-changed']);
    }

    public function testTestsDirectoryFlag(): void
    {
        $flags = ChangeDetector::classify(['tests/Unit/CalculatorTest.php']);

        self::assertTrue($flags['tests-changed']);
        self::assertTrue($flags['php-changed']);
    }
}
