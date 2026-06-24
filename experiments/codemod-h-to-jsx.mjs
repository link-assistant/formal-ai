#!/usr/bin/env node
// Issue #550 codemod: convert classic `h(tag, props, ...children)` render calls
// in src/web/app/main.jsx into modern JSX. The repo's tsconfig pins bun's JSX
// transform to the classic runtime with `h` as the factory, so JSX compiles back
// to the *exact* h() calls this file already uses. That makes the conversion
// provably behaviour-preserving: compiling main.jsx with
//   bun build … --packages external   (unminified)
// before and after this codemod yields byte-identical output (modulo the leading
// `// <path>` banner bun prepends). See experiments/verify-jsx-equivalence.sh.
//
// Strategy: surgical string-splice. We parse the file, find every *top-level*
// h() call (one with no h() ancestor), build an equivalent JSX AST for it, print
// just that subtree with @babel/generator, and splice it back over the call's
// source span. Everything that is not a render call (hooks, handlers, ~10k lines
// of logic) is left byte-for-byte untouched, keeping the PR diff focused.
//
// Usage: node experiments/codemod-h-to-jsx.mjs [path-to-main.jsx]
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const { parse } = require("@babel/parser");
const t = require("@babel/types");
const _generate = require("@babel/generator");
const generate = _generate.default || _generate;
const _traverse = require("@babel/traverse");
const traverse = _traverse.default || _traverse;

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..");
const target = process.argv[2] || path.join(repoRoot, "src/web/app/main.jsx");
const code = fs.readFileSync(target, "utf8");

const ast = parse(code, {
  sourceType: "module",
  plugins: ["jsx"],
  ranges: true,
});

const isHCall = (node) =>
  node &&
  node.type === "CallExpression" &&
  node.callee &&
  node.callee.type === "Identifier" &&
  node.callee.name === "h";

class Bail extends Error {}

// Build a JSX element name node from the h() tag argument.
function tagToName(arg) {
  if (arg.type === "StringLiteral") {
    // Lowercase/standard tag: e.g. "div", "section". Must be a valid JSX name.
    if (!/^[A-Za-z][A-Za-z0-9]*$/.test(arg.value)) throw new Bail();
    return t.jsxIdentifier(arg.value);
  }
  if (arg.type === "Identifier") {
    // A lowercase JSX element name is treated as a literal string tag by the
    // compiler (`<tag>` -> h("tag", …)), NOT as a reference to the variable
    // `tag`. So an h() call whose tag is a lowercase *variable* (e.g. the
    // dynamic `tag` from shape.map(([tag, attrs]) => h(tag, …))) cannot be
    // expressed as JSX without changing behaviour — bail and keep it as h().
    // Capitalised identifiers are components and resolve to the variable.
    if (!/^[A-Z]/.test(arg.name)) throw new Bail();
    return t.jsxIdentifier(arg.name);
  }
  if (arg.type === "MemberExpression" && !arg.computed) {
    // e.g. chakra.div -> JSXMemberExpression
    const objName =
      arg.object.type === "Identifier" ? arg.object.name : null;
    const propName =
      arg.property.type === "Identifier" ? arg.property.name : null;
    if (!objName || !propName) throw new Bail();
    return t.jsxMemberExpression(
      t.jsxIdentifier(objName),
      t.jsxIdentifier(propName),
    );
  }
  throw new Bail();
}

// Build JSX attributes from the h() props argument.
function propsToAttributes(arg) {
  if (!arg) return [];
  if (arg.type === "NullLiteral") return [];
  if (arg.type === "Identifier" && arg.name === "undefined") return [];
  if (arg.type === "ObjectExpression") {
    return arg.properties.map((prop) => {
      if (prop.type === "SpreadElement") {
        return t.jsxSpreadAttribute(prop.argument);
      }
      if (prop.type !== "ObjectProperty" || prop.computed) throw new Bail();
      let name;
      if (prop.key.type === "Identifier") name = prop.key.name;
      else if (prop.key.type === "StringLiteral") name = prop.key.value;
      else throw new Bail();
      // JSX attribute names allow hyphens (data-*, aria-*) and ':' namespaces.
      if (!/^[A-Za-z_][A-Za-z0-9_-]*(:[A-Za-z_][A-Za-z0-9_-]*)?$/.test(name)) {
        throw new Bail();
      }
      const jsxName = t.jsxIdentifier(name);
      const value = prop.value;
      // Shorthand { foo } -> foo={foo}
      if (prop.shorthand) {
        return t.jsxAttribute(jsxName, t.jsxExpressionContainer(value));
      }
      // Plain, quote-free, single-line string -> name="value" (clean JSX).
      if (
        value.type === "StringLiteral" &&
        !/["\n\r{}<>]/.test(value.value)
      ) {
        return t.jsxAttribute(jsxName, t.stringLiteral(value.value));
      }
      // Any other value becomes name={value}, with nested h() inside the value
      // (e.g. children={h("div", …)}, render props, ternaries) rewritten to JSX.
      return t.jsxAttribute(jsxName, t.jsxExpressionContainer(toJSXExpr(value)));
    });
  }
  // props is some other expression (Identifier/Call): spread it. h(C, props)
  // -> <C {...props} />
  return [t.jsxSpreadAttribute(arg)];
}

// Convert an h() call to a JSXElement, but never throw: if the call itself is
// irreducible (e.g. dynamic lowercase tag), leave it as a raw h() call while
// still rewriting any convertible calls nested in its arguments.
function safeHToJSX(node) {
  try {
    return hCallToJSX(node);
  } catch (e) {
    if (!(e instanceof Bail)) throw e;
    node.arguments.forEach((a) =>
      a && typeof a === "object" && a.type ? rewriteNestedH(a) : null,
    );
    return node; // unchanged CallExpression
  }
}

// Rewrite, in place, every nested h() call inside an arbitrary expression
// (descending into ternaries, .map() callbacks, arrays, …) and return it.
function rewriteNestedH(node) {
  if (isHCall(node)) return safeHToJSX(node);
  const replace = (parent, key, idx) => {
    const child = idx === undefined ? parent[key] : parent[key][idx];
    if (!child || typeof child !== "object" || !child.type) return;
    if (isHCall(child)) {
      const res = safeHToJSX(child);
      if (idx === undefined) parent[key] = res;
      else parent[key][idx] = res;
      return; // safeHToJSX handled this subtree
    }
    for (const k in child) {
      if (k === "loc" || k === "range" || k === "start" || k === "end") continue;
      const v = child[k];
      if (Array.isArray(v)) v.forEach((_, i) => replace(child, k, i));
      else if (v && typeof v === "object" && v.type) replace(child, k);
    }
  };
  for (const k in node) {
    if (k === "loc" || k === "range" || k === "start" || k === "end") continue;
    const v = node[k];
    if (Array.isArray(v)) v.forEach((_, i) => replace(node, k, i));
    else if (v && typeof v === "object" && v.type) replace(node, k);
  }
  return node;
}

// Rewrite nested h() in an expression used as an attribute value, returning the
// (possibly JSXElement) expression to wrap in {…}.
function toJSXExpr(node) {
  return isHCall(node) ? safeHToJSX(node) : rewriteNestedH(node);
}

// Convert a child argument node into a JSX child.
function childToJSX(node) {
  const r = toJSXExpr(node);
  // A converted element is a bare child; anything else goes in an {expr}.
  return r.type === "JSXElement" || r.type === "JSXFragment"
    ? r
    : t.jsxExpressionContainer(r);
}

// Core: h(tag, props, ...children) -> <Tag …>{…}</Tag>
function hCallToJSX(node) {
  const [tagArg, propsArg, ...childArgs] = node.arguments;
  if (!tagArg) throw new Bail();
  if (tagArg.type === "SpreadElement") throw new Bail();
  const name = tagToName(tagArg);
  const attributes = propsToAttributes(propsArg);
  const children = childArgs.map((c) => {
    if (c.type === "SpreadElement") throw new Bail();
    return childToJSX(c);
  });
  const selfClosing = children.length === 0;
  const opening = t.jsxOpeningElement(name, attributes, selfClosing);
  const closing = selfClosing ? null : t.jsxClosingElement(name);
  return t.jsxElement(opening, closing, children, selfClosing);
}

// Collect top-level h() calls (no h() ancestor).
const topLevel = [];
traverse(ast, {
  CallExpression(p) {
    if (!isHCall(p.node)) return;
    const hasHAncestor = p.findParent((pp) => isHCall(pp.node));
    if (hasHAncestor) return;
    topLevel.push(p);
  },
});

let converted = 0;
let bailed = 0;
const edits = [];
for (const p of topLevel) {
  const node = p.node;
  try {
    const jsx = hCallToJSX(node);
    const printed = generate(jsx, {
      jsescOption: { minimal: true },
      retainLines: false,
    }).code;
    edits.push({ start: node.start, end: node.end, text: printed });
    converted += 1;
  } catch (e) {
    if (e instanceof Bail) {
      bailed += 1;
      continue;
    }
    throw e;
  }
}

// Apply edits in reverse source order so offsets stay valid.
edits.sort((a, b) => b.start - a.start);
let out = code;
for (const e of edits) {
  out = out.slice(0, e.start) + e.text + out.slice(e.end);
}

fs.writeFileSync(target, out, "utf8");
console.error(
  `codemod-h-to-jsx: converted ${converted} top-level h() call(s), bailed ${bailed}. Wrote ${target}`,
);
