import { For } from 'solid-js';

interface ClassMapTableProps {
  entries: () => [string, string][];
}

export function ClassMapTable(props: ClassMapTableProps) {
  return (
    <table class="map-table">
      <thead>
        <tr>
          <th>Tailwind Classes</th>
          <th>Generated</th>
        </tr>
      </thead>
      <tbody>
        <For each={props.entries()}>
          {([orig, gen]) => (
            <tr>
              <td>{orig}</td>
              <td>{gen}</td>
            </tr>
          )}
        </For>
      </tbody>
    </table>
  );
}
