// Evaluated by the genuine fresh VM loader before client overrides.
// No registration, adoption, or caller-issued profile interface.
const formalAiParserCacheBootstrap = (function () {
  'use strict';
  const global = globalThis, own = Object.getOwnPropertyDescriptor;
  const keys = Reflect.ownKeys, proto = Object.getPrototypeOf;
  const same = Object.is, apply = Reflect.apply, freeze = Object.freeze;
  const names = ['Object','String','Array','RegExp','Number','Boolean','JSON','Reflect','Symbol','Function','Set','Map','WeakSet','WeakMap','Error','TypeError','RangeError','parseInt'];
  const namespace = [], owners = [], globalPrototype = proto(global);
  function value(record,key) { const field=own(record,key); return field ? field.value : undefined; }
  function equal(a,b) {
    if (!a || !b) return a===b;
    const fields=['value','get','set','writable','enumerable','configurable'];
    for(let n=0;n<fields.length;n++) if(!same(value(a,fields[n]),value(b,fields[n]))) return false;
    return true;
  }
  function capture(owner) {
    for(let n=0;n<owners.length;n++) if(owners[n].owner===owner) return;
    const fields=keys(owner), descriptors=[];
    for(let n=0;n<fields.length;n++) descriptors[n]=own(owner,fields[n]);
    owners[owners.length]={owner,fields,descriptors,prototype:proto(owner)};
  }
  for(let n=0;n<names.length;n++) {
    const field=own(global,names[n]);
    if(!field || !own(field,'value')) throw new Error('MissingFreshParserIntrinsic');
    namespace[n]=field;capture(field.value);
    const prototype=own(field.value,'prototype');
    if(prototype && value(prototype,'value')) capture(value(prototype,'value'));
  }
  const regexp=value(namespace[3],'value'), regexGetters=[];
  const regexKeys=keys(regexp);
  for(let n=0;n<regexKeys.length;n++) {
    const field=own(regexp,regexKeys[n]), getter=field && value(field,'get');
    // Symbol.species is a getter but not mutable match state.
    if(typeof regexKeys[n]==='string' && typeof getter==='function') regexGetters[regexGetters.length]=getter;
  }
  function intact() {
    if(proto(global)!==globalPrototype) return false;
    for(let n=0;n<names.length;n++) if(!equal(own(global,names[n]),namespace[n])) return false;
    for(let n=0;n<owners.length;n++) {
      const row=owners[n];if(proto(row.owner)!==row.prototype) return false;
      const current=keys(row.owner);if(current.length!==row.fields.length) return false;
      for(let k=0;k<current.length;k++) if(current[k]!==row.fields[k] || !equal(own(row.owner,current[k]),row.descriptors[k])) return false;
    }
    return true;
  }
  function regexState() {
    if(!intact()) return null;
    const state=[];
    for(let n=0;n<regexGetters.length;n++) state[n]=apply(regexGetters[n],regexp,[]);
    return state;
  }
  return freeze({intact,regexState});
})();
