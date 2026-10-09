import pathlib,json,hashlib
p=pathlib.Path('/private/tmp/pr1188-ci-T4053');changes=[]
def edit(path,fn):
 f=pathlib.Path(path);b=f.read_text();a=fn(b);assert a!=b;changes.append(dict(path=str(f.resolve()),before_sha256=hashlib.sha256(b.encode()).hexdigest(),after_sha256=hashlib.sha256(a.encode()).hexdigest(),content=a,before=b))
def once(s,b,a):assert s.count(b)==1;return s.replace(b,a)
edit('js/agentic/planner.mjs',lambda s:once(s,'  const ownedGoals = await planGoalLedger(task, messages, toolNames, planChatStepResolved);','  const ownedGoals = evidenceRecord.hasTypedEvidenceDelivery(task) ? null\n    : await planGoalLedger(task, messages, toolNames, planChatStepResolved);'))
def js(s):
 s=once(s,"import { composeGeneralChangePlan } from './general_planner.mjs';","import { composeGeneralChangePlan, hasAuthoritativeLiteralWrite } from './general_planner.mjs';")
 at='function exactFieldLines(sentence, target) {';assert s.count(at)==1
 return s.replace(at,'''/** Existing exact field/pinned-line delivery owns its record grammar, outside authoritative literal bytes. */
export function hasTypedEvidenceDelivery(request) {
  if (hasAuthoritativeLiteralWrite(request)) return false;
  const binding = parseObligation(request);
  return binding !== null && (binding.field_lines.length > 0 || binding.first_line !== null);
}

'''+at)
edit('js/agentic/evidence_record.mjs',js)
def rust_planner(s):
 old='''    if let Some(plan) = super::general_planner::plan_owned_goal_step(
        &task,
        messages,
        tool_names,
        plan_chat_step_resolved,
        result,
    )
    .or_else(|| {'''
 new='''    let owned_goal = if evidence_record::has_typed_evidence_delivery(&task) {
        None
    } else {
        super::general_planner::plan_owned_goal_step(
            &task,
            messages,
            tool_names,
            plan_chat_step_resolved,
            result,
        )
    };
    if let Some(plan) = owned_goal.or_else(|| {'''
 return once(s,old,new)
edit('rust/src/agentic_coding/planner.rs',rust_planner)
def rust(s):
 s=once(s,'use super::general_planner::compose_general_change_plan;','use super::general_planner::{compose_general_change_plan, has_authoritative_literal_write};')
 at='fn parse_obligation(request: &str) -> Option<DeliveryBinding> {';index=s.index('/// Declines when the residual is empty:');assert s.count(at)==1
 helper='''/// Exact fields or a pinned first line belong to the existing evidence delivery grammar.
/// Authoritative literal payloads keep their original ownership and never become records.
pub(super) fn has_typed_evidence_delivery(request: &str) -> bool {
    !has_authoritative_literal_write(request)
        && parse_obligation(request)
            .is_some_and(|binding| !binding.field_lines.is_empty() || binding.first_line.is_some())
}

'''
 return s[:index]+helper+s[index:]
edit('rust/src/agentic_coding/evidence_record.rs',rust);(p/'request.json').write_text(json.dumps(dict(changes=changes,original_job=114023635704),indent=2))
