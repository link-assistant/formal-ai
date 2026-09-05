#!/usr/bin/env bash
# Verify the catalog's Laravel hello-world template inside a real Laravel
# application (issue #723, reported as "напиши мне код на PHP Laravel").
#
# The catalog claims `ExecutionStatus::Verified` for the `laravel` target, and
# that claim is only worth what a reader can reproduce: this script builds the
# project the answer's setup hint names, drops in the template the answer
# renders, and runs the command the answer prints -- then compares the bytes to
# the task's expected output.
#
# Usage: run.sh [work-dir]   (defaults to a fresh mktemp directory)
set -euo pipefail

WORK="${1:-$(mktemp -d -t issue-1021-laravel-XXXXXX)}"
mkdir -p "$WORK"
cd "$WORK"

if ! command -v composer >/dev/null 2>&1; then
  # Retries for the same reason every other install here does: a truncated
  # transfer is a dropped connection, and only `--retry-all-errors` covers it.
  curl -sS --retry 3 --retry-delay 2 --retry-all-errors -o composer-setup.php https://getcomposer.org/installer
  php composer-setup.php --quiet --install-dir="$WORK" --filename=composer
  PATH="$WORK:$PATH"
  export PATH
fi

composer create-project laravel/laravel app --no-interaction --quiet
cd app

mkdir -p app/Console/Commands
cat > app/Console/Commands/HelloWorld.php <<'PHP'
<?php

namespace App\Console\Commands;

use Illuminate\Console\Command;

class HelloWorld extends Command
{
    protected $signature = 'hello:world';

    protected $description = 'Print a greeting';

    public function handle(): int
    {
        $this->line('Hello, world!');

        return self::SUCCESS;
    }
}
PHP

echo "php: $(php --version | head -1)"
echo "laravel: $(php artisan --version)"

php -l app/Console/Commands/HelloWorld.php
actual="$(php artisan hello:world)"
expected="Hello, world!"

if [ "$actual" = "$expected" ]; then
  echo "PASS laravel/hello_world: $actual"
else
  echo "FAIL laravel/hello_world: expected [$expected], got [$actual]"
  exit 1
fi
