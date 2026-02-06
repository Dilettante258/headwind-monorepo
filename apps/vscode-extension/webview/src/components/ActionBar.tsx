interface ActionBarProps {
  wasmReady: () => boolean;
  hasResult: () => boolean;
  onTransform: () => void;
  onPreviewDiff: () => void;
  onApply: () => void;
}

export function ActionBar(props: ActionBarProps) {
  return (
    <div class="actions">
      <button
        class="btn btn-primary"
        disabled={!props.wasmReady()}
        onClick={props.onTransform}
      >
        Transform
      </button>
      <button
        class="btn btn-secondary"
        disabled={!props.wasmReady()}
        onClick={props.onPreviewDiff}
      >
        Preview Diff
      </button>
      <button
        class="btn btn-secondary"
        disabled={!props.wasmReady() || !props.hasResult()}
        onClick={props.onApply}
      >
        Apply
      </button>
    </div>
  );
}
