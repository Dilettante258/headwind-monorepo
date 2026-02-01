import * as path from "path";
import type { TransformOptions, TransformResult } from "./types";

interface WasmModule {
  transformJsx(source: string, filename: string, options: unknown): TransformResult;
  transformHtml(source: string, options: unknown): TransformResult;
}

let wasmModule: WasmModule | null = null;

export function loadWasm(): void {
  if (wasmModule) return;
  // __dirname at runtime is dist/ where the WASM files are copied
  const wasmPath = path.join(__dirname, "headwind_wasm.js");
  wasmModule = require(wasmPath) as WasmModule;
}

export function isWasmReady(): boolean {
  return wasmModule !== null;
}

export function transformJsx(
  source: string,
  filename: string,
  options: TransformOptions,
): TransformResult {
  if (!wasmModule) throw new Error("WASM not loaded. Call loadWasm() first.");
  return wasmModule.transformJsx(source, filename, options);
}

export function transformHtml(
  source: string,
  options: TransformOptions,
): TransformResult {
  if (!wasmModule) throw new Error("WASM not loaded. Call loadWasm() first.");
  return wasmModule.transformHtml(source, options);
}

/**
 * Pick the right transform function based on file extension.
 * For JSX global mode, auto-injects the CSS import path when
 * `cssOutputPattern` is provided (e.g. `[name].headwind.css`).
 */
export function runTransform(
  source: string,
  filename: string,
  options: TransformOptions,
  cssOutputPattern?: string,
): TransformResult {
  const ext = path.extname(filename).toLowerCase();
  if (ext === ".html" || ext === ".htm") {
    return transformHtml(source, options);
  }

  // For global mode, derive the CSS import path from the filename
  let opts = options;
  if (options.outputMode.type === "global" && cssOutputPattern) {
    const stem = path.basename(filename, path.extname(filename));
    const cssFileName = cssOutputPattern.replace("[name]", stem);
    opts = {
      ...options,
      outputMode: { ...options.outputMode, importPath: `./${cssFileName}` },
    };
  }

  return transformJsx(source, filename, opts);
}
