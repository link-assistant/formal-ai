//! Templates for the catalog's framework targets — rows whose
//! [`ProgramLanguage::framework_of`](super::ProgramLanguage::framework_of) names
//! the language they are written in.
//!
//! Issue #723 asked for Laravel. A framework's template is not its language's
//! template with a different file name: a Laravel program lives inside a Laravel
//! application, in the directory Laravel discovers it from, and is run by the
//! command that application ships. That is exactly what a request naming the
//! framework is asking to be shown, so it is what the template carries.
//!
//! Coverage here is deliberately sparse: a framework earns a task template only
//! once the task has been run inside a real application of that framework
//! (`experiments/issue-1021-laravel/run.sh`). A task with no template for the
//! framework reaches the honest dead end
//! ([`crate::program_skill_gap`]) rather than an answer in the base language
//! wearing the framework's name.

use super::types::ProgramTemplate;

pub const TEMPLATES_FRAMEWORK: &[ProgramTemplate] = &[ProgramTemplate {
    task_slug: "hello_world",
    language_slug: "laravel",
    code: r"<?php

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
}",
}];
