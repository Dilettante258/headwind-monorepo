#!/usr/bin/env node

import { execSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, "../..");

const wasmSource = join(
  rootDir,
  "target/wasm32-wasip1/release/swc_plugin.wasm"
);

const destinations = [
  join(rootDir, "packages/swc-host/assets/swc_plugin.wasm"),
  join(rootDir, "apps/vscode-extension/resources/plugins/swc_plugin.wasm"),
];

console.log("Building SWC plugin...");

try {
  execSync(
    "cargo build-wasip1 --release",
    {
      cwd: rootDir,
      stdio: "inherit",
    }
  );

  console.log("Copying wasm file to destinations...");

  for (const dest of destinations) {
    mkdirSync(dirname(dest), { recursive: true });
    copyFileSync(wasmSource, dest);
    console.log(`  -> ${dest}`);
  }

  console.log("Done!");
} catch (error) {
  console.error("Build failed:", error.message);
  process.exit(1);
}
