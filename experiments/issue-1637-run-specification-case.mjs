// Runs one native Rust specification case on the JavaScript worker host.
import { specificationCases } from "../scripts/lib/rust-specification-cases.mjs";
import { executeTypedProgram } from "../scripts/lib/rust-specification-programs.mjs";
import { WorkerHost } from "../js/server/worker-host.mjs";
const name = process.argv[2];
const found = specificationCases(process.cwd()).cases.find((item) => item.id.endsWith("::" + name));
if (!found?.program) throw new Error("not a typed program: " + name);
const result = await executeTypedProgram(new WorkerHost(), found.program);
console.log(JSON.stringify({ status: result.status, assertions: result.assertions, failure: result.failure }));
process.exit(0);
