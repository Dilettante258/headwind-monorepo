// ── Transform Types (mirrors WASM interface) ─────────────────

export interface TransformOptions {
  namingMode: 'hash' | 'readable' | 'camelCase';
  outputMode: GlobalOutputMode | CssModulesOutputMode;
  cssVariables: 'var' | 'inline';
  unknownClasses: 'remove' | 'preserve';
  colorMode: 'hex' | 'oklch' | 'hsl' | 'var';
  elementTree?: boolean;
}

export interface GlobalOutputMode {
  type: 'global';
  importPath?: string;
}

export interface CssModulesOutputMode {
  type: 'cssModules';
  access?: 'dot' | 'bracket';
}

export interface TransformResult {
  code: string;
  css: string;
  classMap: Record<string, string>;
  elementTree?: string;
}

export type SupportedLanguage = 'jsx' | 'html';

const SUPPORTED_EXTENSIONS = new Set([".jsx", ".tsx", ".js", ".ts", ".html", ".htm"]);

/** Check if a filename has a supported extension for transformation */
export function isSupportedFile(filename: string): boolean {
  const ext = filename.lastIndexOf(".");
  if (ext === -1) return false;
  return SUPPORTED_EXTENSIONS.has(filename.slice(ext).toLowerCase());
}

export const DEFAULT_OPTIONS: TransformOptions = {
  namingMode: 'hash',
  outputMode: { type: 'global' },
  cssVariables: 'var',
  unknownClasses: 'preserve',
  colorMode: 'hex',
};

// ── Webview <-> Extension Host Message Protocol ──────────────

/** Messages sent FROM the webview TO the extension host */
export type WebviewToHostMessage =
  | { type: 'ready' }
  | { type: 'optionsChanged'; options: TransformOptions }
  | { type: 'requestTransform' }
  | { type: 'requestPreviewDiff' }
  | { type: 'requestApply' }
  | { type: 'copyToClipboard'; text: string };

/** Messages sent FROM the extension host TO the webview */
export type HostToWebviewMessage =
  | { type: 'init'; state: PanelState }
  | { type: 'transformResult'; result: TransformResult; duration: number }
  | { type: 'transformError'; error: string }
  | { type: 'activeFileChanged'; filename: string | null }
  | { type: 'optionsUpdated'; options: TransformOptions };

/** Full state sent on webview init or restore */
export interface PanelState {
  options: TransformOptions;
  result: TransformResult | null;
  activeFilename: string | null;
  duration: number;
}
