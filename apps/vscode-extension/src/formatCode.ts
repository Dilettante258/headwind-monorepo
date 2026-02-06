import * as vscode from "vscode";
import * as path from "path";

/**
 * Map file extension to VSCode language identifier.
 */
function getLanguageId(filename: string): string {
  const ext = path.extname(filename).toLowerCase();
  switch (ext) {
    case ".tsx":
      return "typescriptreact";
    case ".jsx":
      return "javascriptreact";
    case ".ts":
      return "typescript";
    case ".js":
      return "javascript";
    case ".html":
    case ".htm":
      return "html";
    default:
      return "plaintext";
  }
}

/**
 * Apply an array of TextEdits to a plain string, returning the new string.
 *
 * Edits are applied in reverse document order so earlier offsets are not
 * invalidated by later replacements.
 */
function applyTextEdits(text: string, edits: vscode.TextEdit[]): string {
  const sorted = [...edits].sort((a, b) => {
    const lineDiff = b.range.start.line - a.range.start.line;
    if (lineDiff !== 0) return lineDiff;
    return b.range.start.character - a.range.start.character;
  });

  const lines = text.split("\n");

  for (const edit of sorted) {
    const { start, end } = edit.range;
    const prefix = lines[start.line]?.substring(0, start.character) ?? "";
    const suffix = lines[end.line]?.substring(end.character) ?? "";
    const newLines = (prefix + edit.newText + suffix).split("\n");
    lines.splice(start.line, end.line - start.line + 1, ...newLines);
  }

  return lines.join("\n");
}

/**
 * Format a code string using the user's configured formatter.
 *
 * Opens a temporary in-memory document with the correct language,
 * asks the registered formatting provider for edits, applies them
 * to the string, and closes any resulting untitled tab.
 * Returns the original string unchanged if no formatter is available.
 */
export async function formatCodeString(
  code: string,
  filename: string,
): Promise<string> {
  const languageId = getLanguageId(filename);

  try {
    const doc = await vscode.workspace.openTextDocument({
      content: code,
      language: languageId,
    });

    const config = vscode.workspace.getConfiguration("editor", doc.uri);
    const options: vscode.FormattingOptions = {
      tabSize: config.get<number>("tabSize", 2),
      insertSpaces: config.get<boolean>("insertSpaces", true),
    };

    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      "vscode.executeFormatDocumentProvider",
      doc.uri,
      options,
    );

    // Close any untitled tab that may have been created for this document
    // This prevents the untitled window from appearing
    await closeUntitledTab(doc.uri);

    if (!edits || edits.length === 0) {
      return code;
    }

    return applyTextEdits(code, edits);
  } catch {
    // No formatter available or formatting failed — return as-is
    return code;
  }
}

/**
 * Close an untitled tab by its URI if it exists.
 */
async function closeUntitledTab(uri: vscode.Uri): Promise<void> {
  if (uri.scheme !== "untitled") return;

  for (const tabGroup of vscode.window.tabGroups.all) {
    for (const tab of tabGroup.tabs) {
      const input = tab.input;
      if (input instanceof vscode.TabInputText && input.uri.toString() === uri.toString()) {
        await vscode.window.tabGroups.close(tab);
        return;
      }
    }
  }
}
