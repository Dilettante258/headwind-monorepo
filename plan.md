项目目标

我要做一个 代码转换/重写引擎，核心是 SWC 插件（Rust）。使用场景：

VSCode 扩展：在本地 Node 环境运行转换（调用后端 SDK，本地执行）。

Web 在线体验：浏览器不运行 wasm，只做 UI；真正转换在 Node 后端 API 完成（后端用 @swc/core + wasm 插件）。

未来可能有 CLI、更多插件、更多规则。

核心要求：同一套转换规则和测试用例可被 VSCode、本地 API、未来 CLI 复用。

关键技术决策（必须遵守）

浏览器端 不运行 wasm，不运行 SWC。浏览器只调用后端 HTTP API。

Node 侧运行转换：@swc/core 加载 SWC wasm 插件（Rust 编译产物）。

Rust 插件目标：优先 wasm32-wasip1（兼容 SWC 插件机制的主流方式）。

monorepo：JS/TS 用 pnpm workspace + turborepo；Rust 用 cargo workspace。

统一 fixture/golden tests：同一套输入输出用例，用于 Node 端回归测试（可选再扩展 Rust 端测试）。

推荐 monorepo 目录结构
repo/
├─ apps/
│ ├─ api/ # Node 后端：提供 /transform 等接口
│ │ ├─ src/
│ │ │ ├─ server.ts
│ │ │ ├─ routes/transform.ts
│ │ │ └─ types.ts
│ │ ├─ package.json
│ │ └─ README.md
│ │
│ ├─ web-playground/ # 在线体验：纯前端 UI，调用 apps/api
│ │ ├─ src/
│ │ ├─ package.json
│ │ └─ README.md
│ │
│ └─ vscode-extension/ # VSCode 插件：本地调用 transformer SDK
│ ├─ src/
│ ├─ package.json
│ └─ resources/
│ └─ plugins/my_swc_plugin.wasm # 可选：若决定把 wasm 直接打包进扩展
│
├─ packages/
│ ├─ swc-host/ # Node 侧宿主库：@swc/core + wasm 插件加载
│ │ ├─ src/
│ │ │ ├─ index.ts
│ │ │ ├─ loadPlugin.ts
│ │ │ ├─ transform.ts
│ │ │ └─ config.ts
│ │ ├─ assets/
│ │ │ └─ my_swc_plugin.wasm # CI 构建后复制到此处
│ │ ├─ package.json
│ │ └─ README.md
│ │
│ ├─ transformer/ # 统一 SDK：VSCode / API 都复用
│ │ ├─ src/
│ │ │ ├─ index.ts # export transform()
│ │ │ ├─ local.ts # 调用 swc-host
│ │ │ ├─ remote.ts # 调用 api（web-playground 用）
│ │ │ └─ types.ts
│ │ ├─ package.json
│ │ └─ README.md
│ │
│ ├─ config/ # 共享 lint/tsconfig/prettier
│ │ ├─ tsconfig/
│ │ └─ eslint/
│ │
│ └─ test-fixtures/ # 共享用例：输入/输出快照
│ ├─ cases/
│ │ ├─ 001-basic/
│ │ │ ├─ input.ts
│ │ │ └─ output.ts
│ │ └─ ...
│ └─ README.md
│
├─ crates/
│ ├─ transform_core/ # Rust：规则核心（可选，但建议）
│ │ ├─ src/
│ │ └─ Cargo.toml
│ │
│ └─ swc_plugin/ # Rust：真正 SWC 插件（编译 wasm）
│ ├─ src/
│ ├─ Cargo.toml
│ └─ README.md
│
├─ tools/
│ ├─ scripts/
│ │ ├─ build-plugin.mjs # cargo build wasm + copy 到 packages/...
│ │ └─ dev.mjs # 可选：一键起 api + web + watch
│ └─ rust-toolchain.toml # 锁 Rust 版本
│
├─ .github/workflows/ci.yml
├─ Cargo.toml # cargo workspace root
├─ package.json # pnpm root
├─ pnpm-workspace.yaml
├─ turbo.json
└─ README.md

模块职责说明（防止 AI 写偏）
crates/swc_plugin（必须）

产物：my_swc_plugin.wasm

内部：实现 SWC plugin entry + 使用 swc 访问 AST，执行转换规则

允许读取 plugin config（JSON 序列化参数）

packages/swc-host（必须）

目标：在 Node 环境里提供一个函数：

transformWithPlugin(code: string, opts: TransformOptions): Promise<{ code: string, map?: string }>

内部：

依赖 @swc/core

加载 assets/my_swc_plugin.wasm 的绝对路径（注意在 monorepo + bundler 下路径稳定）

jsc.experimental.plugins = [[wasmPath, pluginConfig]]

apps/api（必须）

提供 HTTP API：

POST /transform

body: { code: string, filename?: string, options?: {...}, pluginConfig?: {...} }

return: { code: string, map?: string, diagnostics?: [...] }

内部复用 packages/transformer 或 packages/swc-host

packages/transformer（必须）

给上层统一入口：

transform(code, { mode: 'local' | 'remote', ... })

VSCode 用 local（调用 swc-host）

web-playground 用 remote（调用 apps/api）

apps/web-playground（可后做）

纯 UI：编辑器 + 结果输出 + 调用 transformer(remote)

不做 wasm，不做 SWC

apps/vscode-extension（可后做）

纯 Node：调用 transformer(local)

提供命令：对当前文件/选区运行 transform，并替换文本 or 输出 diff

构建与开发脚本要求
build-plugin（必须）

一个脚本完成：

cargo build -p swc_plugin --release --target wasm32-wasip1

把产物复制到：

packages/swc-host/assets/my_swc_plugin.wasm

（可选）apps/vscode-extension/resources/plugins/my_swc_plugin.wasm

dev（建议）

pnpm dev：并发启动 apps/api、apps/web-playground

支持 hot reload

pnpm test：跑 fixtures 测试（Node 端）

测试策略（必须落地）

packages/test-fixtures/cases/_/input._ 和 output.\*

写一个 Node 测试 runner（vitest/jest 都行），遍历 cases：

读 input

调用 transformer(local) 或 swc-host

结果对比 output（支持更新快照模式）

目标：CI 上保证插件升级不会悄悄改行为

编码优先级（给 IDE AI 的任务顺序）

初始化 monorepo：pnpm workspace + turbo + cargo workspace

实现 Rust SWC 插件最小可用（比如把 var a=1 改成 const a=1 或插入注释，随便一个 deterministic 变换）

实现 packages/swc-host 调用插件 wasm

apps/api 暴露 /transform

写 fixtures runner + 1~2 个用例

web-playground/vscode-extension 先放骨架（后续再完善）

约束与注意事项（坑位提示）

@swc/core 版本和 Rust 侧 swc_core 版本强耦合：必须锁版本，升级要一起升。

wasm 插件文件路径要稳定：在 monorepo 环境不要用相对路径猜测，使用 import.meta.url / fileURLToPath 计算资源路径。

API 端要限制 payload 大小、防止 CPU 滥用（后续可加队列/限流）。
