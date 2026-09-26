import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const corpusUrl = new URL(
  "../../../data/benchmarks/repository-world-model-canaries.lino",
  import.meta.url,
);

function records(text) {
  const result = [];
  let current;
  for (const line of text.split("\n")) {
    if (!line.trim()) continue;
    if (!line.startsWith(" ")) {
      const [kind, id = ""] = line.trim().split(/\s+/, 2);
      current = { kind, id, fields: new Map() };
      result.push(current);
      continue;
    }
    const match = line.trim().match(/^(\S+)\s+"?(.*?)"?$/);
    assert.ok(match && current, `invalid fixture line: ${line}`);
    const [, name, raw] = match;
    const value = raw.endsWith('"') ? raw.slice(0, -1) : raw;
    const values = current.fields.get(name) ?? [];
    values.push(value);
    current.fields.set(name, values);
  }
  return result;
}

test("each canary observation is evidence-bound and each requirement is explicit", async () => {
  const parsed = records(await readFile(corpusUrl, "utf8"));
  const cases = parsed.filter((record) => record.kind === "repository_case");
  const requirements = parsed.filter(
    (record) => record.kind === "repository_requirement",
  );
  const observations = parsed.filter(
    (record) => record.kind === "repository_observation",
  );

  assert.deepEqual(
    cases.map((record) => record.id),
    ["kotlin_claude_latest", "scala_agent_latest", "rust_codex_latest"],
  );
  assert.equal(requirements.length, 42);
  for (const requirement of requirements) {
    for (const field of ["case", "origin", "text", "subject", "predicate", "accepted"]) {
      assert.ok(requirement.fields.get(field)?.[0], `${requirement.id}: ${field}`);
    }
  }
  for (const observation of observations) {
    for (const field of [
      "case",
      "subject",
      "predicate",
      "value",
      "evidence_command",
      "evidence_url",
      "evidence_kind",
      "evidence_source",
    ]) {
      assert.ok(observation.fields.get(field)?.[0], `${observation.id}: ${field}`);
    }
  }
});

test("attribution records preserve tool boundaries and public external repros", async () => {
  const parsed = records(await readFile(corpusUrl, "utf8"));
  const attributions = parsed.filter(
    (record) => record.kind === "repository_attribution",
  );
  const projection = attributions.map((record) => ({
    case: record.fields.get("case")?.[0],
    component: record.fields.get("component")?.[0],
    verdict: record.fields.get("verdict")?.[0],
    externalIssue: record.fields.get("external_issue")?.[0] ?? "",
  }));

  assert.deepEqual(projection, [
    {
      case: "kotlin_claude_latest",
      component: "formal_ai",
      verdict: "defect",
      externalIssue: "",
    },
    {
      case: "kotlin_claude_latest",
      component: "hive_mind",
      verdict: "defect",
      externalIssue: "https://github.com/link-assistant/hive-mind/issues/2263",
    },
    {
      case: "kotlin_claude_latest",
      component: "claude_code",
      verdict: "downstream_corruption",
      externalIssue: "",
    },
    {
      case: "scala_agent_latest",
      component: "agent_cli",
      verdict: "positive_evidence",
      externalIssue: "",
    },
    {
      case: "scala_agent_latest",
      component: "formal_ai",
      verdict: "positive_evidence",
      externalIssue: "",
    },
    {
      case: "scala_agent_latest",
      component: "hive_mind",
      verdict: "positive_evidence",
      externalIssue: "",
    },
    {
      case: "rust_codex_latest",
      component: "hive_mind",
      verdict: "defect",
      externalIssue: "https://github.com/link-assistant/hive-mind/issues/2259",
    },
    {
      case: "rust_codex_latest",
      component: "formal_ai",
      verdict: "not_reached",
      externalIssue: "",
    },
    {
      case: "rust_codex_latest",
      component: "codex",
      verdict: "not_reached",
      externalIssue: "",
    },
  ]);
});

