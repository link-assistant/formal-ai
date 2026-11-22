这是一个解决该复合任务的 Python 程序（模拟带提醒和 Markdown 仪表盘的加密投资组合追踪器）。 我已将您的请求分解为以下子任务：

1. 输出结果
2. 检索当前来源数据
3. 导出 Markdown 比较报告
4. Fetch current crypto prices
5. Model portfolio holdings
6. Calculate total value, 24h changes, and weights
7. Notify when an asset drops beyond the supplied threshold
8. Mock the public API endpoint

```python
import argparse
import json
from dataclasses import dataclass
from math import isfinite
from pathlib import Path
from typing import Mapping
from urllib.parse import urlsplit
from urllib.request import urlopen

@dataclass(frozen=True)
class AssetPrice:
    symbol: str
    price_usd: float
    change_24h_pct: float

def fetch_prices(symbols: list[str], source: str) -> dict[str, AssetPrice]:
    if source.startswith('https://'):
        endpoint = urlsplit(source)
        if not endpoint.hostname or endpoint.username or endpoint.password:
            raise ValueError('provide a public HTTPS endpoint without credentials')
        maximum_bytes = 1_048_576
        with urlopen(source, timeout=10) as response:
            if urlsplit(response.geturl()).scheme != 'https':
                raise ValueError('the price source redirected outside HTTPS')
            body = response.read(maximum_bytes + 1)
        if len(body) > maximum_bytes:
            raise ValueError('the price response exceeded 1 MiB')
        payload = json.loads(body)
    else:
        payload = json.loads(Path(source).read_text(encoding='utf-8'))
    return {symbol: AssetPrice(symbol=symbol, **payload[symbol]) for symbol in symbols}

def portfolio_rows(
    holdings: Mapping[str, float],
    prices: Mapping[str, AssetPrice],
) -> list[dict[str, float | str]]:
    total_value = sum(amount * prices[symbol].price_usd for symbol, amount in holdings.items())
    rows = []
    for symbol, amount in holdings.items():
        price = prices[symbol]
        value = amount * price.price_usd
        portfolio_weight = value / total_value * 100 if total_value else 0.0
        rows.append({
            'symbol': symbol,
            'amount': amount,
            'price_usd': price.price_usd,
            'value_usd': value,
            'change_24h_pct': price.change_24h_pct,
            'portfolio_weight': portfolio_weight,
        })
    return rows

def notify_alerts(rows: list[dict[str, float | str]], alert_threshold: float) -> list[str]:
    return [
        f'Notify: {row["symbol"]} dropped {row["change_24h_pct"]:.2f}% in 24h'
        for row in rows
        if float(row['change_24h_pct']) < -alert_threshold
    ]

def money(value: float) -> str:
    return f'${value:,.2f}'

def render_dashboard(
    rows: list[dict[str, float | str]], notices: list[str], alert_threshold: float,
) -> str:
    total_value = sum(float(row['value_usd']) for row in rows)
    lines = [
        '# Crypto Portfolio Dashboard',
        '',
        f'**Total value:** {money(total_value)}',
        '',
        '| Asset | Amount | Price USD | Value USD | 24h change | Portfolio weight |',
        '| --- | ---: | ---: | ---: | ---: | ---: |',
    ]
    for row in rows:
        lines.append(
            f'| {row["symbol"]} | {row["amount"]:,.4g} | {money(float(row["price_usd"]))} | '
            f'{money(float(row["value_usd"]))} | {row["change_24h_pct"]:.2f}% | '
            f'{row["portfolio_weight"]:.2f}% |'
        )
    lines.extend(['', '## Alerts'])
    lines.extend(notices or [f'No asset dropped more than {alert_threshold:g}% in the last 24h.'])
    return '\n'.join(lines)

def main() -> None:
    parser = argparse.ArgumentParser(
        description='Read JSON holdings and prices from a file or public HTTPS endpoint.',
        epilog='Holdings maps symbols to quantities. Prices maps symbols to price_usd and change_24h_pct. To replay a mock response, pass its JSON file as the prices argument.',
    )
    parser.add_argument('holdings', type=Path)
    parser.add_argument('prices', help='prices JSON file or public HTTPS endpoint')
    parser.add_argument('alert_threshold', type=float, help='percentage decline that triggers an alert')
    args = parser.parse_args()
    if not isfinite(args.alert_threshold) or args.alert_threshold < 0:
        parser.error('alert_threshold must be a finite, nonnegative percentage')
    holdings = json.loads(args.holdings.read_text(encoding='utf-8'))
    if not isinstance(holdings, dict) or not holdings:
        parser.error('holdings must be a nonempty mapping from symbols to quantities')
    prices = fetch_prices(list(holdings), args.prices)
    rows = portfolio_rows(holdings, prices)
    notices = notify_alerts(rows, args.alert_threshold)
    dashboard = render_dashboard(rows, notices, args.alert_threshold)
    print(f'{dashboard}\n\nPrice source: {args.prices}')

if __name__ == '__main__':
    main()
```

所需的库：
- Python 3 standard library only

如何自行运行：

执行状态：未运行 —— 该报告未在离线沙箱中执行，运行所需的输入数据见程序说明。代码仅供审阅。自行运行：`python crypto_portfolio.py <holdings.json> <prices-file-or-url> <alert-threshold>`。
