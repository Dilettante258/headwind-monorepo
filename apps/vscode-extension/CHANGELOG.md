# Changelog

All notable changes to the Headwind VS Code extension will be documented in this file.

## [0.1.0] - 2025-02-02

### Added

- Initial release of Headwind for VS Code.
- Transform atomic CSS utility classes to semantic CSS directly in the editor.
- Control Panel webview with live options, CSS preview, class map table, and action buttons.
- Side-by-side diff preview using VS Code's built-in diff editor.
- One-click apply: replaces source code and writes companion CSS file.
- Transform on save (opt-in via `headwind.transformOnSave`).
- Configurable naming modes: hash, readable, camelCase.
- CSS Modules output with dot or bracket notation.
- CSS variable mode and inline mode.
- Color output formats: hex, oklch, hsl, and CSS custom properties.
- Unknown class handling: preserve or remove.
- Smart file tracking across editor tab switches.
- Theme-aware UI (light, dark, high-contrast).
- Powered by Rust + WASM for near-instant transforms.
