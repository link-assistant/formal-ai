# Issue 753: Grok Build local routing

## Reproduction
The seeded Grok integration exported legacy XAI variables, but Grok Build reads GROK_API_KEY and GROK_BASE_URL, so it ignored the local Formal AI endpoint.

## Implemented behavior
Ephemeral runs now set a nonempty temporary GROK_API_KEY and route GROK_BASE_URL to the local OpenAI-compatible /api/openai/v1 endpoint. Global setup writes the native ~/.grok/user-settings.json apiKey and baseURL fields.

## Verification
Integration tests cover ephemeral routing, native global settings and undo, and a real HTTP request from a Grok-compatible fixture to the configured local chat-completions endpoint. This directory includes the real Agent CLI session log produced through local formal-ai serve.
