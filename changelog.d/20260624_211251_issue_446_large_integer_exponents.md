---
bump: patch
---

### Fixed
- **Issue #446 — large integer exponents in the web calculator were truncated.** Arithmetic fallback evaluation now keeps integer exponentiation exact, so prompts such as `10^100` render the full integer instead of `1e+1`.
