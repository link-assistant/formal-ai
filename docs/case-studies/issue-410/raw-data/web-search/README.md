# @link-assistant/web-search

A web search microservice and library that aggregates results from 20+ search engines and knowledge/paper/code APIs, with intelligent result merging and reranking. Ships as **two first-class implementations** — JavaScript (`@link-assistant/web-search`) and Rust (the `web-search` crate) — that stay in lock-step: the same provider catalog, categories, merge strategies, and CLI/HTTP surface in both languages.

## Features

- **Many providers, four categories**: 22 providers grouped into `search`, `knowledge`, `papers`, and `code` — the same categories `formal-ai` consumes (see [Search Providers](#search-providers)).
- **Descriptor-driven catalog**: Engines are declared as data (URL, request kind, parser) and run through one shared `GenericProvider`, so adding an engine in one place adds it everywhere.
- **web-capture component**: The optional [`@link-assistant/web-capture`](https://github.com/link-assistant/web-capture) library can back any provider; when it is not installed the library degrades gracefully.
- **Result merging**: Combine results using RRF, weighted scoring, or interleaving.
- **Configurable weights**: Adjust provider weights for custom reranking.
- **URL deduplication**: Automatic normalization and deduplication across providers.
- **Typed provider registry**: A single source of truth powering provider discovery (CLI `--list-providers`, HTTP `/providers`, `/categories`) and provider instantiation.
- **Dual language parity**: Identical behavior and an extensive shared test suite across JavaScript and Rust.
- **Multi-runtime support**: The JavaScript build works with Bun, Node.js, and Deno.

## Installation

```bash
# With npm
npm install @link-assistant/web-search

# With bun
bun add @link-assistant/web-search

# With yarn
yarn add @link-assistant/web-search
```

## Quick Start

### As a Library

```javascript
import {
  WebSearchEngine,
  createSearchEngine,
} from '@link-assistant/web-search';

// Create a search engine
const engine = createSearchEngine();

// Search across all providers
const results = await engine.search('artificial intelligence');

// Search with options
const results = await engine.search('machine learning', {
  limit: 20,
  providers: ['google', 'duckduckgo'],
  strategy: 'rrf',
  weights: { google: 1.5, duckduckgo: 1.0 },
});

// Search single provider
const googleResults = await engine.searchSingle('deep learning', 'google');
```

### As a REST API Server

```bash
# Start the server
npx web-search serve --port 3000

# Or with bun
bunx web-search serve --port 3000
```

API Endpoints:

- `GET /search?q=<query>` - Search all providers
- `POST /search` - Search with options in body
- `GET /search/:provider?q=<query>` - Search single provider
- `GET /providers` - List available providers and the typed registry (filter with `?category=<search|knowledge|papers|code>`)
- `GET /categories` - List provider ids grouped by category
- `GET /health` - Health check

Example:

```bash
curl "http://localhost:3000/search?q=rust+programming&limit=10&strategy=rrf"

# Only the scholarly-paper providers
curl "http://localhost:3000/providers?category=papers"

# Provider ids per category
curl "http://localhost:3000/categories"
```

### As a CLI Tool

```bash
# Search from command line
npx web-search "artificial intelligence"

# With options
npx web-search "machine learning" --limit 20 --providers google,bing --format json

# Search category-specific providers
npx web-search "transformer architecture" --providers arxiv,crossref,openalex

# Output just URLs
npx web-search "deep learning" --format urls

# Discover every available provider, grouped by category
npx web-search --list-providers
```

## Merge Strategies

### Reciprocal Rank Fusion (RRF)

Default strategy. Combines results by their rank positions across providers.

```javascript
const results = await engine.search(query, { strategy: 'rrf' });
```

### Weighted Scoring

Score results based on provider weights and rank positions.

```javascript
const results = await engine.search(query, {
  strategy: 'weighted',
  weights: { google: 2.0, duckduckgo: 1.0, bing: 0.5 },
});
```

### Interleaving

Round-robin style interleaving of results from each provider.

```javascript
const results = await engine.search(query, { strategy: 'interleave' });
```

## Search Providers

Providers are organized into the four categories `formal-ai` consumes. Run
`npx web-search --list-providers` (or `cargo run -- --list-providers` in `rust/`)
to print the live catalog; both languages report the same 22 providers.

| Category    | Providers                                                                                                | Access                          |
| ----------- | -------------------------------------------------------------------------------------------------------- | ------------------------------- |
| `search`    | google, bing, duckduckgo, searx, brave, mojeek, ecosia, startpage, yahoo, lite (DuckDuckGo Lite), `wc:*` | API / hybrid / HTML / component |
| `knowledge` | wikipedia, wikidata                                                                                      | API (CORS-readable)             |
| `papers`    | crossref, openalex, arxiv                                                                                | API (CORS-readable)             |
| `code`      | github, hackernews                                                                                       | API (CORS-readable)             |

- **`api`** providers call a JSON/Atom endpoint directly.
- **`html`** providers scrape a search-results page with a per-engine regex through the shared anchor-list parser.
- **`hybrid`** providers (google, bing) use an official API when credentials are configured and fall back to scraping otherwise.
- **`component`** providers (`wc:*`) are backed by the optional `@link-assistant/web-capture` library — see [web-capture component](#web-capture-component).

`GITHUB_TOKEN` is optional but raises the GitHub search rate limit when set.

### Class-based providers (google, bing, duckduckgo)

```javascript
import {
  GoogleProvider,
  BingProvider,
  DuckDuckGoProvider,
} from '@link-assistant/web-search';

// Google: Custom Search API when configured, scraping fallback otherwise
const google = new GoogleProvider({
  apiKey: 'your-api-key',
  searchEngineId: 'your-cx-id',
});

// Bing: Web Search API when configured, scraping fallback otherwise
const bing = new BingProvider({ apiKey: 'your-bing-api-key' });

// DuckDuckGo: HTML scraping, no API key required
const duckduckgo = new DuckDuckGoProvider();
```

### Descriptor-driven providers

Every other engine in the table is declared as a descriptor (id, request kind,
parser) and instantiated through a single `GenericProvider`. The registry can
build the whole catalog so you can pick any provider by id:

```javascript
import { buildProviders, API_ENGINES } from '@link-assistant/web-search';

// Instantiate the full catalog (Map<id, provider>) and select one
const arxiv = buildProviders().get('arxiv');
const results = await arxiv.search('graph neural networks', { limit: 5 });

// Or build directly from a descriptor
import { createGenericProvider } from '@link-assistant/web-search';
const crossref = createGenericProvider(
  API_ENGINES.find((d) => d.id === 'crossref')
);
```

### web-capture component

Any provider can be backed by the optional
[`@link-assistant/web-capture`](https://github.com/link-assistant/web-capture)
component library, exposed through the `wc:*` provider ids
(`wc:wikipedia`, `wc:duckduckgo`, `wc:google`, `wc:bing`, `wc:brave`). The
dependency is loaded lazily; when it is not installed the provider warns once and
returns an empty result set so the rest of the aggregation keeps working. You can
also inject a custom implementation for testing:

```javascript
import { createWebCaptureProvider } from '@link-assistant/web-search';

const provider = createWebCaptureProvider({
  engine: 'wikipedia',
  // Optional: inject a fetch/search implementation (defaults to @link-assistant/web-capture)
  searchImpl: async (query, options) => [
    /* { title, url, snippet } */
  ],
});
```

### Provider registry

A typed registry is the single source of truth for discovery and instantiation:

```javascript
import {
  CATEGORIES, // ['search', 'knowledge', 'papers', 'code']
  getRegistry, // full provider metadata
  getProviderIds, // ids, optionally filtered by category
  getDefaultProviderIds, // ids used when none are specified
  buildProviders, // instantiate the whole catalog
} from '@link-assistant/web-search';

getProviderIds('papers'); // ['crossref', 'openalex', 'arxiv']
```

## API Reference

### WebSearchEngine

```javascript
const engine = new WebSearchEngine(config);

// Search methods
await engine.search(query, options);
await engine.searchSingle(query, providerName, options);

// Provider management
engine.getAvailableProviders();
engine.getProviderStatus();
engine.setProviderWeight(name, weight);
engine.setProviderEnabled(name, enabled);
engine.getProvider(name);
```

### Merge Functions

```javascript
import {
  mergeResults,
  mergeWithRRF,
  mergeWithWeights,
  mergeWithInterleave,
} from '@link-assistant/web-search';

// Merge results from multiple providers
const merged = mergeResults(resultsByProvider, {
  strategy: 'rrf',
  weights: { google: 1.5 },
  rrfK: 60,
  removeDuplicates: true,
});
```

## Rust Library

A first-class Rust implementation lives in the `rust/` directory (crate
`web-search`). It mirrors the JavaScript library: the same descriptor-driven
catalog, the same typed registry, the same four categories, and the same 22
providers — verified by a shared test suite (`cargo test`).

```bash
cd rust
cargo build --release
```

### Rust CLI

```bash
# Search
./target/release/web-search "artificial intelligence" --limit 10

# Category-specific providers
./target/release/web-search "graph neural networks" --providers arxiv,crossref

# List every available provider, grouped by category (matches the JS CLI)
./target/release/web-search --list-providers

# Start server (GET /search, /providers, /categories, /health)
./target/release/web-search serve --port 3000
```

### Rust Library Usage

```rust
use web_search::{WebSearchEngine, SearchOptions, MergeStrategy};

let engine = WebSearchEngine::new();

let results = engine.search_with_options(
    "machine learning",
    SearchOptions { limit: Some(10), ..Default::default() },
    None,
    Some(MergeOptions { strategy: MergeStrategy::Rrf, ..Default::default() })
).await?;
```

## Development

```bash
# Install dependencies
bun install

# Run tests
bun test

# Run with other runtimes
npm test
deno test --allow-read --allow-env --allow-net

# Lint code
bun run lint

# Format code
bun run format
```

### Rust Development

```bash
cd rust

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

## Environment Variables

- `GOOGLE_API_KEY` - Google Custom Search API key
- `GOOGLE_SEARCH_ENGINE_ID` - Google Custom Search Engine ID
- `BING_API_KEY` - Bing Web Search API key

## License

[Unlicense](LICENSE) - Public Domain
