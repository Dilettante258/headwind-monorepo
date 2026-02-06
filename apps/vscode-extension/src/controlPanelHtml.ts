import type * as vscode from "vscode";

export function getWebviewHtml(webview: vscode.Webview): string {
  const nonce = getNonce();

  return /*html*/ `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta http-equiv="Content-Security-Policy"
    content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';" />
  <title>Headwind</title>
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      font-family: var(--vscode-font-family);
      font-size: var(--vscode-font-size);
      color: var(--vscode-foreground);
      background: var(--vscode-editor-background);
      overflow-x: hidden;
    }

    .panel-container {
      display: flex;
      flex-direction: column;
      height: 100vh;
      overflow: hidden;
    }

    /* Header */
    .header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 16px;
      background: var(--vscode-sideBarSectionHeader-background);
      border-bottom: 1px solid var(--vscode-panel-border);
      flex-shrink: 0;
    }
    .header-title {
      font-weight: 600;
      font-size: 13px;
      color: var(--vscode-sideBarSectionHeader-foreground);
    }
    .status {
      font-size: 11px;
      padding: 2px 8px;
      border-radius: 10px;
    }
    .status-ok { background: var(--vscode-testing-iconPassed); color: var(--vscode-editor-background); }
    .status-loading { background: var(--vscode-editorWarning-foreground); color: var(--vscode-editor-background); }

    /* Options */
    .section {
      padding: 12px 16px;
      border-bottom: 1px solid var(--vscode-panel-border);
      flex-shrink: 0;
    }
    .section-title {
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--vscode-descriptionForeground);
      margin-bottom: 8px;
    }
    .options-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px;
    }
    .option-group {
      display: flex;
      flex-direction: column;
      gap: 3px;
    }
    .option-label {
      font-size: 11px;
      color: var(--vscode-descriptionForeground);
    }
    .option-select {
      font-family: var(--vscode-font-family);
      font-size: 12px;
      padding: 4px 8px;
      border-radius: 3px;
      border: 1px solid var(--vscode-input-border);
      background: var(--vscode-input-background);
      color: var(--vscode-input-foreground);
      outline: none;
      cursor: pointer;
      width: 100%;
    }
    .option-select:focus { border-color: var(--vscode-focusBorder); }

    /* Actions */
    .actions {
      display: flex;
      gap: 8px;
      padding: 8px 16px;
      border-bottom: 1px solid var(--vscode-panel-border);
      flex-shrink: 0;
    }
    .btn {
      font-family: var(--vscode-font-family);
      font-size: 12px;
      padding: 5px 14px;
      border-radius: 3px;
      border: none;
      cursor: pointer;
      flex: 1;
      text-align: center;
    }
    .btn-primary {
      background: var(--vscode-button-background);
      color: var(--vscode-button-foreground);
    }
    .btn-primary:hover { background: var(--vscode-button-hoverBackground); }
    .btn-secondary {
      background: var(--vscode-button-secondaryBackground);
      color: var(--vscode-button-secondaryForeground);
    }
    .btn-secondary:hover { background: var(--vscode-button-secondaryHoverBackground); }
    .btn:disabled { opacity: 0.5; cursor: default; }

    /* Info bar */
    .info-bar {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 4px 16px;
      font-size: 11px;
      color: var(--vscode-descriptionForeground);
      background: var(--vscode-sideBarSectionHeader-background);
      border-bottom: 1px solid var(--vscode-panel-border);
      flex-shrink: 0;
    }
    .info-bar .file-name {
      font-family: var(--vscode-editor-font-family);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .info-bar .duration {
      font-family: var(--vscode-editor-font-family);
    }

    /* Tabs */
    .tab-bar {
      display: flex;
      background: var(--vscode-editorGroupHeader-tabsBackground);
      border-bottom: 1px solid var(--vscode-panel-border);
      flex-shrink: 0;
    }
    .tab {
      font-family: var(--vscode-font-family);
      font-size: 12px;
      padding: 6px 14px;
      border: none;
      background: none;
      color: var(--vscode-tab-inactiveForeground);
      cursor: pointer;
      border-bottom: 2px solid transparent;
      transition: color 0.1s, border-color 0.1s;
    }
    .tab:hover { color: var(--vscode-tab-activeForeground); }
    .tab.active {
      color: var(--vscode-tab-activeForeground);
      border-bottom-color: var(--vscode-focusBorder);
    }
    .tab-actions {
      margin-left: auto;
      display: flex;
      align-items: center;
      padding-right: 8px;
    }
    .copy-btn {
      font-family: var(--vscode-font-family);
      font-size: 11px;
      padding: 2px 10px;
      border-radius: 3px;
      border: 1px solid var(--vscode-input-border);
      background: var(--vscode-input-background);
      color: var(--vscode-descriptionForeground);
      cursor: pointer;
    }
    .copy-btn:hover {
      color: var(--vscode-foreground);
      border-color: var(--vscode-focusBorder);
    }

    /* Output */
    .output-area {
      flex: 1;
      overflow: auto;
      background: var(--vscode-editor-background);
    }
    .output-pre {
      font-family: var(--vscode-editor-font-family);
      font-size: var(--vscode-editor-font-size);
      line-height: 1.5;
      padding: 12px 16px;
      margin: 0;
      white-space: pre;
      color: var(--vscode-editor-foreground);
    }
    .output-css { color: var(--vscode-debugTokenExpression-string); }
    .output-error { color: var(--vscode-errorForeground); white-space: pre-wrap; }

    /* Class Map table */
    .map-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 12px;
    }
    .map-table th {
      text-align: left;
      padding: 6px 16px;
      font-weight: 600;
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--vscode-descriptionForeground);
      background: var(--vscode-sideBarSectionHeader-background);
      border-bottom: 1px solid var(--vscode-panel-border);
      position: sticky;
      top: 0;
      z-index: 1;
    }
    .map-table td {
      padding: 4px 16px;
      border-bottom: 1px solid var(--vscode-panel-border);
      font-family: var(--vscode-editor-font-family);
      font-size: var(--vscode-editor-font-size);
    }
    .map-table tr:hover td { background: var(--vscode-list-hoverBackground); }

    .placeholder {
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100%;
      color: var(--vscode-descriptionForeground);
      font-style: italic;
      padding: 20px;
      text-align: center;
    }

    ::-webkit-scrollbar { width: 10px; height: 10px; }
    ::-webkit-scrollbar-track { background: transparent; }
    ::-webkit-scrollbar-thumb {
      background: var(--vscode-scrollbarSlider-background);
      border-radius: 5px;
    }
    ::-webkit-scrollbar-thumb:hover { background: var(--vscode-scrollbarSlider-hoverBackground); }
    ::-webkit-scrollbar-thumb:active { background: var(--vscode-scrollbarSlider-activeBackground); }
  </style>
</head>
<body>
  <div class="panel-container">
    <div class="header">
      <span class="header-title">Headwind</span>
      <span id="wasm-status" class="status status-loading">Loading...</span>
    </div>

    <div class="section">
      <div class="section-title">Transform Options</div>
      <div class="options-grid">
        <div class="option-group">
          <label class="option-label">Naming Mode</label>
          <select id="opt-naming" class="option-select">
            <option value="hash">Hash (c_a1b2c3)</option>
            <option value="readable">Readable (p4_m2)</option>
            <option value="camelCase">CamelCase (p4M2)</option>
          </select>
        </div>
        <div class="option-group">
          <label class="option-label">Output Mode</label>
          <select id="opt-output" class="option-select">
            <option value="global">Global</option>
            <option value="cssModules">CSS Modules</option>
          </select>
        </div>
        <div class="option-group">
          <label class="option-label">CSS Values</label>
          <select id="opt-cssVars" class="option-select">
            <option value="var">Variables (var(--x))</option>
            <option value="inline">Inline (1.875rem)</option>
          </select>
        </div>
        <div class="option-group">
          <label class="option-label">Unknown Classes</label>
          <select id="opt-unknown" class="option-select">
            <option value="preserve">Preserve</option>
            <option value="remove">Remove</option>
          </select>
        </div>
        <div class="option-group">
          <label class="option-label">Color Format</label>
          <select id="opt-color" class="option-select">
            <option value="hex">Hex (#3b82f6)</option>
            <option value="oklch">OKLCH</option>
            <option value="hsl">HSL</option>
            <option value="var">CSS Var</option>
          </select>
        </div>
        <div class="option-group" id="access-group" style="display:none;">
          <label class="option-label">CSS Modules Access</label>
          <select id="opt-access" class="option-select">
            <option value="dot">Dot (styles.xxx)</option>
            <option value="bracket">Bracket (styles["xxx"])</option>
          </select>
        </div>
        <div class="option-group" style="justify-content:center;">
          <label class="option-label" style="display:flex;align-items:center;gap:6px;cursor:pointer;">
            <input type="checkbox" id="opt-elementTree" />
            Element Tree
          </label>
        </div>
      </div>
    </div>

    <div class="actions">
      <button id="btn-transform" class="btn btn-primary" disabled>Transform</button>
      <button id="btn-preview" class="btn btn-secondary" disabled>Preview Diff</button>
      <button id="btn-apply" class="btn btn-secondary" disabled>Apply</button>
    </div>

    <div class="info-bar">
      <span id="file-name" class="file-name">No file</span>
      <span id="duration" class="duration"></span>
    </div>

    <div class="tab-bar">
      <button class="tab active" data-tab="css">Generated CSS</button>
      <button class="tab" data-tab="map">Class Map <span id="map-count"></span></button>
      <button class="tab" data-tab="code">Output Code</button>
      <button class="tab" data-tab="tree" id="tab-tree" style="display:none;">Element Tree</button>
      <div class="tab-actions">
        <button id="btn-copy" class="copy-btn">Copy</button>
      </div>
    </div>

    <div class="output-area" id="output-area">
      <div class="placeholder">Click "Transform" to see results</div>
    </div>
  </div>

  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();

    let currentTab = 'css';
    let result = null;
    let wasmReady = false;

    const $naming   = document.getElementById('opt-naming');
    const $output   = document.getElementById('opt-output');
    const $cssVars  = document.getElementById('opt-cssVars');
    const $unknown  = document.getElementById('opt-unknown');
    const $access   = document.getElementById('opt-access');
    const $colorMode = document.getElementById('opt-color');
    const $accessGr = document.getElementById('access-group');
    const $btnTrans = document.getElementById('btn-transform');
    const $btnPrev  = document.getElementById('btn-preview');
    const $btnApply = document.getElementById('btn-apply');
    const $btnCopy  = document.getElementById('btn-copy');
    const $fileName = document.getElementById('file-name');
    const $duration = document.getElementById('duration');
    const $mapCount = document.getElementById('map-count');
    const $outArea  = document.getElementById('output-area');
    const $wasmStat = document.getElementById('wasm-status');
    const $elemTree = document.getElementById('opt-elementTree');
    const $tabTree  = document.getElementById('tab-tree');
    const $tabs     = document.querySelectorAll('.tab[data-tab]');

    function gatherOptions() {
      const outputType = $output.value;
      return {
        namingMode: $naming.value,
        outputMode: outputType === 'global'
          ? { type: 'global' }
          : { type: 'cssModules', access: $access.value },
        cssVariables: $cssVars.value,
        unknownClasses: $unknown.value,
        colorMode: $colorMode.value,
        elementTree: $elemTree.checked,
      };
    }

    function sendOptions() {
      vscode.postMessage({ type: 'optionsChanged', options: gatherOptions() });
    }

    function updateAccessVisibility() {
      $accessGr.style.display = $output.value === 'cssModules' ? '' : 'none';
    }

    [$naming, $output, $cssVars, $unknown, $colorMode, $access].forEach(function(el) {
      el.addEventListener('change', function() {
        updateAccessVisibility();
        sendOptions();
      });
    });

    $elemTree.addEventListener('change', function() {
      $tabTree.style.display = $elemTree.checked ? '' : 'none';
      sendOptions();
    });

    $btnTrans.addEventListener('click', function() {
      vscode.postMessage({ type: 'requestTransform' });
    });
    $btnPrev.addEventListener('click', function() {
      vscode.postMessage({ type: 'requestPreviewDiff' });
    });
    $btnApply.addEventListener('click', function() {
      vscode.postMessage({ type: 'requestApply' });
    });
    $btnCopy.addEventListener('click', function() {
      if (!result) return;
      var text = '';
      if (currentTab === 'css') text = result.css;
      else if (currentTab === 'code') text = result.code;
      else if (currentTab === 'tree') text = result.elementTree || '';
      else {
        text = Object.entries(result.classMap)
          .map(function(e) { return e[0] + ' -> ' + e[1]; })
          .join('\\n');
      }
      vscode.postMessage({ type: 'copyToClipboard', text: text });
    });

    $tabs.forEach(function(tab) {
      tab.addEventListener('click', function() {
        $tabs.forEach(function(t) { t.classList.remove('active'); });
        tab.classList.add('active');
        currentTab = tab.dataset.tab;
        renderOutput();
      });
    });

    function escapeHtml(s) {
      return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }

    function renderOutput() {
      if (!result) {
        $outArea.innerHTML = '<div class="placeholder">Click "Transform" to see results</div>';
        return;
      }
      if (currentTab === 'css') {
        $outArea.innerHTML = '<pre class="output-pre output-css"></pre>';
        $outArea.querySelector('pre').textContent = result.css || '/* No CSS generated */';
      } else if (currentTab === 'code') {
        $outArea.innerHTML = '<pre class="output-pre"></pre>';
        $outArea.querySelector('pre').textContent = result.code;
      } else if (currentTab === 'tree') {
        if (!result.elementTree) {
          $outArea.innerHTML = '<div class="placeholder">No element tree (enable Element Tree option)</div>';
          return;
        }
        $outArea.innerHTML = '<pre class="output-pre"></pre>';
        $outArea.querySelector('pre').textContent = result.elementTree;
      } else if (currentTab === 'map') {
        var entries = Object.entries(result.classMap);
        if (entries.length === 0) {
          $outArea.innerHTML = '<div class="placeholder">No class mappings</div>';
          return;
        }
        var html = '<table class="map-table"><thead><tr><th>Tailwind Classes</th><th>Generated</th></tr></thead><tbody>';
        for (var i = 0; i < entries.length; i++) {
          html += '<tr><td>' + escapeHtml(entries[i][0]) + '</td><td>' + escapeHtml(entries[i][1]) + '</td></tr>';
        }
        html += '</tbody></table>';
        $outArea.innerHTML = html;
      }
    }

    function renderError(msg) {
      $outArea.innerHTML = '<pre class="output-pre output-error"></pre>';
      $outArea.querySelector('pre').textContent = msg;
    }

    function updateButtons() {
      $btnTrans.disabled = !wasmReady;
      $btnPrev.disabled = !wasmReady;
      $btnApply.disabled = !wasmReady || !result;
    }

    function setOptionsFromState(opts) {
      $naming.value = opts.namingMode || 'hash';
      var outType = (opts.outputMode && opts.outputMode.type) || 'global';
      $output.value = outType;
      $cssVars.value = opts.cssVariables || 'var';
      $unknown.value = opts.unknownClasses || 'preserve';
      $colorMode.value = opts.colorMode || 'hex';
      $elemTree.checked = !!opts.elementTree;
      $tabTree.style.display = $elemTree.checked ? '' : 'none';
      if (outType === 'cssModules' && opts.outputMode.access) {
        $access.value = opts.outputMode.access;
      }
      updateAccessVisibility();
    }

    window.addEventListener('message', function(event) {
      var msg = event.data;
      switch (msg.type) {
        case 'init':
          wasmReady = true;
          setOptionsFromState(msg.state.options);
          if (msg.state.result) {
            result = msg.state.result;
            $mapCount.textContent = '(' + Object.keys(result.classMap).length + ')';
          }
          if (msg.state.activeFilename) {
            var parts = msg.state.activeFilename.split('/');
            $fileName.textContent = parts[parts.length - 1] || 'No file';
          }
          if (msg.state.duration > 0) {
            $duration.textContent = msg.state.duration.toFixed(1) + 'ms';
          }
          $wasmStat.className = 'status status-ok';
          $wasmStat.textContent = 'Ready';
          updateButtons();
          renderOutput();
          break;

        case 'transformResult':
          result = msg.result;
          $duration.textContent = msg.duration.toFixed(1) + 'ms';
          $mapCount.textContent = '(' + Object.keys(result.classMap).length + ')';
          updateButtons();
          renderOutput();
          break;

        case 'transformError':
          renderError(msg.error);
          break;

        case 'activeFileChanged':
          if (msg.filename) {
            var fparts = msg.filename.split('/');
            $fileName.textContent = fparts[fparts.length - 1];
          } else {
            $fileName.textContent = 'No file';
          }
          break;

        case 'optionsUpdated':
          setOptionsFromState(msg.options);
          break;
      }
    });

    vscode.postMessage({ type: 'ready' });
  </script>
</body>
</html>`;
}

function getNonce(): string {
  let text = "";
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  for (let i = 0; i < 32; i++) {
    text += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return text;
}
