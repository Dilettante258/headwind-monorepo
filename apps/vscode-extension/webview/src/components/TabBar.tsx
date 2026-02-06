import { Show, For } from 'solid-js';

export type TabId = 'css' | 'map' | 'code' | 'tree';

interface TabBarProps {
  activeTab: () => TabId;
  onTabChange: (tab: TabId) => void;
  mapCount: () => number;
  showTreeTab: () => boolean;
  onCopy: () => void;
  aiLoading: () => boolean;
  aiError: () => string | null;
  aiApplied: () => boolean;
  hasElementTree: () => boolean;
  onAiRename: () => void;
  onResetAi: () => void;
}

const TABS: { id: TabId; label: string }[] = [
  { id: 'css', label: 'Generated CSS' },
  { id: 'map', label: 'Class Map' },
  { id: 'code', label: 'Output Code' },
];

export function TabBar(props: TabBarProps) {
  return (
    <div class="tab-bar">
      <For each={TABS}>
        {(tab) => (
          <button
            class={`tab ${props.activeTab() === tab.id ? 'active' : ''}`}
            onClick={() => props.onTabChange(tab.id)}
          >
            {tab.label}
            <Show when={tab.id === 'map' && props.mapCount() > 0}>
              {` (${props.mapCount()})`}
            </Show>
          </button>
        )}
      </For>
      <Show when={props.showTreeTab()}>
        <button
          class={`tab ${props.activeTab() === 'tree' ? 'active' : ''}`}
          onClick={() => props.onTabChange('tree')}
        >
          Element Tree
        </button>
      </Show>
      <div class="tab-actions">
        <Show when={props.aiError()}>
          <span class="ai-error">{props.aiError()}</span>
        </Show>
        <Show when={!props.aiApplied()}>
          <button
            class="ai-btn"
            onClick={props.onAiRename}
            disabled={!props.hasElementTree() || props.aiLoading()}
          >
            {props.aiLoading() ? 'Renaming...' : 'AI Rename'}
          </button>
        </Show>
        <Show when={props.aiApplied()}>
          <button class="ai-btn ai-btn-reset" onClick={props.onResetAi}>
            Reset AI
          </button>
        </Show>
        <button class="copy-btn" onClick={props.onCopy}>Copy</button>
      </div>
    </div>
  );
}
