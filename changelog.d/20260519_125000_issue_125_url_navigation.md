### Fixed
- Split URL navigation and HTTP fetch into two distinct intents so that
  `Navigate to github.com`, `Go to google.com`, and the Russian `Перейди на github.com`
  return a direct HTTPS link with iframe preview controls (no fetch attempted),
  while `Make a request to example.com`, `Fetch example.com`, and the Russian
  `Сделай запрос к example.com` keep the previous HTTP GET with CORS-fallback to
  an iframe. Added many more navigation variations (`open`, `visit`, `show`,
  `display`, `load`, `take me to`, `preview`, `view`, `browse to`, `открой`,
  `покажи`, `загрузи`, `посети`, `зайди на`, `просмотри`, `отобрази`) and bare
  URLs (`github.com`).
