import { Show } from 'solid-js';

interface HeaderProps {
  wasmReady: () => boolean;
}

export function Header(props: HeaderProps) {
  return (
    <div class="header">
      <span class="header-title">Headwind</span>
      <Show when={props.wasmReady()} fallback={<span class="status status-loading">Loading...</span>}>
        <span class="status status-ok">Ready</span>
      </Show>
    </div>
  );
}
