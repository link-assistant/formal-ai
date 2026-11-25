// Prints why a native Rust specification case does not parse into a JS-replayable program.
import { specificationCases } from "../scripts/lib/rust-specification-cases.mjs";
const name = process.argv[2] ?? "procedural_elaboration_followup_rebinds_to_prior_how_to";
const found = specificationCases(process.cwd()).cases.find((item) => item.id.endsWith("::" + name));
console.log(JSON.stringify(found && { ...found, program: found.program ? "parsed" : found.program }, null, 1).slice(0, 3000));
