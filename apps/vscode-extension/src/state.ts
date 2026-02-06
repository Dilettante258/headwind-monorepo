import { EventEmitter } from "events";
import type * as vscode from "vscode";
import type { TransformOptions, TransformResult, PanelState } from "./types";
import { DEFAULT_OPTIONS, isSupportedFile } from "./types";

class HeadwindState extends EventEmitter {
  options: TransformOptions = { ...DEFAULT_OPTIONS };
  lastResult: TransformResult | null = null;
  activeFilename: string | null = null;
  /** The URI of the last focused supported file — used for transform operations */
  activeFileUri: vscode.Uri | null = null;
  lastDuration = 0;

  /** Monotonically increasing version for virtual document cache invalidation */
  private _version = 0;

  get version(): number {
    return this._version;
  }

  setOptions(opts: TransformOptions): void {
    this.options = opts;
    this.emit("optionsChanged", opts);
  }

  setResult(result: TransformResult, duration: number): void {
    this.lastResult = result;
    this.lastDuration = duration;
    this._version++;
    this.emit("resultChanged", result, duration);
  }

  /** Update the result without emitting events (used for AI rename). */
  updateResultSilent(result: TransformResult): void {
    this.lastResult = result;
    this._version++;
  }

  /**
   * Update the active file. Only updates if the file is a supported type
   * (.jsx/.tsx/.html etc). Non-supported files are ignored so the panel
   * keeps showing the last valid file.
   *
   * @returns true if the active file was updated
   */
  setActiveFile(filename: string | null, uri?: vscode.Uri | null): boolean {
    if (filename && isSupportedFile(filename)) {
      this.activeFilename = filename;
      this.activeFileUri = uri ?? null;
      this.emit("activeFileChanged", filename);
      return true;
    }
    // Non-supported file or null — keep the previous value
    return false;
  }

  toPanelState(): PanelState {
    return {
      options: this.options,
      result: this.lastResult,
      activeFilename: this.activeFilename,
      duration: this.lastDuration,
    };
  }
}

export const state = new HeadwindState();
