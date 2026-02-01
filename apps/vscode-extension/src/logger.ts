import * as vscode from "vscode";

let outputChannel: vscode.OutputChannel;

export function initLogger(): vscode.OutputChannel {
  outputChannel = vscode.window.createOutputChannel("Headwind");
  return outputChannel;
}

export function log(message: string): void {
  const ts = new Date().toISOString();
  outputChannel.appendLine(`[${ts}] ${message}`);
}

export function logError(message: string, error?: unknown): void {
  const ts = new Date().toISOString();
  outputChannel.appendLine(`[${ts}] ERROR: ${message}`);
  if (error instanceof Error) {
    outputChannel.appendLine(`  ${error.message}`);
    if (error.stack) outputChannel.appendLine(`  ${error.stack}`);
  }
}
