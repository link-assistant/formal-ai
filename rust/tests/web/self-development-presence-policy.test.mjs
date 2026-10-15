// Source/data controls only. Rust unit tests require hosted native acceptance.
import test from 'node:test';
 import assert from 'node:assert/strict';
 import fs from 'node:fs';
const source=p=>fs.readFileSync(new URL('../../../'+p,
 import.meta.url),
 'utf8');
test('reviewed general policy is explicitly report and grants no recipe credit',
 ()=>{const config=source('data/meta/self-development-policy.lino');
 assert.match(config,
 /self-development-policy\s+schema-version "1"\s+presence-mode "report"/);
 assert.match(config,
 /zero autonomous authorship credit/);
 assert.doesNotMatch(config,
 /1221|1202|head.ref|branch/);
 });
test('shared parser fails closed for unknown, duplicate and incomplete policy',
 ()=>{const code=source('scripts/self-development-policy.rs');
 for(const text of ['unknown presence-mode',
 'unknown or duplicate policy field',
 'policy root and schema-version 1 required',
 'presence-mode required',
 'could not read',
 'could not write self-development summary'])assert.ok(code.includes(text),
 text);
 assert.ok(code.includes('if line == "self-development-policy"'));
 assert.ok(code.includes('["schema-version", "\\\"1\\\""]'));
 assert.ok(code.includes('["presence-mode", value]'));
 assert.ok(code.includes('mode.is_none()'));
 assert.ok(code.includes('if !version'));
 assert.ok(code.includes('if self == Self::Enforce { return Ok(false); }'));
 assert.ok(code.includes('targetSatisfied={target_status}'));
 assert.ok(code.includes('floorSatisfied={floor_satisfied}'));
 assert.ok(code.includes('recipeTransportCredit=0'));
 });
test('standalone contribution uses actual strict measurement with compatible Rust edition',
 ()=>{const code=source('scripts/check-formal-ai-contribution.rs');
 assert.match(code,
 /edition = "2024"/);
 assert.match(code,
 /self_hosting_metric::measure\(&repo, &since, &until\)/);
 assert.doesNotMatch(code,
 /fn count_formal_ai_commits|strip_prefix\(MODEL_TRAILER\)/);
 assert.ok(code.includes('measurement.self_authored_commits'));
 assert.ok(code.indexOf('self_hosting_metric::measure')<code.indexOf('policy.report_unmet'));
 assert.match(code,
 /if commits == 0/);
 assert.match(code,
 /false, None/);
 assert.match(code,
 /if verdict.is_failure\(\)/);
 });
test('cycle reports independent floor and target after strict validation without swallowing failures',
 ()=>{const code=source('scripts/check-self-development-release.rs');
 assert.ok(code.includes('let measurement = self_hosting_metric::measure(&repo, &since, "HEAD")?;'));
 assert.ok(code.indexOf('let measurement = self_hosting_metric::measure')<code.indexOf('policy.report_unmet'));
 assert.ok(code.includes('self_hosting_metric::EvidencePolicy::Strict'));
 assert.ok(code.includes('!pull_requests.is_empty(), Some(projected >= target)'));
 assert.ok(code.includes('return Err(reason);'));
 assert.ok(code.includes('Some(hours) if hours > RELAXATION_HOURS'));
 assert.doesNotMatch(code,
 /unwrap_or_default\(\).*policy|continue-on-error|\|\| true/);
 });
test('strict attribution, debt workflow and historical exceptions remain independently enforced',
 ()=>{for(const workflow of ['.github/workflows/evidence-check.yml',
 '.github/workflows/self-development-status.yml']){const code=source(workflow);
 assert.ok(code.includes('rust-script scripts/self-hosting-metric.rs'));
 assert.doesNotMatch(code,
 /continue-on-error|\|\| true/);
 }assert.ok(source('.github/workflows/self-development-status.yml').includes('rust-script scripts/check-debt-ratchet.rs --base origin/main'));
 const helper=source('scripts/check-release-recovery-authoring-deferral.mjs');
 assert.doesNotMatch(helper,
 /1221/);
 for(const n of ['1196',
 '1197',
 '1200',
 '1202'])assert.ok(helper.includes(n));
 });
test('existing strict metric and zero-credit historical measurement definitions remain intact',
 ()=>{const metric=source('scripts/self-hosting-metric.rs');
 assert.match(metric,
 /pub fn measure\([\s\S]*?EvidencePolicy::Strict/);
 assert.ok(metric.includes('merged_self_authored_pull_requests'));
 const loop=source('scripts/self-development-loop.rs');
 assert.ok(loop.includes('pub fn merged_self_authored_pull_requests'));
 assert.ok(loop.includes('if pull_requests.is_empty()'));
 assert.ok(loop.includes('if projected < target'));
 });
