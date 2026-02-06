import { createSignal, createMemo } from 'solid-js';
import { useVSCodeBridge } from './hooks/useVSCodeBridge';
import { Header } from './components/Header';
import { OptionsPanel } from './components/OptionsPanel';
import { ActionBar } from './components/ActionBar';
import { InfoBar } from './components/InfoBar';
import { TabBar, type TabId } from './components/TabBar';
import { OutputArea } from './components/OutputArea';

export function App() {
  const bridge = useVSCodeBridge();
  const [activeTab, setActiveTab] = createSignal<TabId>('css');
  const [showTree, setShowTree] = createSignal(false);

  const hasResult = createMemo(() => bridge.result() !== null);
  const hasElementTree = createMemo(() => !!bridge.result()?.elementTree);
  const mapCount = createMemo(() => {
    const r = bridge.result();
    return r ? Object.keys(r.classMap).length : 0;
  });

  function handleCopy() {
    const r = bridge.result();
    if (!r) return;
    const tab = activeTab();
    let text = '';
    if (tab === 'css') text = r.css;
    else if (tab === 'code') text = r.code;
    else if (tab === 'tree') text = r.elementTree ?? '';
    else {
      text = Object.entries(r.classMap)
        .map(([k, v]) => `${k} -> ${v}`)
        .join('\n');
    }
    bridge.copyToClipboard(text);
  }

  return (
    <div class="panel-container">
      <Header wasmReady={bridge.wasmReady} />
      <OptionsPanel
        options={bridge.options}
        onOptionsChanged={bridge.sendOptions}
        onElementTreeChanged={setShowTree}
      />
      <ActionBar
        wasmReady={bridge.wasmReady}
        hasResult={hasResult}
        onTransform={bridge.requestTransform}
        onPreviewDiff={bridge.requestPreviewDiff}
        onApply={bridge.requestApply}
      />
      <InfoBar
        activeFilename={bridge.activeFilename}
        duration={bridge.duration}
      />
      <TabBar
        activeTab={activeTab}
        onTabChange={setActiveTab}
        mapCount={mapCount}
        showTreeTab={showTree}
        onCopy={handleCopy}
        aiLoading={bridge.aiLoading}
        aiError={bridge.aiError}
        aiApplied={bridge.aiApplied}
        hasElementTree={hasElementTree}
        onAiRename={bridge.requestAiRename}
        onResetAi={bridge.resetAiRename}
      />
      <OutputArea
        activeTab={activeTab}
        result={bridge.result}
        error={bridge.error}
      />
    </div>
  );
}
