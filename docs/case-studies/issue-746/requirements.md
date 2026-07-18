# Requirements

1. Recognize OpenAI function and type-only hosted tool advertisements.
2. Normalize `web_search_preview`, `file_search`, `computer_use_preview`, and `code_interpreter` to stable capabilities.
3. Preserve Anthropic function tools and dated server-tool advertisements.
4. Recognize Gemini `functionDeclarations`, `google_search`, `googleSearch`, and `url_context`.
5. Emit real tool-call objects instead of the wildcard refusal or browser-demo prose.
6. Preserve multilingual web intent routing through the normalized capability set.
7. Declare search/fetch routing as a first-class HTTP-server environment capability.
8. Verify each requirement and their composition through public HTTP surfaces and real installed agentic CLIs.

