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
search_recurrence="$base?action=query&format=json&formatversion=2&list=wikilambdasearch_functions&wikilambdasearch_functions_search=nth%20Fibonacci%20number&wikilambdasearch_functions_language=en&wikilambdasearch_functions_limit=5"
search_factorial="$base?action=query&format=json&formatversion=2&list=wikilambdasearch_functions&wikilambdasearch_functions_search=factorial&wikilambdasearch_functions_language=en&wikilambdasearch_functions_limit=5"
fetch_function="$base?action=wikilambda_fetch&format=json&zids=Z13612&language=en"
fetch_implementations="$base?action=wikilambda_fetch&format=json&zids=Z14707%7CZ14857%7CZ29084%7CZ13642%7CZ13639&language=en"
fetch_testers="$base?action=wikilambda_fetch&format=json&zids=Z13614%7CZ13615%7CZ13616%7CZ13613&language=en"
fetch_recurrence_function="$base?action=wikilambda_fetch&format=json&zids=Z13835&language=en"
fetch_recurrence_abstract="$base?action=wikilambda_fetch&format=json&zids=Z13864&language=en"
fetch_recurrence_operators="$base?action=wikilambda_fetch&format=json&zids=Z802%7CZ13695%7CZ13521%7CZ13582%7CZ13569%7CZ13522%7CZ13539&language=en"
fetch_recurrence_testers="$base?action=wikilambda_fetch&format=json&zids=Z13869%7CZ17391%7CZ17398%7CZ34276&language=en"
fetch_factorial_function="$base?action=wikilambda_fetch&format=json&zids=Z13667&language=en"
fetch_factorial_abstract="$base?action=wikilambda_fetch&format=json&zids=Z13863&language=en"
fetch_factorial_testers="$base?action=wikilambda_fetch&format=json&zids=Z13840%7CZ13865%7CZ13866%7CZ13867&language=en"
fetch_fibonacci_concept="https://www.wikidata.org/wiki/Special:EntityData/Q23835349.json"
fetch_factorial_concept="https://www.wikidata.org/wiki/Special:EntityData/Q120976.json"

capture search-greatest-common-divisor-en.json "$search_gcd"
capture search-no-match-en.json "$search_empty"
capture search-product-of-list-en.json "$search_product"
capture search-nth-fibonacci-number-en.json "$search_recurrence"
capture search-factorial-en.json "$search_factorial"
capture fetch-Z13612-en.json "$fetch_function"
capture fetch-gcd-implementations-en.json "$fetch_implementations"
capture fetch-gcd-testers-en.json "$fetch_testers"
capture fetch-Z13835-en.json "$fetch_recurrence_function"
capture fetch-Z13864-abstract-en.json "$fetch_recurrence_abstract"
capture fetch-recurrence-operators-en.json "$fetch_recurrence_operators"
capture fetch-recurrence-testers-en.json "$fetch_recurrence_testers"
capture fetch-Z13667-en.json "$fetch_factorial_function"
capture fetch-Z13863-abstract-en.json "$fetch_factorial_abstract"
capture fetch-factorial-testers-en.json "$fetch_factorial_testers"
capture fetch-Q23835349.json "$fetch_fibonacci_concept"
capture fetch-Q120976.json "$fetch_factorial_concept"

captured_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
manifest="$root/capture-manifest.lino"
{
  echo "capture_manifest"
  for record in \
    "search-greatest-common-divisor-en.json|$search_gcd|CC0-1.0" \
    "search-no-match-en.json|$search_empty|CC0-1.0" \
    "search-product-of-list-en.json|$search_product|CC0-1.0" \
    "search-nth-fibonacci-number-en.json|$search_recurrence|CC0-1.0" \
    "search-factorial-en.json|$search_factorial|CC0-1.0" \
    "fetch-Z13612-en.json|$fetch_function|CC0-1.0" \
    "fetch-gcd-implementations-en.json|$fetch_implementations|Apache-2.0" \
    "fetch-gcd-testers-en.json|$fetch_testers|CC0-1.0" \
    "fetch-Z13835-en.json|$fetch_recurrence_function|CC0-1.0" \
    "fetch-Z13864-abstract-en.json|$fetch_recurrence_abstract|CC0-1.0" \
    "fetch-recurrence-operators-en.json|$fetch_recurrence_operators|CC0-1.0" \
    "fetch-recurrence-testers-en.json|$fetch_recurrence_testers|CC0-1.0" \
    "fetch-Z13667-en.json|$fetch_factorial_function|CC0-1.0" \
    "fetch-Z13863-abstract-en.json|$fetch_factorial_abstract|CC0-1.0" \
    "fetch-factorial-testers-en.json|$fetch_factorial_testers|CC0-1.0" \
    "fetch-Q23835349.json|$fetch_fibonacci_concept|CC0-1.0" \
    "fetch-Q120976.json|$fetch_factorial_concept|CC0-1.0"
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
