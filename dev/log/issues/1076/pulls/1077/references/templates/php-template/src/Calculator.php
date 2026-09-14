<?php

declare(strict_types=1);

namespace LinkFoundation\Template;

/**
 * A tiny example API used to demonstrate the testing and release pipeline.
 *
 * Replace this class with your own package code. It exists only so the
 * template ships with a real, tested public surface that the CI/CD pipeline
 * can build, lint, test and release end-to-end.
 */
final class Calculator
{
    /**
     * Add two numbers.
     */
    public function add(float $a, float $b): float
    {
        return $a + $b;
    }

    /**
     * Multiply two numbers.
     */
    public function multiply(float $a, float $b): float
    {
        return $a * $b;
    }
}
