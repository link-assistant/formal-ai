// Browser/WASM mirror of `lowered_candidate_is_closed`
// (src/coding/composition_search/binders.rs): reject a derived candidate when
// a catalog placeholder atom would survive lowering as a free target-language
// name. The typed frontier intentionally introduces those atoms because a
// fragment can bind them inside a comprehension or lambda; keeping a
// candidate where the binding never materialized would turn a type-correct
// expression into a guaranteed NameError at verification time.

function browserSourceTokens(source) {
  return String(source || "").match(/[A-Za-z_][A-Za-z0-9_]*/g) || [];
}

function browserCollectBindingsUntil(tokens, start, delimiter, bound) {
  for (let index = start; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (token === delimiter) break;
    bound.add(token);
  }
}

function browserLoweredCandidateIsClosed(source, placeholderNames, parameterNames) {
  const tokens = browserSourceTokens(source);
  const bound = new Set(parameterNames);
  for (let index = 0; index < tokens.length; index += 1) {
    if (tokens[index] === "for") browserCollectBindingsUntil(tokens, index + 1, "in", bound);
    else if (tokens[index] === "lambda") browserCollectBindingsUntil(tokens, index + 1, ":", bound);
  }
  return [...placeholderNames].every((name) =>
    bound.has(name) || !tokens.includes(name));
}

function browserInputDomainCoherence(arguments_) {
  // Ground an application in its input domain: a membership-style predicate is
  // coherent when the member is a plain identifier and the collection argument
  // carries a concrete collection type, not an unresolved unknown. Mirrors the
  // native binder applies being grounded in their arguments and input domain.
  return arguments_.length >= 2 &&
    !arguments_[1].type.startsWith("unknown:") &&
    /^[A-Za-z_]\w*$/.test(arguments_[0].source) ? 1 : 0;
}
