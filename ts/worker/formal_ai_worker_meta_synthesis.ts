// @worker-prerequisite js/seed_loader.js
// @worker-prerequisite js/worker/formal_ai_worker_meta_reasoner.js
// The meta reasoner's synthesis half (see formal_ai_worker_meta_reasoner.js):
// typed program enumeration over the instruction set in
// data/seed/meta-reasoning.lino, parameter and threshold inference from the
// request's examples, rendering, and verification of the rendered source.
// Domain-free: every operation, filter and probe is seed data.
/**
 * The type of a runtime value in the instruction set's vocabulary.
 * @param {*} value
 * @returns {string}
 */
function metaTypeOf(value) {
  if (typeof value === "string") return "text";
  if (typeof value === "number") return "number";
  if (Array.isArray(value)) {
    if (value.length && value.every((item) => typeof item === "number")) return "list_number";
    if (value.length && value.every((item) => typeof item === "string")) return "list_text";
    return "list_any";
  }
  return "unknown";
}
/**
 * The type a primitive produces from a concrete input type, or null.
 * @param {object} primitive
 * @param {string} type
 * @returns {string|null}
 */
function metaApply(primitive, type) {
  if (primitive.from === type) return primitive.to === "list_any" ? type : primitive.to;
  if (primitive.from === "list_any" && type.startsWith("list_")) return primitive.to === "list_any" ? type : primitive.to;
  return null;
}
const metaMemoSourceFunctions = (() => {
  const profile = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaSeedIntrinsicProfile') === undefined && typeof metaSeedIntrinsicProfile === 'object' && metaSeedIntrinsicProfile !== null ? metaSeedIntrinsicProfile : null;
  const helpers = {apply: metaApply, measures: metaMeasures, uses: metaStepUsesAny};
  return profile !== null && profile.check() ? profile.freeze(helpers) : formalAiWorkerPrimordials.freeze(helpers);
})();
/**
 * A parameter as the JavaScript literal its code is given: a number as
 * written, a text or a list of values as JSON.
 * @param {*} value
 * @returns {string}
 */
function metaLiteral(value) {
  return typeof value === "string" || Array.isArray(value) ? JSON.stringify(value) : String(value);
}
/**
 * An operation's code with its parameter slots filled: `{k}` takes the whole
 * parameter, `{k0}`, `{k1}` ... the values of a list parameter in order.
 * @param {string} code
 * @param {*} parameter
 * @returns {string}
 */
function metaSubstitute(code, parameter) {
  const values = Array.isArray(parameter) ? parameter : [parameter];
  let out = String(code).split("{k}").join(metaLiteral(parameter));
  for (const [index, value] of values.entries()) out = out.split(`{k${index}}`).join(metaLiteral(value));
  return out;
}
/**
 * Compiled primitive functions, keyed by id and parameter.
 * @param {object} primitive
 * @param {*} parameter
 * @returns {function}
 */
function metaCompile(primitive, parameter) {
  const source = metaSubstitute(primitive.code, parameter);
  const cached = metaCompiled.get(source);
  if (cached) return cached;
  // eslint-disable-next-line no-new-func -- the instruction set is seed data.
  const fn = new Function(`return (${source});`)();
  metaCompiled.set(source, fn);
  return fn;
}
/**
 * The measures a list of `element` can be filtered by: every primitive from
 * the element type to a number, and a number element itself. A text measure
 * is the element's content: the element itself when it is text, or what one
 * operation reads from it ("the files that contain": read the file).
 * @param {string} element
 * @param {string} [type] the measure's result type, number by default
 * @returns {Array<object>}
 */
const metaMeasureProducer = (() => {
  let issuing = false;
  const earlyProfile = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaSeedIntrinsicProfile') === undefined && typeof metaSeedIntrinsicProfile === 'object' && metaSeedIntrinsicProfile !== null ? metaSeedIntrinsicProfile : null;
  if (earlyProfile === null || !earlyProfile.check()) return formalAiWorkerPrimordials.freeze({produce: produceMeasures, owns: () => false});
  const apply = Reflect.apply, descriptor = Object.getOwnPropertyDescriptor;
  const descriptors = Object.getOwnPropertyDescriptors, keys = Reflect.ownKeys;
  const proto = Object.getPrototypeOf, equal = Object.is;
  const setHas = Set.prototype.has, setAdd = Set.prototype.add;
  const setValues = Set.prototype.values, mapEntries = Map.prototype.entries;
  const weakHas = WeakSet.prototype.has, weakAdd = WeakSet.prototype.add;
  const allocated = new WeakSet();
  const parser = descriptor(globalThis, 'metaSeedProducer') === undefined
    && typeof metaSeedProducer === 'object' ? metaSeedProducer : null;
  const parserOwns = parser === null ? null : parser.owns;
  const documentProducer = descriptor(globalThis, 'metaDocIndexProducer') === undefined
    && typeof metaDocIndexProducer === 'object' && metaDocIndexProducer !== null ? metaDocIndexProducer : null;
  const documentQuery = documentProducer === null ? null : descriptor(documentProducer, 'owns');
  const documentOwns = documentQuery && descriptor(documentQuery, 'value') && documentQuery.writable === false && documentQuery.configurable === false && typeof documentQuery.value === 'function' ? documentQuery.value : null;
  const globals = ['Object', 'Array', 'String', 'Map', 'Set', 'WeakSet', 'Reflect'];
  const values = globals.map(name => descriptor(globalThis, name).value);
  const owners = [Object, Array, String, Map, Set, WeakSet, Reflect, Function.prototype, Object.prototype, Array.prototype, String.prototype, Map.prototype, Set.prototype, WeakSet.prototype,
    proto(apply(mapEntries, new Map(), [])), proto(apply(setValues, new Set(), [])), proto(apply(Array.prototype.values, [], [])), proto(proto(apply(Array.prototype.values, [], [])))];
  const profiles = owners.map(owner => descriptors(owner));
  const objectKeys = keys(Object.prototype);
  const sourceSeed = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaMemoSourceSeed') === undefined && typeof metaMemoSourceSeed === 'function' ? metaMemoSourceSeed : null;
  const sourceHelpers = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaMemoSourceFunctions') === undefined && typeof metaMemoSourceFunctions === 'object' && metaMemoSourceFunctions !== null ? metaMemoSourceFunctions : null;
  const seedFactory = sourceSeed, applyPrimitive = sourceHelpers === null ? null : sourceHelpers.apply;
  function own(value) {return apply(weakHas, allocated, [value]) || parserOwns !== null && parserOwns(value) || documentOwns !== null && documentOwns(value);}
  function unchanged(owner, saved) {
    const fields = descriptors(owner), names = keys(saved), current = keys(fields);
    if (names.length !== current.length) return false;
    for (let i = 0; i < names.length; i++) {
      if (names[i] !== current[i]) return false;
      const a = fields[names[i]], b = saved[names[i]], namesA = keys(a), namesB = keys(b);
      if (namesA.length !== namesB.length) return false;
      for (let j = 0; j < namesA.length; j++) if (namesA[j] !== namesB[j] || !equal(a[namesA[j]], b[namesB[j]])) return false;
    }
    return true;
  }
  function intact() {
    if (!earlyProfile.check()) return false;
    const now = keys(Object.prototype);
    if (now.length !== objectKeys.length) return false;
    for (let i = 0; i < now.length; i++) if (now[i] !== objectKeys[i]) return false;
    for (let i = 0; i < globals.length; i++) {const field = descriptor(globalThis, globals[i]); if (!field || !descriptor(field, 'value') || field.value !== values[i]) return false;}
    if (seedFactory === null || applyPrimitive === null || parser === null || documentOwns === null) return false;
    const currentDocumentQuery = descriptor(documentProducer, 'owns');
    if (!currentDocumentQuery || !descriptor(currentDocumentQuery, 'value') || currentDocumentQuery.value !== documentOwns || currentDocumentQuery.writable !== false || currentDocumentQuery.configurable !== false) return false;
    for (let i = 0; i < owners.length; i++) if (!unchanged(owners[i], profiles[i])) return false;
    const seedNow = descriptor(globalThis, 'metaSeed'), applyNow = descriptor(globalThis, 'metaApply');
    if (!seedNow || !applyNow || !descriptor(seedNow, 'value') || !descriptor(applyNow, 'value') || seedNow.value !== seedFactory || applyNow.value !== applyPrimitive) return false;
    return true;
  }
  function admitted(value, seen = new Set()) {
    if (value === null || typeof value !== 'object') return typeof value !== 'function' && typeof value !== 'symbol';
    if (!own(value)) return false;
    if (apply(setHas, seen, [value])) return true;
    apply(setAdd, seen, [value]);
    const parent = proto(value);
    if (parent !== Object.prototype && parent !== Array.prototype && parent !== Map.prototype && parent !== Set.prototype && parent !== null) return false;
    const fields = descriptors(value);
    for (const key of keys(fields)) {if (typeof key !== 'string' || !descriptor(fields[key], 'value') || !admitted(fields[key].value, seen)) return false;}
    if (parent === Map.prototype) for (const pair of apply(mapEntries, value, [])) if (!admitted(pair[0], seen) || !admitted(pair[1], seen)) return false;
    if (parent === Set.prototype) for (const member of apply(setValues, value, [])) if (!admitted(member, seen)) return false;
    return true;
  }
  function allocation(value) {if (issuing) apply(weakAdd, allocated, [value]); return value;}
  function produceMeasures(element, type) {
  const seed = metaSeed();
  if (!seed.measureCache) seed.measureCache = allocation(new Map());
  const key = `${element} ${type || "number"}`;
  if (seed.measureCache.has(key)) return seed.measureCache.get(key);
  const out = allocation([]);
  if (type === "text") {
    if (element === "text") out.push(allocation({ id: "value", from: "text", to: "text", doc: "", code: "(input) => input", infer: "", environment: "", takes: allocation([]) }));
    for (const primitive of seed.primitives) {
      if (primitive.from !== element || primitive.to !== "text" || element === "text" || primitive.infer || primitive.effect || primitive.takes.length) continue;
      out.push(allocation({ ...primitive, parts: allocation([primitive.id]) }));
    }
    seed.measureCache.set(key, out);
    return out;
  }
  if (element === "number") out.push(allocation({ id: "value", from: "number", to: "number", doc: "", code: "(input) => input", infer: "", environment: "", takes: allocation([]) }));
  // A measure is any short parameter-free program from the element to a
  // number ("the lines of a file": read the file, split its lines, count).
  let frontier = [{ ids: [], codes: [], type: element, environment: "" }];
  for (let size = 1; size <= META_BOUNDS.measureLength; size += 1) {
    const next = [];
    for (const partial of frontier) {
      for (const primitive of seed.primitives) {
        if (primitive.infer || primitive.effect || primitive.takes.length) continue;
        const type = metaApply(primitive, partial.type);
        if (!type) continue;
        const grown = {
          ids: allocation(partial.ids.concat(primitive.id)),
          codes: partial.codes.concat(primitive.code),
          type,
          environment: partial.environment || primitive.environment || "",
        };
        if (type === "number") {
          const code = `(input) => ${grown.codes.reduce((inner, step) => `(${step})(${inner})`, "input")}`;
          out.push(allocation({ id: grown.ids.join("∘"), parts: grown.ids, from: element, to: "number", doc: "", code, infer: "", environment: grown.environment, takes: allocation([]) }));
        } else {
          next.push(grown);
        }
      }
    }
    frontier = next;
  }
  seed.measureCache.set(key, out);
  return out;
}
  function produce(element, type) {
    const prior = issuing;
    issuing = intact() && typeof element === 'string' && (type === undefined || typeof type === 'string') && admitted(seedFactory());
    try {return produceMeasures(element, type);} finally {issuing = prior;}
  }
  return Object.freeze({produce, owns: value => apply(weakHas, allocated, [value])});
})();
function metaMeasures(element, type) {return metaMeasureProducer.produce(element, type);}

/**
 * All typed programs from a type, shortest first, each step a primitive or a
 * primitive mapped over a list.
 * @param {string} fromType
 * @param {number} length
 * @param {Set<string>|null} [mustUse] when given, only the programs using one
 *   of these operations (see `metaStepUsesAny`) are returned
 * @returns {Array<{steps: Array<object>, type: string}>}
 */
function createMetaProgramMemo() {
  const original = function originalMetaPrograms(fromType, length, mustUse = null) {
  // Call-local descriptors cannot retain a different request, seed or worker realm.
  const transitionsByType = new Map();
  const primitives = metaSeed().primitives;
  let frontier = [mustUse === null ? { steps: [], type: fromType } : { steps: [], type: fromType, uses: false }];
  const out = [];
  for (let size = 1; size <= length; size += 1) {
    const next = [];
    // A last-length program using none of `mustUse` is never built.
    const extend = (program, step, type) => {
      const uses = mustUse === null ? null : program.uses || metaStepUsesAny(step, mustUse);
      if (mustUse !== null && size === length && !uses) return;
      const grown = { steps: program.steps.concat(step), type };
      if (mustUse !== null) grown.uses = uses;
      next.push(grown);
    };
    for (const program of frontier) {
      let typedTransitions = transitionsByType.get(program.type);
      if (typedTransitions === undefined) {
        typedTransitions = [];
        const recordTransition = (step, type) => typedTransitions.push({ step, type });

        for (const primitive of primitives) {
          const direct = metaApply(primitive, program.type);
          if (direct) recordTransition({ primitive, mapped: false }, direct);
          if (program.type.startsWith("list_")) {
            const element = program.type.slice(5);
            const mapped = metaApply(primitive, element);
            if (mapped && !mapped.startsWith("list_")) recordTransition({ primitive, mapped: true }, `list_${mapped}`);
          }
        }
        // A file is edited in place by any text transformation: read it,
        // transform the text, write it back ("rewrite_file").
        if (program.type === "path" || program.type === "list_path") {
          for (const primitive of primitives) {
            if (primitive.from !== "text" || primitive.to !== "text" || primitive.infer || primitive.takes.length) continue;
            recordTransition({ primitive, mapped: program.type === "list_path", rewrite: true }, program.type);
          }
        }
        if (program.type.startsWith("list_")) {
          for (const measure of metaMeasures(program.type.slice(5))) {
            for (const filter of metaSeed().filters) {
              if (filter.measure === "number") recordTransition({ primitive: measure, mapped: false, filter }, program.type);
            }
          }
          // A filter comparing the element's content with a value.
          for (const filter of metaSeed().filters) {
            if (filter.measure === "number") continue;
            for (const measure of metaMeasures(program.type.slice(5), filter.measure)) recordTransition({ primitive: measure, mapped: false, filter }, program.type);
          }
          // A selector picks one element by its measure ("the longest word").
          for (const measure of metaMeasures(program.type.slice(5))) {
            for (const select of metaSeed().selectors) recordTransition({ primitive: measure, mapped: false, select }, program.type.slice(5));
          }
        }

        transitionsByType.set(program.type, typedTransitions);
      }
      for (const transition of typedTransitions) extend(program, { ...transition.step }, transition.type);
    }
    for (const program of next) if (mustUse === null || program.uses) out.push(program);
    frontier = next;
  }
  return out;
};
  const earlyProfile = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaSeedIntrinsicProfile') === undefined && typeof metaSeedIntrinsicProfile === 'object' && metaSeedIntrinsicProfile !== null ? metaSeedIntrinsicProfile : null;
  if (earlyProfile === null || !earlyProfile.check()) return {lookup: (fromType, length, mustUse) => ({done: true, result: original(fromType, length, mustUse)}), store() {}};
  const sourceSeed = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaMemoSourceSeed') === undefined && typeof metaMemoSourceSeed === 'function' ? metaMemoSourceSeed : null;
  const sourceHelpers = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaMemoSourceFunctions') === undefined && typeof metaMemoSourceFunctions === 'object' && metaMemoSourceFunctions !== null ? metaMemoSourceFunctions : null;
  const seedFactory = sourceSeed;
  // The producer is a genuine source-declared lexical constant, not a
  // mutable global callback or caller-created property of the VM global.
  const producer = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaSeedProducer') === undefined
    && typeof metaSeedProducer === 'object' && metaSeedProducer !== null ? metaSeedProducer : null;
  const seedObjectOwnership = producer === null ? null : producer.owns;
  const documentObjectOwnership = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaDocIndexProducer') === undefined && typeof metaDocIndexProducer === 'object' && metaDocIndexProducer !== null ? metaDocIndexProducer.owns : null;
  const measureProducer = formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaMeasureProducer') === undefined && typeof metaMeasureProducer === 'object' ? metaMeasureProducer : null;
  const measureObjectOwnership = measureProducer === null ? null : measureProducer.owns;
  const originalOwnershipQuery = producer === null ? null : formalAiWorkerPrimordials.ownDescriptor(globalThis, 'metaSeedOriginalOwnershipQuery') === undefined && typeof metaSeedOriginalOwnershipQuery === 'function' ? metaSeedOriginalOwnershipQuery : null;
  const apply = sourceHelpers === null ? null : sourceHelpers.apply;
  const measures = sourceHelpers === null ? null : sourceHelpers.measures;
  const uses = sourceHelpers === null ? null : sourceHelpers.uses;
  const nativeApply = Reflect.apply;
  const descriptor = Object.getOwnPropertyDescriptor;
  const descriptors = Object.getOwnPropertyDescriptors;
  const prototype = Object.getPrototypeOf;
  const keys = Reflect.ownKeys;
  const mapEntries = Map.prototype.entries;
  const setValues = Set.prototype.values;
  const setHas = Set.prototype.has;
  const operationBrandWitness = {};
  const globalOwner = globalThis;
  const constructorFields = ['Object', 'Array', 'Map', 'Set', 'String'];
  const constructorValues = [Object, Array, Map, Set, String];
  const intrinsicFields = [[Object, 'getOwnPropertyDescriptor', descriptor],
    [Object, 'getOwnPropertyDescriptors', descriptors], [Object, 'getPrototypeOf', prototype],
    [Reflect, 'ownKeys', keys], [Object, 'is', Object.is], [Array, 'from', Array.from],
    [Array.prototype, Symbol.iterator, Array.prototype[Symbol.iterator]],
    [globalOwner, 'metaSeed', seedFactory], [globalOwner, 'metaSeedOwnsObject', originalOwnershipQuery], [globalOwner, 'metaApply', apply],
    [globalOwner, 'metaMeasures', measures], [globalOwner, 'metaStepUsesAny', uses]];
  const equal = Object.is;
  const prototypeOwners = [Object, Array, Map, Set, String, Object.prototype, Array.prototype, Map.prototype, Set.prototype,
    String.prototype, Function.prototype, JSON, Reflect,
    prototype(nativeApply(mapEntries, new Map(), [])), prototype(nativeApply(setValues, new Set(), [])),
    prototype([][Symbol.iterator]()), prototype(prototype([][Symbol.iterator]()))];
  const prototypeFields = prototypeOwners.map(owner => descriptors(owner));
  const bounds = typeof META_BOUNDS === 'object' && META_BOUNDS !== null ? META_BOUNDS : null;
  const boundsBinding = descriptor(globalOwner, 'META_BOUNDS');
  const objectPrototype = Object.prototype;
  const originalObjectKeys = keys(objectPrototype);
  const initialBounds = bounds === null ? null : descriptors(bounds);
  let latest = null;
  function fieldsMatch(owner, expected) {
    const actual = descriptors(owner);
    const names = keys(expected), current = keys(actual);
    if (names.length !== current.length) return false;
    for (let i = 0; i < names.length; i += 1) {
      if (names[i] !== current[i]) return false;
      const a = actual[names[i]], b = expected[names[i]];
      // Captured native reflection produces own data fields only. Compare
      // exact own keys first so missing fields never read inherited accessors.
      const attributes = keys(a), expectedAttributes = keys(b);
      if (attributes.length !== expectedAttributes.length) return false;
      for (let j = 0; j < attributes.length; j += 1) {
        const name = attributes[j];
        if (name !== expectedAttributes[j] || !equal(a[name], b[name])) return false;
      }
    }
    return true;
  }
  function intact() {
    if (seedFactory === null || apply === null || measures === null || uses === null || initialBounds === null) return false;
    const currentObjectKeys = keys(objectPrototype);
    if (currentObjectKeys.length !== originalObjectKeys.length) return false;
    for (let i = 0; i < originalObjectKeys.length; i += 1) if (currentObjectKeys[i] !== originalObjectKeys[i]) return false;
    for (let i = 0; i < prototypeOwners.length; i += 1) if (!fieldsMatch(prototypeOwners[i], prototypeFields[i])) return false;
    if (boundsBinding !== undefined) {
      const current = descriptor(globalOwner, 'META_BOUNDS');
      const value = current === undefined ? undefined : descriptor(current, 'value');
      if (value === undefined || value.value !== bounds) return false;
    }
    for (let i = 0; i < constructorFields.length; i += 1) {
      const field = descriptor(globalOwner, constructorFields[i]);
      const value = field === undefined ? undefined : descriptor(field, 'value');
      if (value === undefined || value.value !== constructorValues[i]) return false;
    }
    for (let i = 0; i < intrinsicFields.length; i += 1) {
      const owner = intrinsicFields[i][0], name = intrinsicFields[i][1], expected = intrinsicFields[i][2];
      const field = descriptor(owner, name);
      const value = field === undefined ? undefined : descriptor(field, 'value');
      if (value === undefined || value.value !== expected) return false;
    }
    if (!fieldsMatch(bounds, initialBounds)) return false;

    return true;
  }
  function snapshot(value) {
    const out = [];
    const visited = new Set();
    function visit(item) {
      if (item === null || typeof item !== 'object') {
        if (typeof item === 'function' || typeof item === 'symbol') return false;
        out.push(item); return true;
      }
      if (!(seedObjectOwnership !== null && seedObjectOwnership(item) || measureObjectOwnership !== null && measureObjectOwnership(item) || documentObjectOwnership !== null && documentObjectOwnership(item))) return false;
      out.push(item);
      if (visited.has(item)) return true;
      visited.add(item);
      const proto = prototype(item);
      if (proto !== Object.prototype && proto !== Array.prototype && proto !== Map.prototype && proto !== Set.prototype && proto !== null) return false;
      out.push(proto);
      const fields = descriptors(item);
      for (const key of keys(fields)) {
        if (typeof key !== 'string') return false;
        const field = fields[key];
        const value = descriptor(field, 'value');
        if (value === undefined) return false;
        // A native data descriptor has these three own data attributes.
        out.push(key, field.writable, field.enumerable, field.configurable);
        if (!visit(value.value)) return false;
      }
      if (proto === Map.prototype) {
        for (const pair of nativeApply(mapEntries, item, [])) if (!visit(pair[0]) || !visit(pair[1])) return false;
      }
      if (proto === Set.prototype) for (const member of nativeApply(setValues, item, [])) if (!visit(member)) return false;
      out.push('end'); return true;
    }
    return visit(value) ? out : null;
  }
  function same(a, b) {
    if (!a || !b || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i += 1) if (!equal(a[i], b[i])) return false;
    return true;
  }
  function copy(programs) {
    const steps = new Map();
    const out = [];
    for (const program of programs) {
      const next = {steps: [], type: program.type};
      if ('uses' in program) next.uses = program.uses;
      for (const step of program.steps) {
        let fresh = steps.get(step);
        if (fresh === undefined) {fresh = {...step}; steps.set(step, fresh);}
        next.steps.push(fresh);
      }
      out.push(next);
    }
    return out;
  }
  return {
    lookup(fromType, length, mustUse) {
      const fallback = () => ({done: true, result: original(fromType, length, mustUse)});
      if (!intact() || typeof metaSeedCache === 'undefined' || metaSeedCache === null) return fallback();
      if (mustUse !== null) {
        // Native Set brand validation invokes no user getters or Proxy traps.
        try {nativeApply(setHas, mustUse, [operationBrandWitness]);} catch {return fallback();}
        if (prototype(mustUse) !== Set.prototype || keys(descriptors(mustUse)).length !== 0) return fallback();
      }
      const required = mustUse === null ? null : Array.from(nativeApply(setValues, mustUse, []));
      const before = snapshot(metaSeedCache);
      if (!before || required !== null && required.some(value => typeof value !== 'string')) return fallback();
      const knownTypes = new Set();
      for (const primitive of metaSeedCache.primitives) {
        for (const type of [primitive.from, primitive.to]) {
          knownTypes.add(type);
          if (typeof type === 'string' && !type.startsWith('list_')) knownTypes.add(`list_${type}`);
        }
      }
      if (!knownTypes.has(fromType)) return fallback();
      if (!(latest && latest.seed === metaSeedCache && latest.length === length
        && (latest.required === null && required === null || same(latest.required, required))
        && same(latest.snapshot, before))) {
        latest = {seed: metaSeedCache, length, required, snapshot: before, programs: new Map()};
      }
      if (latest.programs.has(fromType)) return {done: true, result: copy(latest.programs.get(fromType))};
      return {done: false, seed: metaSeedCache, type: fromType, length, required, group: latest};
    },
    store(state, programs) {
      if (!intact() || state.seed !== metaSeedCache) {latest = null; return;}
      const after = snapshot(metaSeedCache);
      if (!after || state.group !== latest) {latest = null; return;}
      if (!same(latest.snapshot, after)) latest.programs.clear();
      latest.snapshot = after;
      latest.programs.set(state.type, copy(programs));
    },
  };
}
const metaProgramMemo = createMetaProgramMemo();

function metaPrograms(fromType, length, mustUse = null) {
  const memo = metaProgramMemo.lookup(fromType, length, mustUse);
  if (memo.done) return memo.result;
  // Call-local descriptors cannot retain a different request, seed or worker realm.
  const transitionsByType = new Map();
  const primitives = metaSeed().primitives;
  const transitionsFor = (type) => {
    const existing = transitionsByType.get(type);
    if (existing !== undefined) return existing;
    const typedTransitions = [];
    const recordTransition = (step, type) => typedTransitions.push({ step, type });

    for (const primitive of primitives) {
      const direct = metaApply(primitive, type);
      if (direct) recordTransition({ primitive, mapped: false }, direct);
      if (type.startsWith("list_")) {
        const element = type.slice(5);
        const mapped = metaApply(primitive, element);
        if (mapped && !mapped.startsWith("list_")) recordTransition({ primitive, mapped: true }, `list_${mapped}`);
      }
    }
    // A file is edited in place by any text transformation: read it,
    // transform the text, write it back ("rewrite_file").
    if (type === "path" || type === "list_path") {
      for (const primitive of primitives) {
        if (primitive.from !== "text" || primitive.to !== "text" || primitive.infer || primitive.takes.length) continue;
        recordTransition({ primitive, mapped: type === "list_path", rewrite: true }, type);
      }
    }
    if (type.startsWith("list_")) {
      for (const measure of metaMeasures(type.slice(5))) {
        for (const filter of metaSeed().filters) {
          if (filter.measure === "number") recordTransition({ primitive: measure, mapped: false, filter }, type);
        }
      }
      // A filter comparing the element's content with a value.
      for (const filter of metaSeed().filters) {
        if (filter.measure === "number") continue;
        for (const measure of metaMeasures(type.slice(5), filter.measure)) recordTransition({ primitive: measure, mapped: false, filter }, type);
      }
      // A selector picks one element by its measure ("the longest word").
      for (const measure of metaMeasures(type.slice(5))) {
        for (const select of metaSeed().selectors) recordTransition({ primitive: measure, mapped: false, select }, type.slice(5));
      }
    }

    transitionsByType.set(type, typedTransitions);
    return typedTransitions;
  };
  const reachabilityByType = new Map();
  const canUseWithin = (type, remaining) => {
    if (remaining === 0) return false;
    let depths = reachabilityByType.get(type);
    if (depths === undefined) {
      depths = new Map();
      reachabilityByType.set(type, depths);
    }
    if (depths.has(remaining)) return depths.get(remaining);
    const reachable = transitionsFor(type).some(transition =>
      metaStepUsesAny(transition.step, mustUse)
      || canUseWithin(transition.type, remaining - 1));
    depths.set(remaining, reachable);
    return reachable;
  };
  let frontier = [mustUse === null ? { steps: [], type: fromType } : { steps: [], type: fromType, uses: false }];
  const out = [];
  for (let size = 1; size <= length; size += 1) {
    const next = [];
    // A last-length program using none of `mustUse` is never built.
    const extend = (program, step, type) => {
      const uses = mustUse === null ? null : program.uses || metaStepUsesAny(step, mustUse);
      if (mustUse !== null && size === length && !uses) return;
      const grown = { steps: program.steps.concat(step), type };
      if (mustUse !== null) grown.uses = uses;
      next.push(grown);
    };
    for (const program of frontier) {
      if (mustUse !== null && !program.uses
        && !canUseWithin(program.type, length - size + 1)) continue;
      const typedTransitions = transitionsFor(program.type);
      for (const transition of typedTransitions) extend(program, { ...transition.step }, transition.type);
    }
    for (const program of next) if (mustUse === null || program.uses) out.push(program);
    frontier = next;
  }
  metaProgramMemo.store(memo, out);
  return out;
}


/**
 * A step's operations for evidence scoring.
 * @param {object} step
 * @returns {Array<string>}
 */
function metaStepOperations(step) {
  if (step.rewrite) return step.mapped ? [step.primitive.id, "rewrite_file", "map_each"] : [step.primitive.id, "rewrite_file"];
  const chooser = step.filter || step.select;
  if (chooser) return step.primitive.id === "value" ? [chooser.id] : [step.primitive.id, chooser.id];
  return step.mapped ? [step.primitive.id, "map_each"] : [step.primitive.id];
}

/**
 * Whether a step uses one of `operations` -- in `metaStepOperations(step)` or,
 * for a filter or selector, its measure's parts -- without building a list.
 */
function metaStepUsesAny(step, operations) {
  const chooser = step.filter || step.select;
  if (!(chooser && step.primitive.id === "value") && operations.has(step.primitive.id)) return true;
  if (chooser) return operations.has(chooser.id) || (step.primitive.parts || []).some((part) => operations.has(part));
  return (step.rewrite && operations.has("rewrite_file")) || (step.mapped && operations.has("map_each"));
}

/**
 * A step's operations with a filter's or selector's measure folded into it:
 * the measure serves the filter, not a word of the request.
 * @param {object} step
 * @returns {Array<string>}
 */
function metaStepMainOperations(step) {
  const chooser = step.filter || step.select;
  return chooser ? [chooser.id] : metaStepOperations(step);
}

/**
 * True when a step takes the program's parameter.
 * @param {object} step
 * @returns {boolean}
 */
function metaStepIsParametric(step) {
  return Boolean(step.filter || step.primitive.infer || (step.primitive.takes || []).length);
}

/**
 * The types of the values a parametric step reads: a filter's threshold
 * (its measure type), an inferred number, or the values an operation takes.
 * @param {object} step
 * @returns {Array<string>}
 */
function metaStepParameterTypes(step) {
  if (step.filter) return [step.filter.measure];
  if (step.primitive.infer) return ["number"];
  return (step.primitive.takes || []).slice();
}

/**
 * Run a program on one input; a parametric last step takes `parameter`.
 * @param {Array<object>} steps
 * @param {*} input
 * @param {*} parameter
 * @returns {*}
 */
function metaRun(steps, input, parameter) {
  let value = input;
  for (const step of steps) {
    const fn = metaCompile(step.primitive, step.primitive.infer || (step.primitive.takes || []).length ? parameter : null);
    if (step.select) {
      const test = metaCompile({ code: step.select.test }, null);
      value = value.reduce((best, item) => (test(fn(item), fn(best)) ? item : best));
    } else if (step.rewrite) {
      const rewrite = metaCompile({ code: metaRewriteCode(step.primitive.code) }, null);
      value = step.mapped ? value.map((item) => rewrite(item)) : rewrite(value);
    } else if (step.filter) {
      const test = metaCompile({ code: step.filter.test }, null);
      value = value.filter((item) => test(fn(item), parameter));
    } else {
      value = step.mapped ? value.map((item) => fn(item)) : fn(value);
    }
  }
  return value;
}

/**
 * Infer the parameter of a program's parametric last step: an arithmetic
 * parameter from the first example, a filter threshold from all of them.
 * @param {Array<object>} steps
 * @param {Array<{input: *, output: *}>} examples
 * @returns {*}
 */
function metaInferParameter(steps, examples) {
  const last = steps[steps.length - 1];
  const example = examples[0];
  if (steps.slice(0, -1).some(metaStepIsParametric)) return undefined;
  if (last.filter) return last.filter.measure === "number" ? metaInferThreshold(steps, examples) : undefined;
  if ((last.primitive.takes || []).length) return undefined;
  if (!last.primitive.infer) return null;
  const before = metaRun(steps.slice(0, -1), example.input, null);
  // eslint-disable-next-line no-new-func -- the inference rule is seed data.
  const infer = new Function(`return (${last.primitive.infer});`)();
  const pair = last.mapped ? [before[0], Array.isArray(example.output) ? example.output[0] : undefined] : [before, example.output];
  if (pair[0] === undefined || pair[1] === undefined) return undefined;
  const parameter = infer(pair[0], pair[1]);
  return typeof parameter === "number" && Number.isFinite(parameter) ? parameter : undefined;
}

/**
 * The threshold separating the kept elements from the dropped ones across
 * every example, or undefined when the output is not a filtering of the
 * input or no threshold separates them.
 * @param {Array<object>} steps
 * @param {Array<{input: *, output: *}>} examples
 * @returns {number|undefined}
 */
function metaInferThreshold(steps, examples) {
  const last = steps[steps.length - 1];
  const measure = metaCompile(last.primitive, null);
  const kept = [];
  const dropped = [];
  for (const example of examples) {
    const before = metaRun(steps.slice(0, -1), example.input, null);
    if (!Array.isArray(before) || !Array.isArray(example.output)) return undefined;
    let cursor = 0;
    for (const item of before) {
      if (cursor < example.output.length && JSON.stringify(item) === JSON.stringify(example.output[cursor])) {
        kept.push(measure(item));
        cursor += 1;
      } else {
        dropped.push(measure(item));
      }
    }
    if (cursor !== example.output.length) return undefined;
  }
  if (!kept.length || !dropped.length) return undefined;
  const test = metaCompile({ code: last.filter.test }, null);
  for (const candidate of dropped.concat(kept).sort((a, b) => a - b)) {
    if (kept.every((value) => test(value, candidate)) && dropped.every((value) => !test(value, candidate))) return candidate;
  }
  return undefined;
}

/**
 * Render a program as the JavaScript the answer shows and the verifier runs.
 * @param {Array<object>} steps
 * @param {*} parameter
 * @returns {string}
 */
function metaRender(steps, parameter) {
  const lines = ["function solution(input) {", "  let value = input;"];
  for (const step of steps) {
    const code = metaSubstitute(step.primitive.code, parameter);
    if (step.select) {
      lines.push(`  value = value.reduce((best, item) => ((${step.select.test})((${code})(item), (${code})(best)) ? item : best)); // ${step.select.id} by ${step.primitive.id}`);
      continue;
    }
    if (step.rewrite) {
      const rewrite = metaRewriteCode(code);
      lines.push(step.mapped ? `  value = value.map(${rewrite}); // each: rewrite_file by ${step.primitive.id}` : `  value = (${rewrite})(value); // rewrite_file by ${step.primitive.id}`);
      continue;
    }
    if (step.filter) {
      const test = step.filter.test.replace(/^\(measure, threshold\) => /u, "");
      lines.push(`  value = value.filter((item) => { const measure = (${code})(item); const threshold = ${metaLiteral(parameter)}; return ${test}; }); // ${step.filter.id} by ${step.primitive.id}`);
      continue;
    }
    lines.push(step.mapped ? `  value = value.map(${code}); // each: ${step.primitive.id}` : `  value = (${code})(value); // ${step.primitive.id}`);
  }
  lines.push("  return value;", "}");
  return lines.join("\n");
}
/**
 * Run rendered source on every example; the verifier never trusts the
 * enumerator's own evaluation.
 * @param {string} source
 * @param {Array<object>} examples
 * @returns {{passed: number, total: number, failures: Array<object>}}
 */
function metaVerify(source, examples) {
  // eslint-disable-next-line no-new-func -- running the composed answer is the verification.
  const solution = new Function(`${source}\nreturn solution;`)();
  const failures = [];
  for (const example of examples) {
    let actual;
    try {
      actual = solution(example.input);
    } catch (error) {
      actual = `error: ${error.message}`;
    }
    if (JSON.stringify(actual) !== JSON.stringify(example.output)) failures.push({ input: example.input, expected: example.output, actual });
  }
  return { passed: examples.length - failures.length, total: examples.length, failures };
}
/**
 * Evidence for a program: the grounded score of every operation it uses.
 * @param {Array<object>} steps
 * @param {Map<string, number>} evidence
 * @returns {number}
 */
function metaEvidenceScore(steps, evidence) {
  let score = 0;
  for (const operation of new Set(steps.flatMap(metaStepOperations))) score += evidence.get(operation) || 0;
  return score;
}
/**
 * Coverage of the request: each grounded word credits once, with the best
 * of its hypotheses the program uses.
 * @param {Array<object>} steps
 * @param {Array<Array<{operation: string, score: number}>>} words
 * @returns {number}
 */
function metaCoverageScore(steps, words) {
  // A filter's measure serves the filter, not a word of the request.
  const used = new Set(steps.flatMap(metaStepMainOperations));
  let score = 0;
  for (const hypotheses of words) {
    let best = 0;
    for (const hypothesis of hypotheses) if (used.has(hypothesis.operation) && hypothesis.score > best) best = hypothesis.score;
    score += best;
  }
  return score;
}
/**
 * Search for a program whose difference from the goal (failing examples) is
 * zero: shortest first, then most evidenced.
 * @param {Array<object>} examples
 * @param {Map<string, number>} evidence
 * @param {object} trace
 * @returns {object|null}
 */
function metaSynthesizeFromExamples(examples, evidence, trace) {
  const fromType = metaTypeOf(examples[0].input);
  const toType = metaTypeOf(examples[0].output);
  trace.emit("goal", metaNote("examples_goal", { from: fromType, to: toType, count: examples.length }));
  let evaluated = 0;
  let rejected = 0;
  for (let length = 1; length <= META_BOUNDS.programLength; length += 1) {
    const programs = metaPrograms(fromType, length)
      .filter((program) => program.steps.length === length && (program.type === toType || (toType === "list_any" && program.type.startsWith("list_"))))
      .sort((a, b) => metaEvidenceScore(b.steps, evidence) - metaEvidenceScore(a.steps, evidence));
    const passing = [];
    for (const program of programs) {
      if (evaluated >= META_BOUNDS.candidateBudget) break;
      evaluated += 1;
      let parameter;
      try {
        parameter = metaInferParameter(program.steps, examples);
      } catch {
        parameter = undefined;
      }
      if (parameter === undefined) continue;
      let difference = 0;
      let counterexample = null;
      for (const example of examples) {
        let actual;
        try {
          actual = metaRun(program.steps, example.input, parameter);
        } catch {
          actual = undefined;
        }
        if (JSON.stringify(actual) !== JSON.stringify(example.output)) {
          difference += 1;
          if (!counterexample) counterexample = { input: example.input, expected: example.output, actual };
        }
      }
      if (difference === 0) {
        passing.push({ steps: program.steps, parameter, evidence: metaEvidenceScore(program.steps, evidence) });
      } else {
        rejected += 1;
        if (rejected <= META_BOUNDS.rejectionsTraced && metaEvidenceScore(program.steps, evidence) > 0) {
          trace.emit("counterexample", `${program.steps.map(metaStepLabel).join(" ∘ ")}: ${JSON.stringify(counterexample.input)} gave ${JSON.stringify(counterexample.actual)}, expected ${JSON.stringify(counterexample.expected)} (difference ${difference})`);
        }
      }
    }
    trace.emit("search", `length ${length}: ${programs.length} typed candidate(s), ${passing.length} with difference 0`);
    if (passing.length) {
      passing.sort((a, b) => b.evidence - a.evidence);
      const best = passing[0];
      const ties = passing.filter((item) => item.evidence === best.evidence);
      if (ties.length > 1) trace.emit("tie", metaNote("tie", { count: ties.length, kept: ties.slice(0, 3).map((item) => item.steps.map(metaStepLabel).join(" ∘ ")).join(" | ") }));
      return { steps: best.steps, parameter: best.parameter, alternatives: ties.length - 1, evaluated };
    }
  }
  trace.emit("impasse", metaNote("no_program", { length: META_BOUNDS.programLength, evaluated }));
  return null;
}
/**
 * A short label for one program step.
 * @param {object} step
 * @returns {string}
 */
function metaStepLabel(step) {
  if (step.rewrite) return step.mapped ? `each(rewrite(${step.primitive.id}))` : `rewrite(${step.primitive.id})`;
  if (step.filter || step.select) return `${(step.filter || step.select).id}(${step.primitive.id})`;
  return step.mapped ? `each(${step.primitive.id})` : step.primitive.id;
}
/**
 * Without examples, the goal is a program that uses one operation from every
 * grounded word's hypothesis group, preferring a type-preserving program.
 * @param {Array<Array<string>>} groups
 * @param {Map<string, number>} evidence
 * @param {object} trace
 * @param {Array<{value: *, type: string}>} values the values the request names, in order, bound to a parametric step
 * @param {Array<Array<object>>} words every grounded word's hypotheses
 * @param {Array<string>} inputTypes argument types the request's data words name
 * @param {Array<object>} clauses the request's clauses (see metaClauses)
 * @param {{weakWords?: Array<Array<object>>, weakViews?: Array<Array<object>>, universal?: boolean, measureWords?: Array<Array<object>>|null}} [hints]
 *   words tied across many operations, the views tied plural nouns name,
 *   whether the request quantifies
 *   universally ("every file"), and the bound's unit word naming a measure
 * @returns {object|null}
 */
function metaSynthesizeFromMeaning(groups, evidence, trace, values, words, inputTypes, clauses, hints) {
  const { weakWords = [], weakViews = [], universal = false, measureWords = null } = hints || {};
  trace.emit("goal", `program serving ${groups.map((group) => `{${group.join("|")}}`).join(" ")}`);
  const types = ["text", "list_number", "list_text", "number", "path"];
  const operations = metaSeed().primitives;
  // The last clause's head is the outermost operation, so its result type is
  // the program's.
  const lastHead = clauses.length ? clauses[clauses.length - 1].head : groups[0];
  const headTypes = lastHead.map((id) => (operations.find((primitive) => primitive.id === id) || {}).to).filter(Boolean);
  let best = null;
  const allWords = words.concat(weakWords);
  const strongest = new Map();
  for (const hypotheses of allWords) {
    for (const hypothesis of hypotheses) strongest.set(hypothesis.operation, Math.max(strongest.get(hypothesis.operation) || 0, hypothesis.score));
  }
  const headOperations = new Set(lastHead);
  for (const fromType of types) {
    // Only programs using the head operation are enumerated at full length.
    for (const program of metaPrograms(fromType, META_BOUNDS.programLength - 1, headOperations)) {
      const parametric = program.steps.filter(metaStepIsParametric);
      if (parametric.length > (values.length ? 1 : 0)) continue;
      const bound = parametric.length ? metaBindValues(metaStepParameterTypes(parametric[0]), values) : null;
      if (bound === undefined) continue;
      // A measure's parts serve the words they ground ("80 characters").
      const used = new Set(program.steps.flatMap((step) => metaStepOperations(step).concat(step.filter || step.select ? step.primitive.parts || [] : [])));
      if (!lastHead.some((operation) => used.has(operation))) continue;
      const uncovered = groups.filter((group) => !group.some((operation) => used.has(operation))).length;
      // A filter's measure is internal to the filter: it is not charged as
      // an ungrounded operation; its grounded parts only break ties.
      const ungrounded = program.steps.flatMap(metaStepMainOperations).filter((operation) => !evidence.get(operation)).length;
      const main = new Set(program.steps.flatMap(metaStepMainOperations));
      const parts = new Set(program.steps.filter((step) => step.filter || step.select).flatMap((step) => step.primitive.parts || [step.primitive.id]));
      let measureEvidence = 0;
      // A word the main program already explains (it is that operation's
      // strongest word) does not name the measure; a word whose meaning the
      // main program owes to another word may ("prints ... lines").
      for (const hypotheses of measureWords || allWords) {
        if (!measureWords && hypotheses.some((hypothesis) => main.has(hypothesis.operation) && hypothesis.score >= strongest.get(hypothesis.operation) * 0.99)) continue;
        measureEvidence += Math.max(0, ...hypotheses.filter((hypothesis) => parts.has(hypothesis.operation)).map((hypothesis) => hypothesis.score));
      }
      const measureSize = parts.size;
      const order = metaClauseOrder(program, fromType, clauses);
      const statedInput = inputTypes.length && !inputTypes.includes(fromType) ? 1 : 0;
      // A rewrite writes the head's result back to the file: it fits there.
      const rewritesHead = program.steps.some((step) => step.rewrite && lastHead.includes(step.primitive.id));
      const headFits = rewritesHead || headTypes.some((type) => type === program.type || (type === "list_any" && program.type.startsWith("list_"))) ? 0 : 1;
      // Grounded evidence decides first (every operation asked for, none
      // invented), then the request's own composition order, then shape.
      // "every file" asks for iteration: a program never holding a list
      // does not quantify over anything.
      const unquantified = universal && !order.types.some((type) => type.startsWith("list_")) ? 1 : 0;
      const rank = [ungrounded, uncovered, -metaCoverageScore(program.steps, words), -measureEvidence,
        order.violations, order.objectMismatch, unquantified, statedInput,
        headFits, -metaCoverageScore(program.steps, weakViews), program.type === fromType ? 0 : 1, program.steps.length, -metaCoverageScore(program.steps, weakWords), measureSize];
      let better = !best;
      for (let position = 0; !better && position < rank.length; position += 1) {
        if (rank[position] !== best.rank[position]) {
          better = rank[position] < best.rank[position];
          break;
        }
      }
      if (better) {
        best = { steps: program.steps, fromType, rank, parameter: bound, uncovered };
      }
    }
  }
  if (!best) trace.emit("impasse", metaNote("no_head_program", {}));
  else if (best.uncovered) trace.emit("evidence", metaNote("uncovered", { count: best.uncovered }));
  return best;
}
/**
 * The request's values a parametric step reads, by type in the order the
 * request names them, or undefined when one is missing. One value is bound
 * as itself, several as a list.
 * @param {Array<string>} types
 * @param {Array<{value: *, type: string}>} values
 * @returns {*}
 */
function metaBindValues(types, values) {
  const used = new Set();
  const out = [];
  for (const type of types) {
    const at = values.findIndex((item, index) => !used.has(index) && item.type === type);
    if (at < 0) return undefined;
    used.add(at);
    out.push(values[at].value);
  }
  return out.length === 1 ? out[0] : out;
}
/**
 * How far a program departs from the request's composition: coordinated
 * clauses apply in order; within a clause the head acts last, after its
 * object's modifiers; the head consumes the type its object noun names.
 * Representation changes (split / join) are free.
 * @param {{steps: Array<object>}} program
 * @param {string} fromType
 * @param {Array<object>} clauses
 * @returns {{violations: number, objectMismatch: number, types: Array<string>}}
 */
function metaClauseOrder(program, fromType, clauses) {
  const inputs = [];
  let type = fromType;
  for (const step of program.steps) {
    inputs.push(step.rewrite ? "text" : step.mapped ? type.slice(5) : type);
    type = step.select ? type.slice(5) : step.filter || step.rewrite ? type : step.mapped ? `list_${step.primitive.to}` : metaApply(step.primitive, type);
  }
  const positionOf = (ids) => program.steps.findIndex((step) => metaStepOperations(step).some((id) => ids.includes(id) && !metaIsView(id)));
  let violations = 0;
  let objectMismatch = 0;
  let previousHead = -1;
  for (const clause of clauses) {
    const head = positionOf(clause.head);
    if (head < 0) continue;
    if (head < previousHead) violations += 1;
    previousHead = head;
    for (const [position, step] of program.steps.entries()) {
      const ids = metaStepOperations(step).filter((id) => !metaIsView(id));
      if (position > head && ids.some((id) => clause.others.includes(id))) violations += 1;
    }
    if (clause.objectType && inputs[head] !== clause.objectType && !(clause.objectType.startsWith("list_") && inputs[head] === "list_any")) objectMismatch += 1;
  }
  return { violations, objectMismatch, types: inputs.concat(type) };
}
/**
 * The in-place edit of one file by a text transformation: the seeded
 * combinator `rewrite_file` applied to an operation's code.
 * @param {string} code a text -> text function source
 * @returns {string}
 */
function metaRewriteCode(code) {
  const combinator = metaSeed().combinators.find((item) => item.id === "rewrite_file");
  return String(combinator ? combinator.code : "").split("{f}").join(code);
}
