import * as vscode from "vscode";
import { state } from "./state";
import { runTransform } from "./wasm";
import { getCssOutputPattern } from "./config";
import { isSupportedFile } from "./types";
import type { WebviewToHostMessage, HostToWebviewMessage } from "./types";
import { fetchSemanticNames, applyAiNames } from "@headwind/common-utils";
import { previewCurrentResult } from "./diffPreview";

let panel: vscode.WebviewPanel | undefined;

export function registerControlPanel(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("headwind.openPanel", () => {
      createOrShowPanel(context);
    }),
  );

  // Sync active editor changes to the panel — only update for supported files
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor) {
        const updated = state.setActiveFile(
          editor.document.fileName,
          editor.document.uri,
        );
        if (updated) {
          postMessage({
            type: "activeFileChanged",
            filename: state.activeFilename,
          });
        }
      }
      // When editor is undefined (e.g. panel focused) or file is unsupported,
      // we keep the previous valid file — no update needed.
    }),
  );

  // When diff preview updates state, push result to panel
  state.on("resultChanged", (result, duration) => {
    postMessage({ type: "transformResult", result, duration });
  });
}

function createOrShowPanel(context: vscode.ExtensionContext): void {
  if (panel) {
    panel.reveal(vscode.ViewColumn.Beside);
    return;
  }

  const webviewDir = vscode.Uri.joinPath(context.extensionUri, "dist", "webview");

  panel = vscode.window.createWebviewPanel(
    "headwind.controlPanel",
    "Headwind",
    { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true },
    {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [webviewDir],
    },
  );

  panel.webview.html = getWebviewHtml(panel.webview, webviewDir);

  panel.webview.onDidReceiveMessage(
    (msg: WebviewToHostMessage) => handleWebviewMessage(msg),
    undefined,
    context.subscriptions,
  );

  panel.onDidDispose(
    () => {
      panel = undefined;
    },
    null,
    context.subscriptions,
  );
}

function handleWebviewMessage(msg: WebviewToHostMessage): void {
  switch (msg.type) {
    case "ready":
      postMessage({ type: "init", state: state.toPanelState() });
      break;

    case "optionsChanged":
      state.setOptions(msg.options);
      postMessage({ type: "optionsUpdated", options: msg.options });
      break;

    case "requestTransform":
      void doTransform();
      break;

    case "requestPreviewDiff":
      void previewCurrentResult();
      break;

    case "requestApply":
      vscode.commands.executeCommand("headwind.applyTransform");
      break;

    case "copyToClipboard":
      vscode.env.clipboard.writeText(msg.text);
      vscode.window.showInformationMessage("Copied to clipboard");
      break;

    case "requestAiRename":
      void doAiRename();
      break;
  }
}

/**
 * Resolve the document to use for transform operations.
 * Prefers the active text editor's document if it's a supported file,
 * otherwise silently reads the last tracked file from state (no new tab).
 */
async function resolveDocument(): Promise<vscode.TextDocument | undefined> {
  const active = vscode.window.activeTextEditor;
  if (active && isSupportedFile(active.document.fileName)) {
    return active.document;
  }

  // Silently open the document without showing it in a tab
  if (state.activeFileUri) {
    return await vscode.workspace.openTextDocument(state.activeFileUri);
  }

  return undefined;
}

async function doTransform(): Promise<void> {
  const doc = await resolveDocument();
  if (!doc) {
    postMessage({
      type: "transformError",
      error:
        "No supported file selected. Please focus a .jsx, .tsx, or .html file first.",
    });
    return;
  }

  try {
    const source = doc.getText();
    const filename = doc.fileName;
    const start = performance.now();
    const result = runTransform(source, filename, state.options, getCssOutputPattern());
    const duration = performance.now() - start;
    state.setResult(result, duration);
    postMessage({ type: "transformResult", result, duration });
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    postMessage({ type: "transformError", error: message });
  }
}

async function doAiRename(): Promise<void> {
  const result = state.lastResult;
  if (!result?.elementTree) {
    const msg = "No element tree available. Enable Element Tree option and transform first.";
    postMessage({ type: "aiRenameError", error: msg });
    vscode.window.showWarningMessage(`Headwind: ${msg}`);
    return;
  }

  const apiUrl = vscode.workspace.getConfiguration("headwind").get<string>("aiApiUrl", "");
  if (!apiUrl) {
    const action = await vscode.window.showWarningMessage(
      "Headwind: AI API URL not configured.",
      "Open Settings",
    );
    if (action === "Open Settings") {
      vscode.commands.executeCommand("workbench.action.openSettings", "headwind.aiApiUrl");
    }
    postMessage({ type: "aiRenameError", error: "AI API URL not configured. Set headwind.aiApiUrl in settings." });
    return;
  }

  try {
    const names = await fetchSemanticNames(result.elementTree, apiUrl);
    const { outputMode } = state.options;
    const useCamelCase = outputMode.type === "cssModules" && outputMode.access === "dot";
    const renamed = applyAiNames(result, names, { camelCase: useCamelCase });
    state.updateResultSilent(renamed);
    postMessage({ type: "aiRenameResult", result: renamed });
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    postMessage({ type: "aiRenameError", error: message });
    vscode.window.showErrorMessage(`Headwind AI Rename failed: ${message}`);
  }
}

function postMessage(msg: HostToWebviewMessage): void {
  panel?.webview.postMessage(msg);
}

function getWebviewHtml(webview: vscode.Webview, webviewDir: vscode.Uri): string {
  const nonce = getNonce();
  const scriptUri = webview.asWebviewUri(vscode.Uri.joinPath(webviewDir, "index.js"));
  const styleUri = webview.asWebviewUri(vscode.Uri.joinPath(webviewDir, "index.css"));

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta http-equiv="Content-Security-Policy"
    content="default-src 'none'; style-src ${webview.cspSource}; script-src 'nonce-${nonce}';" />
  <link rel="stylesheet" href="${styleUri}" />
  <title>Headwind</title>
</head>
<body>
  <div id="root"></div>
  <script nonce="${nonce}" src="${scriptUri}"></script>
</body>
</html>`;
}

function getNonce(): string {
  let text = "";
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  for (let i = 0; i < 32; i++) {
    text += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return text;
}
