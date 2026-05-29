"""Fetch JSON from a URL and report the mean and median of every number in it.

Dependencies:  pip install requests
"""

import statistics
import sys

import requests


def collect_numbers(value):
    """Recursively collect every int/float out of a decoded JSON value."""
    if isinstance(value, bool):  # bool subclasses int, so skip it explicitly
        return []
    if isinstance(value, (int, float)):
        return [float(value)]
    if isinstance(value, list):
        return [number for item in value for number in collect_numbers(item)]
    if isinstance(value, dict):
        return [number for item in value.values() for number in collect_numbers(item)]
    return []


def main():
    # 1. Read the target URL from the first command-line argument.
    if len(sys.argv) < 2:
        raise SystemExit("usage: stats.py <url-returning-json>")
    url = sys.argv[1]

    # 2. Make the HTTP GET request and parse the JSON body, turning any HTTP
    #    error status into an exception before we try to decode it.
    response = requests.get(url, timeout=30)
    response.raise_for_status()
    document = response.json()

    # 3. Gather every number, then guard against an empty data set.
    numbers = collect_numbers(document)
    if not numbers:
        raise SystemExit("the JSON response contained no numbers")

    # 4. Compute and print the statistics.
    print(f"count:  {len(numbers)}")
    print(f"mean:   {statistics.mean(numbers):.4f}")
    print(f"median: {statistics.median(numbers):.4f}")


if __name__ == "__main__":
    main()
