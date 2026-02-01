import * as vscode from "vscode";
import { loadWasm } from "./wasm";
import { state } from "./state";
import { registerDiffPreview } from "./diffPreview";
import { registerControlPanel } from "./controlPanel";
import { registerTransformOnSave } from "./transformOnSave";
import { initLogger, log, logError } from "./logger";
import { initDiagnostics } from "./diagnostics";

export function activate(context: vscode.ExtensionContext): void {
  // Initialize services
  const outputChannel = initLogger();
  const diagnostics = initDiagnostics();
  context.subscriptions.push(outputChannel, diagnostics);

  // Load WASM engine
  try {
    loadWasm();
    log("Headwind WASM engine loaded");
  } catch (err) {
    logError("Failed to load WASM engine", err);
    vscode.window.showErrorMessage(
      "Headwind: Failed to load WASM engine. See Output channel for details.",
    );
    return;
  }

  // Set initial active file (with URI for fallback operations)
  if (vscode.window.activeTextEditor) {
    const editor = vscode.window.activeTextEditor;
    state.setActiveFile(editor.document.fileName, editor.document.uri);
  }

  // Register features
  registerDiffPreview(context);
  registerControlPanel(context);
  registerTransformOnSave(context);

  // headwind.transform is a shortcut for previewTransform
  context.subscriptions.push(
    vscode.commands.registerCommand("headwind.transform", () => {
      vscode.commands.executeCommand("headwind.previewTransform");
    }),
  );

  log("Headwind extension activated");
}

export function deactivate(): void {}
