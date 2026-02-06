import { createSignal, createEffect, Show } from 'solid-js';
import type { TransformOptions } from '@ext/types';

interface OptionsPanelProps {
  options: () => TransformOptions | null;
  onOptionsChanged: (opts: TransformOptions) => void;
  onElementTreeChanged: (enabled: boolean) => void;
}

export function OptionsPanel(props: OptionsPanelProps) {
  const [naming, setNaming] = createSignal('hash');
  const [outputType, setOutputType] = createSignal('global');
  const [cssVars, setCssVars] = createSignal('var');
  const [unknown, setUnknown] = createSignal('preserve');
  const [colorMode, setColorMode] = createSignal('hex');
  const [access, setAccess] = createSignal('dot');
  const [elementTree, setElementTree] = createSignal(false);

  // Sync from host state
  createEffect(() => {
    const opts = props.options();
    if (!opts) return;
    setNaming(opts.namingMode);
    setOutputType(opts.outputMode.type);
    setCssVars(opts.cssVariables);
    setUnknown(opts.unknownClasses);
    setColorMode(opts.colorMode);
    setElementTree(!!opts.elementTree);
    if (opts.outputMode.type === 'cssModules' && opts.outputMode.access) {
      setAccess(opts.outputMode.access);
    }
  });

  function gather(): TransformOptions {
    return {
      namingMode: naming() as TransformOptions['namingMode'],
      outputMode:
        outputType() === 'global'
          ? { type: 'global' }
          : { type: 'cssModules', access: access() as 'dot' | 'bracket' },
      cssVariables: cssVars() as TransformOptions['cssVariables'],
      unknownClasses: unknown() as TransformOptions['unknownClasses'],
      colorMode: colorMode() as TransformOptions['colorMode'],
      elementTree: elementTree(),
    };
  }

  function handleChange() {
    props.onOptionsChanged(gather());
  }

  function handleTreeChange(checked: boolean) {
    setElementTree(checked);
    props.onElementTreeChanged(checked);
    props.onOptionsChanged(gather());
  }

  return (
    <div class="section">
      <div class="section-title">Transform Options</div>
      <div class="options-grid">
        <div class="option-group">
          <label class="option-label">Naming Mode</label>
          <select class="option-select" value={naming()} onChange={(e) => { setNaming(e.currentTarget.value); handleChange(); }}>
            <option value="hash">Hash (c_a1b2c3)</option>
            <option value="readable">Readable (p4_m2)</option>
            <option value="camelCase">CamelCase (p4M2)</option>
          </select>
        </div>
        <div class="option-group">
          <label class="option-label">Output Mode</label>
          <select class="option-select" value={outputType()} onChange={(e) => { setOutputType(e.currentTarget.value); handleChange(); }}>
            <option value="global">Global</option>
            <option value="cssModules">CSS Modules</option>
          </select>
        </div>
        <div class="option-group">
          <label class="option-label">CSS Values</label>
          <select class="option-select" value={cssVars()} onChange={(e) => { setCssVars(e.currentTarget.value); handleChange(); }}>
            <option value="var">Variables (var(--x))</option>
            <option value="inline">Inline (1.875rem)</option>
          </select>
        </div>
        <div class="option-group">
          <label class="option-label">Unknown Classes</label>
          <select class="option-select" value={unknown()} onChange={(e) => { setUnknown(e.currentTarget.value); handleChange(); }}>
            <option value="preserve">Preserve</option>
            <option value="remove">Remove</option>
          </select>
        </div>
        <div class="option-group">
          <label class="option-label">Color Format</label>
          <select class="option-select" value={colorMode()} onChange={(e) => { setColorMode(e.currentTarget.value); handleChange(); }}>
            <option value="hex">Hex (#3b82f6)</option>
            <option value="oklch">OKLCH</option>
            <option value="hsl">HSL</option>
            <option value="var">CSS Var</option>
          </select>
        </div>
        <Show when={outputType() === 'cssModules'}>
          <div class="option-group">
            <label class="option-label">CSS Modules Access</label>
            <select class="option-select" value={access()} onChange={(e) => { setAccess(e.currentTarget.value); handleChange(); }}>
              <option value="dot">Dot (styles.xxx)</option>
              <option value="bracket">Bracket (styles["xxx"])</option>
            </select>
          </div>
        </Show>
        <div class="option-group" style={{ "justify-content": "center" }}>
          <label class="option-checkbox">
            <input
              type="checkbox"
              checked={elementTree()}
              onChange={(e) => handleTreeChange(e.currentTarget.checked)}
            />
            Element Tree
          </label>
        </div>
      </div>
    </div>
  );
}
