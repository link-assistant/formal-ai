// Fetch JSON from a URL and report the mean and median of every number in it.
//
// Requirements: Node.js 18+ (built-in global fetch; no extra packages).

// Recursively collect every finite number out of a decoded JSON value.
function collectNumbers(value) {
  if (typeof value === "number" && Number.isFinite(value)) return [value];
  if (Array.isArray(value)) return value.flatMap(collectNumbers);
  if (value && typeof value === "object") {
    return Object.values(value).flatMap(collectNumbers);
  }
  return [];
}

// Arithmetic mean of the samples (the caller guarantees a non-empty array).
function mean(samples) {
  return samples.reduce((total, sample) => total + sample, 0) / samples.length;
}

// Median of the samples; averages the two middle values for an even count.
function median(samples) {
  const sorted = [...samples].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 0
    ? (sorted[middle - 1] + sorted[middle]) / 2
    : sorted[middle];
}

async function main() {
  // 1. Read the target URL from the first command-line argument.
  const url = process.argv[2];
  if (!url) throw new Error("usage: node stats.js <url-returning-json>");

  // 2. Make the HTTP GET request and parse the JSON body, failing fast on a
  //    non-2xx status before we try to decode it.
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} ${response.statusText}`);
  }
  const document = await response.json();

  // 3. Gather every number, then guard against an empty data set.
  const numbers = collectNumbers(document);
  if (numbers.length === 0) {
    throw new Error("the JSON response contained no numbers");
  }

  // 4. Compute and print the statistics.
  console.log(`count:  ${numbers.length}`);
  console.log(`mean:   ${mean(numbers).toFixed(4)}`);
  console.log(`median: ${median(numbers).toFixed(4)}`);
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
