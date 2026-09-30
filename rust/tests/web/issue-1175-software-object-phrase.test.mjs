// Issue #1175 R5: the software-project object-phrase guard on the worker
// side. The artifact kind is the head noun of the authoring verb's object
// phrase — mirrored from object_phrase_artifact in
// rust/src/solver_handlers/software_project.rs — so an artifact word that
// merely modifies another noun ("an optional 4-digit extension" of a ZIP
// code) must not turn a regex request into an extension project.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

async function softwareContext() {
  const context = createWorkerContext();
  await evaluate(context, "loadSeed()");
  return context;
}

function formalize(context, prompt) {
  return plain(evaluate(context, `formalizeSoftwareProjectRequest(${JSON.stringify(prompt)})`));
}

test("a regex request mentioning a ZIP-code extension is not a software project", async () => {
  const context = await softwareContext();
  // The headline case of issue #1175: `extension` only modifies "ZIP code";
  // the head of what is written is "regular expression", so the worker must
  // not route it to the software-project plan. The any-position artifact
  // scan this replaces matched `extension` wherever it appeared.
  const meaning = formalize(
    context,
    "Write a regular expression that matches a US ZIP code with an optional 4-digit extension",
  );
  assert.equal(meaning, null, "an artifact word in a modifier must not claim the request");
});

test("a genuine artifact request still formalizes with its head noun", async () => {
  const context = await softwareContext();
  // Relative-clause and prepositional postmodifiers end the object phrase,
  // and the head noun is still an artifact surface.
  const extension = formalize(context, "write a browser extension that blocks ads");
  assert.ok(extension, "a real extension request must formalize");
  assert.equal(extension.artifact, "browser extension");
  assert.equal(extension.action, "write");

  // The benefactive "me a" between verb and head is skipped, and the "for"
  // postmodifier ends the phrase before its incidental words.
  const webApp = formalize(context, "build me a web app for tracking workouts");
  assert.ok(webApp, "a real web-app request must formalize");
  assert.equal(webApp.artifact, "web app");
  assert.equal(webApp.action, "build");
});

test("a verb-final request reads the object from before the verb", async () => {
  const context = await softwareContext();
  // Hindi writes the object before the verb (subject-object-verb): the
  // object phrase is the text immediately before the sentence-final verb.
  const meaning = formalize(context, "एक ब्राउज़र एक्सटेंशन बनाओ");
  assert.ok(meaning, "a verb-final authoring request must formalize");
  assert.equal(meaning.artifact, "browser extension");
});

test("a prompt with no authoring verb never claims", async () => {
  const context = await softwareContext();
  assert.equal(
    formalize(context, "a browser extension, please"),
    null,
    "a noun phrase without a verb is not a project request",
  );
  assert.equal(
    formalize(context, "solve this puzzle"),
    null,
    "a verb whose object is not an artifact does not claim",
  );
});

test("loadSeed hydrates the lexicon: the registry mirrors the seed precedence", async () => {
  // The object-phrase guard reads its artifact and verb surfaces from the
  // seed lexicon, and loadSeed asserts the worker registry is an exact
  // permutation of data/seed/handler-precedence.lino before hydrating it.
  // A precedence row without its registry twin throws inside loadSeed's
  // catch, every seed-driven worker surface silently goes empty, and this
  // suite's guards above stop proving anything — so pin the hydration.
  const context = await softwareContext();
  const lexiconSize = plain(evaluate(context, "meaningLexicon().length"));
  assert.ok(lexiconSize > 0, "the meaning lexicon must hydrate from the seed files");
  const artifacts = plain(evaluate(context, "softwareArtifactTable().length"));
  assert.ok(artifacts > 0, "the artifact surface table must fill from the seed lexicon");
});
