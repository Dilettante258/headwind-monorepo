import { Show, createMemo } from 'solid-js';
import type { TransformResult } from '@ext/types';
import type { TabId } from './TabBar';
import { ClassMapTable } from './ClassMapTable';

interface OutputAreaProps {
  activeTab: () => TabId;
  result: () => TransformResult | null;
  error: () => string | null;
}

export function OutputArea(props: OutputAreaProps) {
  const classMapEntries = createMemo(() => {
    const r = props.result();
    if (!r) return [];
    return Object.entries(r.classMap);
  });

  return (
    <div class="output-area">
      <Show when={props.error()}>
        <pre class="output-pre output-error">{props.error()}</pre>
      </Show>

      <Show when={!props.error()}>
        <Show when={!props.result()}>
          <div class="placeholder">Click "Transform" to see results</div>
        </Show>

        <Show when={props.result() && props.activeTab() === 'css'}>
          <pre class="output-pre output-css">
            {props.result()!.css || '/* No CSS generated */'}
          </pre>
        </Show>

        <Show when={props.result() && props.activeTab() === 'code'}>
          <pre class="output-pre">{props.result()!.code}</pre>
        </Show>

        <Show when={props.result() && props.activeTab() === 'map'}>
          <Show
            when={classMapEntries().length > 0}
            fallback={<div class="placeholder">No class mappings</div>}
          >
            <ClassMapTable entries={classMapEntries} />
          </Show>
        </Show>

        <Show when={props.result() && props.activeTab() === 'tree'}>
          <Show
            when={props.result()!.elementTree}
            fallback={<div class="placeholder">No element tree (enable Element Tree option)</div>}
          >
            <pre class="output-pre">{props.result()!.elementTree}</pre>
          </Show>
        </Show>
      </Show>
    </div>
  );
}
