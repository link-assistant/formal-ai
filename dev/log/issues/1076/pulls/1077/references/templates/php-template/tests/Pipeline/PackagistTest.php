<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\Http;
use LinkFoundation\Template\Pipeline\Packagist;
use PHPUnit\Framework\TestCase;

final class PackagistTest extends TestCase
{
    private function packagistReturning(int $status, string $body): Packagist
    {
        $http = new Http(static fn (string $method, string $url): array => [
            'status' => $status,
            'body' => $body,
        ]);

        return new Packagist($http);
    }

    public function testMetadataUrlIsLowercasedP2(): void
    {
        $packagist = new Packagist();

        self::assertSame(
            'https://repo.packagist.org/p2/link-foundation/example.json',
            $packagist->metadataUrl('Link-Foundation/Example'),
        );
    }

    public function testPublishedVersionsAreNormalised(): void
    {
        $json = json_encode([
            'packages' => [
                'vendor/pkg' => [
                    ['version' => 'v1.2.0'],
                    ['version' => '1.1.0'],
                ],
            ],
        ]);
        self::assertIsString($json);

        $versions = $this->packagistReturning(200, $json)->publishedVersions('Vendor/Pkg');

        self::assertContains('1.2.0', $versions);
        self::assertContains('1.1.0', $versions);
    }

    public function testUnknownPackageReturnsNoVersions(): void
    {
        $versions = $this->packagistReturning(404, '')->publishedVersions('vendor/missing');

        self::assertSame([], $versions);
    }

    public function testHasVersionIgnoresLeadingV(): void
    {
        $json = json_encode(['packages' => ['vendor/pkg' => [['version' => '2.0.0']]]]);
        self::assertIsString($json);
        $packagist = $this->packagistReturning(200, $json);

        self::assertTrue($packagist->hasVersion('vendor/pkg', 'v2.0.0'));
        self::assertTrue($packagist->hasVersion('vendor/pkg', '2.0.0'));
        self::assertFalse($packagist->hasVersion('vendor/pkg', '3.0.0'));
    }
}
