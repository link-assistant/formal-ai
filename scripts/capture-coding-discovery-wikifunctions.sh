#!/usr/bin/env bash
set -euo pipefail

root="${1:-tests/fixtures/coding-discovery/wikifunctions}"
mkdir -p "$root"

base="https://www.wikifunctions.org/w/api.php"
user_agent="formal-ai/${FORMAL_AI_VERSION:-coding-discovery} (https://github.com/link-assistant/formal-ai; fixture capture)"

capture() {
  local file="$1"
  local url="$2"
  curl --fail --silent --show-error --location --compressed --max-time 30 \
    --user-agent "$user_agent" "$url" --output "$root/$file"
}

search_gcd="$base?action=query&format=json&formatversion=2&list=wikilambdasearch_functions&wikilambdasearch_functions_search=greatest%20common%20divisor&wikilambdasearch_functions_language=en&wikilambdasearch_functions_limit=5"
search_empty="$base?action=query&format=json&formatversion=2&list=wikilambdasearch_functions&wikilambdasearch_functions_search=qzxjvkplmnbvcx&wikilambdasearch_functions_language=en&wikilambdasearch_functions_limit=5"
search_product="$base?action=query&format=json&formatversion=2&list=wikilambdasearch_functions&wikilambdasearch_functions_search=product%20of%20list&wikilambdasearch_functions_language=en&wikilambdasearch_functions_limit=5"
fetch_function="$base?action=wikilambda_fetch&format=json&zids=Z13612&language=en"
fetch_implementations="$base?action=wikilambda_fetch&format=json&zids=Z14707%7CZ14857%7CZ29084%7CZ13642%7CZ13639&language=en"
fetch_testers="$base?action=wikilambda_fetch&format=json&zids=Z13614%7CZ13615%7CZ13616%7CZ13613&language=en"

capture search-greatest-common-divisor-en.json "$search_gcd"
capture search-no-match-en.json "$search_empty"
capture search-product-of-list-en.json "$search_product"
capture fetch-Z13612-en.json "$fetch_function"
capture fetch-gcd-implementations-en.json "$fetch_implementations"
capture fetch-gcd-testers-en.json "$fetch_testers"

captured_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
manifest="$root/capture-manifest.lino"
{
  echo "capture_manifest"
  for record in \
    "search-greatest-common-divisor-en.json|$search_gcd|CC0-1.0" \
    "search-no-match-en.json|$search_empty|CC0-1.0" \
    "search-product-of-list-en.json|$search_product|CC0-1.0" \
    "fetch-Z13612-en.json|$fetch_function|CC0-1.0" \
    "fetch-gcd-implementations-en.json|$fetch_implementations|Apache-2.0" \
    "fetch-gcd-testers-en.json|$fetch_testers|CC0-1.0"
  do
    IFS='|' read -r file url license <<<"$record"
    sha256="$(shasum -a 256 "$root/$file" | awk '{print $1}')"
    bytes="$(wc -c <"$root/$file" | tr -d ' ')"
    echo "  capture \"$file\""
    echo "    url \"$url\""
    echo "    fetched_at \"$captured_at\""
    echo "    sha256 \"$sha256\""
    echo "    bytes \"$bytes\""
    echo "    license \"$license\""
  done
} >"$manifest"
