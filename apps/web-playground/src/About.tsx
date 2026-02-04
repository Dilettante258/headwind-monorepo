import type { Component } from 'solid-js';
import { navigate } from './App';

const About: Component = () => {
  const goHome = (e: MouseEvent) => { e.preventDefault(); navigate('/'); };

  return (
    <div class="about">
      <header class="header">
        <div class="header-left">
          <a href="/" class="logo" onClick={goHome}>Headwind</a>
          <span class="badge">About</span>
        </div>
        <div class="header-right">
          <a href="/" class="nav-link" onClick={goHome}>Playground</a>
        </div>
      </header>
      <div class="about-content">
        <section class="about-hero">
          <h2 class="about-title">Headwind</h2>
          <p class="about-subtitle">
            Atomic CSS to Semantic CSS Compiler
          </p>
          <p class="about-desc">
            Headwind is an open-source tool that transforms Tailwind CSS utility classes into
            optimized, semantic CSS. Powered by a high-performance Rust core compiled to
            WebAssembly, it brings near-native speed to CSS transformation — right in your
            browser or your editor.
          </p>
        </section>

        <section class="about-section">
          <h3 class="about-heading">How It Works</h3>
          <p>
            Write your components with familiar Tailwind utility classes. Headwind parses
            your JSX, TSX, or HTML, identifies every utility class, and replaces them with
            short, generated class names backed by optimized CSS rules. The result is smaller
            HTML, deduplicated styles, and a clean separation between markup and styling.
          </p>
          <div class="about-flow">
            <div class="flow-step">
              <span class="flow-num">1</span>
              <div>
                <strong>Parse</strong>
                <p>Scans your source code and extracts Tailwind class strings from JSX/HTML attributes.</p>
              </div>
            </div>
            <div class="flow-step">
              <span class="flow-num">2</span>
              <div>
                <strong>Resolve</strong>
                <p>Maps each utility class to its CSS declarations, handling variants, modifiers, and responsive breakpoints.</p>
              </div>
            </div>
            <div class="flow-step">
              <span class="flow-num">3</span>
              <div>
                <strong>Bundle</strong>
                <p>Deduplicates identical rule sets, merges shorthands, and generates a minimal CSS stylesheet.</p>
              </div>
            </div>
            <div class="flow-step">
              <span class="flow-num">4</span>
              <div>
                <strong>Output</strong>
                <p>Rewrites your source with semantic class names and produces a companion CSS file.</p>
              </div>
            </div>
          </div>
        </section>

        <section class="about-section">
          <h3 class="about-heading">Features</h3>
          <div class="feature-grid">
            <div class="feature-card">
              <div class="feature-icon">Rust + WASM</div>
              <p>Core engine written in Rust and compiled to WebAssembly for near-native performance in the browser.</p>
            </div>
            <div class="feature-card">
              <div class="feature-icon">JSX / TSX / HTML</div>
              <p>Supports React JSX, TypeScript TSX, and plain HTML with full className and class attribute parsing.</p>
            </div>
            <div class="feature-card">
              <div class="feature-icon">CSS Modules</div>
              <p>Output global CSS or CSS Modules with dot or bracket notation — ready for your build pipeline.</p>
            </div>
            <div class="feature-card">
              <div class="feature-icon">Tailwind v4</div>
              <p>Full support for Tailwind CSS v4 utilities, responsive breakpoints, pseudo-classes, and variants.</p>
            </div>
            <div class="feature-card">
              <div class="feature-icon">Naming Modes</div>
              <p>Choose hash-based, readable, or camelCase class names to match your project conventions.</p>
            </div>
            <div class="feature-card">
              <div class="feature-icon">Color Formats</div>
              <p>Output colors in Hex, OKLCH, HSL, or CSS custom properties — your choice.</p>
            </div>
          </div>
        </section>

        <section class="about-section">
          <h3 class="about-heading">VSCode Extension</h3>
          <p>
            Headwind also ships as a VSCode extension. Transform files on save, preview
            diffs before applying, and configure naming, output mode, and color format
            directly from your editor settings. Install it from the
            {' '}<a href="https://marketplace.visualstudio.com/items?itemName=Dilettante258.headwind-vscode" target="_blank" rel="noopener noreferrer">Visual Studio Marketplace</a>.
          </p>
        </section>

        <section class="about-section">
          <h3 class="about-heading">Open Source</h3>
          <p>
            Headwind is MIT-licensed and open source. Contributions, issues, and ideas are welcome.
          </p>
          <div class="about-links">
            <a href="https://github.com/Dilettante258/headwind-monorepo" target="_blank" rel="noopener noreferrer" class="about-link">
              GitHub Repository
            </a>
            <a href="https://github.com/Dilettante258/headwind-monorepo/issues" target="_blank" rel="noopener noreferrer" class="about-link">
              Report an Issue
            </a>
          </div>
        </section>

        <footer class="about-footer">
          <p>Built with Rust, Solid.js, and a love for clean CSS.</p>
        </footer>
      </div>
    </div>
  );
};

export default About;
