#!/usr/bin/env bun

/**
 * 从 Tailwind CSS 官网文档中提取 class → CSS 映射
 *
 * 使用方法：
 * 1. bun run setup  （首次运行，clone tailwindcss.com）
 * 2. bun run extract
 */

import { readdir, readFile, writeFile, mkdir } from 'fs/promises';
import { join, resolve } from 'path';
import { existsSync } from 'fs';

interface Mapping {
  class: string;
  css: string;
  source?: string; // 来源文件（用于调试）
}

const REPO_PATH = resolve(import.meta.dir, '../data/tailwindcss.com');
const DOCS_PATH = join(REPO_PATH, 'src/docs');
const OUTPUT_PATH = resolve(import.meta.dir, '../../crates/tw_index/fixtures/official-mappings.json');

async function findMdxFiles(dir: string): Promise<string[]> {
  const files: string[] = [];

  try {
    const entries = await readdir(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = join(dir, entry.name);

      if (entry.isDirectory()) {
        const subFiles = await findMdxFiles(fullPath);
        files.push(...subFiles);
      } else if (entry.name.endsWith('.mdx')) {
        files.push(fullPath);
      }
    }
  } catch (error) {
    // 忽略无法访问的目录
  }

  return files;
}

function extractApiTables(content: string, filePath: string): Mapping[] {
  const mappings: Mapping[] = [];

  // 匹配 <ApiTable rows={[...]} /> 或 <ApiTable rows={[...]}> ... </ApiTable>
  // 这个正则需要能处理多行和嵌套的情况
  const apiTableRegex = /<ApiTable\s+rows=\{(\[[\s\S]*?\])\}\s*(?:\/?>|>[\s\S]*?<\/ApiTable>)/g;

  let match;
  while ((match = apiTableRegex.exec(content)) !== null) {
    try {
      const rowsStr = match[1];

      // 尝试解析数组
      // 注意：这里使用 eval 是因为 MDX 中的数据是 JavaScript 表达式
      // 仅在受信任的本地数据上使用
      const rows = eval(rowsStr) as [string, string][];

      for (const [className, css] of rows) {
        if (!className || !css) continue;

        // 跳过包含占位符的条目（如 perspective-origin-[]）
        if (className.includes('[]') || className.includes('<')) continue;

        // 清理 CSS（移除多余的空格和分号）
        const cleanCss = css.trim().replace(/;\s*$/, '');

        mappings.push({
          class: className,
          css: cleanCss,
          source: filePath.replace(REPO_PATH, ''),
        });
      }
    } catch (error) {
      console.warn(`⚠️  Failed to parse ApiTable in ${filePath}:`, (error as Error).message);
    }
  }

  return mappings;
}

async function main() {
  console.log('🔍 Extracting Tailwind CSS mappings from official docs...\n');

  // 检查仓库是否存在
  if (!existsSync(REPO_PATH)) {
    console.error('❌ tailwindcss.com repository not found!');
    console.error('   Please run: bun run setup');
    process.exit(1);
  }

  if (!existsSync(DOCS_PATH)) {
    console.error('❌ Docs directory not found:', DOCS_PATH);
    process.exit(1);
  }

  // 查找所有 MDX 文件
  console.log('📁 Scanning MDX files...');
  const mdxFiles = await findMdxFiles(DOCS_PATH);
  console.log(`   Found ${mdxFiles.length} MDX files\n`);

  // 提取所有映射
  const allMappings: Mapping[] = [];
  let processedFiles = 0;
  let filesWithTables = 0;

  for (const file of mdxFiles) {
    const content = await readFile(file, 'utf-8');
    const mappings = extractApiTables(content, file);

    if (mappings.length > 0) {
      allMappings.push(...mappings);
      filesWithTables++;
      console.log(`✓ ${file.replace(REPO_PATH, '')}: ${mappings.length} mappings`);
    }

    processedFiles++;
  }

  console.log(`\n📊 Summary:`);
  console.log(`   Processed files: ${processedFiles}`);
  console.log(`   Files with ApiTable: ${filesWithTables}`);
  console.log(`   Total mappings: ${allMappings.length}`);

  // 去重（同一个 class 可能在多个文件中）
  const uniqueMappings = new Map<string, Mapping>();
  for (const mapping of allMappings) {
    if (!uniqueMappings.has(mapping.class)) {
      uniqueMappings.set(mapping.class, mapping);
    }
  }

  console.log(`   Unique classes: ${uniqueMappings.size}\n`);

  // 按 class 名称排序
  const sortedMappings = Array.from(uniqueMappings.values()).sort((a, b) =>
    a.class.localeCompare(b.class)
  );

  // 确保输出目录存在
  const outputDir = join(OUTPUT_PATH, '..');
  if (!existsSync(outputDir)) {
    await mkdir(outputDir, { recursive: true });
  }

  // 写入 JSON 文件
  await writeFile(
    OUTPUT_PATH,
    JSON.stringify(sortedMappings, null, 2) + '\n',
    'utf-8'
  );

  console.log('✅ Successfully extracted mappings!');
  console.log(`   Output: ${OUTPUT_PATH}`);
  console.log(`\n💡 Tip: Run 'git diff ${OUTPUT_PATH}' to see changes`);
}

main().catch((error) => {
  console.error('❌ Error:', error);
  process.exit(1);
});
