# Changelog

All notable changes to the Headwind VS Code extension will be documented in this file.

## [0.2.0] - 2025-02-04

### Added

- **Auto-format after transform**: transformed code is now automatically formatted using the user's configured default formatter (e.g. Prettier, Biome) in preview, apply, and transform-on-save modes.
- **Empty line preservation**: empty lines in JSX/TSX source are no longer stripped during transformation. Uses comment-based placeholders to survive SWC's AST pipeline.
- **Alpha / opacity modifiers**: support `text-white/60`, `bg-blue-500/50` and other color-with-alpha syntaxes across all color modes (hex, oklch, hsl, var).
- **`color-mix()` support**: opt-in `color-mix(in oklab, …)` output for alpha transparency via var-mode colors.
- **CSS variable syntax `-(…)`**: support Tailwind v4 `bg-(--my-color)`, `text-(--my-var)`, `p-(--spacing)` and type-hinted variants like `bg-(image:--my-bg)`.
- **Gradient utilities**: `bg-linear-*`, `bg-radial-*`, `bg-conic-*` with standard angles, arbitrary values, and CSS variable syntaxes.
- **Border width utilities**: `border` (1px), `border-2` (2px), `border-[3px]`, `border-(length:--my-width)`.
- **`space-x-*` / `space-y-*`**: mapped to `column-gap` / `row-gap`.
- **Scroll padding / margin**: full `scroll-p*`, `scroll-m*` family including axis variants (`scroll-px`, `scroll-py`, `scroll-mx`, `scroll-my`).
- **Color plugins**: `accent-*`, `caret-*`, `fill-*`, `stroke-*`, `outline-*`, `decoration-*` with standard colors, arbitrary values, and CSS variables.
- **Shadow / ring system**: `shadow-sm` through `shadow-2xl`, `ring-*`, `inset-shadow-*`, `inset-ring-*` with named sizes, numeric widths, colors, and CSS variables.
- **Extension README** with full feature docs, commands, and settings reference.

### Fixed

- `bg-conic` gradient output no longer wraps CSS variable values in `conic-gradient(…)`.
- Diff preview and apply now open in the main editor column (`ViewColumn.One`), so the Headwind control panel is never obscured.

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
