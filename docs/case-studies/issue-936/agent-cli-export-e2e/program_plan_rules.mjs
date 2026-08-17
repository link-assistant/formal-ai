// Generated JavaScript interop; substitution semantics remain in Rust/WASM.
// Compile program_plan_rules_wasm.rs to a WebAssembly module before loading this file.

export async function instantiateSubstitution(wasmBytes) {
  const { instance } = await WebAssembly.instantiate(wasmBytes, {});
  return (input) => {
    const bytes = new TextEncoder().encode(input);
    if (bytes.length > instance.exports.input_capacity()) {
      throw new Error("input exceeds WASM capacity");
    }
    new Uint8Array(instance.exports.memory.buffer, instance.exports.input_ptr(), bytes.length).set(bytes);
    const outputLength = instance.exports.run(bytes.length);
    const output = new Uint8Array(
      instance.exports.memory.buffer,
      instance.exports.output_ptr(),
      outputLength,
    );
    return new TextDecoder().decode(output);
  };
}

if (typeof process !== "undefined" && process.release?.name === "node") {
  const wasmPath = process.argv[2];
  if (!wasmPath) throw new Error("usage: node INTEROP.mjs PROGRAM.wasm");
  const { readFile } = await import("node:fs/promises");
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const runSubstitution = await instantiateSubstitution(await readFile(wasmPath));
  process.stdout.write(runSubstitution(Buffer.concat(chunks).toString("utf8")));
}
