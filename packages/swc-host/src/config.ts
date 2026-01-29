export interface TransformOptions {
  filename?: string;
  sourceMaps?: boolean;
  pluginConfig?: Record<string, unknown>;
}

export interface TransformResult {
  code: string;
  map?: string;
}
