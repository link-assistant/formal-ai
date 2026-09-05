<?php

declare(strict_types=1);

namespace LinkFoundation\Template\Pipeline;

/**
 * Tiny HTTP client used to probe the Packagist registry.
 *
 * Network access is wrapped behind a swappable fetcher callable so tests can
 * inject canned responses without hitting the network.
 *
 * @phpstan-type Fetcher callable(string, string): array{status:int, body:string}
 */
final class Http
{
    /** @var callable(string, string): array{status:int, body:string} */
    private $fetcher;

    /**
     * @param (callable(string, string): array{status:int, body:string})|null $fetcher
     */
    public function __construct(?callable $fetcher = null)
    {
        $this->fetcher = $fetcher ?? self::defaultFetcher();
    }

    /**
     * @return array{status:int, body:string}
     */
    public function get(string $url): array
    {
        return ($this->fetcher)('GET', $url);
    }

    /**
     * @return array{status:int, body:string}
     */
    public function head(string $url): array
    {
        return ($this->fetcher)('HEAD', $url);
    }

    /**
     * @return callable(string, string): array{status:int, body:string}
     */
    private static function defaultFetcher(): callable
    {
        return static function (string $method, string $url): array {
            $context = stream_context_create([
                'http' => [
                    'method' => $method,
                    'timeout' => 30,
                    'ignore_errors' => true,
                    'header' => "User-Agent: php-ai-driven-development-pipeline-template\r\n",
                ],
            ]);

            // PHP populates $http_response_header in the local scope after the
            // request; seed it so static analysis sees a defined list and a
            // failed request still yields an empty header set.
            $http_response_header = [];
            $body = @file_get_contents($url, false, $context);
            $status = self::statusFromHeaders($http_response_header);

            return ['status' => $status, 'body' => $body === false ? '' : $body];
        };
    }

    /**
     * @param list<string> $headers
     */
    private static function statusFromHeaders(array $headers): int
    {
        foreach ($headers as $header) {
            if (preg_match('#^HTTP/\S+\s+(\d+)#', $header, $m)) {
                return (int) $m[1];
            }
        }

        return 0;
    }
}
