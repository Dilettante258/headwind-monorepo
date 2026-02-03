import init, {
  transformJsx,
  transformHtml,
} from '../wasm-pkg/headwind_wasm.js';

let ready = false;

export async function loadWasm(): Promise<void> {
  if (ready) return;
  await init();
  ready = true;
}

export function isReady(): boolean {
  return ready;
}

export interface TransformOptions {
  namingMode?: 'hash' | 'readable' | 'camelCase';
  outputMode?: GlobalOutputMode | CssModulesOutputMode;
  cssVariables?: 'var' | 'inline';
  unknownClasses?: 'remove' | 'preserve';
  colorMode?: 'hex' | 'oklch' | 'hsl' | 'var';
  colorMix?: boolean;
}

interface GlobalOutputMode {
  type: 'global';
}

interface CssModulesOutputMode {
  type: 'cssModules';
  bindingName?: string;
  importPath?: string;
  access?: 'dot' | 'bracket';
}

export interface TransformResult {
  code: string;
  css: string;
  classMap: Record<string, string>;
}

export function runTransformJsx(
  source: string,
  filename: string,
  options?: TransformOptions,
): TransformResult {
  return transformJsx(source, filename, options ?? {}) as TransformResult;
}

export function runTransformHtml(
  source: string,
  options?: TransformOptions,
): TransformResult {
  return transformHtml(source, options ?? {}) as TransformResult;
}
