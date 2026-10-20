// Projecting planner arguments onto the client's advertised tool schema
// (rust/src/protocol_responses.rs).

import { currentDirectory } from '../host.mjs';
import { jsonText } from '../plan.mjs';
import { findToolDefinition } from '../protocol_policy.mjs';
import { applyPatchInput } from './protocol_responses_apply_patch.mjs';
import { groundedIdentityArgument, isIdentityArgument } from './tool_scope.mjs';

export { applyPatchInput };

const isObject = (value) => Boolean(value) && typeof value === 'object' && !Array.isArray(value);
const has = (object, key) => Object.prototype.hasOwnProperty.call(object, key);
const sortedKeys = (object) => Object.keys(object).sort();

/** Mirrors `fn is_custom_response_tool` in rust/src/protocol_responses.rs. */
export function isCustomResponseTool(tools, toolName) {
  return findToolDefinition(tools, toolName)?.type === 'custom';
}

/** Mirrors `fn custom_response_tool_input` in rust/src/protocol_responses.rs. */
export function customResponseToolInput(tools, toolName, args) {
  const definition = findToolDefinition(tools, toolName);
  if (!definition || definition.type !== 'custom') return args;
  const leaf = toolName.split('__').pop();
  if (!leaf.toLowerCase().includes('patch')) return args;
  return applyPatchInput(args) ?? args;
}

/**
 * Mirrors `fn response_arguments_for_tool` in rust/src/protocol_responses.rs.
 * @param {Array<object>} tools
 * @param {string} toolName
 * @param {string} args
 * @param {string} userPrompt
 * @param {string|null} workspace
 * @returns {string}
 */
export function responseArgumentsForTool(tools, toolName, args, userPrompt, workspace) {
  const definition = findToolDefinition(tools, toolName);
  if (!definition) return args;
  const schema = toolParametersSchema(definition);
  if (schema === undefined) return args;
  const properties = isObject(schema) && isObject(schema.properties) ? schema.properties : null;
  if (!properties || !Object.keys(properties).length) return args;
  let source;
  try {
    source = JSON.parse(args);
  } catch {
    return args;
  }
  if (!isObject(source)) return args;
  return jsonText(projectSchemaValue(source, schema, definition, userPrompt, workspace, 0));
}

/** Mirrors `fn project_schema_value` in rust/src/protocol_responses.rs; invalid values remain available to boundary validation. */
function projectSchemaValue(source, schema, definition, userPrompt, workspace, depth) {
  if (depth > 32 || !isObject(schema) || has(schema, '$ref')) return source;
  if (Array.isArray(source)) {
    if (!has(schema, 'items')) return source;
    return source.map((item) => projectSchemaValue(item, schema.items, definition, userPrompt, workspace, depth + 1));
  }
  if (!isObject(source)) return source;
  const properties = isObject(schema.properties) ? schema.properties : null;
  if (!properties || !Object.keys(properties).length) return source;
  const required = Array.isArray(schema.required) ? schema.required : [];
  const createsFile = callWritesBytes(properties, source);
  const projected = Object.create(null);
  for (const name of sortedKeys(properties)) {
    const propertySchema = properties[name];
    let value = has(source, name) ? source[name] : semanticAlias(name, source, userPrompt);
    if (value === undefined && required.some((entry) => entry === name)) {
      value = missingRequiredValue(propertySchema, name, userPrompt);
    }
    if (value === undefined) continue;
    if (typeof value === 'string' && demandsAbsolutePath(definition, name, propertySchema)) {
      const absolute = absolutePath(value, workspace, createsFile);
      if (absolute !== null) value = absolute;
    }
    projected[name] = projectSchemaValue(value, propertySchema, definition, userPrompt, workspace, depth + 1);
  }
  return projected;
}

/** Mirrors `fn tool_parameters_schema`; `undefined` when absent. */
function toolParametersSchema(tool) {
  if (!isObject(tool)) return undefined;
  if (has(tool, 'parameters')) return tool.parameters;
  if (has(tool, 'input_schema')) return tool.input_schema;
  if (isObject(tool.function) && has(tool.function, 'parameters')) return tool.function.parameters;
  if (isObject(tool.function) && has(tool.function, 'input_schema')) return tool.function.input_schema;
  return undefined;
}

const ALIASES = [
  [['path', 'filePath', 'file_path', 'absolute_path'], ['path', 'filePath', 'file_path', 'absolute_path']],
  [['command', 'cmd'], ['command', 'cmd']],
  [['query', 'pattern', 'input'], ['query', 'pattern', 'input']],
  [['paths', 'file_paths', 'files'], ['paths', 'file_paths', 'files']],
  [['old', 'oldString', 'old_string', 'old_str'], ['old', 'oldString', 'old_string', 'old_str']],
  [['new', 'newString', 'new_string', 'new_str'], ['new', 'newString', 'new_string', 'new_str']],
];

/** Mirrors `fn semantic_alias`; `undefined` for `None`. */
function semanticAlias(name, source, userPrompt) {
  if (name === 'prompt' || name === 'instruction') return userPrompt;
  const row = ALIASES.find(([names]) => names.includes(name));
  if (!row) return undefined;
  const alias = row[1].find((candidate) => has(source, candidate));
  return alias === undefined ? undefined : source[alias];
}

const PATH_PROPERTIES = ['path', 'filePath', 'file_path', 'absolute_path', 'dir_path', 'directory', 'notebook_path'];

/** Mirrors `fn demands_absolute_path`. */
function demandsAbsolutePath(definition, name, propertySchema) {
  if (!PATH_PROPERTIES.includes(name)) return false;
  if (name === 'absolute_path') return true;
  const saysAbsolute = (value) => typeof value === 'string' && value.toLowerCase().includes('absolute');
  return saysAbsolute(isObject(propertySchema) ? propertySchema.description : undefined)
    || saysAbsolute(toolDescription(definition));
}

/** Mirrors `fn tool_description`. */
function toolDescription(definition) {
  if (has(definition, 'description')) return definition.description;
  return isObject(definition.function) ? definition.function.description : undefined;
}

/**
 * `Path::join` for a relative `path` under `root`.
 * @param {string} root
 * @param {string} path
 * @returns {string}
 */
function joinPath(root, path) {
  return root.endsWith('/') ? `${root}${path}` : `${root}/${path}`;
}

/** `std::path::absolute` on Unix: join with the cwd, drop `.` and empty components. */
function stdAbsolute(path) {
  const base = currentDirectory();
  if (base === null) return path;
  const joined = joinPath(base, path);
  const parts = joined.split('/').filter((part, index) => index === 0 || (part !== '' && part !== '.'));
  const out = parts.join('/');
  return joined.endsWith('/') && !out.endsWith('/') ? out : out || '/';
}

/** Mirrors `fn absolute_path`. */
function absolutePath(path, workspace, createsFile) {
  if (path.startsWith('/')) return path;
  if (workspace && workspace.startsWith('/')) return joinPath(workspace, path);
  if (createsFile) return null;
  return stdAbsolute(path);
}

const CONTENT_PROPERTIES = ['content', 'contents', 'file_text', 'text', 'new_string'];

/** Mirrors `fn call_writes_bytes`. */
function callWritesBytes(properties, source) {
  return CONTENT_PROPERTIES.some((name) => has(properties, name) && has(source, name));
}

const FREE_TEXT = ['prompt', 'instruction', 'message', 'description', 'commit_message', 'commitMessage'];

/** Explicit schema defaults/constants and grounded semantic fields only. */
function missingRequiredValue(schema, name, userPrompt) {
  if (isObject(schema) && has(schema, 'default')) return schema.default;
  if (isObject(schema) && has(schema, 'const')) return schema.const;
  if (isIdentityArgument(name)) return groundedIdentityArgument(name, userPrompt) ?? undefined;
  if (isObject(schema) && Array.isArray(schema.enum)) return schema.enum.length === 1 ? schema.enum[0] : undefined;
  if (FREE_TEXT.includes(name)) return userPrompt;
  if (name === 'is_background') return false;
  if (name === 'login') return true;
  return undefined;
}
