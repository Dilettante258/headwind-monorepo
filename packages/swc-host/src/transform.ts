import { transform } from "@swc/core";
import { loadPluginPath } from "./loadPlugin.js";
import type { TransformOptions, TransformResult } from "./config.js";

export async function transformWithPlugin(
  code: string,
  options: TransformOptions = {}
): Promise<TransformResult> {
  const pluginPath = loadPluginPath();
  const pluginConfig = options.pluginConfig ?? {};

  const result = await transform(code, {
    filename: options.filename ?? "input.ts",
    sourceMaps: options.sourceMaps ?? false,
    jsc: {
      parser: {
        syntax: "typescript",
        tsx: options.filename?.endsWith(".tsx") ?? false,
      },
      target: "es2022",
      experimental: {
        plugins: [[pluginPath, pluginConfig]],
      },
    },
  });

  return {
    code: result.code,
    map: result.map,
  };
}
