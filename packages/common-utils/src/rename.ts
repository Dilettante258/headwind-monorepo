import { parseTreeRefs } from './parse.ts';
import type { TransformResult } from './types.ts';

/** 应用 AI 命名结果，替换 code/css 中的生成类名 */
export function applyAiNames(
  result: TransformResult,
  aiNames: Record<string, string>,
): TransformResult {
  const refClasses = parseTreeRefs(result.elementTree!);
  let { code, css } = result;
  const newClassMap: Record<string, string> = {};

  // 构建 tailwind → semantic 查找表（第一个 ref 的名称优先）
  const tailwindToSemantic = new Map<string, string>();
  for (const [ref, semantic] of Object.entries(aiNames)) {
    const tailwind = refClasses.get(ref);
    if (tailwind && !tailwindToSemantic.has(tailwind)) {
      tailwindToSemantic.set(tailwind, semantic);
    }
  }

  // 遍历 classMap，替换生成名
  for (const [tailwind, generated] of Object.entries(result.classMap)) {
    const semantic = tailwindToSemantic.get(tailwind) ?? generated;
    newClassMap[tailwind] = semantic;
    if (semantic !== generated) {
      const escaped = generated.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      code = code.replace(new RegExp(`\\b${escaped}\\b`, 'g'), semantic);
      css = css.replace(new RegExp(`\\b${escaped}\\b`, 'g'), semantic);
    }
  }

  return { code, css, classMap: newClassMap, elementTree: result.elementTree };
}
