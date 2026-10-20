import { callbackValue as canonicalUnaryValue } from '../callback_values.mjs';
import {cached, parseLino, readText} from '../host.mjs';
const PATH='data/seed/text-formalization.lino';
export function grammarRoot() {
  return cached('dialogue-formalization-grammar-root',()=>{
    const parsed=parseLino(readText(PATH));
    return parsed.name==='text-formalization'?parsed:
      (parsed.children??[]).find(node=>node.name==='text-formalization');
  });
}

/** Exact language-neutral facts from the existing shared formalization grammar. */
export function structuralRuleValues(rule,kind) {
  return cached('formalization-structural-'+rule+'-'+kind,()=>{
    const records=(grammarRoot()?.children??[]).filter(node=>node.name==='structural-rule'&&node.value===rule);
    if(records.length!==1)throw Error('unique_structural_grammar_rule');
    const values=(records[0].children??[]).filter(node=>node.name===kind).map(canonicalUnaryValue);
    if(!values.length||values.some(value=>!value)||new Set(values).size!==values.length)throw Error('nonempty_unique_structural_grammar_facts');
    return Object.freeze(values);
  });
}

export function numericIdentityPrefix(unresolved) {
  const values=structuralRuleValues('numeric-identity',unresolved?'unresolved-prefix':'resolved-prefix');
  if(values.length!==1)throw Error('unique_numeric_identity_prefix');
  return values[0];
}


/** Local values override defaults only for a declared supported language. */
export function grammarVocabulary(language, kind) {
  const root = grammarRoot();
  const record = (root?.children ?? []).find(node =>
    node.name === 'language' && node.value === language);
  if (!record) return [];
  const local = (record.children ?? []).filter(node => node.name === kind);
  const defaults = (root.children ?? []).find(node => node.name === 'defaults');
  const values = local.length ? local :
    (defaults?.children ?? []).filter(node => node.name === kind);
  return values.map(canonicalUnaryValue);
}
