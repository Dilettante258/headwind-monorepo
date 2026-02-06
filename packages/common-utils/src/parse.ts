/** 解析 element tree 文本，提取 ref → tailwind classes 映射 */
export function parseTreeRefs(tree: string): Map<string, string> {
  const map = new Map<string, string>();
  for (const line of tree.split('\n')) {
    const m = line.match(/^\s*-\s+(\S+)\s+(.*?)\s*\[ref=(e\d+)\]/);
    if (!m) continue;
    const [, _tag, middle, ref] = m;
    const classes = middle!.replace(/"[^"]*"/, '').replace(/^:.*/, '').trim();
    if (classes) map.set(ref!, classes);
  }
  return map;
}
