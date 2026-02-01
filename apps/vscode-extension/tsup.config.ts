import { defineConfig } from "tsup";
import { copyFileSync, existsSync, mkdirSync } from "fs";
import { join } from "path";

export default defineConfig({
  entry: ["src/extension.ts"],
  format: ["cjs"],
  external: ["vscode"],
  dts: false,
  clean: true,
  sourcemap: true,
  onSuccess: async () => {
    const distDir = "dist";
    const wasmDir = "wasm";

    if (!existsSync(distDir)) {
      mkdirSync(distDir, { recursive: true });
    }

    // Copy WASM artifacts to dist/ so __dirname resolves correctly at runtime
    const files = ["headwind_wasm.js", "headwind_wasm_bg.wasm"];
    for (const file of files) {
      const src = join(wasmDir, file);
      const dest = join(distDir, file);
      if (existsSync(src)) {
        copyFileSync(src, dest);
        console.log(`Copied ${src} -> ${dest}`);
      } else {
        console.warn(`Warning: ${src} not found. Run 'pnpm build:wasm' first.`);
      }
    }
  },
});
