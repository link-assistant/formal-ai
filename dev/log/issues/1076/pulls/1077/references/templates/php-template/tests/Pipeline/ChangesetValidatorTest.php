<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Pipeline;

use LinkFoundation\Template\Pipeline\ChangelogFragments;
use LinkFoundation\Template\Pipeline\ChangesetValidator;
use PHPUnit\Framework\TestCase;

final class ChangesetValidatorTest extends TestCase
{
    private string $dir;

    protected function setUp(): void
    {
        $this->dir = sys_get_temp_dir() . '/cv-' . bin2hex(random_bytes(6));
        mkdir($this->dir);
    }

    protected function tearDown(): void
    {
        foreach (glob($this->dir . '/*') ?: [] as $file) {
            @unlink($file);
        }

        @rmdir($this->dir);
    }

    private function validator(): ChangesetValidator
    {
        return new ChangesetValidator(new ChangelogFragments($this->dir));
    }

    public function testMissingFragmentFails(): void
    {
        $result = $this->validator()->validate(0);

        self::assertFalse($result['ok']);
        self::assertNotEmpty($result['errors']);
    }

    public function testValidSingleFragmentPasses(): void
    {
        file_put_contents($this->dir . '/a.md', "---\nbump: minor\n---\n### Added\n- A.");

        $result = $this->validator()->validate(1);

        self::assertTrue($result['ok']);
        self::assertSame([], $result['errors']);
    }

    public function testMultipleFragmentsWarnButPass(): void
    {
        file_put_contents($this->dir . '/a.md', "---\nbump: minor\n---\n- A.");
        file_put_contents($this->dir . '/b.md', "---\nbump: patch\n---\n- B.");

        $result = $this->validator()->validate(2);

        self::assertTrue($result['ok']);
        self::assertNotEmpty($result['warnings']);
    }

    public function testMissingBumpIsAnError(): void
    {
        file_put_contents($this->dir . '/a.md', "### Added\n- No frontmatter.");

        $result = $this->validator()->validate(1);

        self::assertFalse($result['ok']);
    }

    public function testEmptyBodyIsAnError(): void
    {
        file_put_contents($this->dir . '/a.md', "---\nbump: patch\n---\n");

        $result = $this->validator()->validate(1);

        self::assertFalse($result['ok']);
    }
}
