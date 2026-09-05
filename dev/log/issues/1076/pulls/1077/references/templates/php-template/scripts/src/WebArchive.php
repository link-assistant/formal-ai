<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Checks broken links against the Internet Archive's Wayback Machine.
 *
 * When lychee reports a broken link we do not fail immediately: a permanent
 * snapshot on archive.org is an acceptable replacement, so the link is only a
 * hard failure when no archived copy exists. This mirrors the JavaScript
 * template's web-archive fallback, rewritten as native PHP.
 */
final class WebArchive
{
    public const AVAILABILITY_API = 'https://archive.org/wayback/available';

    /** Number of times to query the availability API before giving up. */
    public const MAX_ATTEMPTS = 3;

    /** @var callable(int): void */
    private $sleeper;

    /**
     * @param (callable(int): void)|null $sleeper backoff hook (defaults to sleep);
     *                                             injectable so tests run instantly
     */
    public function __construct(
        private readonly Http $http = new Http(),
        ?callable $sleeper = null,
    ) {
        $this->sleeper = $sleeper ?? static function (int $seconds): void {
            sleep($seconds);
        };
    }

    /**
     * Extract the broken-link URLs from a lychee Markdown report.
     *
     * lychee renders every broken link as a Markdown list item that carries its
     * failing status and the URL as an autolink, e.g.
     *
     *     * [404] <https://example.com/page> | Rejected status code: 404 Not Found
     *     * [ERROR] <https://example.com/page> | Network error: ...
     *
     * Only these entries are genuine broken links. Everything else in the report
     * must be ignored: the summary table, the section headings, and — crucially —
     * the `[Full Github Actions output](…?check_suite_focus=true)` footer that
     * lychee-action appends to the Markdown output. Probing that footer URL
     * against the Web Archive (it has no snapshot) used to fail the job even when
     * every real broken link was archived. Anchoring extraction to the status
     * tag plus the `<…>` autolink keeps us to the actual broken links.
     *
     * @return list<string>
     */
    public static function extractUrls(string $report): array
    {
        preg_match_all('#^\s*[*-]\s*\[[^\]]+\]\s*<(https?://[^>]+)>#m', $report, $matches);

        $urls = [];

        foreach ($matches[1] as $url) {
            // Trim trailing punctuation that the regex may have captured.
            $url = rtrim($url, '.,;:');

            // Ignore the archive itself to avoid recursive suggestions.
            if (str_contains($url, 'web.archive.org') || str_contains($url, 'archive.org/wayback')) {
                continue;
            }

            $urls[$url] = true;
        }

        return array_keys($urls);
    }

    /**
     * Return the archived snapshot URL for $url, or null when none exists.
     *
     * The Wayback availability API is intermittently rate-limited: instead of a
     * 429 it answers HTTP 200 with an empty `archived_snapshots` object even
     * when a snapshot exists. An empty/failed answer is therefore treated as
     * inconclusive and retried with a linear backoff before we conclude there is
     * genuinely no snapshot.
     */
    public function snapshot(string $url): ?string
    {
        $endpoint = self::AVAILABILITY_API . '?url=' . rawurlencode($url);

        for ($attempt = 1; $attempt <= self::MAX_ATTEMPTS; ++$attempt) {
            $response = $this->http->get($endpoint);

            if ($response['status'] === 200 && $response['body'] !== '') {
                /** @var array{archived_snapshots?: array{closest?: array{available?: bool, url?: string}}} $data */
                $data = json_decode($response['body'], true) ?: [];
                $closest = $data['archived_snapshots']['closest'] ?? null;

                if (\is_array($closest) && ($closest['available'] ?? false) === true && isset($closest['url'])) {
                    return (string) $closest['url'];
                }
            }

            // Inconclusive (non-200, empty body, or empty snapshot set): back off
            // and retry, unless this was the final attempt.
            if ($attempt < self::MAX_ATTEMPTS) {
                ($this->sleeper)($attempt);
            }
        }

        return null;
    }

    /**
     * Look up every URL and partition them into archived (url => snapshot) and
     * missing (list of urls with no snapshot).
     *
     * @param list<string> $urls
     * @return array{archived: array<string, string>, missing: list<string>}
     */
    public function review(array $urls): array
    {
        $archived = [];
        $missing = [];

        foreach ($urls as $url) {
            $snapshot = $this->snapshot($url);

            if ($snapshot !== null) {
                $archived[$url] = $snapshot;
            } else {
                $missing[] = $url;
            }
        }

        return ['archived' => $archived, 'missing' => $missing];
    }
}
