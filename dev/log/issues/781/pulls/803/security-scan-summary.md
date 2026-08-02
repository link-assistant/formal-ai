# Evidence security scan

The evidence bundle was scanned before publication for common GitHub, OpenAI,
Anthropic, Slack, AWS, private-key, and bearer-token shapes. One file in
`security-candidate-files.txt` contains three `sk-`-like substrings inside
opaque encrypted Codex reasoning ciphertext. Redacted-context review confirmed
that these are random ciphertext matches, not request headers or API keys.

One raw, previously public PR diff and its later complete-PR snapshot contain a
browser-published Google Maps API identifier copied from captured marketplace
HTML. Both paths are listed in `security-google-key-files.txt`. This is
page-source evidence rather than a server credential or an authorization
header. Exact per-dialog records also deliberately omit HTTP authorization
headers.
