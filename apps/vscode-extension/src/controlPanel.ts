import * as vscode from "vscode";
import { state } from "./state";
import { runTransform } from "./wasm";
import { getWebviewHtml } from "./controlPanelHtml";
import { getCssOutputPattern } from "./config";
import { isSupportedFile } from "./types";
import type { WebviewToHostMessage, HostToWebviewMessage } from "./types";

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

  panel = vscode.window.createWebviewPanel(
    "headwind.controlPanel",
    "Headwind",
    { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true },
    {
      enableScripts: true,
      retainContextWhenHidden: true,
    },
  );

  panel.webview.html = getWebviewHtml(panel.webview);

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
      vscode.commands.executeCommand("headwind.previewTransform");
      break;

    case "requestApply":
      vscode.commands.executeCommand("headwind.applyTransform");
      break;

    case "copyToClipboard":
      vscode.env.clipboard.writeText(msg.text);
      vscode.window.showInformationMessage("Copied to clipboard");
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

function postMessage(msg: HostToWebviewMessage): void {
  panel?.webview.postMessage(msg);
}
