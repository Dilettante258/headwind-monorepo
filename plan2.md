# HeadWind 长期规划 Plan（Rust-first）

## 0. 北极星目标（North Star）

输入：任意前端代码（HTML / React JSX / TSX / 甚至以后 Vue/Svelte）
输出：等价代码，满足：

1. 原本元素上的 Tailwind class 串被替换为**一个稳定生成的类名**（或 CSS Module 的引用）
2. 同时生成对应的 **CSS**（或 `.module.css`）
3. 行为保持一致（layout/颜色/响应式/伪类等都尽可能等价）
4. 更远将来：引入 AI，根据语义推理生成更可读的类名（例如 `primaryButton`, `cardHeader`）
5. 提供 VSCode 插件：预览、批量转换、diff、回滚、规则配置

---

## 1. 总体架构（长期稳定的模块边界）

把系统拆成 4 层，每层都可单测：

### A. 语义内核（Rust Core，确定性）

- 负责：class 列表 → 规范化 → 冲突/合并 → 生成 CSS 规则 → 生成 className（hash 或策略）
- **不关心** HTML/JSX 怎么解析、怎么替换、文件系统、VSCode

### B. 语言适配层（Rust Adapters）

- HTML 适配器：从 HTML AST 找到 class 属性，做替换
- JSX/TSX 适配器：从 JS AST 找到 `className`（字符串、模板字符串、条件拼接等），做替换
- 适配层只做：**提取 class tokens** 与 **回写修改后的文本/AST**

### C. 工程集成层（Host）

- Node 后端：提供 API，执行转换，写 CSS 文件/输出
- CLI：批处理、写入文件、生成报告
- VSCode：调用 CLI/API，提供交互

### D. 智能命名层（AI Naming，可插拔）

- AI 不是“改代码”，AI 只提供：`classes + context → name suggestions`
- 最终落地仍由确定性系统执行（可回归、可重现）

> 关键原则：**AI 永远不直接参与不可回放的决定**。所有命名都要可记录、可复现、可锁定。

---

## 2. Rust 模块规划（从现在开始的主线）

建议 `cargo workspace`，每个 crate 都有单测。长期目录形态：

```
crates/
  core/                 # 语义内核：规范化、合并、css生成、命名策略
  css/                  # CSS 规则表示与输出（支持 media/pseudo/layer）
  tw_index/             # Tailwind 规则索引（从编译产物/规则表得到 class→css）
  html_adapter/         # HTML 解析与替换
  jsx_adapter/          # JSX/TSX 解析与替换（基于 swc_ecma_*）
  pipeline/             # 端到端 pipeline：输入代码→输出代码+css+报告
  fixtures/             # 测试夹具与 golden 用例加载器（可选单独 crate）
```

下面按阶段讲每个模块要做什么、怎么测试、如何逐步走向 HTML/JSX 全覆盖。

---

# 阶段规划（Milestones）

## Milestone 1：确定性内核 v0（2~3 周量级的工作，但可拆很细）

**目标**：给定 `Vec<String>` tailwind 类名 + `class→css` 映射表，输出稳定的 `{ new_class, css_rule }`。

### crates/core（v0）

功能：

- `normalize_classes(classes)`: 去空格、拆分、排序策略（可选：保持原顺序用于 hash）
- `merge_declarations(decls)`: 属性冲突后者覆盖
- `bundle(classes, tw_index, naming_strategy) -> BundleResult`
- `naming_strategy`：先做 `HashName`（稳定 hash），再加 `ReadableName`（调试）

测试（必须）：

- 相同输入稳定输出（className 和 cssText 都稳定）
- 属性覆盖逻辑正确
- 未知类处理策略明确（忽略/报 warning）
- 输出 CSS 属性排序稳定

> v0 不需要 Tailwind 全能力，只要你能提供一个 `tw_index` 映射表就能跑通。

---

## Milestone 2：Tailwind 索引 tw_index v0（让系统“接上真实 Tailwind”）

**目标**：能从 Tailwind 编译产物（或预生成 JSON）得到 `class → CSS declarations`。

### crates/tw_index（v0）

输入来源策略（建议按可行性排序）：

1. **推荐 v0**：从一个预先生成的 JSON 文件加载
   - 你用 Node 工具（tailwind build）生成 CSS，再用脚本解析为 JSON 映射
   - Rust 端只负责读取 JSON（简单、稳定）

2. v1：Rust 端直接解析 CSS 文件，抽出 “单类选择器” 的规则
   - 覆盖 `.p-4 { padding: 1rem }` 这种
   - 暂不处理复杂选择器/媒体查询

测试：

- 加载固定 JSON fixture，查得到期望 class
- 不同平台换行符/排序不影响结果

> 这一步的意义是：**从玩具映射表进入真实世界**。

---

## Milestone 3：CSS 表示层 css v1（为响应式/伪类铺路）

**目标**：把 CSS 不是当字符串拼，而是当结构化 IR（中间表示），以后才能支持：

- `hover:`
- `md:`
- `dark:`
- `focus-visible:`
- 任意组合 `md:hover:...`

### crates/css（v1）

设计一个 IR：

- `Selector`（`.class` + optional pseudo）
- `MediaQuery`（min-width 等）
- `Rule { selector, declarations }`
- `StyleSheet { rules }`
- `emit_css(stylesheet) -> String`（稳定排序）

测试：

- IR → CSS 输出稳定
- 多规则排序稳定（selector、media、prop 排序）

---

## Milestone 4：Pipeline v0（端到端：输入类串 → 输出 css + className + report）

**目标**：形成统一入口，后续 HTML/JSX 适配层都只要调用 pipeline。

### crates/pipeline（v0）

输入：

- `BundleRequest { class_tokens, file_context, naming_mode }`
  输出：
- `BundleResponse { new_class, stylesheet_delta, removed, diagnostics }`

测试：

- 用固定 tw_index fixture + 输入 tokens，断言输出

---

## Milestone 5：HTML Adapter v0（先把最简单场景打通）

**目标**：处理纯 HTML 文件中 `class="..."`。

### crates/html_adapter（v0）

能力：

- 解析 HTML（选型：`html5ever` 或 `lol_html`；偏向可修改 AST 的）
- 找到所有 `class` 属性
- 提取 tokens → pipeline → 替换为新 class
- 输出修改后的 HTML + CSS 文件（或 CSS delta）

v0 限制（写进 spec）：

- 只处理静态 `class="..."` 字符串
- 不处理条件 class（HTML 里也很少）

测试（必须）：

- golden test：input.html → output.html + output.css（固定快照）
- 保证格式变化可控（比如只改 class 值，尽量不重排其他内容）

---

## Milestone 6：JSX/TSX Adapter v0（React 最小可用）

**目标**：处理 `className="..."` 的字符串字面量。

### crates/jsx_adapter（v0）

选型：用 SWC 的 Rust AST（`swc_ecma_parser` + visit/mutate）
能力：

- parse TSX/JSX
- 找到 JSXAttribute `className`
- 仅处理字符串字面量 `"..."` / `'...'`
- 替换为新类名字符串
- 输出代码（swc 代码生成器）

测试：

- golden：input.tsx → output.tsx + output.css
- 保证只改必要节点（diff 友好）

---

## Milestone 7：JSX Adapter v1（现实世界：表达式 className）

**目标**：逐步覆盖常见写法：

- 模板字符串：`` `p-4 ${active ? 'text-red-500' : ''}` ``
- 条件运算：`active ? '...' : '...'`
- `clsx()` / `classnames()` 调用
- 数组 join：`['p-4', active && '...'].filter(Boolean).join(' ')`

策略（非常关键）：

- 把 className 表达式转成一个 **Class Token IR**（静态可判定部分 + 动态部分）
- v1 先只做“静态可判定部分”的替换，把动态部分保留（或生成多个 bundle 并条件引用）

测试：

- 每种表达式至少一个 fixture
- 记录 diagnostics（哪些部分无法静态化）

---

## Milestone 8：输出模式扩展（CSS Modules）

**目标**：支持两种输出：

1. 全局 CSS：`.c_xxx { ... }`
2. CSS Module：生成 `X.module.css` + 代码里变成 `styles.c_xxx`

### crates/pipeline（v1）

- 增加 `OutputMode::{GlobalCss, CssModule}`
- 对 JSX：注入 `import styles from './X.module.css'`（或复用已有 import）
- 对 HTML：CSS Modules 不适用（通常还是 global）

测试：

- golden 覆盖两种模式

---

## Milestone 9：Node 后端 API + Web Playground（工程层）

**目标**：浏览器不跑 wasm，只调用后端。

- Node API：接收源码 → 调 rust/或 swc 处理 → 返回结果
- Web UI：展示 diff + CSS 输出 + 一键复制

> Rust 侧此时已经稳定，工程层只是包壳。

---

## Milestone 10：VSCode 插件（稳定工程化）

**目标**：开发体验闭环：

- 命令：Convert current file / selection / workspace
- 预览：生成 diff（不直接改），确认后写入
- 回滚：通过 git 或生成备份
- 配置：输出模式、命名策略、忽略规则

技术建议：

- VSCode 先调用 CLI/本地 Node API（不要把复杂逻辑塞进 extension host）
- 所有结果可复现：把转换报告（json）存到 `.tailwind-bundler/`

---

## Milestone 11：AI 命名（可插拔、可回放）

**目标**：在不破坏确定性的前提下引入智能命名。

### 命名系统设计（关键）

- core 里定义 trait：
  - `NameProvider`：输入 `{classes, context}` 输出 `Vec<NameSuggestion>`

- 默认 provider：`HashNameProvider`（永远可用）
- AI provider：只提供建议，不直接生效
- 最终选择结果写入一个 **Name Lockfile**（例如 `bundler.names.json`）：
  - key：classes 的规范化 hash
  - value：最终 chosen name

- 这样以后再跑转换，**即使 AI 没开**也能得到相同命名。

测试：

- 不启用 AI 时结果一致
- 启用 AI 但锁文件存在时结果一致
- 锁文件缺失时，使用 hash fallback

---

# 测试体系（贯穿所有阶段）

你要求“每个模块要有单测，方便回归”，我建议三层测试：

1. **Unit tests（crate 内）**
   - 核心算法、IR、排序稳定性

2. **Golden tests（fixtures）**
   - `input` → `output` 快照
   - 覆盖 HTML / TSX / CSS module / 响应式等

3. **Property-based（可选，后期）**
   - 随机 class 序列，确保排序/稳定性/不 panic

建议 fixtures 目录结构：

```
fixtures/
  html/
    001-basic/
      input.html
      output.html
      output.css
  jsx/
    001-static/
      input.tsx
      output.tsx
      output.css
  jsx/
    010-clsx/
      input.tsx
      output.tsx
      output.css
```

---

# 最先开始做什么（Rust-first 的落地顺序）

你现在从 Rust 开始，最小闭环推荐顺序：

1. `crates/core`：bundle classes → className + CSS(IR)
2. `crates/css`：CSS IR + emit
3. `crates/tw_index`：从 JSON fixture 加载映射
4. `crates/pipeline`：统一入口 + diagnostics
5. `crates/html_adapter`：静态 class 替换（先打通最简单的 end-to-end）
6. `crates/jsx_adapter`：再上 React/TSX

这条线的好处：**每一步都能写单测 + golden，回归成本极低**。

---

# 风险清单（提前标出来，免得以后摔跤）

- Tailwind 真正复杂点在：
  - variant 组合（`md:hover:`）
  - arbitrary values（`w-[13px]`）
  - arbitrary selectors（`[&>p]:...`）
  - layers 和优先级

- 解决策略是：**CSS IR + tw_index 的表达能力要逐步增强**，不要一开始就字符串拼。
