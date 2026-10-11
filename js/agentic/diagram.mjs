// The issue-#538 diagram recipe (rust/src/agentic_coding/diagram.rs): the
// agentic recipes rendered as split-into-parts mermaid diagrams from the
// planner's own recipe table. The table's wording (titles, routes, step
// notes) and every prose line live in data/meta/agentic-messages.lino.

import { agenticMessage } from './messages.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** Mirrors `const RECIPES`: keys, issues and step tools; wording by message key. */
const RECIPES = [
  { key: 'formalize', issue: '#468', tools: ['web_search', 'web_fetch', 'write_file', 'run_command'] },
  { key: 'meaning', issue: '#538', tools: ['web_search', 'web_fetch', 'write_file', 'run_command'] },
  { key: 'diagram', issue: '#538', tools: ['write_file', 'run_command'] },
  { key: 'self_ast', issue: '#538', tools: ['write_file', 'run_command'] },
];

const title = (recipe) => agenticMessage(`diagram_${recipe.key}_title`);
const route = (recipe) => agenticMessage(`diagram_${recipe.key}_route`);
const note = (recipe, index) => agenticMessage(`diagram_${recipe.key}_step_${index}`);

/** Mirrors `const DIAGRAM_PATH`. */
export const DIAGRAM_PATH = 'agentic-recipes.md';

const DIAGRAM_KEYWORDS = ['mermaid', 'diagram', 'visual overview', 'flowchart'];

/**
 * Mirrors `fn is_diagram_task` in rust/src/agentic_coding/diagram.rs.
 * @param {string} prompt
 */
export function isDiagramTask(prompt) {
  // A cue inside a quoted literal is payload, not the request: "Insert the
  // line 'js/mermaid.bundle.js' after …" is an edit (PR #1188 dogfood T70).
  let unquoted = '';
  let cursor = 0;
  for (const segment of quotedSegmentSpans(prompt)) {
    unquoted += `${prompt.slice(cursor, segment.start)} `;
    cursor = segment.end;
  }
  const lower = `${unquoted}${prompt.slice(cursor)}`.toLowerCase();
  return DIAGRAM_KEYWORDS.some((keyword) => lower.includes(keyword));
}

/** Mirrors `fn render_overview`. */
function renderOverview() {
  let out = `${agenticMessage('diagram_overview_heading')}\n`;
  out += '```mermaid\n';
  out += 'flowchart TD\n';
  out += `${agenticMessage('diagram_overview_router_line')}\n`;
  for (const recipe of RECIPES) {
    out += `    router -->|"${route(recipe)}"| ${recipe.key}["${title(recipe)} (${recipe.issue})"]\n`;
  }
  out += `${agenticMessage('diagram_overview_otherwise_line')}\n`;
  out += '```\n\n';
  return out;
}

/** Mirrors `fn render_recipe`. */
function renderRecipe(part, recipe) {
  let out = `${agenticMessage('diagram_recipe_heading', { part, title: title(recipe), issue: recipe.issue })}\n`;
  out += '```mermaid\n';
  out += 'flowchart LR\n';
  let previous = `${recipe.key}_task`;
  out += `${agenticMessage('diagram_recipe_task_line', { previous, route: route(recipe) })}\n`;
  recipe.tools.forEach((tool, index) => {
    const node = `${recipe.key}_${index}`;
    out += `    ${previous} --> ${node}["${index + 1}. ${tool}<br/>${note(recipe, index)}"]\n`;
    previous = node;
  });
  out += `${agenticMessage('diagram_recipe_final_line', { previous, final_node: `${recipe.key}_final` })}\n`;
  out += '```\n\n';
  return out;
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/diagram.rs. */
export function renderDocument() {
  let out = `${agenticMessage('diagram_document_heading')}\n`;
  out += `${agenticMessage('diagram_document_comment')}\n`;
  out += `${agenticMessage('diagram_document_intro')}\n`;
  out += renderOverview();
  RECIPES.forEach((recipe, offset) => {
    out += renderRecipe(offset + 2, recipe);
  });
  return `${trimEnd(out)}\n`;
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/diagram.rs. */
export function finalAnswer(document) {
  return agenticMessage('diagram_final_answer', { parts: RECIPES.length + 1, path: DIAGRAM_PATH, document: trimEnd(document) });
}
