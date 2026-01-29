import { transformWithPlugin } from "@headwind/swc-host";
import type { TransformInput, TransformOutput } from "./types.js";

export async function transform(input: TransformInput): Promise<TransformOutput> {
  const result = await transformWithPlugin(input.code, {
    filename: input.filename,
    sourceMaps: input.sourceMaps,
    pluginConfig: input.pluginConfig,
  });

  return {
    code: result.code,
    map: result.map,
  };
}
