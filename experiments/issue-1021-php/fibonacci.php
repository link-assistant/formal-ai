<?php

function fibonacci(int $n): int
{
    if ($n <= 2) {
        return 1;
    }

    return fibonacci($n - 1) + fibonacci($n - 2);
}

echo fibonacci(10), PHP_EOL;
