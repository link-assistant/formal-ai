# Online Research

## Sources

- Bun Bundler documentation: https://bun.com/docs/bundler
- Bun GitHub Actions setup guidance: https://bun.com/guides/install/cicd
- MDN CORS error guide: https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS/Errors

## Notes

- Bun's current CLI bundles browser code through `bun build`, with `--target browser`, `--outfile`, and `--format iife` available for this repo's non-module HTML entry shape.
- The official Bun CI guidance uses `oven-sh/setup-bun@v2`, which matches the workflow change in this PR.
- MDN describes the exact browser class of failure shown in the issue screenshot: a cross-origin response that is not permitted by the remote server's CORS headers is blocked before application code can read it.
