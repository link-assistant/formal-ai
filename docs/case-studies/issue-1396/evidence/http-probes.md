# HTTP observations on November 18, 2026

Requests used `Origin: https://link-assistant.github.io` where applicable. The recorded result for each request is from this environment on that date.

| Request | Result |
| --- | --- |
| `https://ru.wikihow.com/хорошо-покакать` | HTTP 200 HTML. The response omitted `Access-Control-Allow-Origin`. The HTML contained wikiHow's step-list markup. |
| `https://ru.wikihow.com/api.php?action=parse&page=хорошо-покакать&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*` with UTF-8 percent encoding | HTTP 200 JSON with `Access-Control-Allow-Origin: *`. The response contained 87,755 bytes including ordered article sections. Chromium also accepted the live response and displayed a citation for the capture. |
| Same API with `Хорошо покакать` as the page title | HTTP 200 with a MediaWiki `missingtitle` error. |
| `https://archive.org/wayback/available` with the exact Russian article as the `url` query parameter | HTTP 200 JSON with `Access-Control-Allow-Origin: *`, matching the snapshot dated 2024-12-22 16:00:07 UTC. [Response](wayback-availability.json) |
| `https://web.archive.org/web/20241222160007id_/https://ru.wikihow.com/хорошо-покакать` | HTTP 429 during the live probe. Chromium's deterministic replay used the same URL and a fixture with its canonical reference and steps. |

```sh
article='https://ru.wikihow.com/%D1%85%D0%BE%D1%80%D0%BE%D1%88%D0%BE-%D0%BF%D0%BE%D0%BA%D0%B0%D0%BA%D0%B0%D1%82%D1%8C'
curl -sSL -D article.headers -o article.html -H 'Origin: https://link-assistant.github.io' "$article"
curl -sS -D parse.headers -o parse.json -H 'Origin: https://link-assistant.github.io' -G 'https://ru.wikihow.com/api.php' --data-urlencode 'action=parse' --data-urlencode 'page=хорошо-покакать' --data-urlencode 'prop=text|sections|displaytitle' --data-urlencode 'format=json' --data-urlencode 'origin=*'
curl -sS -D wayback.headers -o wayback.json -G 'https://archive.org/wayback/available' --data-urlencode "url=$article"
curl -sSL -D replay.headers -o replay.html "https://web.archive.org/web/20241222160007id_/$article"
```
