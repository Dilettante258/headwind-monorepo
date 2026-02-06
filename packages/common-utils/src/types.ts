export interface TransformResult {
  code: string;
  css: string;
  classMap: Record<string, string>;
  elementTree?: string;
}
