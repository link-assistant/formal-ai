---
bump: patch
---

### Fixed
- Issue #1101: the same documentation question is now answered the same way in every language. `how does pandas DataFrame.join work?` was answered from the documentation rule, while its Russian, Hindi and Chinese translations were answered with the web-search handler's offline-fetch notice. The cause was not the precedence order the issue suspected: the last branch of the web-search cascade — an interrogative naming an engineered brand, carrying no search imperative — claimed all four, and English escaped only by accident. Once the question opener is stripped, the English residual begins with `does`, which the seed lists as a `non_referential_subject` so that "does it …" is rejected; Russian, Hindi and Chinese form the same question without do-support, so nothing rescued them. That branch now asks the rule set whether a documentation rule already answers the prompt, which is language-neutral by construction. An explicit search imperative still reaches web search in all four languages, and a brand question no documentation rule covers is still searched for.
