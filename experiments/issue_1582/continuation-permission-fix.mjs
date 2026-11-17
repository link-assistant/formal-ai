// Supplied reusable seed/recognition repair; no autonomous-authorship claim.
import fs from 'node:fs';
function replace(file,before,after){const source=fs.readFileSync(file,'utf8');if(!source.includes(before))throw Error('Missing replacement in '+file);fs.writeFileSync(file,source.replace(before,after));}
const helper=`// Permission guidance owns a direct request, not a phrase quoted by another task.
function internetAccessHelpRequested(normalized) {
  return roleWordForms("internet-access-help").some((form) => {
    const surface = normalizePrompt(form.text);
    return surface && (normalized === surface || normalized.startsWith(surface + " ") ||
      (containsCjk(surface) && normalized.startsWith(surface)));
  });
}

`;
const worker='js/worker/formal_ai_worker_capabilities_and_runtime_rules.js';
replace(worker,'function tryFeatureCapabilityStatus(',helper+'function tryFeatureCapabilityStatus(');
for(const file of [worker,'js/worker/formal_ai_worker_solve.js'])replace(file,'lexiconMentionsRole("internet-access-help", normalized)','internetAccessHelpRequested(normalized)');
const native='rust/src/solver_handler_how.rs';
replace(native,'seed::lexicon().mentions_role("internet-access-help", normalized)','internet_access_help_requested(normalized)');
fs.appendFileSync(native,`
/// A permission phrase embedded in a translation or code task is not a grant.
pub(crate) fn internet_access_help_requested(normalized: &str) -> bool {
    seed::lexicon()
        .words_for_role("internet-access-help")
        .iter()
        .any(|surface| {
            normalized.strip_prefix(surface.as_str()).is_some_and(|tail| {
                tail.is_empty() || tail.starts_with(' ') || crate::coding::contains_cjk(surface)
            })
        })
}
`);
replace('rust/src/solver_handlers/feature_capability.rs','seed::lexicon().mentions_role("internet-access-help", normalized)','crate::solver_handler_how::internet_access_help_requested(normalized)');
const seed='data/seed/meanings-feature-capability.lino';
for(const [before,after] of [
 ['lexeme en "how do i enable internet access"','lexeme en "allow internet access" "what should i do to give you access to the internet" "how do i enable internet access"'],
 ['lexeme ru "как включить доступ к интернету"','lexeme ru "что мне сделать чтобы дать тебе доступ в интернет" "как мне разрешить доступ к интернету" "как включить доступ к интернету"'],
 ['lexeme hi "इंटरनेट की अनुमति कैसे दें"','lexeme hi "मैं आपको इंटरनेट का उपयोग करने की अनुमति कैसे दूँ" "इंटरनेट की अनुमति कैसे दें"'],
 ['lexeme zh "如何启用互联网访问"','lexeme zh "我怎样才能允许你访问互联网" "如何启用互联网访问"'],
 ['lexeme es "cómo habilitar el acceso a internet"','lexeme es "cómo puedo darte acceso a internet" "cómo habilitar el acceso a internet"'],
])replace(seed,before,after);
fs.writeFileSync('rust/tests/unit/issue_1582_permissions.rs',`//! Shared positive and negative permission requests, with networking disabled.
use formal_ai::{SolverConfig, UniversalSolver};

#[test]
fn permission_guidance_owns_direct_requests_in_every_supported_language() {
    let fixtures: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../data/parity/issue-1582-permissions.json"),
    ).expect("shared fixture")).expect("fixture JSON");
    let solver = UniversalSolver::new(SolverConfig { offline: true, ..SolverConfig::default() });
    for row in fixtures.as_array().expect("fixture rows") {
        let prompt = row["prompt"].as_str().expect("prompt");
        let answer = solver.solve(prompt);
        if row["guidance"] == true {
            assert_eq!(answer.intent, "capabilities", "{prompt}: {}", answer.answer);
            assert!(answer.answer.contains("/permissions"), "{prompt}: {}", answer.answer);
            assert!(answer.answer.contains("FORMAL_AI_OFFLINE"));
        } else {
            assert_ne!(answer.intent, "capabilities", "{prompt}: {}", answer.answer);
        }
    }
}
`);
replace('rust/tests/unit/mod.rs','mod issue_1588_intent_spans;','mod issue_1582_permissions;\nmod issue_1588_intent_spans;');
fs.appendFileSync('rust/tests/web/issue-1582-permissions.test.mjs',`
test('the generated TypeScript recognizer executes the same shared cases',{timeout:15000},async()=>{
  const source=readFileSync(new URL('../../../ts/worker/formal_ai_worker_capabilities_and_runtime_rules.ts',import.meta.url),'utf8');
  const helper=source.match(/function internetAccessHelpRequested[\\s\\S]*?(?=function tryFeatureCapabilityStatus)/u)?.[0];
  assert(helper,'the TypeScript projection must include the shared request recognizer');
  await host.boot();
  await host.run(helper);
  for(const row of fixtures){
    const accepted=await host.run('internetAccessHelpRequested(normalizePrompt(__prompt))',{__prompt:row.prompt});
    assert.equal(accepted,row.guidance,row.prompt);
  }
});
`);
console.log('Applied seed aliases, anchored request ownership and native/TypeScript twins');
