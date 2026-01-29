import type { TransformInput, TransformOutput } from "./types.js";

export async function transformRemote(
  input: TransformInput,
  apiUrl: string = "http://localhost:3000"
): Promise<TransformOutput> {
  const response = await fetch(`${apiUrl}/transform`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      code: input.code,
      filename: input.filename,
      options: { sourceMaps: input.sourceMaps },
      pluginConfig: input.pluginConfig,
    }),
  });

  if (!response.ok) {
    throw new Error(`Transform API error: ${response.statusText}`);
  }

  return response.json() as Promise<TransformOutput>;
}
