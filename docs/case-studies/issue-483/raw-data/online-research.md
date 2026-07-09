# Issue 483 Online Research

## Local Prior Art

- `link-assistant/model-in-browser`:
  <https://github.com/link-assistant/model-in-browser>

  The referenced repository is described as an "Experiment to run model in
  browser" and was last pushed on 2026-06-11 in the captured metadata
  (`model-in-browser-repo.json`). This supports a browser-only experiment, but
  not bundling a model into Formal AI.

## Browser Runtime Candidates

- WebLLM home: <https://webllm.mlc.ai/>
- WebLLM GitHub repository: <https://github.com/mlc-ai/web-llm>
- WebLLM paper: <https://arxiv.org/abs/2402.18520>

  WebLLM is a browser LLM runtime built around WebGPU and WebAssembly, exposes
  OpenAI-compatible APIs, and supports worker-based execution so inference does
  not block the UI. This fits the issue's browser fallback direction, but the
  PR avoids adding WebLLM as a dependency because no runtime may load until
  explicit opt-in.

- Transformers.js documentation: <https://huggingface.co/docs/transformers.js>
- Transformers.js WebGPU guide:
  <https://huggingface.co/docs/transformers.js/guides/webgpu>

  Transformers.js can run models in the browser and supports WebGPU and
  quantized dtypes. It remains a possible runtime, but it does not remove the
  need for the formalization-safe option-selection contract.

## Model Metadata Captures

The public rating used by the catalog is the captured Hugging Face downloads
plus likes for models with available API pages.

| Model | Captured downloads | Captured likes | Public rating | Raw capture |
| --- | ---: | ---: | ---: | --- |
| `mlc-ai/SmolLM2-360M-Instruct-q4f16_1-MLC` | 80,150 | 0 | 80,150 | `hf-smollm2-360m.json` |
| `mlc-ai/Qwen2.5-0.5B-Instruct-q4f16_1-MLC` | 34,920 | 4 | 34,924 | `hf-qwen2.5-0.5b.json` |
| `mlc-ai/SmolLM2-1.7B-Instruct-q4f16_1-MLC` | 503 | 0 | 503 | `hf-smollm2-1.7b.json` |
| `Phi-3.5-mini-instruct-q4f16_1-MLC-1k` | unavailable | unavailable | 0 | `hf-phi3.5-mini-1k.json` |

Relevant WebLLM catalog values captured during research:

| WebLLM model id | Approximate VRAM gate | Low-resource | Required feature | Context |
| --- | ---: | --- | --- | ---: |
| `SmolLM2-360M-Instruct-q4f16_1-MLC` | 376.06 MB | yes | `shader-f16` | 4096 |
| `Qwen2.5-0.5B-Instruct-q4f16_1-MLC` | 944.62 MB | yes | none | 4096 |
| `SmolLM2-1.7B-Instruct-q4f16_1-MLC` | 1774.19 MB | yes | `shader-f16` | 4096 |
| `Phi-3.5-mini-instruct-q4f16_1-MLC-1k` | 2520.07 MB | yes | none | 1024 |

The implementation rounds these gates up to whole megabytes for UI display and
filtering.

## Formal-First Hybrid Pattern

- Habr, "SymFSM" article: <https://habr.com/ru/articles/1047344/>

  The useful idea is not to let an LLM own state or decisions. Instead, the LLM
  may fill constrained slots while a formal mechanism validates and repairs the
  result. Issue 483 applies the same principle to formalization: a model may
  name an existing option, while Formal AI owns candidate generation,
  validation, and final selection.
