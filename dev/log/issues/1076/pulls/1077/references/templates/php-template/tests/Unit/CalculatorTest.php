<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Tests\Unit;

use LinkFoundation\Template\Calculator;
use PHPUnit\Framework\TestCase;

final class CalculatorTest extends TestCase
{
    public function testAddsTwoNumbers(): void
    {
        $calculator = new Calculator();

        self::assertSame(5.0, $calculator->add(2, 3));
        self::assertSame(0.0, $calculator->add(-2, 2));
    }

    public function testMultipliesTwoNumbers(): void
    {
        $calculator = new Calculator();

        self::assertSame(6.0, $calculator->multiply(2, 3));
        self::assertSame(0.0, $calculator->multiply(0, 99));
    }
}
