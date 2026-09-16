#!/usr/bin/env bash
set -euo pipefail

# Replays candidate anticipation prompts through the production CLI while
# keeping the shared append-only memory untouched. The Responses projection
# exposes the symbolic intent used to build Markov classes.
export FORMAL_AI_RECORD_CHAT=0

prompts=(
  "hello"
  "2 + 2"
  "translate apple to Russian"
  "frobulator705 resonance vellum"
  "frobulator705 detailed resonance"
  "frobulator705 describe resonance"
  "reverse the words frobulator705 resonance"
)

for prompt in "${prompts[@]}"; do
  cargo run --quiet --bin formal-ai -- chat --format responses --prompt "$prompt"
done
