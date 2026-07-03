// Capture Binance daily-kline min-low / max-high summaries for multiple assets
// and years, used as the versioned reference data for the market-price
// verification topic (issue #493). Writes raw klines and a compact summary per
// (symbol, year) under docs/case-studies/issue-493/raw-data/.
//
// Usage: node experiments/issue_493_capture_market_data.mjs
import { writeFileSync } from "node:fs";

const RAW_DIR = "docs/case-studies/issue-493/raw-data";

// UTC year boundaries in milliseconds.
const YEARS = {
  2021: [Date.UTC(2021, 0, 1), Date.UTC(2022, 0, 1) - 1],
  2022: [Date.UTC(2022, 0, 1), Date.UTC(2023, 0, 1) - 1],
  2023: [Date.UTC(2023, 0, 1), Date.UTC(2024, 0, 1) - 1],
  2024: [Date.UTC(2024, 0, 1), Date.UTC(2025, 0, 1) - 1],
};

// (symbol, [years]) to capture.
const TARGETS = [
  ["ETHUSDT", [2021, 2022, 2023, 2024]],
  ["BTCUSDT", [2024]],
];

const isoDate = (ms) => new Date(ms).toISOString().slice(0, 10);

async function fetchKlines(symbol, startTime, endTime) {
  const url =
    `https://api.binance.com/api/v3/klines?symbol=${symbol}` +
    `&interval=1d&startTime=${startTime}&endTime=${endTime}&limit=1000`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${symbol} ${startTime}: HTTP ${response.status}`);
  }
  return response.json();
}

function summarize(symbol, year, klines) {
  let minLow = Infinity;
  let minLowDate = "";
  let maxHigh = -Infinity;
  let maxHighDate = "";
  for (const candle of klines) {
    const openTime = candle[0];
    const high = Number(candle[2]);
    const low = Number(candle[3]);
    if (low < minLow) {
      minLow = low;
      minLowDate = isoDate(openTime);
    }
    if (high > maxHigh) {
      maxHigh = high;
      maxHighDate = isoDate(openTime);
    }
  }
  return {
    symbol,
    year,
    candles: klines.length,
    observed_min_price: minLow,
    observed_min_date: minLowDate,
    observed_max_price: maxHigh,
    observed_max_date: maxHighDate,
  };
}

const summaries = [];
for (const [symbol, years] of TARGETS) {
  for (const year of years) {
    const [start, end] = YEARS[year];
    const klines = await fetchKlines(symbol, start, end);
    const base = `${symbol.toLowerCase()}-${year}-daily-klines`;
    writeFileSync(`${RAW_DIR}/${base}.json`, JSON.stringify(klines));
    const summary = summarize(symbol, year, klines);
    summaries.push(summary);
    console.log(JSON.stringify(summary));
  }
}
writeFileSync(
  `${RAW_DIR}/market-price-references-summary.json`,
  JSON.stringify(summaries, null, 2) + "\n",
);
console.log(`Wrote ${summaries.length} summaries.`);
