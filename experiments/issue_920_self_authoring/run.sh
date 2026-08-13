#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI authorship proof for one issue #920 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-920/self-hosting-authorship"
CANONICAL="$ROOT/data/seed/question-necessity.lino"
TASK='Finish Formal AI issue #920 by proving that each user question was necessary before asking it. As one smallest leaf of that same task, create file question-necessity.lino containing exactly
question_necessity_protocol
  record_type "question_necessity_protocol"
  required_stage "memory"
  required_stage "workspace"
  required_stage "sources"
  default_class "factual"
  maximum_questions_per_answer "1"
  source_attempt_budget "3"
question_necessity_class_requirement
  record_type "question_necessity_class"
  class "requirement"
  section_marker "clarifying questions:"
  section_marker "уточняющие вопросы:"
  section_marker "स्पष्टीकरण के प्रश्न:"
  section_marker "澄清问题:"
  section_marker "still needed from you:"
  section_marker "нужно от вас:"
  section_marker "आपसे चाहिए:"
  section_marker "需要您提供:"
  cue "should i read"
  cue "which interpretation did you mean"
  cue "would you like"
  cue "do you want"
  cue "is there anything else you want"
  cue "should we"
  cue "which exact statement do you want"
  cue "in which axiom system"
  cue "do you have a preferred"
  cue "хотите"
  cue "нужно ли"
  cue "какой именно"
  cue "в какой системе аксиом"
  cue "есть ли предпочитаемая"
  cue "क्या आप"
  cue "कौन-सा"
  cue "किस अभिगृहीत प्रणाली"
  cue "क्या कोई वांछित"
  cue "是否"
  cue "哪一个命题"
  cue "在哪一个公理系统"
  cue "偏好的证明技术"
  cue "需要我"
  cue "¿quieres"
question_necessity_class_factual
  record_type "question_necessity_class"
  class "factual"
  cue "which one source or missing fact"
  cue "какой один источник или факт"
  cue "कौन सा एक source या missing fact"
  cue "哪一个来源或缺失事实"
question_necessity_ratchet
  record_type "question_necessity_ratchet"
  metric "questions_per_100_tasks"
  direction "down"
  maximum "60"'

TASK="$TASK" \
EXPECT_FILE="question-necessity.lino" \
EXPECT_TEXT='direction "down"' \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8920}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/question-necessity.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/question-necessity.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
