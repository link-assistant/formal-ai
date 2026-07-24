# Online Research for Issue #834

Research performed 2026-07-24. Primary legal texts, regulator guidance, model
licenses/cards, and provider terms were preferred. These notes support an
engineering policy and are not legal advice.

## Public domain and AI output

- The [U.S. Copyright Office AI initiative](https://www.copyright.gov/ai/) links
  its reports on output copyrightability and training. Part 2 concludes that
  human expressive contribution can be protected even when a work contains AI
  material; merely using AI does not settle ownership of a complete work.
- The Office's [Part 2 announcement](https://www.copyright.gov/newsnet/2025/1060.html)
  says protection depends on sufficient human-determined expressive elements,
  including human arrangement or modification. This contradicts any blanket
  premise that all model output is ownerless.
- The [Creative Commons public-domain tools page](https://creativecommons.org/public-domain/)
  explains that CC0 seeks the fullest relinquishment possible but cannot
  guarantee public-domain status in every jurisdiction. Formal AI currently
  uses the Unlicense, so the operational need is an explicit inbound rule and
  third-party boundary, not a claim that a label erases rights the contributor
  does not own.

Conclusion: keep provenance and output review even where raw machine expression
may be uncopyrightable; human curation and third-party material remain relevant.

## Fair use, training, and EU text-and-data mining

- [17 U.S.C. §107](https://www.copyright.gov/title17/92chap1.html) supplies four
  fair-use factors. The [Copyright Office Fair Use Index](https://www.copyright.gov/fair-use/)
  emphasizes case-by-case analysis and rejects a fixed word/percentage safe
  harbor.
- The Copyright Office's [Part 3 pre-publication training report](https://www.copyright.gov/ai/Copyright-and-Artificial-Intelligence-Part-3-Generative-AI-Training-Report-Pre-Publication-Version.pdf)
  treats the analysis as fact-specific and says knowing use of pirated copies
  should weigh against fair use. It is a pre-publication report, not a judicial
  holding.
- Directive (EU) 2019/790
  [Article 4](https://eur-lex.europa.eu/eli/dir/2019/790/oj) permits certain
  text-and-data mining of lawfully accessible works only while the rightsholder
  has not expressly reserved the relevant rights, including by machine-readable
  means for online content.

Conclusion: research/criticism can support a narrow issue excerpt, but it does
not automatically clear leaked sources, paywalled datasets, long works, or a
later change from debugging evidence to training input.

## Privacy and regional data governance

- GDPR [Article 5](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679)
  sets lawfulness/fairness/transparency, purpose limitation, data minimisation,
  accuracy, storage limitation, integrity/confidentiality, and accountability
  principles.
- The UK ICO's current [AI and data-protection guidance](https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/artificial-intelligence/guidance-on-ai-and-data-protection/about-this-guidance/)
  provides a risk-based audit methodology and warns that its guidance is under
  review after the Data (Use and Access) Act. That is a concrete example of why
  review dates and territories must be recorded.
- Canadian privacy regulators published
  [generative-AI privacy principles](https://www.priv.gc.ca/en/opc-news/news-and-announcements/2023/nr-c_231207/)
  spanning federal, provincial, and territorial authorities.
- Japan's Personal Information Protection Commission published an
  [international resolution on generative AI](https://www.ppc.go.jp/files/pdf/231115_shiryou-1-3.pdf)
  that treats development-data collection as a privacy concern even when
  personal data is publicly accessible.
- The U.S. FTC's [COPPA FAQ](https://www.ftc.gov/business-guidance/resources/complying-coppa-frequently-asked-questions)
  documents parental-consent and other duties for covered children's data.

Conclusion: "public" and "pseudonymous" are not safe training categories. A
no-real-personal-data rule is simpler and safer for the current project, with a
new assessment required for any claimed anonymous source.

## EU AI Act

- Regulation (EU) 2024/1689
  [Article 53](https://eur-lex.europa.eu/eli/reg/2024/1689/oj) requires
  general-purpose-model technical documentation, downstream information, a
  copyright-compliance policy, and a public training-content summary. Its
  free/open-source exception removes only the first two for qualifying models
  and is unavailable for systemic-risk models.
- The European Commission's
  [GPAI provider guidelines](https://digital-strategy.ec.europa.eu/en/policies/guidelines-gpai-providers)
  state that GPAI obligations applied from 2 August 2025 and Commission
  enforcement begins 2 August 2026, with a later transition for qualifying
  older models.
- The Commission's
  [training-content summary FAQ](https://digital-strategy.ec.europa.eu/en/faqs/template-general-purpose-ai-model-providers-summarise-their-training-content)
  says the summary obligation also applies to open-source providers.
- The Commission's
  [GPAI Code of Practice page](https://digital-strategy.ec.europa.eu/en/policies/contents-code-gpai)
  separates transparency/copyright commitments for GPAI from safety/security
  commitments for systemic-risk models.

Conclusion: Formal AI has no trained GPAI weights and is not currently claiming
an exemption. A future open release may avoid Article 53(1)(a)-(b) only if its
facts qualify; copyright policy, training summary, privacy, prohibited
practices, and systemic-risk analysis remain.

## Model and API terms

- Meta's [Llama 3.3 model card](https://github.com/meta-llama/llama-models/blob/main/models/llama3_3/MODEL_CARD.md)
  identifies synthetic-data generation and distillation as intended uses.
  Meta's exact [Llama 3.3 license](https://github.com/meta-llama/llama-models/blob/main/models/llama3_3/LICENSE)
  contains **Built with Llama**, notice, trained-model naming, acceptable-use,
  patent, redistribution, and 700-million-MAU provisions.
- Mistral's [Mistral 7B v0.1 model card](https://docs.mistral.ai/models/model-cards/mistral-7b-0-1)
  identifies Apache 2.0 for those weights. Mistral's
  [license overview](https://help.mistral.ai/en/articles/347393-under-which-license-are-mistral-s-open-models-available)
  shows that licenses vary across its catalog.
- OpenRouter's [distillation guide](https://openrouter.ai/docs/cookbook/evaluate-and-optimize/distillation)
  calls `enforce_distillable_text` best-effort and says to verify each model's
  license. Its [Terms of Service](https://openrouter.ai/terms) leave users
  responsible for model-specific terms.
- The current [OpenAI Services Agreement](https://openai.com/policies/services-agreement/)
  restricts use of output to develop competing AI except defined permitted
  exceptions.
- Anthropic's [Commercial Terms](https://www.anthropic.com/legal/commercial-terms)
  restrict using services to build a competing product or train a competing AI
  without approval.
- The [Gemini API Additional Terms](https://ai.google.dev/gemini-api/terms)
  distinguish paid and unpaid services, restrict competing-model uses, and put
  additional limits on grounded results.

Conclusion: open weights, API output rights, hosted-service contracts,
operator-input rights, and downstream release obligations are separate review
layers. A provider filter cannot replace exact model/version/route evidence.

## Warranty and safety

The existing `LICENSE` already provides Formal AI **AS IS** and without
warranty. No researched source supports treating that language as permission
to ignore mandatory consumer, privacy, product-safety, or intentional-harm
rules. The policy therefore treats disclaimers as allocation language and uses
testing, human review, containment, and counsel as substantive controls.

