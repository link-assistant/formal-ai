// This real CommonJS entry compiles after the journal preload installs its hook.
// The fixed ESM planner remains the sole planner entry; its rejection fails the process.
const plannerEntrypoint = './main.mjs';
void import(plannerEntrypoint);
