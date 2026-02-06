// Mock the VS Code API for browser-based development
const state: Record<string, unknown> = {};

(window as any).acquireVsCodeApi = () => ({
  postMessage: (msg: any) => {
    console.log('[vscode mock] postMessage:', msg);
    if (msg.type === 'ready') {
      setTimeout(() => {
        window.postMessage(
          {
            type: 'init',
            state: {
              options: {
                namingMode: 'hash',
                outputMode: { type: 'global' },
                cssVariables: 'var',
                unknownClasses: 'preserve',
                colorMode: 'hex',
                elementTree: false,
              },
              result: null,
              activeFilename: 'App.tsx',
              duration: 0,
            },
          },
          '*',
        );
      }, 100);
    }
    if (msg.type === 'requestTransform') {
      setTimeout(() => {
        window.postMessage(
          {
            type: 'transformResult',
            result: {
              code: 'export default function App() {\n  return <div className="c_abc123">Hello</div>;\n}',
              css: '.c_abc123 {\n  display: flex;\n  flex-direction: column;\n}',
              classMap: {
                'flex flex-col': 'c_abc123',
                'text-blue-600': 'c_def456',
              },
              elementTree: '## App\n- div flex flex-col [ref=e1]\n  - span text-blue-600 "Hello" [ref=e2]',
            },
            duration: 2.5,
          },
          '*',
        );
      }, 200);
    }
    if (msg.type === 'requestAiRename') {
      setTimeout(() => {
        window.postMessage(
          {
            type: 'aiRenameResult',
            result: {
              code: 'export default function App() {\n  return <div className="app_layout">Hello</div>;\n}',
              css: '.app_layout {\n  display: flex;\n  flex-direction: column;\n}',
              classMap: {
                'flex flex-col': 'app_layout',
                'text-blue-600': 'heading_text',
              },
              elementTree: '## App\n- div flex flex-col [ref=e1]\n  - span text-blue-600 "Hello" [ref=e2]',
            },
          },
          '*',
        );
      }, 500);
    }
    if (msg.type === 'copyToClipboard') {
      navigator.clipboard.writeText(msg.text).then(() => {
        console.log('[vscode mock] copied to clipboard');
      });
    }
  },
  getState: () => state,
  setState: (s: any) => Object.assign(state, s),
});
