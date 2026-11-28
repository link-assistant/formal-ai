import { readFileSync } from "node:fs";

export const quoteDate = "2026-11-17";
const endpoint = "https://api.frankfurter.dev/v2";

export const currencyCaptures = new Map(
  ["usd-rub", "rub-usd", "eur-rub", "eur-usd", "usd-eur", "jpy-cad"].map(pair => [
    `${endpoint}${pair.includes("rub") ? "/providers/cbr" : ""}/rate/${pair.replace("-", "/")}?date=${quoteDate}`,
    readFileSync(new URL(`../../fixtures/currency-quotes/${pair}.json`, import.meta.url), "utf8"),
  ])
);

export function currencyFetch(url) {
  const body = currencyCaptures.get(String(url));
  return new Response(body, { status: body ? 200 : 404 });
}

export const currencyExamples = [
  [`Посчитай 1000 рублей в долларах ${quoteDate}`, "12.12 USD", "rub/usd", "0.012119"],
  [`Convert 1000 RUB to USD on ${quoteDate}`, "12.12 USD", "rub/usd", "0.012119"],
  [`Calculate 1 USD in RUB on ${quoteDate}`, "82.51 RUB", "usd/rub", "82.5143"],
  [`Calculate 1 EUR in RUB on ${quoteDate}`, "91.19 RUB", "eur/rub", "91.1948"],
  [`Calculate 1 EUR in USD on ${quoteDate}`, "1.10 USD", "eur/usd", "1.1006"],
  [`Convert 2500 JPY to CAD on ${quoteDate}`, "22.41 CAD", "jpy/cad", "0.0089627"],
];

export const localizedUsdToEur = [
  [`请计算 1000 美元 换成 欧元 ${quoteDate}`, "908.60 EUR", "usd/eur", "0.9086"],
  [`गणना करें 1000 डॉलर में यूरो ${quoteDate}`, "908.60 EUR", "usd/eur", "0.9086"],
];
