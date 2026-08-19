<?php

$names = array_filter(scandir("."), "is_file");
sort($names);
foreach ($names as $name) {
    echo $name, PHP_EOL;
}
