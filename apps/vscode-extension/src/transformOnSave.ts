import * as vscode from "vscode";
import * as path from "path";
import { state } from "./state";
import { runTransform } from "./wasm";
import { isTransformOnSave, getCssOutputPattern } from "./config";
import { writeCssOutput } from "./cssOutput";
import { clearDiagnostics, reportError } from "./diagnostics";
import { log, logError } from "./logger";
import type { SupportedLanguage } from "./types";
import { formatCodeString } from "./formatCode";

function detectLanguage(filename: string): SupportedLanguage | null {
  const ext = path.extname(filename).toLowerCase();
  if ([".jsx", ".tsx"].includes(ext)) return "jsx";
  if ([".html", ".htm"].includes(ext)) return "html";
  return null;
}

export function registerTransformOnSave(context: vscode.ExtensionContext): void {
  const disposable = vscode.workspace.onWillSaveTextDocument((event) => {
    if (!isTransformOnSave()) return;

    const document = event.document;
    const filename = path.basename(document.uri.fsPath);
    const language = detectLanguage(filename);
    if (!language) return;

    event.waitUntil(
      (async (): Promise<vscode.TextEdit[]> => {
        try {
          const source = document.getText();
          const result = runTransform(source, filename, state.options, getCssOutputPattern());

          clearDiagnostics(document.uri);

          // Write CSS file (fire-and-forget)
          if (result.css.trim().length > 0) {
            writeCssOutput(document.uri, result.css, state.options.outputMode).catch((err) =>
              logError("CSS write failed", err),
            );
          }

          log(
            `Transform on save: ${filename} (${Object.keys(result.classMap).length} classes)`,
          );

          // Format the transformed code with the user's formatter
          const formattedCode = await formatCodeString(result.code, filename);

          const fullRange = new vscode.Range(
            document.positionAt(0),
            document.positionAt(source.length),
          );
          return [vscode.TextEdit.replace(fullRange, formattedCode)];
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err);
          logError(`Transform on save failed for ${filename}`, err);
          reportError(document.uri, message);
          return [];
        }
      })(),
    );
  });

  context.subscriptions.push(disposable);
}
