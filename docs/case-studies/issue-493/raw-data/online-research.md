# Issue 493 Online Research

Collected on 2026-07-03 for
<https://github.com/link-assistant/formal-ai/issues/493>.

## Sources

| Source | URL | Use |
|---|---|---|
| Binance Spot API klines | <https://api.binance.com/api/v3/klines?symbol=ETHUSDT&interval=1d&startTime=1704067200000&endTime=1735689599999&limit=1000> | Captured ETHUSDT daily candles for 2024. |
| Binance API documentation | <https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints#klinecandlestick-data> | Confirms `/api/v3/klines` returns candlestick bars identified by open time, with open/high/low/close fields. |
| Binance Vision API mirror | <https://data-api.binance.vision/api/v3/klines?symbol=ETHUSDT&interval=1d&startTime=1704067200000&endTime=1735689599999&limit=1000> | Independent Binance-hosted mirror of the same kline query. |
| CoinGecko market chart range API | <https://api.coingecko.com/api/v3/coins/ethereum/market_chart/range?vs_currency=usd&from=1704067200&to=1735689599> | Attempted public historical range query. The captured response reports the free public API range is limited to the past 365 days. |
| CoinGecko API documentation | <https://docs.coingecko.com/reference/coins-id-market-chart-range> | Confirms the historical chart range endpoint shape. |
| CoinGecko Ethereum historical page | <https://www.coingecko.com/en/coins/ethereum/historical_data> | Captured HTML fallback for review evidence when the public API rejected the full 2024 range. |
| Tesseract.js API documentation | <https://github.com/naptha/tesseract.js/blob/master/docs/api.md> | Confirms `createWorker("eng")` and `worker.recognize(image)` OCR workflow used by the experiment. |

## Binance ETHUSDT 2024 Summary

The captured Binance daily kline JSON contains 366 rows for
2024-01-01 through 2024-12-31 UTC. The extracted low/high range is:

| Metric | Date | Price |
|---|---:|---:|
| Minimum daily low | 2024-01-03 | 2100.00 |
| Maximum daily high | 2024-12-16 | 4107.80 |

Therefore the screenshot claim `ETH in 2024: $1,700` is contradicted by the
captured Binance ETHUSDT 2024 daily candle data.
