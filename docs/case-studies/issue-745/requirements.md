# Issue 745 requirements

- Exercise at least 15 natural-language variations per primary tool intent in every currently supported language: English, Russian, Hindi, and Chinese.
- Route URL objects to fetch, local file paths to read, explicit online topics to web search, explicit content plus file targets to write, and local directory/code requests to shell execution.
- Reject cross-tool collisions such as `display sample.txt` becoming web navigation or `tell me about URL` becoming a web search.
- Keep natural-language vocabulary in Links Notation seed data and combine it with validated URL/path/content/query shapes in Rust.
- Preserve a red/green regression, real Agent CLI authorship evidence, and a release fragment.
