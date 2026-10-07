// Issues #1176 and #700 browser parity: exact unit conversion.
//
// The worker twin (js/worker/formal_ai_worker_units.js) of
// rust/src/solver_handlers/unit_conversion.rs and rust/src/si_units.rs. The
// prompts come from rust/tests/unit/issue_1176_quantities_dates.rs; the SI
// algebra cases from rust/tests/unit/issue_700_si_units.rs.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

/** `tag:num/den` of a worker-side SI conversion. */
function convert(value, from, to) {
  return evaluate(
    worker,
    `(() => { const o = siConvertThroughSi(${value}n, 1n, ${JSON.stringify(from)}, ${JSON.stringify(to)});
      return o.tag === "converted" ? \`converted:\${o.numerator}/\${o.denominator}\`
        : o.tag === "incompatible" ? \`incompatible:\${o.from}:\${o.to}\`
        : \`\${o.tag}:\${o.unit}\`; })()`,
  );
}

function assertConverts(value, from, to, expected) {
  const outcome = convert(value, from, to);
  assert.ok(outcome.startsWith("converted:"), `${from} to ${to} did not convert: ${outcome}`);
  const [num, den] = outcome.slice("converted:".length).split("/");
  const got = Number(num) / Number(den);
  assert.ok(Math.abs(got - expected) < 1e-9, `${value} ${from} in ${to}: got ${got}, expected ${expected}`);
}

test("unit conversion multiplies by the seed factor", async () => {
  const answer = await solve("How many kilometers are 26.2 miles?");
  assert.equal(answer.intent, "unit_conversion");
  assert.ok(answer.evidence.includes("response:unit_conversion"));
  assert.equal(
    answer.content,
    "26.2 miles is 42.1648128 kilometers. 26.2 × 1.609344 = 42.1648128, because 1 miles = 1.609344 kilometers. Wikidata grounds the factor: miles is Q253276 and kilometers is Q828224, whose conversion to SI unit (P2370, captured 2026-10-07) is 1609.344 and 1000 of Q11573, so 1609.344 ÷ 1000 = 1.609344.",
  );
});

test("the reverse direction divides by the same factor", async () => {
  const answer = await solve("10 kilometers in miles");
  assert.equal(answer.intent, "unit_conversion");
  // The divide template renders the derivation itself after the marked
  // result, so the quotient carries exactly one ≈ in the prose.
  assert.ok(
    answer.evidence.includes("unit_conversion:derivation:10 ÷ 1.609344 ≈ 6.2137119"),
    JSON.stringify(answer.evidence),
  );
  assert.equal(
    answer.content,
    "10 kilometers is ≈6.2137119 miles. 10 ÷ 1.609344 ≈ 6.2137119, because 1 miles = 1.609344 kilometers. Wikidata grounds the factor: miles is Q253276 and kilometers is Q828224, whose conversion to SI unit (P2370, captured 2026-10-07) is 1609.344 and 1000 of Q11573, so 1609.344 ÷ 1000 = 1.609344.",
  );
});

test("temperature conversion shows every formula step", async () => {
  const answer = await solve("convert 25 celsius to fahrenheit");
  assert.equal(answer.intent, "unit_conversion");
  assert.ok(answer.content.includes("25 × 9/5 + 32 = 77"), answer.content);
});

test("unknown and ambiguous units are declined", async () => {
  for (const prompt of ["How many meters are 5 furlongs?", "5 feet 9 inches in cm"]) {
    const answer = await solve(prompt);
    assert.notEqual(answer.intent, "unit_conversion", `${prompt} -> ${answer.content}`);
  }
});

test("a pair without a seed record converts exactly through SI", async () => {
  const answer = await solve("How many watts is 3 horsepower?");
  assert.equal(answer.intent, "unit_conversion");
  assert.equal(answer.content, "3 horsepower = 69909363/31250 watts");
  assert.ok(answer.evidence.includes("response:unit_conversion_si"));
});

test("the SI seed loads with base dimensions and positive factors", () => {
  const count = evaluate(worker, "siUnits().length");
  assert.ok(count >= 60, `the dimension table carries at least 60 units, got ${count}`);
  assert.equal(evaluate(worker, "siUnits().every((u) => u.numerator > 0n && u.denominator > 0n)"), true);
  assert.equal(
    evaluate(worker, "siUnits().every((u) => siDimensionString(siParseDimension(siDimensionString(u.dimension))) === siDimensionString(u.dimension))"),
    true,
  );
  assert.equal(evaluate(worker, 'siDimensionString(siParseDimension("L2MT-3"))'), "L2MT-3");
  assert.equal(evaluate(worker, 'siParseDimension("LX")'), null);
  assert.equal(evaluate(worker, 'String(siParseFactor("0.0254").numerator) + "/" + String(siParseFactor("254/10000").denominator)'), "127/5000");
  assert.equal(evaluate(worker, 'siParseFactor("-1")'), null);
});

test("forty conversions through SI", () => {
  assertConverts(5, "kilometer", "mile", (5.0 * 1000.0) / 1609.344);
  assertConverts(6, "foot", "meter", 6.0 * 0.3048);
  assertConverts(1, "inch", "millimeter", 25.4);
  assertConverts(1, "nautical_mile", "kilometer", 1.852);
  assertConverts(1, "astronomical_unit", "kilometer", 149597870.7);
  assertConverts(1, "light_year", "kilometer", 9460730472580.8);
  assertConverts(3, "yard", "meter", 3.0 * 0.9144);
  assertConverts(150, "pound", "kilogram", 150.0 * 0.45359237);
  assertConverts(1, "ounce", "gram", 28.349523125);
  assertConverts(1, "stone", "kilogram", 6.35029318);
  assertConverts(1, "us_ton", "tonne", 0.90718474);
  assertConverts(1, "long_ton", "tonne", 1.0160469088);
  assertConverts(5, "hour", "minute", 300.0);
  assertConverts(1, "week", "day", 7.0);
  assertConverts(1, "julian_year", "day", 365.25);
  assertConverts(90, "kilometer_per_hour", "meter_per_second", 25.0);
  assertConverts(60, "mile_per_hour", "meter_per_second", 60.0 * 0.44704);
  assertConverts(100, "mile_per_hour", "kilometer_per_hour", (100.0 * 0.44704) / (1000.0 / 3600.0));
  assertConverts(10, "knot", "kilometer_per_hour", 18.52);
  assertConverts(3, "horsepower", "watt", 3.0 * 745.699872);
  assertConverts(1, "horsepower", "kilowatt", 0.745699872);
  assertConverts(1, "horsepower_metric", "watt", 735.49875);
  assertConverts(2000, "calorie", "kilojoule", 8.368);
  assertConverts(1, "kilocalorie", "kilojoule", 4.184);
  assertConverts(1, "watt_hour", "joule", 3600.0);
  assertConverts(1, "kilowatt_hour", "kilojoule", 3600.0);
  assertConverts(1, "btu", "joule", 1055.05585262);
  assertConverts(70, "kilogram_force", "newton", 70.0 * 9.80665);
  assertConverts(1000, "dyne", "newton", 0.01);
  assertConverts(1, "pound_force", "newton", 4.4482216152605);
  assertConverts(1, "atmosphere", "pascal", 101325.0);
  assertConverts(1, "atmosphere", "bar", 1.01325);
  assertConverts(32, "psi", "kilopascal", (32.0 * 6894.757293168) / 1000.0);
  assertConverts(1, "psi", "torr", 6894.757293168 / (101325.0 / 760.0));
  assertConverts(1, "acre", "hectare", 0.40468564224);
  assertConverts(1, "square_mile", "square_kilometer", 2.589988110336);
  assertConverts(1, "hectare", "square_meter", 10000.0);
  assertConverts(1, "square_foot", "square_meter", 0.09290304);
  assertConverts(2, "liter", "us_gallon", 2.0 / 3.785411784);
  assertConverts(1, "imperial_gallon", "liter", 4.54609);
  assertConverts(2, "us_cup", "milliliter", 2.0 * 236.5882365);
  assertConverts(1, "gibibyte", "mebibyte", 1024.0);
  assertConverts(1, "kibibyte", "byte", 1024.0);
  assertConverts(180, "degree", "radian", Math.PI);
});

test("exact conversions stay rational; mismatches and unknown units fail by name", () => {
  assert.equal(convert(3, "horsepower", "watt"), "converted:69909363/31250");
  assert.equal(convert(90, "kilometer_per_hour", "meter_per_second"), "converted:25/1");
  assert.equal(convert(5, "meter", "second"), "incompatible:L:T");
  assert.ok(convert(1, "joule", "watt").startsWith("incompatible:"));
  assert.equal(convert(1, "furlong", "meter"), "unknown_unit:furlong");
});

test("surfaces in four languages resolve to canonical units", () => {
  const expected = [
    ["miles", "mile"], ["inches", "inch"], ["gallons", "us_gallon"],
    ["horsepower", "horsepower_mechanical"],
    ["километров", "kilometer"], ["метров", "meter"], ["миль", "mile"], ["ватт", "watt"],
    ["часов", "hour"], ["фунтов", "pound"], ["калорий", "calorie"], ["гектара", "hectare"],
    ["лошадиных сил", "horsepower_mechanical"],
    ["मील", "mile"], ["किलोमीटर", "kilometer"], ["घंटे", "hour"], ["कैलोरी", "calorie"],
    ["अश्वशक्ति", "horsepower_mechanical"], ["पाउंड", "pound"],
    ["公里", "kilometer"], ["米", "meter"], ["小时", "hour"], ["马力", "horsepower_mechanical"],
    ["升", "liter"], ["卡路里", "calorie"], ["英里", "mile"],
  ];
  for (const [surface, unit] of expected) {
    assert.equal(evaluate(worker, `siUnitNamedBy(${JSON.stringify(surface)})`), unit, surface);
  }
});

test("multilingual prompts resolve both surfaces and convert through SI", () => {
  const cases = [
    ["Сколько метров в 5 километрах?", 5, "километрах", "метров", 5000.0],
    ["How many watts is 3 horsepower?", 3, "horsepower", "watts", 3.0 * 745.699872],
    ["5 मील कितने किलोमीटर हैं?", 5, "मील", "किलोमीटर", (5.0 * 1609.344) / 1000.0],
    ["3 马力是多少瓦?", 3, "马力", "瓦", 3.0 * 745.699872],
  ];
  for (const [prompt, value, fromSurface, toSurface, expected] of cases) {
    assert.ok(prompt.includes(fromSurface));
    const from = evaluate(worker, `siUnitNamedBy(${JSON.stringify(fromSurface)})`);
    const to = evaluate(worker, `siUnitNamedBy(${JSON.stringify(toSurface)})`);
    assert.ok(from && to, `${prompt}: surfaces resolve`);
    assertConverts(value, from, to, expected);
  }
});
