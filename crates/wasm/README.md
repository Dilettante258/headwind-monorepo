# headwind-wasm

WebAssembly 绑定，通过 `wasm-bindgen` 将 Rust 转换引擎暴露给 JavaScript/TypeScript。

## 导出 API

### `transformJsx(source, filename, options?)`

转换 JSX/TSX 源码。

```typescript
import init, { transformJsx } from 'headwind-wasm';

await init();

const result = transformJsx(
  `export default function App() {
    return <div className="p-4 text-center">Hello</div>;
  }`,
  'App.tsx',
  {
    namingMode: 'camelCase',
    outputMode: {
      type: 'cssModules',
      bindingName: 'styles',
      access: 'dot',
    },
    colorMode: 'hex',
  }
);

console.log(result.code);      // 转换后的源码
console.log(result.css);       // 生成的 CSS
console.log(result.classMap);  // { "p-4 text-center": "p4TextCenter" }
```

### `transformHtml(source, options?)`

转换 HTML 源码。

```typescript
const result = transformHtml(
  '<div class="p-4 text-center">Hello</div>',
  { namingMode: 'hash' }
);
```

## TypeScript 选项接口

```typescript
interface TransformOptions {
  namingMode?: 'hash' | 'readable' | 'camelCase';
  outputMode?: GlobalMode | CssModulesMode;
  cssVariables?: 'var' | 'inline';
  unknownClasses?: 'remove' | 'preserve';
  colorMode?: 'hex' | 'oklch' | 'hsl' | 'var';
  colorMix?: boolean;
  elementTree?: boolean;
}

interface GlobalMode {
  type: 'global';
  importPath?: string;
}

interface CssModulesMode {
  type: 'cssModules';
  bindingName?: string;   // 默认 "styles"
  importPath?: string;     // 默认从文件名推导
  access?: 'dot' | 'bracket';  // 默认 "dot"
}

interface TransformResult {
  code: string;
  css: string;
  classMap: Record<string, string>;
  elementTree?: string;
}
```

## 构建

```bash
cd crates/wasm && wasm-pack build --target web
```

构建产物在 `pkg/` 目录下，可直接在浏览器或 Node.js 中使用。

## 依赖

- `headwind-transform` — Rust 源码变换引擎
- `headwind-core` — 共享类型定义
- `wasm-bindgen` — Rust ↔ JS 绑定
- `serde-wasm-bindgen` — JsValue ↔ Rust 结构体转换
- `console_error_panic_hook` — WASM panic 调试信息
