import * as vscode from "vscode";
import * as path from "path";
import { state } from "./state";
import { runTransform } from "./wasm";
import { writeCssOutput } from "./cssOutput";
import { getCssOutputPattern } from "./config";
import { clearDiagnostics, reportError } from "./diagnostics";
import { log, logError } from "./logger";
import { isSupportedFile } from "./types";
import { formatCodeString } from "./formatCode";

const SCHEME = "headwind-preview";

/**
 * Content provider for the "headwind-preview" scheme.
 * Serves the most recent transformed code for a given original file URI.
 */
class HeadwindPreviewProvider implements vscode.TextDocumentContentProvider {
  private _onDidChange = new vscode.EventEmitter<vscode.Uri>();
  readonly onDidChange = this._onDidChange.event;

  private _cache = new Map<string, string>();

  setContent(originalUri: vscode.Uri, code: string): void {
    this._cache.set(originalUri.fsPath, code);
    const virtualUri = this.toVirtualUri(originalUri);
    this._onDidChange.fire(virtualUri);
  }

  provideTextDocumentContent(uri: vscode.Uri): string {
    const originalPath = uri.query;
    return this._cache.get(originalPath) ?? "";
  }

  /**
   * Convert an original file URI to its virtual preview URI.
   *
   * Scheme:   headwind-preview
   * Path:     /<filename> (for display in tab title)
   * Query:    <original fsPath> (for cache lookup)
   * Fragment: v=<version> (to bust VS Code's caching)
   */
  toVirtualUri(originalUri: vscode.Uri): vscode.Uri {
    const filename = path.basename(originalUri.fsPath);
    return vscode.Uri.parse(
      `${SCHEME}:///${filename}?${encodeURIComponent(originalUri.fsPath)}#v=${state.version}`,
    );
  }

  dispose(): void {
    this._onDidChange.dispose();
  }
}

let previewProvider: HeadwindPreviewProvider;

export function registerDiffPreview(context: vscode.ExtensionContext): void {
  previewProvider = new HeadwindPreviewProvider();

  context.subscriptions.push(
    vscode.workspace.registerTextDocumentContentProvider(SCHEME, previewProvider),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("headwind.previewTransform", executePreviewTransform),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("headwind.applyTransform", executeApplyTransform),
  );
}

/**
 * Resolve the document to use for transform.
 * Prefers the active text editor if it's a supported file,
 * otherwise falls back to the last tracked file URI from state.
 */
async function resolveDocument(): Promise<vscode.TextDocument | undefined> {
  const active = vscode.window.activeTextEditor;
  if (active && isSupportedFile(active.document.fileName)) {
    return active.document;
  }

  if (state.activeFileUri) {
    return await vscode.workspace.openTextDocument(state.activeFileUri);
  }

  return undefined;
}

const NO_FILE_MSG =
  "No supported file selected. Please focus a .jsx, .tsx, or .html file first.";

async function executePreviewTransform(): Promise<void> {
  const document = await resolveDocument();
  if (!document) {
    vscode.window.showErrorMessage(NO_FILE_MSG);
    return;
  }
  const source = document.getText();
  const filename = path.basename(document.uri.fsPath);

  try {
    const start = performance.now();
    const result = runTransform(source, filename, state.options, getCssOutputPattern());
    const duration = performance.now() - start;

    state.setResult(result, duration);
    clearDiagnostics(document.uri);

    log(
      `Transform preview: ${filename} in ${duration.toFixed(1)}ms (${Object.keys(result.classMap).length} class groups)`,
    );

    // Format the transformed code with the user's formatter before previewing
    const formattedCode = await formatCodeString(result.code, filename);

    // Update the virtual document
    previewProvider.setContent(document.uri, formattedCode);

    // Open the diff editor
    const originalUri = document.uri;
    const previewUri = previewProvider.toVirtualUri(originalUri);
    const title = `Headwind: ${filename} (Preview)`;

    await vscode.commands.executeCommand("vscode.diff", originalUri, previewUri, title, {
      preview: true,
    });
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    logError(`Transform failed for ${filename}`, e);
    reportError(document.uri, message);
    vscode.window.showErrorMessage(`Headwind transform failed: ${message}`);
  }
}

async function executeApplyTransform(): Promise<void> {
  const result = state.lastResult;
  if (!result) {
    vscode.window.showErrorMessage("No transform result available. Run Preview first.");
    return;
  }

  // Resolve target document — prefer active editor if supported, else fallback
  const doc = await resolveDocument();
  if (!doc) {
    vscode.window.showErrorMessage(NO_FILE_MSG);
    return;
  }

  const editor = await vscode.window.showTextDocument(doc);
  const source = doc.getText();
  const fullRange = new vscode.Range(
    doc.positionAt(0),
    doc.positionAt(source.length),
  );

  await editor.edit((editBuilder) => {
    editBuilder.replace(fullRange, result.code);
  });

  // Format the document with the user's default formatter
  await vscode.commands.executeCommand("editor.action.formatDocument");

  // Write CSS file
  if (result.css.trim().length > 0) {
    const cssUri = await writeCssOutput(doc.uri, result.css, state.options.outputMode);
    log(`CSS written to ${path.basename(cssUri.fsPath)}`);
    vscode.window.showInformationMessage(
      `Headwind: Applied! CSS written to ${path.basename(cssUri.fsPath)}`,
    );
  } else {
    vscode.window.showInformationMessage("Headwind: Applied!");
  }
}
