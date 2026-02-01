import * as vscode from "vscode";

let diagnosticCollection: vscode.DiagnosticCollection;

export function initDiagnostics(): vscode.DiagnosticCollection {
  diagnosticCollection = vscode.languages.createDiagnosticCollection("headwind");
  return diagnosticCollection;
}

export function clearDiagnostics(uri: vscode.Uri): void {
  diagnosticCollection.delete(uri);
}

export function reportError(uri: vscode.Uri, message: string): void {
  const diagnostic = new vscode.Diagnostic(
    new vscode.Range(0, 0, 0, 0),
    `Headwind: ${message}`,
    vscode.DiagnosticSeverity.Error,
  );
  diagnosticCollection.set(uri, [diagnostic]);
}
