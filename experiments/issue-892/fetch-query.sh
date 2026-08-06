#!/usr/bin/env bash
# Wikidata Query Service fetch with retries: the shared runner IP is
# rate-limited/blocked intermittently (HTTP 403), so retry until a 200 arrives.
set -u
query_file="$1"
output_file="$2"
for attempt in $(seq 1 40); do
  code=$(curl -sS -o "$output_file.raw" -w "%{http_code}" \
    -H 'Accept: application/sparql-results+json' \
    -A 'formal-ai-grounding/1.0 (https://github.com/link-assistant/formal-ai)' \
    --data-urlencode "query@$query_file" \
    https://query.wikidata.org/sparql) || code=000
  if [ "$code" = "200" ]; then
    mv "$output_file.raw" "$output_file"
    echo "attempt $attempt: 200"
    exit 0
  fi
  echo "attempt $attempt: $code"
  sleep 15
done
exit 1
