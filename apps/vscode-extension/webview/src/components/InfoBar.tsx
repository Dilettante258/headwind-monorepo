import { createMemo } from 'solid-js';

interface InfoBarProps {
  activeFilename: () => string | null;
  duration: () => number;
}

export function InfoBar(props: InfoBarProps) {
  const displayName = createMemo(() => {
    const name = props.activeFilename();
    if (!name) return 'No file';
    const parts = name.split('/');
    return parts[parts.length - 1] ?? 'No file';
  });

  return (
    <div class="info-bar">
      <span class="file-name">{displayName()}</span>
      <span class="duration">
        {props.duration() > 0 ? `${props.duration().toFixed(1)}ms` : ''}
      </span>
    </div>
  );
}
