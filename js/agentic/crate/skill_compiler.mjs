// `crate::skill_compiler::looks_like_skill_description`
// (rust/src/skill_compiler.rs): whether a request teaches a skill.

import { afterSlot, beforeSlot, roleWordForms, slotOf } from './seed_meanings.mjs';

/** Mirrors `fn direct_role_surface_present`. */
function directRoleSurfacePresent(role, lower) {
  return roleWordForms(role).some((form) => form.text !== '' && lower.includes(form.text));
}

/** Mirrors `fn explicit_teaching_form`. */
function explicitTeachingForm(lower) {
  return (directRoleSurfacePresent('skill_teaching_trigger_lead', lower)
    && directRoleSurfacePresent('skill_teaching_response_verb', lower))
    || directRoleSurfacePresent('behavior_rule_edit_directive', lower);
}

/**
 * Mirrors `fn looks_like_skill_description` in rust/src/skill_compiler.rs.
 * Offsets found in the lowercased text slice the original, as in Rust.
 * @param {string} description
 */
export function looksLikeSkillDescription(description) {
  const lower = description.toLowerCase();
  if (explicitTeachingForm(lower)) return true;
  return roleWordForms('skill_when_then_pair')
    .filter((form) => slotOf(form) === 'circumfix')
    .some((form) => {
      const head = beforeSlot(form);
      const link = afterSlot(form);
      const headPos = lower.indexOf(head);
      if (headPos < 0) return false;
      const linkPos = lower.slice(headPos + head.length).indexOf(link);
      if (linkPos < 0) return false;
      const absoluteLinkPos = headPos + head.length + linkPos;
      const beforeLink = description.slice(headPos, absoluteLinkPos);
      const afterLink = description.slice(absoluteLinkPos + link.length);
      return beforeLink.includes('`') && afterLink.includes('`');
    });
}
