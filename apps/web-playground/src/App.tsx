import { createSignal, createEffect, onMount, onCleanup, type Component, Show } from 'solid-js';
import { Title, Meta, Link } from '@solidjs/meta';
import About from "./About.tsx";
import {
  loadWasm,
  runTransformJsx,
  runTransformHtml,
  type TransformResult,
  type TransformOptions,
} from "./wasm.ts";
import { applyAiNames } from "@headwind/common-utils";
import { fetchSemanticNames } from "./api.ts";

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

function Playground() {
  const [wasmLoaded, setWasmLoaded] = createSignal(false);
  const [wasmError, setWasmError] = createSignal("");
  const [language, setLanguage] = createSignal<Language>("jsx");
  const [source, setSource] = createSignal(DEFAULT_JSX);
  const [namingMode, setNamingMode] = createSignal<NamingMode>("camelCase");
  const [outputModeType, setOutputModeType] =
    createSignal<OutputModeType>("cssModules");
  const [accessMode, setAccessMode] = createSignal<AccessMode>("dot");
  const [cssVariables, setCssVariables] =
    createSignal<CssVariablesMode>("inline");
  const [unknownClasses, setUnknownClasses] =
    createSignal<UnknownClassesMode>("preserve");
  const [colorMode, setColorMode] = createSignal<ColorModeType>("hex");
  const [colorMix, setColorMix] = createSignal(false);
  const [elementTree, setElementTree] = createSignal(true);
  const [result, setResult] = createSignal<TransformResult | null>(null);
  const [error, setError] = createSignal("");
  const [duration, setDuration] = createSignal(0);
  const [activeTab, setActiveTab] = createSignal<
    "code" | "css" | "map" | "tree"
  >("code");
  const [copied, setCopied] = createSignal(false);
  const [aiLoading, setAiLoading] = createSignal(false);
  const [aiError, setAiError] = createSignal("");
  const [originalResult, setOriginalResult] =
    createSignal<TransformResult | null>(null);
  const [aiApplied, setAiApplied] = createSignal(false);
  let settingsDialogRef: HTMLDialogElement | undefined;

  onMount(async () => {
    try {
      await loadWasm();
      setWasmLoaded(true);
    } catch (e: any) {
      setWasmError(e.message || "Failed to load WASM");
    }
  });

  // Switch default source when language changes
  createEffect(() => {
    const lang = language();
    setSource(lang === "jsx" ? DEFAULT_JSX : DEFAULT_HTML);
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
    const mix = colorMix();
    const tree = elementTree();

    const options: TransformOptions = {
      namingMode: naming,
      outputMode:
        outType === "global"
          ? { type: "global" }
          : { type: "cssModules", access },
      cssVariables: cssVars,
      unknownClasses: unknown,
      colorMode: color,
      colorMix: mix,
      elementTree: tree,
    };

    try {
      const start = performance.now();
      let res: TransformResult;
      if (language() === "jsx") {
        res = runTransformJsx(src, "App.tsx", options);
      } else {
        res = runTransformHtml(src, options);
      }
      const elapsed = performance.now() - start;
      setDuration(elapsed);
      setResult(res);
      setError("");
      setAiApplied(false);
      setOriginalResult(null);
      setAiError("");
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
    if (!r) return "";
    if (tab === "code") return r.code;
    if (tab === "css") return r.css;
    if (tab === "tree") return r.elementTree ?? "";
    return classMapEntries()
      .map(([orig, gen]) => `${orig} → ${gen}`)
      .join("\n");
  };

  const copyOutput = async () => {
    const text = currentOutputText();
    if (!text) return;
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const handleAiRename = async () => {
    const r = result();
    if (!r?.elementTree || aiLoading()) return;
    setAiLoading(true);
    setAiError("");
    try {
      const names = await fetchSemanticNames(r.elementTree);
      setOriginalResult(r);
      const useCamelCase = outputModeType() === "cssModules" && accessMode() === "dot";
      const renamed = applyAiNames(r, names, { camelCase: useCamelCase });
      setResult(renamed);
      setAiApplied(true);
    } catch (e: any) {
      setAiError(e.message || "AI rename failed");
    } finally {
      setAiLoading(false);
    }
  };

  const handleResetAi = () => {
    const orig = originalResult();
    if (orig) {
      setResult(orig);
      setOriginalResult(null);
      setAiApplied(false);
      setAiError("");
    }
  };

  return (
    <>
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
          <a
            href="/about"
            class="nav-link"
            onClick={(e) => {
              e.preventDefault();
              navigate("/about");
            }}
          >
            About
          </a>
          <a
            href="https://github.com/Dilettante258/headwind-monorepo"
            target="_blank"
            rel="noopener noreferrer"
            class="github-link"
            aria-label="GitHub"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z" />
            </svg>
          </a>
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
            onChange={(e) =>
              setOutputModeType(e.currentTarget.value as OutputModeType)
            }
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
            onChange={(e) =>
              setCssVariables(e.currentTarget.value as CssVariablesMode)
            }
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
            onChange={(e) =>
              setUnknownClasses(e.currentTarget.value as UnknownClassesMode)
            }
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
            onChange={(e) =>
              setColorMode(e.currentTarget.value as ColorModeType)
            }
          >
            <option value="hex">Hex (#3b82f6)</option>
            <option value="oklch">OKLCH</option>
            <option value="hsl">HSL</option>
            <option value="var">CSS Var</option>
          </select>
        </div>

        <Show when={outputModeType() === "cssModules"}>
          <div class="toolbar-group">
            <label class="toolbar-label">Access</label>
            <select
              class="toolbar-select"
              value={accessMode()}
              onChange={(e) =>
                setAccessMode(e.currentTarget.value as AccessMode)
              }
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

        <button
          class="settings-btn"
          onClick={() => settingsDialogRef?.showModal()}
        >
          Settings
        </button>
        <dialog
          ref={settingsDialogRef}
          class="settings-dialog"
          onClick={(e) => {
            if (e.target === e.currentTarget) settingsDialogRef?.close();
          }}
        >
          <div onClick={(e) => e.stopPropagation()}>
            <h3>Advanced Settings</h3>
            <label>
              <input
                type="checkbox"
                checked={colorMix()}
                onChange={(e) => setColorMix(e.currentTarget.checked)}
              />
              Use color-mix() for opacity
            </label>
            <p class="settings-hint">
              Generate <code>color-mix(in oklab, …)</code> for alpha modifiers
              (e.g. text-white/60). Useful with CSS Var color mode.
            </p>
            <label>
              <input
                type="checkbox"
                checked={elementTree()}
                onChange={(e) => setElementTree(e.currentTarget.checked)}
              />
              Element Tree
            </label>
            <p class="settings-hint">
              Generate a structured element tree with <code>[ref=eN]</code>{" "}
              identifiers, useful for passing to AI for secondary processing.
            </p>
          </div>
        </dialog>
      </div>

      {/* Main panels */}
      <div class="panels">
        {/* Input panel */}
        <div class="panel panel-input">
          <div class="panel-header">
            <span class="panel-title">
              Input ({language() === "jsx" ? "JSX" : "HTML"})
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
              class={`tab ${activeTab() === "code" ? "tab-active" : ""}`}
              onClick={() => setActiveTab("code")}
            >
              Output Code
            </button>
            <button
              class={`tab ${activeTab() === "css" ? "tab-active" : ""}`}
              onClick={() => setActiveTab("css")}
            >
              Generated CSS
            </button>
            <button
              class={`tab ${activeTab() === "map" ? "tab-active" : ""}`}
              onClick={() => setActiveTab("map")}
            >
              Class Map ({classMapEntries().length})
            </button>
            <Show when={elementTree()}>
              <button
                class={`tab ${activeTab() === "tree" ? "tab-active" : ""}`}
                onClick={() => setActiveTab("tree")}
              >
                Element Tree
              </button>
            </Show>
            <Show when={aiError()}>
              <span class="ai-error">{aiError()}</span>
            </Show>
            <Show when={!aiApplied()}>
              <button
                class="ai-btn"
                onClick={handleAiRename}
                disabled={!result()?.elementTree || aiLoading()}
              >
                {aiLoading() ? "Renaming..." : "AI Rename"}
              </button>
            </Show>
            <Show when={aiApplied()}>
              <button class="ai-btn ai-btn-reset" onClick={handleResetAi}>
                Reset AI
              </button>
            </Show>
            <button class="copy-btn" onClick={copyOutput} disabled={!result()}>
              {copied() ? "Copied!" : "Copy"}
            </button>
          </div>

          <Show when={error()}>
            <pre class="error-output">{error()}</pre>
          </Show>

          <Show when={!error()}>
            <Show when={activeTab() === "code"}>
              <pre class="output">{result()?.code ?? ""}</pre>
            </Show>
            <Show when={activeTab() === "css"}>
              <pre class="output output-css">{result()?.css ?? ""}</pre>
            </Show>
            <Show when={activeTab() === "map"}>
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
                        <td>
                          <code>{orig}</code>
                        </td>
                        <td>
                          <code>{gen}</code>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </Show>
            <Show when={activeTab() === "tree"}>
              <pre class="output">{result()?.elementTree ?? ""}</pre>
            </Show>
          </Show>
        </div>
      </div>
    </>
  );
}

type Page = 'playground' | 'about';

function getPageFromPath(): Page {
  return window.location.pathname === '/about' ? 'about' : 'playground';
}

export function navigate(path: string) {
  window.history.pushState(null, '', path);
  window.dispatchEvent(new PopStateEvent('popstate'));
}

const App: Component = () => {
  const [page, setPage] = createSignal<Page>(getPageFromPath());

  onMount(() => {
    const onPopState = () => setPage(getPageFromPath());
    window.addEventListener('popstate', onPopState);
    onCleanup(() => window.removeEventListener('popstate', onPopState));
  });

  return (
    <div class="app">
      {/* SEO Meta Tags */}
      <Show when={page() === 'playground'}>
        <Title>Headwind Playground — Atomic CSS to Semantic CSS Converter</Title>
        <Link rel="canonical" href="https://headwind-playground.kairi.cc/" />
      </Show>
      <Show when={page() === 'about'}>
        <Title>About — Headwind | Atomic CSS to Semantic CSS Compiler</Title>
        <Link rel="canonical" href="https://headwind-playground.kairi.cc/about" />
      </Show>
      <Meta
        name="description"
        content="Try Headwind online: convert Tailwind atomic utility classes to optimized semantic CSS in real time. Supports JSX, TSX, and HTML with configurable naming, CSS Modules, and color modes."
      />
      <Meta
        name="keywords"
        content="Tailwind CSS, atomic CSS, semantic CSS, CSS optimizer, CSS converter, Headwind, utility-first CSS, CSS Modules, WASM, playground"
      />
      <Meta name="author" content="Headwind" />
      <Meta name="msvalidate.01" content="E02F8D47396FC1DC9F6F91870B428BF9" />
      {/* Open Graph */}
      <Meta property="og:type" content="website" />
      <Meta
        property="og:title"
        content="Headwind Playground — Atomic to Semantic CSS"
      />
      <Meta
        property="og:description"
        content="Convert Tailwind atomic utility classes to optimized semantic CSS in real time. Supports JSX, TSX, and HTML."
      />
      <Meta property="og:url" content="https://headwind-playground.kairi.cc/" />
      <Meta property="og:site_name" content="Headwind Playground" />
      <Meta property="og:locale" content="en_US" />
      {/* Twitter Card */}
      <Meta name="twitter:card" content="summary_large_image" />
      <Meta
        name="twitter:title"
        content="Headwind Playground — Atomic to Semantic CSS"
      />
      <Meta
        name="twitter:description"
        content="Convert Tailwind atomic utility classes to optimized semantic CSS in real time. Supports JSX, TSX, and HTML."
      />
      <Show when={page() === 'playground'}>
        <Playground />
      </Show>
      <Show when={page() === 'about'}>
        <About />
      </Show>
    </div>
  );
};

export default App;
