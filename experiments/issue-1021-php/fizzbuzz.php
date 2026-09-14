<?php

foreach (range(1, 15) as $number) {
    if ($number % 15 === 0) {
        echo "FizzBuzz", PHP_EOL;
    } elseif ($number % 3 === 0) {
        echo "Fizz", PHP_EOL;
    } elseif ($number % 5 === 0) {
        echo "Buzz", PHP_EOL;
    } else {
        echo $number, PHP_EOL;
    }
}
