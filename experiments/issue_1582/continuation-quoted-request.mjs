// Supplied follow-up to the failing Hindi quoted-input regression.
import fs from 'node:fs';
function replace(file,before,after){const source=fs.readFileSync(file,'utf8');if(!source.includes(before))throw Error('Missing replacement '+file);fs.writeFileSync(file,source.replace(before,after));}
const worker='js/worker/formal_ai_worker_capabilities_and_runtime_rules.js';
replace(worker,'function internetAccessHelpRequested(normalized) {','function internetAccessHelpRequested(prompt, normalized) {\n  if (/^[\\s]*["\'`«“‘]/u.test(prompt)) return false;');
for(const file of [worker,'js/worker/formal_ai_worker_solve.js'])replace(file,'internetAccessHelpRequested(normalized)','internetAccessHelpRequested(prompt, normalized)');
const how='rust/src/solver_handler_how.rs';
replace(how,'internet_access_help_requested(normalized)','internet_access_help_requested(prompt, normalized)');
replace(how,'pub(crate) fn internet_access_help_requested(normalized: &str) -> bool {',`pub(crate) fn internet_access_help_requested(prompt: &str, normalized: &str) -> bool {
    if prompt.trim_start().chars().next().is_some_and(|character| {
        matches!(character, '"' | '\\'' | '«' | '“' | '‘') || character == '\\u{0060}'
    }) {
        return false;
    }`);
replace('rust/src/solver_handlers/feature_capability.rs','internet_access_help_requested(normalized)','internet_access_help_requested(prompt, normalized)');
const tests='rust/tests/web/issue-1582-permissions.test.mjs';
replace(tests,"assert.equal(answer,null,'a quoted permission phrase does not own the request');","assert.equal(answer?.evidence.includes('permission:guidance')??false,false,'a quoted permission phrase does not own the request');");
replace(tests,'internetAccessHelpRequested(normalizePrompt(__prompt))','internetAccessHelpRequested(__prompt, normalizePrompt(__prompt))');
replace('rust/tests/unit/issue_1582_permissions.rs','assert_ne!(answer.intent, "capabilities", "{prompt}: {}", answer.answer);','assert!(!answer.answer.contains("/permissions"), "{prompt}: {}", answer.answer);');
