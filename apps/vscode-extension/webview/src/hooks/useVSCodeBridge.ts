import { createSignal, onMount, onCleanup } from 'solid-js';
import type {
  WebviewToHostMessage,
  HostToWebviewMessage,
  TransformOptions,
  TransformResult,
} from '@ext/types';

interface VSCodeApi {
  postMessage(msg: WebviewToHostMessage): void;
  getState(): unknown;
  setState(state: unknown): void;
}

declare function acquireVsCodeApi(): VSCodeApi;

export function useVSCodeBridge() {
  const vscode = acquireVsCodeApi();

  const [wasmReady, setWasmReady] = createSignal(false);
  const [options, setOptions] = createSignal<TransformOptions | null>(null);
  const [result, setResult] = createSignal<TransformResult | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [activeFilename, setActiveFilename] = createSignal<string | null>(null);
  const [duration, setDuration] = createSignal(0);
  const [aiLoading, setAiLoading] = createSignal(false);
  const [aiError, setAiError] = createSignal<string | null>(null);
  const [aiApplied, setAiApplied] = createSignal(false);

  let originalResult: TransformResult | null = null;

  function send(msg: WebviewToHostMessage) {
    vscode.postMessage(msg);
  }

  onMount(() => {
    function handleMessage(event: MessageEvent<HostToWebviewMessage>) {
      const msg = event.data;
      switch (msg.type) {
        case 'init':
          setWasmReady(true);
          setOptions(msg.state.options);
          if (msg.state.result) {
            setResult(msg.state.result);
          }
          setActiveFilename(msg.state.activeFilename);
          if (msg.state.duration > 0) {
            setDuration(msg.state.duration);
          }
          break;
        case 'transformResult':
          setResult(msg.result);
          setDuration(msg.duration);
          setError(null);
          // Reset AI state on new transform
          setAiApplied(false);
          setAiError(null);
          originalResult = null;
          break;
        case 'transformError':
          setError(msg.error);
          break;
        case 'activeFileChanged':
          setActiveFilename(msg.filename);
          break;
        case 'optionsUpdated':
          setOptions(msg.options);
          break;
        case 'aiRenameResult':
          setResult(msg.result);
          setAiLoading(false);
          setAiApplied(true);
          setAiError(null);
          break;
        case 'aiRenameError':
          setAiLoading(false);
          setAiError(msg.error);
          break;
      }
    }

    window.addEventListener('message', handleMessage);
    onCleanup(() => window.removeEventListener('message', handleMessage));

    send({ type: 'ready' });
  });

  function requestAiRename() {
    const r = result();
    if (!r?.elementTree || aiLoading()) return;
    originalResult = r;
    setAiLoading(true);
    setAiError(null);
    send({ type: 'requestAiRename' });
  }

  function resetAiRename() {
    if (originalResult) {
      setResult(originalResult);
      originalResult = null;
      setAiApplied(false);
      setAiError(null);
    }
  }

  return {
    wasmReady,
    options,
    result,
    error,
    activeFilename,
    duration,
    aiLoading,
    aiError,
    aiApplied,
    requestTransform: () => send({ type: 'requestTransform' }),
    requestPreviewDiff: () => send({ type: 'requestPreviewDiff' }),
    requestApply: () => send({ type: 'requestApply' }),
    copyToClipboard: (text: string) => send({ type: 'copyToClipboard', text }),
    sendOptions: (opts: TransformOptions) =>
      send({ type: 'optionsChanged', options: opts }),
    requestAiRename,
    resetAiRename,
  };
}
