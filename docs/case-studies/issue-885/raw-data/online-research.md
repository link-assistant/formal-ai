# Primary online research for issue 885

Checked on 2026-08-01. These are the first-party or primary sources used by the
reviewed documentation; search-result snippets and aggregator labels were not
used as permission.

## Output, copyright, and service terms

- [OpenAI Services Agreement](https://openai.com/policies/services-agreement/):
  current business terms separate output ownership from restrictions on using
  output to develop competing AI models and list limited permitted exceptions.
- [OpenAI Europe Terms of Use](https://openai.com/policies/eu-terms-of-use/):
  current consumer terms assign output to the user to the extent permitted by
  law, restrict automatic extraction/competing-model use, and warn about
  similarity and third-party accuracy.
- [OpenAI `gpt-oss` announcement](https://openai.com/index/introducing-gpt-oss/):
  first-party announcement identifies downloadable open weights under
  Apache-2.0. This is distinct from hosted-service output.
- [U.S. Copyright Office AI initiative](https://www.copyright.gov/ai/): links
  the Office's copyrightability report and records its human-authorship analysis.
- [Open Data Commons ODC-By 1.0](https://opendatacommons.org/licenses/by/1-0/):
  database license; its scope must not be silently extended to individual
  contents.

## Dataset records

- [FineWeb2 card](https://huggingface.co/datasets/HuggingFaceFW/fineweb-2):
  ODC-By-1.0, Common Crawl terms, 2025 paper, and web-content limitations.
- [FineWeb card](https://huggingface.co/datasets/HuggingFaceFW/fineweb):
  ODC-By-1.0 English Common Crawl corpus and exact crawl configs.
- [Dolma card](https://huggingface.co/datasets/allenai/dolma): v1.7 release,
  ODC-By, and express continued application of original-source terms.
- [Common Pile organization](https://huggingface.co/common-pile): 2025 raw and
  filtered collections and the maintainers' license-accuracy warning.
- [SmolTalk card](https://huggingface.co/datasets/HuggingFaceTB/smoltalk) and
  [Smol-SmolTalk card](https://huggingface.co/datasets/HuggingFaceTB/smol-smoltalk):
  mixture-level provenance and the named Apache-2.0 synthetic subset.
- [PleIAs SYNTH card](https://huggingface.co/datasets/PleIAs/SYNTH): synthetic
  collection with seed URL/license metadata; row-level review remains required.
- [DCLM repository](https://github.com/mlfoundations/dclm) and
  [baseline dataset card](https://huggingface.co/datasets/mlfoundations/dclm-baseline-1.0):
  curation method and source identity; code licensing is not payload clearance.
- [The Stack v2 repository](https://github.com/bigcode-project/the-stack-v2):
  curation code and source-specific licensing/opt-out context.
- [Tulu 3 SFT mixture card](https://huggingface.co/datasets/allenai/tulu-3-sft-mixture):
  component mixture whose source licenses and generator provenance vary.
- [RedPajama-Data repository](https://github.com/togethercomputer/RedPajama-Data):
  Common Crawl pipeline; its Apache code license does not license every page.

## Model records

- [GLM-5.1](https://huggingface.co/zai-org/GLM-5.1): official MIT-licensed
  repository for the GLM-5 successor.
- [DeepSeek-V4-Pro](https://huggingface.co/deepseek-ai/DeepSeek-V4-Pro): official
  card states repository and model weights use MIT.
- [Granite 4.1 30B](https://huggingface.co/ibm-granite/granite-4.1-30b): official
  release date and Apache-2.0 signal.
- [Qwen3.6 35B-A3B](https://huggingface.co/Qwen/Qwen3.6-35B-A3B): official
  post-trained weight repository and Apache-2.0 signal.
- [Mistral Small 4 announcement](https://mistral.ai/news/mistral-small-4/):
  first-party 2026 open-weight Apache-2.0 release.
- [Apertus 70B 2509](https://huggingface.co/swiss-ai/Apertus-70B-2509): official
  Apache-2.0 release with training and EU documentation.
- [OpenAI gpt-oss-120b](https://huggingface.co/openai/gpt-oss-120b): official
  Apache-2.0 local weights.
- [SmolLM3 3B](https://huggingface.co/HuggingFaceTB/SmolLM3-3B): official
  Apache-2.0 model card.
- [OLMo 2 0325 32B](https://huggingface.co/allenai/OLMo-2-0325-32B): official
  Apache-2.0 model card.
- [Phi-4-mini-instruct](https://huggingface.co/microsoft/Phi-4-mini-instruct):
  official MIT model card.

## Mathematical and repository evidence

- [Encyclopedia of Mathematics: Normal algorithm](https://encyclopediaofmath.org/wiki/Normal_algorithm):
  a normal algorithm is an ordered finite system of substitution formulas; the
  formalism is universal. It does not support calling each isolated rule
  Turing-complete.
- `src/solver.rs`, `src/engine.rs`, `src/link_store.rs`, and `data/seed/` ground
  the symbolic-runtime description.
- `src/statement_audit/` grounds statement extraction, evidence weighting,
  contradictions, learned memory, reference links, and relative probability.
- `data/training/source-registry.json` remains the authority for approval and
  contains zero sources at review time.
