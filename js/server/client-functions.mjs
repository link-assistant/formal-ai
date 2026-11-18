// Forced OpenAI functions return a validated data payload for the caller.
import { computerPrimitiveFromToolName, toolDefinitionNames } from '../agentic/protocol_policy.mjs';
import { agenticToolCapabilities } from '../agentic/crate/seed_agentic_tool_capabilities.mjs';
import { normalizePrompt } from '../agentic/crate/engine.mjs';
import { invalidArguments } from '../agentic/crate/tool_call_validation.mjs';
import { mentionsRole } from '../agentic/crate/seed_meanings.mjs';
import { agenticMessage } from '../agentic/messages.mjs';
import { jsonText } from '../agentic/plan.mjs';
import { estimateTokens, stableId } from './ids.mjs';
import { ensureNodeHost } from './node-host-install.mjs';

const isObject = value => value !== null && typeof value === 'object' && !Array.isArray(value);

async function selectedFunctions(ctx, { tool_choice: choice, tools = [], functions = [], function_call: legacyChoice }) {
  if (functions.length || (legacyChoice != null && legacyChoice !== 'none')) return [];
  const required = choice === 'required';
  const forcedName = choice?.type === 'function' ? (choice.function?.name ?? choice.name) : null;
  if (!required && typeof forcedName !== 'string') return [];
  if (!tools.some(tool => tool?.type === 'function')) return [];

  await ensureNodeHost(ctx);
  const names = new Map();
  for (const name of tools.flatMap(toolDefinitionNames)) names.set(name, (names.get(name) ?? 0) + 1);
  const aliases = new Set(agenticToolCapabilities().flatMap(capability => capability.aliases).map(alias => alias.toLowerCase()));
  return tools.flatMap(tool => {
    if (tool?.type !== 'function') return [];
    const definition = tool.function ?? tool;
    const name = definition.name;
    if (typeof name !== 'string' || !/^[a-zA-Z0-9_-]{1,64}$/u.test(name) || (!required && name !== forcedName)) return [];
    if (names.get(name) !== 1 || computerPrimitiveFromToolName(name) || aliases.has(name.toLowerCase())) return [];
    return [definition];
  });
}

export function sentimentLabel(definition, name, field, prompt) {
  const labels = ['negative', 'neutral', 'positive'];
  if (!field.enum || field.enum.length < 2 || !field.enum.every(label => labels.includes(label))) return undefined;
  const description = normalizePrompt([definition.description, name, field.description].join(' '));
  if (!mentionsRole('client-function-sentiment', description)) return undefined;
  const text = normalizePrompt(prompt);
  if (mentionsRole('client-function-sentiment-negation', text)) return undefined;
  const matched = labels.filter(label => mentionsRole(`client-function-sentiment-${label}`, text));
  return matched.length === 1 ? matched[0] : undefined;
}

function deriveArguments(definition, schema, prompt) {
  let supplied;
  try { supplied = JSON.parse(prompt); } catch { /* The user can also supply plain text. */ }
  if (isObject(supplied)) return supplied;

  const properties = Object.entries(schema.properties ?? {});
  if (!properties.length) return {};
  if (properties.length !== 1) return undefined;
  const [[name, field]] = properties;
  if (!isObject(field)) return undefined;
  const text = prompt.trim();
  const value = supplied === undefined
    ? (field.enum?.includes(text) ? text : sentimentLabel(definition, name, field, prompt))
    : supplied;
  return value === undefined ? undefined : { [name]: value };
}

// Refuse constraints outside the protocol validator's supported vocabulary.
// Annotation keys do not affect validation.
function supportedSchema(schema, depth = 0) {
  if (depth > 32) return false;
  if (typeof schema === 'boolean') return true;
  if (!isObject(schema)) return false;
  const nested = value => supportedSchema(value, depth + 1);
  return Object.entries(schema).every(([key, value]) => {
    switch (key) {
      case 'properties': return isObject(value) && Object.values(value).every(nested);
      case 'additionalProperties':
      case 'items': return nested(value);
      case 'allOf':
      case 'anyOf': return Array.isArray(value) && value.length > 0 && value.every(nested);
      case 'required': return Array.isArray(value) && value.every(item => typeof item === 'string');
      case 'enum': return Array.isArray(value) && value.length > 0;
      case 'minLength':
      case 'maxLength':
      case 'minItems':
      case 'maxItems': return Number.isSafeInteger(value) && value >= 0;
      case 'minimum':
      case 'maximum': return Number.isFinite(value);
      case 'type': {
        const kinds = Array.isArray(value) ? value : [value];
        return kinds.length > 0 && kinds.every(kind =>
          ['null', 'boolean', 'object', 'array', 'string', 'number', 'integer'].includes(kind));
      }
      case 'title':
      case 'description':
      case 'default':
      case 'examples':
      case 'const': return true;
      default: return false;
    }
  });
}

function serializableArguments(value, depth = 0) {
  if (depth > 32) return false;
  if (typeof value === 'number') return Number.isFinite(value) && (!Number.isInteger(value) || Number.isSafeInteger(value));
  if (value && typeof value === 'object') return Object.values(value).every(item => serializableArguments(item, depth + 1));
  return true;
}

export async function forcedFunction(ctx, request, prompt) {
  const definitions = await selectedFunctions(ctx, request);
  if (!definitions.length) return null;
  for (const definition of definitions) {
    const { parameters: schema = { additionalProperties: false } } = definition;
    if (!isObject(schema) || !supportedSchema(schema)) continue;
    const value = deriveArguments(definition, schema, prompt);
    if (value !== undefined && serializableArguments(value) && !invalidArguments(value, schema)) {
      return { name: definition.name, arguments: jsonText(value) };
    }
  }
  return { refusal: agenticMessage('client-function-arguments-underivable', { tool: definitions[0].name }) };
}

export function clientFunctionOutput(result, prompt, messageCount) {
  if (result.refusal) return { outputTokens: estimateTokens(result.refusal) };
  return {
    callId: stableId('call', `${prompt}|${messageCount}|0|${result.name}|${result.arguments}`),
    outputTokens: estimateTokens(result.name) + estimateTokens(result.arguments),
  };
}
