import * as vscode from "vscode";
import type { TransformOptions } from "./types";

export function getTransformOptions(): TransformOptions {
  const config = vscode.workspace.getConfiguration("headwind");

  const outputModeType = config.get<string>("outputMode", "global");
  const outputMode =
    outputModeType === "cssModules"
      ? {
          type: "cssModules" as const,
          access: config.get<"dot" | "bracket">("cssModulesAccess", "dot"),
        }
      : { type: "global" as const };

  return {
    namingMode: config.get<"hash" | "readable" | "camelCase">("namingMode", "hash"),
    outputMode,
    cssVariables: config.get<"var" | "inline">("cssVariables", "var"),
    unknownClasses: config.get<"remove" | "preserve">("unknownClasses", "preserve"),
    colorMode: config.get<"hex" | "oklch" | "hsl" | "var">("colorMode", "hex"),
  };
}

export function getCssOutputPattern(): string {
  return vscode.workspace
    .getConfiguration("headwind")
    .get<string>("cssOutputPattern", "[name].css");
}

export function getIncludeGlob(): string {
  return vscode.workspace
    .getConfiguration("headwind")
    .get<string>("include", "**/*.{jsx,tsx,html}");
}

export function isTransformOnSave(): boolean {
  return vscode.workspace
    .getConfiguration("headwind")
    .get<boolean>("transformOnSave", false);
}
