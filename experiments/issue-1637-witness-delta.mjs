// Lists governed specification cases whose parse outcome differs between HEAD and the working tree witness.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { deserialize } from "node:v8";
const path = "experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8";
const before = deserialize(execFileSync("git", ["show", "HEAD:" + path], { maxBuffer: 1 << 30 }));
const after = deserialize(readFileSync(path));
const outcome = (w) => new Map([...w.inventory.cases.map((c) => [c.id, "case"]), ...w.inventory.unsupported.map((u) => [u.id, "unsupported: " + u.reason])]);
const [b, a] = [outcome(before), outcome(after)];
for (const id of new Set([...b.keys(), ...a.keys()])) if (b.get(id) !== a.get(id)) console.log(id, "\n  before:", b.get(id), "\n  after: ", a.get(id));
console.log("total", before.inventory.total, "->", after.inventory.total, "cases", before.inventory.cases.length, "->", after.inventory.cases.length);
