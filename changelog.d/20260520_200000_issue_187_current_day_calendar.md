### Fixed

- Route current-day questions such as `Какой сегодня день?` through calendar reasoning instead of the unknown fallback.
- Cover current-day prompts across every supported language (`en`, `ru`, `hi`, and `zh`) in Rust and browser tests.
- Mirror the browser worker behavior with local time-zone evidence for current date and weekday answers, and add a CI coverage guard for multilingual feature matrices.
