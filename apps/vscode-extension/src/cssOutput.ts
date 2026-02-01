import * as vscode from "vscode";
import * as path from "path";
import { getCssOutputPattern } from "./config";

export function getCssOutputUri(sourceUri: vscode.Uri): vscode.Uri {
  const pattern = getCssOutputPattern();
  const dir = path.dirname(sourceUri.fsPath);
  const ext = path.extname(sourceUri.fsPath);
  const name = path.basename(sourceUri.fsPath, ext);
  const cssFileName = pattern.replace("[name]", name);
  return vscode.Uri.file(path.join(dir, cssFileName));
}

export async function writeCssOutput(
  sourceUri: vscode.Uri,
  css: string,
): Promise<vscode.Uri> {
  const cssUri = getCssOutputUri(sourceUri);
  const encoder = new TextEncoder();
  await vscode.workspace.fs.writeFile(cssUri, encoder.encode(css));
  return cssUri;
}
