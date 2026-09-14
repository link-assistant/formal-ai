import { build } from "esbuild";

// Playwright is server-side runtime code with package-relative metadata and
// browser assets, so its maintainers do not support bundling it (playwright
// #33031). Keep both package entry points external; .vscodeignore explicitly
// includes these two production packages in the otherwise self-contained VSIX.
export const VSIX_RUNTIME_EXTERNALS = ["playwright", "playwright-core"];

export async function bundleWebTools({ entryPoint, outfile, nodeModulesDir }) {
  return build({
    entryPoints: [entryPoint],
    outfile,
    bundle: true,
    platform: "node",
    format: "cjs",
    target: "node22",
    nodePaths: [nodeModulesDir],
    external: VSIX_RUNTIME_EXTERNALS,
    logLevel: "warning",
  });
}
