export type TransformMode = "local" | "remote";

export interface TransformInput {
  code: string;
  filename?: string;
  sourceMaps?: boolean;
  pluginConfig?: Record<string, unknown>;
}

export interface TransformOutput {
  code: string;
  map?: string;
  diagnostics?: Diagnostic[];
}

export interface Diagnostic {
  message: string;
  severity: "error" | "warning" | "info";
  line?: number;
  column?: number;
}
