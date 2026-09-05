<?php

declare(strict_types=1);

/**
 * Shared bootstrap for the PHP CI/CD scripts. Loads Composer's autoloader so
 * the pipeline classes under scripts/src/ (and the package under src/) are
 * available to every entrypoint.
 *
 * The pipeline classes have no third-party runtime dependencies, so when the
 * Composer autoloader is absent (e.g. the lightweight link-checker workflow,
 * which never runs `composer install`) we fall back to a tiny PSR-4 autoloader
 * covering this template's namespaces. Scripts that genuinely need vendor code
 * still fail loudly when they try to use it.
 */

$root = dirname(__DIR__);
$autoload = $root . '/vendor/autoload.php';

if (is_file($autoload)) {
    require_once $autoload;
} else {
    spl_autoload_register(static function (string $class) use ($root): void {
        /** @var array<string, string> $prefixes */
        $prefixes = [
            'LinkFoundation\\Template\\Pipeline\\' => $root . '/scripts/src/',
            'LinkFoundation\\Template\\Tests\\' => $root . '/tests/',
            'LinkFoundation\\Template\\' => $root . '/src/',
        ];

        foreach ($prefixes as $prefix => $baseDir) {
            if (!str_starts_with($class, $prefix)) {
                continue;
            }

            $relative = substr($class, strlen($prefix));
            $file = $baseDir . str_replace('\\', '/', $relative) . '.php';

            if (is_file($file)) {
                require_once $file;
            }

            return;
        }
    });
}
