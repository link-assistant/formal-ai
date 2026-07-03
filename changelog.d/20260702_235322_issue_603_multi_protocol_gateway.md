### Added

- Added `/api/<protocol>/...` gateway routes for OpenAI, Anthropic, Gemini,
  Vertex, and formal-ai native APIs, with per-protocol model discovery.

### Fixed

- Added named OpenAI Responses SSE events for streaming Responses clients,
  including the final `response.completed` event.
