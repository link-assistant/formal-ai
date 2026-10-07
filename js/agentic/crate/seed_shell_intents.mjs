// `crate::seed::shell_intent_vocabulary` (rust/src/seed/shell_intents.rs),
// read from data/seed/shell-intents.lino. Shapes keep the Rust field names;
// `ShellIntentArgument` values are 'none' | 'path' | 'name_lead' |
// 'one_path' | 'two_paths' | 'remainder' | 'search_query'.

import { cached, readText } from '../host.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';

const childrenNamed = (node, name) => (node.children || []).filter((child) => child.name === name);

const collectLanguageValues = (group, name) => childrenNamed(group, 'language')
  .flatMap((language) => childrenNamed(language, name).map((child) => child.value));

const lowerAll = (values) => values.map((value) => value.toLowerCase());

const ARGUMENTS = ['path', 'name_lead', 'one_path', 'two_paths', 'remainder', 'search_query'];

/** Mirrors `fn parse_intent`. */
function parseIntent(node) {
  const argument = findChildValue(node, 'argument');
  const effectNode = (node.children || []).find((child) => child.name === 'effect');
  const templates = (name) => (effectNode ? childrenNamed(effectNode, name).map((child) => child.value) : []);
  return {
    command: findChildValue(node, 'command'),
    argument: ARGUMENTS.includes(argument) ? argument : 'none',
    cues: lowerAll(collectLanguageValues(node, 'cue')),
    effect: { before: templates('before'), prepare: templates('prepare'), after: templates('after') },
  };
}

/** Mirrors `ShellIntentEffect::is_declared`. */
export function effectIsDeclared(effect) {
  return effect.before.length > 0 || effect.prepare.length > 0 || effect.after.length > 0;
}

/** Mirrors `fn shell_intent_vocabulary`. */
export function shellIntentVocabulary() {
  return cached('shell-intent-vocabulary', () => {
    const vocab = {
      name_leads: [], argument_noise: [], cue_fillers: [], local_search_scopes: [], local_path_search_command_template: '',
      local_path_search_actions: [], local_path_search_scopes: [], local_path_search_kinds: [],
      workspace_commands: [], path_objects: [],
      directory_listing: { verbs: [], questions: [], objects: [], scopes: [] },
      intents: [],
    };
    const root = parseRoot(readText('data/seed/shell-intents.lino')).children[0];
    if (!root) return vocab;
    for (const group of root.children || []) {
      const first = (name) => (group.children || []).find((child) => child.name === name);
      switch (group.name) {
        case 'name_leads':
          vocab.name_leads = collectLanguageValues(group, 'lead');
          break;
        case 'argument_noise':
          vocab.argument_noise = collectLanguageValues(group, 'word');
          break;
        case 'cue_fillers':
          vocab.cue_fillers = lowerAll(collectLanguageValues(group, 'word'));
          break;
        case 'local_search_scopes':
          vocab.local_search_scopes = collectLanguageValues(group, 'scope');
          break;
        case 'local_path_search': {
          vocab.local_path_search_command_template = findChildValue(group, 'command_template');
          const actions = first('actions');
          vocab.local_path_search_actions = actions ? collectLanguageValues(actions, 'action') : [];
          const scopes = first('scopes');
          vocab.local_path_search_scopes = scopes ? childrenNamed(scopes, 'scope').map((scope) => ({
            name: scope.value,
            root: findChildValue(scope, 'root'),
            cues: lowerAll(collectLanguageValues(scope, 'cue')),
          })) : [];
          const kinds = first('kinds');
          vocab.local_path_search_kinds = kinds ? childrenNamed(kinds, 'kind').map((kind) => ({
            predicate: findChildValue(kind, 'predicate'),
            cues: lowerAll(collectLanguageValues(kind, 'cue')),
          })) : [];
          break;
        }
        case 'workspace_commands':
          vocab.workspace_commands = childrenNamed(group, 'workspace').map((node) => ({
            marker: findChildValue(node, 'marker'),
            test: findChildValue(node, 'test'),
            install: findChildValue(node, 'install'),
            build: findChildValue(node, 'build'),
          }));
          break;
        case 'path_objects':
          vocab.path_objects = lowerAll(collectLanguageValues(group, 'object'));
          break;
        case 'directory_listing': {
          const part = (name, child) => {
            const node = first(name);
            return node ? lowerAll(collectLanguageValues(node, child)) : [];
          };
          vocab.directory_listing = {
            verbs: part('verbs', 'verb'),
            questions: part('questions', 'question'),
            objects: part('objects', 'object'),
            scopes: part('scopes', 'scope'),
          };
          break;
        }
        case 'intents':
          vocab.intents = childrenNamed(group, 'intent').map(parseIntent);
          break;
        default:
          break;
      }
    }
    return vocab;
  });
}
