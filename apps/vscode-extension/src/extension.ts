import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext) {
  const disposable = vscode.commands.registerCommand(
    "headwind.transform",
    async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showErrorMessage("No active editor");
        return;
      }

      // TODO: Implement transformation using @headwind/transformer
      vscode.window.showInformationMessage("Headwind transform - coming soon");
    }
  );

  context.subscriptions.push(disposable);
}

export function deactivate() {}
