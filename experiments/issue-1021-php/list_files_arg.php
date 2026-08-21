<?php

$path = $argv[1] ?? ".";
$names = array_filter(scandir($path), fn($name) => is_file($path . DIRECTORY_SEPARATOR . $name));
sort($names);
foreach ($names as $name) {
    echo $name, PHP_EOL;
}
