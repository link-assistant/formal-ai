### Fixed
- For URL navigation prompts, check CORS-readable frame-policy metadata before
  rendering an iframe preview. Pages that send blocking `X-Frame-Options` or
  CSP `frame-ancestors` headers now get a polite direct new-tab link instead of
  a broken embedded preview. Markdown links in chat messages now open in a new
  tab and show an external-link indicator.
