#!/usr/bin/env node

import { execSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, "../..");
const wasmCrate = join(rootDir, "crates/wasm");
const outDir = join(rootDir, "apps/vscode-extension/wasm");

const isRelease = !process.argv.includes("--dev");
const profile = isRelease ? "--release" : "--dev";

console.log(`Building headwind-wasm for Node.js (${isRelease ? "release" : "dev"})...`);

try {
  execSync(
    `wasm-pack build ${wasmCrate} --target nodejs ${profile} --out-dir ${outDir} --out-name headwind_wasm`,
    {
      cwd: rootDir,
      stdio: "inherit",
    }
  );

  console.log(`Done! Output: ${outDir}`);
} catch (error) {
  console.error("WASM build failed:", error.message);
  process.exit(1);
}
