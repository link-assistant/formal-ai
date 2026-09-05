<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\FileSizeChecker;
use PHPUnit\Framework\TestCase;

final class FileSizeCheckerTest extends TestCase
{
    private string $root;

    protected function setUp(): void
    {
        $this->root = sys_get_temp_dir() . '/fsc-' . bin2hex(random_bytes(6));
        mkdir($this->root . '/src', 0o775, true);
        mkdir($this->root . '/vendor', 0o775, true);
    }

    protected function tearDown(): void
    {
        $this->rrmdir($this->root);
    }

    private function rrmdir(string $dir): void
    {
        foreach (glob($dir . '/*') ?: [] as $path) {
            is_dir($path) ? $this->rrmdir($path) : @unlink($path);
        }

        @rmdir($dir);
    }

    public function testReportsFilesOverTheLimit(): void
    {
        file_put_contents($this->root . '/src/Big.php', str_repeat("<?php\n", 12));
        file_put_contents($this->root . '/src/Small.php', "<?php\n");

        $violations = (new FileSizeChecker($this->root, 10))->violations();

        self::assertArrayHasKey('src/Big.php', $violations);
        self::assertArrayNotHasKey('src/Small.php', $violations);
    }

    public function testIgnoresExcludedDirectories(): void
    {
        file_put_contents($this->root . '/vendor/Huge.php', str_repeat("<?php\n", 50));

        $violations = (new FileSizeChecker($this->root, 10))->violations();

        self::assertSame([], $violations);
    }

    public function testIgnoresNonPhpFilesByDefault(): void
    {
        file_put_contents($this->root . '/src/data.txt', str_repeat("x\n", 50));

        self::assertSame([], (new FileSizeChecker($this->root, 10))->violations());
    }
}
