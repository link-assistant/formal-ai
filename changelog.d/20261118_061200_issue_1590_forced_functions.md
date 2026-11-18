### Fixed

- Honor forced client function calls in OpenAI-compatible Chat Completions and Responses, including streaming. Validate arguments before returning the call and expose underivable arguments as a structured refusal. Server tool permissions still apply to shell, file and web tools.
