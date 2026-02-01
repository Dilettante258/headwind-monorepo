import * as vscode from "vscode";
import * as path from "path";
import { getCssOutputPattern } from "./config";
import type { TransformOptions } from "./types";

export function getCssOutputUri(
  sourceUri: vscode.Uri,
  outputMode: TransformOptions["outputMode"],
): vscode.Uri {
  const dir = path.dirname(sourceUri.fsPath);
  const ext = path.extname(sourceUri.fsPath);
  const name = path.basename(sourceUri.fsPath, ext);

  let cssFileName: string;
  if (outputMode.type === "cssModules") {
    // CSS Modules always uses [name].module.css to match Rust import derivation
    cssFileName = `${name}.module.css`;
  } else {
    const pattern = getCssOutputPattern();
    cssFileName = pattern.replace("[name]", name);
  }

  return vscode.Uri.file(path.join(dir, cssFileName));
}

export async function writeCssOutput(
  sourceUri: vscode.Uri,
  css: string,
  outputMode: TransformOptions["outputMode"],
): Promise<vscode.Uri> {
  const cssUri = getCssOutputUri(sourceUri, outputMode);
  const encoder = new TextEncoder();
  await vscode.workspace.fs.writeFile(cssUri, encoder.encode(css));
  return cssUri;
}
