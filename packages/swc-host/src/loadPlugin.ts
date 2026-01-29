import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));

export function loadPluginPath(): string {
  // In development, the wasm file is in assets/
  // After build, it's still relative to the dist folder
  return join(__dirname, "..", "assets", "swc_plugin.wasm");
}
