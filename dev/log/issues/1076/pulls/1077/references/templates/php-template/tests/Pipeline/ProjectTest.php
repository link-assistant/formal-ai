<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\Project;
use PHPUnit\Framework\TestCase;

final class ProjectTest extends TestCase
{
    private string $root;

    protected function setUp(): void
    {
        $this->root = sys_get_temp_dir() . '/proj-' . bin2hex(random_bytes(6));
        mkdir($this->root);
    }

    protected function tearDown(): void
    {
        @unlink($this->root . '/composer.json');
        @rmdir($this->root);
    }

    private function writeComposer(string $name, string $version): Project
    {
        $json = json_encode(['name' => $name, 'version' => $version], \JSON_PRETTY_PRINT);
        self::assertIsString($json);
        file_put_contents($this->root . '/composer.json', $json . "\n");

        return new Project($this->root);
    }

    public function testReadsNameAndVersion(): void
    {
        $project = $this->writeComposer('vendor/pkg', '1.2.3');

        self::assertSame('vendor/pkg', $project->packageName());
        self::assertSame('1.2.3', $project->version());
    }

    public function testSetVersionRewritesOnlyTheVersionField(): void
    {
        $project = $this->writeComposer('vendor/pkg', '1.2.3');
        $project->setVersion('2.0.0');

        self::assertSame('2.0.0', $project->version());
        // Name must be untouched.
        self::assertSame('vendor/pkg', $project->packageName());
    }

    public function testDetectsTemplateSentinel(): void
    {
        $sentinel = $this->writeComposer(Project::SENTINEL_PACKAGE_NAME, '0.1.0');
        self::assertTrue($sentinel->isTemplateSentinel());

        $renamed = $this->writeComposer('vendor/real', '0.1.0');
        self::assertFalse($renamed->isTemplateSentinel());
    }
}
