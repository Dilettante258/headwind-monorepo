import { createSignal, createEffect, onMount, type Component, Show } from 'solid-js';
import { Title, Meta, Link } from '@solidjs/meta';
import {
  loadWasm,
  runTransformJsx,
  runTransformHtml,
  type TransformResult,
  type TransformOptions,
} from './wasm';

const DEFAULT_JSX = `export default function App() {
  return (
    <div className="flex flex-col items-center p-8">
      <h1 className="text-3xl font-bold text-blue-600">
        Hello Headwind
      </h1>
      <p className="mt-4 text-gray-500 text-center">
        Atomic CSS is converted to semantic CSS
      </p>
      <button className="mt-6 px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-700">
        Click me
      </button>
    </div>
  );
}`;

const DEFAULT_HTML = `<div class="flex flex-col items-center p-8">
  <h1 class="text-3xl font-bold text-blue-600">
    Hello Headwind
  </h1>
  <p class="mt-4 text-gray-500 text-center">
    Atomic CSS is converted to semantic CSS
  </p>
  <button class="mt-6 px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-700">
    Click me
  </button>
</div>`;

type Language = 'jsx' | 'html';
type NamingMode = 'hash' | 'readable' | 'camelCase';
type OutputModeType = 'global' | 'cssModules';
type AccessMode = 'dot' | 'bracket';
type CssVariablesMode = 'var' | 'inline';
type UnknownClassesMode = 'remove' | 'preserve';
type ColorModeType = 'hex' | 'oklch' | 'hsl' | 'var';

const App: Component = () => {
  const [wasmLoaded, setWasmLoaded] = createSignal(false);
  const [wasmError, setWasmError] = createSignal('');
  const [language, setLanguage] = createSignal<Language>('jsx');
  const [source, setSource] = createSignal(DEFAULT_JSX);
  const [namingMode, setNamingMode] = createSignal<NamingMode>('hash');
  const [outputModeType, setOutputModeType] = createSignal<OutputModeType>('global');
  const [accessMode, setAccessMode] = createSignal<AccessMode>('dot');
  const [cssVariables, setCssVariables] = createSignal<CssVariablesMode>('inline');
  const [unknownClasses, setUnknownClasses] = createSignal<UnknownClassesMode>('preserve');
  const [colorMode, setColorMode] = createSignal<ColorModeType>('hex');
  const [result, setResult] = createSignal<TransformResult | null>(null);
  const [error, setError] = createSignal('');
  const [duration, setDuration] = createSignal(0);
  const [activeTab, setActiveTab] = createSignal<'code' | 'css' | 'map'>('code');
  const [copied, setCopied] = createSignal(false);

  onMount(async () => {
    try {
      await loadWasm();
      setWasmLoaded(true);
    } catch (e: any) {
      setWasmError(e.message || 'Failed to load WASM');
    }
  });

  // Switch default source when language changes
  createEffect(() => {
    const lang = language();
    setSource(lang === 'jsx' ? DEFAULT_JSX : DEFAULT_HTML);
  });

  // Run transform whenever inputs change
  createEffect(() => {
    if (!wasmLoaded()) return;

    const src = source();
    const naming = namingMode();
    const outType = outputModeType();
    const access = accessMode();
    const cssVars = cssVariables();
    const unknown = unknownClasses();
    const color = colorMode();

    const options: TransformOptions = {
      namingMode: naming,
      outputMode:
        outType === 'global'
          ? { type: 'global' }
          : { type: 'cssModules', access },
      cssVariables: cssVars,
      unknownClasses: unknown,
      colorMode: color,
    };

    try {
      const start = performance.now();
      let res: TransformResult;
      if (language() === 'jsx') {
        res = runTransformJsx(src, 'App.tsx', options);
      } else {
        res = runTransformHtml(src, options);
      }
      const elapsed = performance.now() - start;
      setDuration(elapsed);
      setResult(res);
      setError('');
    } catch (e: any) {
      setError(e.message || String(e));
      setResult(null);
    }
  });

  const classMapEntries = () => {
    const r = result();
    if (!r) return [];
    return Object.entries(r.classMap);
  };

  const currentOutputText = () => {
    const tab = activeTab();
    const r = result();
    if (!r) return '';
    if (tab === 'code') return r.code;
    if (tab === 'css') return r.css;
    return classMapEntries().map(([orig, gen]) => `${orig} → ${gen}`).join('\n');
  };

  const copyOutput = async () => {
    const text = currentOutputText();
    if (!text) return;
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div class="app">
      {/* SEO Meta Tags */}
      <Title>Headwind Playground — Atomic CSS to Semantic CSS Converter</Title>
      <Meta name="description" content="Try Headwind online: convert Tailwind atomic utility classes to optimized semantic CSS in real time. Supports JSX, TSX, and HTML with configurable naming, CSS Modules, and color modes." />
      <Meta name="keywords" content="Tailwind CSS, atomic CSS, semantic CSS, CSS optimizer, CSS converter, Headwind, utility-first CSS, CSS Modules, WASM, playground" />
      <Meta name="author" content="Headwind" />
      <Link rel="canonical" href="https://headwind-playground.kairi.cc/" />
      {/* Open Graph */}
      <Meta property="og:type" content="website" />
      <Meta property="og:title" content="Headwind Playground — Atomic to Semantic CSS" />
      <Meta property="og:description" content="Convert Tailwind atomic utility classes to optimized semantic CSS in real time. Supports JSX, TSX, and HTML." />
      <Meta property="og:url" content="https://headwind-playground.kairi.cc/" />
      <Meta property="og:site_name" content="Headwind Playground" />
      <Meta property="og:locale" content="en_US" />
      {/* Twitter Card */}
      <Meta name="twitter:card" content="summary_large_image" />
      <Meta name="twitter:title" content="Headwind Playground — Atomic to Semantic CSS" />
      <Meta name="twitter:description" content="Convert Tailwind atomic utility classes to optimized semantic CSS in real time. Supports JSX, TSX, and HTML." />

      {/* Header */}
      <header class="header">
        <div class="header-left">
          <h1 class="logo">Headwind</h1>
          <span class="badge">Playground</span>
        </div>
        <div class="header-right">
          <Show when={wasmLoaded()}>
            <span class="status status-ok">WASM Ready</span>
          </Show>
          <Show when={!wasmLoaded() && !wasmError()}>
            <span class="status status-loading">Loading WASM...</span>
          </Show>
          <Show when={wasmError()}>
            <span class="status status-err">{wasmError()}</span>
          </Show>
        </div>
      </header>

      {/* Toolbar */}
      <div class="toolbar">
        <div class="toolbar-group">
          <label class="toolbar-label">Language</label>
          <select
            class="toolbar-select"
            value={language()}
            onChange={(e) => setLanguage(e.currentTarget.value as Language)}
          >
            <option value="jsx">JSX / TSX</option>
            <option value="html">HTML</option>
          </select>
        </div>

        <div class="toolbar-group">
          <label class="toolbar-label">Naming</label>
          <select
            class="toolbar-select"
            value={namingMode()}
            onChange={(e) => setNamingMode(e.currentTarget.value as NamingMode)}
          >
            <option value="hash">Hash (c_a1b2c3)</option>
            <option value="readable">Readable (p4_m2)</option>
            <option value="camelCase">CamelCase (p4M2)</option>
          </select>
        </div>

        <div class="toolbar-group">
          <label class="toolbar-label">Output</label>
          <select
            class="toolbar-select"
            value={outputModeType()}
            onChange={(e) => setOutputModeType(e.currentTarget.value as OutputModeType)}
          >
            <option value="global">Global</option>
            <option value="cssModules">CSS Modules</option>
          </select>
        </div>

        <div class="toolbar-group">
          <label class="toolbar-label">Values</label>
          <select
            class="toolbar-select"
            value={cssVariables()}
            onChange={(e) => setCssVariables(e.currentTarget.value as CssVariablesMode)}
          >
            <option value="inline">Inline (1.875rem)</option>
            <option value="var">Variables (var(--text-3xl))</option>
          </select>
        </div>

        <div class="toolbar-group">
          <label class="toolbar-label">Unknown</label>
          <select
            class="toolbar-select"
            value={unknownClasses()}
            onChange={(e) => setUnknownClasses(e.currentTarget.value as UnknownClassesMode)}
          >
            <option value="preserve">Preserve</option>
            <option value="remove">Remove</option>
          </select>
        </div>

        <div class="toolbar-group">
          <label class="toolbar-label">Colors</label>
          <select
            class="toolbar-select"
            value={colorMode()}
            onChange={(e) => setColorMode(e.currentTarget.value as ColorModeType)}
          >
            <option value="hex">Hex (#3b82f6)</option>
            <option value="oklch">OKLCH</option>
            <option value="hsl">HSL</option>
            <option value="var">CSS Var</option>
          </select>
        </div>

        <Show when={outputModeType() === 'cssModules'}>
          <div class="toolbar-group">
            <label class="toolbar-label">Access</label>
            <select
              class="toolbar-select"
              value={accessMode()}
              onChange={(e) => setAccessMode(e.currentTarget.value as AccessMode)}
            >
              <option value="dot">Dot (styles.xxx)</option>
              <option value="bracket">Bracket (styles["xxx"])</option>
            </select>
          </div>
        </Show>

        <Show when={result()}>
          <div class="toolbar-info">
            <span class="duration">{duration().toFixed(1)}ms</span>
          </div>
        </Show>
      </div>

      {/* Main panels */}
      <div class="panels">
        {/* Input panel */}
        <div class="panel panel-input">
          <div class="panel-header">
            <span class="panel-title">
              Input ({language() === 'jsx' ? 'JSX' : 'HTML'})
            </span>
          </div>
          <textarea
            class="editor"
            value={source()}
            onInput={(e) => setSource(e.currentTarget.value)}
            spellcheck={false}
          />
        </div>

        {/* Output panel */}
        <div class="panel panel-output">
          <div class="panel-header">
            <button
              class={`tab ${activeTab() === 'code' ? 'tab-active' : ''}`}
              onClick={() => setActiveTab('code')}
            >
              Output Code
            </button>
            <button
              class={`tab ${activeTab() === 'css' ? 'tab-active' : ''}`}
              onClick={() => setActiveTab('css')}
            >
              Generated CSS
            </button>
            <button
              class={`tab ${activeTab() === 'map' ? 'tab-active' : ''}`}
              onClick={() => setActiveTab('map')}
            >
              Class Map ({classMapEntries().length})
            </button>
            <button
              class="copy-btn"
              onClick={copyOutput}
              disabled={!result()}
            >
              {copied() ? 'Copied!' : 'Copy'}
            </button>
          </div>

          <Show when={error()}>
            <pre class="error-output">{error()}</pre>
          </Show>

          <Show when={!error()}>
            <Show when={activeTab() === 'code'}>
              <pre class="output">{result()?.code ?? ''}</pre>
            </Show>
            <Show when={activeTab() === 'css'}>
              <pre class="output output-css">{result()?.css ?? ''}</pre>
            </Show>
            <Show when={activeTab() === 'map'}>
              <div class="map-table-wrap">
                <table class="map-table">
                  <thead>
                    <tr>
                      <th>Tailwind Classes</th>
                      <th>Generated Name</th>
                    </tr>
                  </thead>
                  <tbody>
                    {classMapEntries().map(([orig, gen]) => (
                      <tr>
                        <td><code>{orig}</code></td>
                        <td><code>{gen}</code></td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </Show>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default App;
