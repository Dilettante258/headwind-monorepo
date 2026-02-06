pub mod collector;
pub mod element_tree;
pub mod html;
pub mod jsx_visitor;

use indexmap::IndexMap;
use jsx_visitor::JsxClassVisitor;
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::sync::Lrc;
use swc_core::common::{FileName, Globals, SourceMap, DUMMY_SP, GLOBALS};
use swc_core::ecma::ast::*;
use swc_core::ecma::codegen::text_writer::JsWriter;
use swc_core::ecma::codegen::{Config as CodegenConfig, Emitter};
use swc_core::ecma::parser::{parse_file_as_module, EsSyntax, Syntax, TsSyntax};
use swc_core::ecma::visit::VisitMutWith;

// Re-exports
pub use collector::ClassCollector;
pub use headwind_core::{ColorMode, CssVariableMode, NamingMode, UnknownClassMode};

/// CSS Modules 属性访问方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssModulesAccess {
    /// 点号访问: `styles.textCenterP4`
    /// 要求类名是合法 JS 标识符
    Dot,
    /// 方括号访问: `styles["text-center-p4"]`
    /// 支持任意字符串作为键名
    Bracket,
}

impl Default for CssModulesAccess {
    fn default() -> Self {
        CssModulesAccess::Dot
    }
}

/// 输出模式
#[derive(Debug, Clone)]
pub enum OutputMode {
    /// 全局模式：直接替换为类名字符串
    /// `className="p-4"` → `className="c_abc123"`
    Global {
        /// CSS import 路径。设置后注入 side-effect import `import '<path>'`。
        /// None 时不注入（向后兼容）。
        import_path: Option<String>,
    },
    /// CSS Modules 模式：替换为 styles.xxx 或 styles["xxx"] 引用，
    /// 并在文件头部注入 `import styles from './xxx.module.css'`
    CssModules {
        /// import 绑定名（默认 "styles"）
        binding_name: String,
        /// CSS 模块文件 import 路径。
        /// None 时自动从文件名推导：`App.tsx` → `./App.module.css`
        import_path: Option<String>,
        /// 属性访问方式（默认 Dot）
        access: CssModulesAccess,
    },
}

impl Default for OutputMode {
    fn default() -> Self {
        OutputMode::Global { import_path: None }
    }
}

impl OutputMode {
    /// 快捷构造 CssModules 模式（Dot 访问，binding = "styles"）
    pub fn css_modules() -> Self {
        OutputMode::CssModules {
            binding_name: "styles".to_string(),
            import_path: None,
            access: CssModulesAccess::Dot,
        }
    }

    /// CssModules 模式 + Bracket 访问
    pub fn css_modules_bracket() -> Self {
        OutputMode::CssModules {
            binding_name: "styles".to_string(),
            import_path: None,
            access: CssModulesAccess::Bracket,
        }
    }

    /// CssModules 模式 + 自定义 import 路径
    pub fn css_modules_with_path(path: impl Into<String>) -> Self {
        OutputMode::CssModules {
            binding_name: "styles".to_string(),
            import_path: Some(path.into()),
            access: CssModulesAccess::Dot,
        }
    }
}

/// 转换选项
pub struct TransformOptions {
    /// 类名生成策略（默认 Hash）
    pub naming_mode: NamingMode,
    /// 输出模式（默认 Global）
    pub output_mode: OutputMode,
    /// CSS 变量模式（默认 Var）
    pub css_variables: CssVariableMode,
    /// 未知类名处理模式（默认 Remove）
    pub unknown_classes: UnknownClassMode,
    /// 颜色输出模式（默认 Hex）
    pub color_mode: ColorMode,
    /// 是否使用 color-mix() 函数处理颜色透明度（默认 false）
    pub color_mix: bool,
    /// 是否生成元素树（默认 false）
    ///
    /// 开启后 `TransformResult.element_tree` 会包含结构化的元素树文本，
    /// 每个元素附带 `[ref=eN]` 引用标识，方便传给 AI 做二次处理。
    pub element_tree: bool,
}

impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            naming_mode: NamingMode::Hash,
            output_mode: OutputMode::default(),
            css_variables: CssVariableMode::Var,
            unknown_classes: UnknownClassMode::Remove,
            color_mode: ColorMode::default(),
            color_mix: false,
            element_tree: false,
        }
    }
}

/// 转换结果
pub struct TransformResult {
    /// 转换后的源码
    pub code: String,
    /// 生成的 CSS
    pub css: String,
    /// 类名映射（原始类字符串 -> 生成的类名）
    pub class_map: IndexMap<String, String>,
    /// 元素树文本（仅当 `TransformOptions.element_tree == true` 时生成）
    ///
    /// 格式示例：
    /// ```text
    /// - div w-full h-20 border [ref=e1]
    ///   - h2 text-xl text-red-500 [ref=e2]
    ///   - p: xxxx [ref=e3]
    /// ```
    pub element_tree: Option<String>,
}

/// 转换 JSX/TSX 源码
///
/// 遍历 AST，将 `className="..."` 和 `class="..."` 中的
/// Tailwind 类替换为生成的类名，并产出对应的 CSS。
///
/// # 参数
///
/// - `source`: JSX/TSX 源码字符串
/// - `filename`: 文件名（用于判断语法类型：.tsx/.jsx/.ts/.js）
/// - `options`: 转换选项
///
/// # 示例
///
/// ```no_run
/// use headwind_transform::{transform_jsx, TransformOptions};
///
/// let source = r#"
///     export default function App() {
///         return <div className="p-4 text-center hover:text-left">Hello</div>;
///     }
/// "#;
///
/// let result = transform_jsx(source, "App.tsx", TransformOptions::default()).unwrap();
/// println!("Code:\n{}", result.code);
/// println!("CSS:\n{}", result.css);
/// println!("Mappings: {:?}", result.class_map);
/// ```
pub fn transform_jsx(
    source: &str,
    filename: &str,
    options: TransformOptions,
) -> Result<TransformResult, String> {
    // 根据文件名选择语法
    let syntax = if filename.ends_with(".tsx") {
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    } else if filename.ends_with(".ts") {
        Syntax::Typescript(TsSyntax {
            tsx: false,
            ..Default::default()
        })
    } else {
        // .jsx / .js 默认支持 JSX
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    };

    // 用占位符注释保留空行位置，防止 SWC parse→emit 吞掉空行
    let preserved_source = preserve_empty_lines(source);

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom(filename.to_string()).into(),
        preserved_source,
    );

    // 解析（保留注释）
    let comments = SingleThreadedComments::default();
    let mut errors = vec![];
    let mut module = parse_file_as_module(&fm, syntax, EsVersion::latest(), Some(&comments), &mut errors)
        .map_err(|e| format!("解析错误: {:?}", e))?;

    if !errors.is_empty() {
        return Err(format!("解析警告: {:?}", errors));
    }

    // 生成元素树（在 AST 变更前遍历）
    let tree_text = if options.element_tree {
        let components = element_tree::build_jsx_element_tree(&module);
        if components.is_empty() {
            None
        } else {
            Some(element_tree::format_component_trees(&components))
        }
    } else {
        None
    };

    // 遍历并替换
    let mut collector = ClassCollector::new(options.naming_mode, options.css_variables, options.unknown_classes, options.color_mode, options.color_mix);
    let css_modules_config = match &options.output_mode {
        OutputMode::CssModules {
            binding_name,
            access,
            ..
        } => Some((binding_name.clone(), *access)),
        OutputMode::Global { .. } => None,
    };
    {
        let mut visitor = JsxClassVisitor::new(
            &mut collector,
            css_modules_config
                .as_ref()
                .map(|(b, a)| (b.as_str(), *a)),
        );
        module.visit_mut_with(&mut visitor);
    }

    // 注入 import 语句（仅在有类名映射时）
    if !collector.class_map().is_empty() {
        match &options.output_mode {
            OutputMode::Global {
                import_path: Some(path),
            } => {
                let import = create_side_effect_import(path);
                module.body.insert(0, import);
            }
            OutputMode::CssModules {
                binding_name,
                import_path,
                ..
            } => {
                let path = import_path
                    .clone()
                    .unwrap_or_else(|| derive_css_module_path(filename));
                let import = create_css_module_import(binding_name, &path);
                module.body.insert(0, import);
            }
            _ => {}
        }
    }

    // 输出代码（携带注释）
    let code = GLOBALS.set(&Globals::new(), || emit_module(&cm, &module, Some(&comments)))?;

    // 还原空行占位符
    let code = restore_empty_lines(&code);

    Ok(TransformResult {
        code,
        css: collector.combined_css(),
        class_map: collector.into_class_map(),
        element_tree: tree_text,
    })
}

/// 转换 HTML 源码
///
/// 扫描 HTML 中的 `class="..."` 属性，
/// 将 Tailwind 类替换为生成的类名，并产出对应的 CSS。
///
/// # 参数
///
/// - `source`: HTML 源码字符串
/// - `options`: 转换选项
///
/// # 示例
///
/// ```no_run
/// use headwind_transform::{transform_html, TransformOptions};
///
/// let html = r#"
///     <div class="p-4 text-center">
///         <span class="text-red-500 hover:text-blue-500">Hello</span>
///     </div>
/// "#;
///
/// let result = transform_html(html, TransformOptions::default()).unwrap();
/// println!("HTML:\n{}", result.code);
/// println!("CSS:\n{}", result.css);
/// ```
pub fn transform_html(source: &str, options: TransformOptions) -> Result<TransformResult, String> {
    // 生成元素树（在转换前）
    let tree_text = if options.element_tree {
        let nodes = element_tree::build_html_element_tree(source);
        if nodes.is_empty() {
            None
        } else {
            Some(element_tree::format_element_tree(&nodes))
        }
    } else {
        None
    };

    let mut collector = ClassCollector::new(options.naming_mode, options.css_variables, options.unknown_classes, options.color_mode, options.color_mix);
    let code = html::transform_html_source(source, &mut collector);

    Ok(TransformResult {
        code,
        css: collector.combined_css(),
        class_map: collector.into_class_map(),
        element_tree: tree_text,
    })
}

/// 从文件名推导 CSS Module 的 import 路径
/// `App.tsx` → `./App.module.css`
fn derive_css_module_path(filename: &str) -> String {
    let base = filename.rsplit('/').next().unwrap_or(filename);
    let stem = base.rsplit_once('.').map(|(name, _)| name).unwrap_or(base);
    format!("./{}.module.css", stem)
}

/// 创建 side-effect import 声明 AST 节点
/// `import './App.css'`
fn create_side_effect_import(import_path: &str) -> ModuleItem {
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        specifiers: vec![],
        src: Box::new(Str {
            span: DUMMY_SP,
            value: import_path.into(),
            raw: None,
        }),
        type_only: false,
        with: None,
        phase: Default::default(),
    }))
}

/// 创建 CSS Module 的 import 声明 AST 节点
/// `import styles from './App.module.css'`
fn create_css_module_import(binding_name: &str, import_path: &str) -> ModuleItem {
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
            span: DUMMY_SP,
            local: Ident {
                span: DUMMY_SP,
                ctxt: Default::default(),
                sym: binding_name.into(),
                optional: false,
            },
        })],
        src: Box::new(Str {
            span: DUMMY_SP,
            value: import_path.into(),
            raw: None,
        }),
        type_only: false,
        with: None,
        phase: Default::default(),
    }))
}

/// 空行占位符
///
/// SWC 的 AST 不保留空行信息，parse → emit 后空行会被吞掉。
/// 解法：在解析前把空行替换为注释占位符（SWC 会保留注释），
/// 代码生成后再把占位符还原为空行。
const EMPTY_LINE_MARKER: &str = "// __HEADWIND_EMPTY_LINE__";

/// 将源码中的空行替换为占位符注释，使 SWC 保留空行位置
fn preserve_empty_lines(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();

    // 找到最后一个非空行的索引，避免处理末尾空行
    let last_non_empty = lines.iter().rposition(|l| !l.trim().is_empty());

    lines
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            // 只在最后一个非空行之前的空行添加 marker
            if line.trim().is_empty() && last_non_empty.is_some_and(|last| i < last) {
                EMPTY_LINE_MARKER
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 将占位符注释还原为空行
fn restore_empty_lines(code: &str) -> String {
    code.lines()
        .map(|line| {
            if line.trim() == EMPTY_LINE_MARKER {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 使用 SWC codegen 输出 JS/TS 模块代码
fn emit_module(
    cm: &Lrc<SourceMap>,
    module: &swc_core::ecma::ast::Module,
    comments: Option<&SingleThreadedComments>,
) -> Result<String, String> {
    let mut buf = vec![];
    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
        let mut emitter = Emitter {
            cfg: CodegenConfig::default().with_target(EsVersion::latest()),
            cm: cm.clone(),
            comments: comments.map(|c| c as &dyn swc_core::common::comments::Comments),
            wr: writer,
        };
        emitter
            .emit_module(module)
            .map_err(|e| format!("代码生成错误: {:?}", e))?;
    }
    String::from_utf8(buf).map_err(|e| format!("UTF-8 编码错误: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_jsx_basic() {
        let source = r#"export default function App() {
    return <div className="p-4 text-center">Hello</div>;
}"#;

        let result = transform_jsx(source, "App.tsx", TransformOptions::default()).unwrap();

        println!("=== Transformed Code ===\n{}", result.code);
        println!("\n=== Generated CSS ===\n{}", result.css);
        println!("\n=== Class Map ===\n{:?}", result.class_map);

        // 代码中不再包含 Tailwind 类
        assert!(!result.code.contains("p-4 text-center"));
        // 生成了 CSS
        assert!(result.css.contains("padding"));
        assert!(result.css.contains("text-align"));
        // 有映射记录
        assert_eq!(result.class_map.len(), 1);
        assert!(result.class_map.contains_key("p-4 text-center"));
    }

    #[test]
    fn test_transform_jsx_with_hover() {
        let source = r#"function Btn() {
    return <button className="bg-blue-500 hover:bg-blue-700 text-white">Click</button>;
}"#;

        let result = transform_jsx(source, "Btn.jsx", TransformOptions::default()).unwrap();

        println!("=== Code ===\n{}", result.code);
        println!("\n=== CSS ===\n{}", result.css);

        assert!(!result.code.contains("bg-blue-500"));
        assert!(result.css.contains("background"));
        assert!(result.css.contains(":hover"));
    }

    #[test]
    fn test_transform_jsx_multiple_elements() {
        let source = r#"function Page() {
    return (
        <div className="flex flex-col">
            <h1 className="text-2xl font-bold">Title</h1>
            <p className="text-gray-500 mt-4">Content</p>
        </div>
    );
}"#;

        let result = transform_jsx(source, "Page.tsx", TransformOptions::default()).unwrap();

        println!("=== Code ===\n{}", result.code);
        println!("\n=== CSS ===\n{}", result.css);

        assert_eq!(result.class_map.len(), 3);
    }

    #[test]
    fn test_transform_jsx_class_attr() {
        // 有些 JSX 库（如 Preact）使用 class 而非 className
        let source = r#"function App() {
    return <div class="p-4 m-2">Hello</div>;
}"#;

        let result = transform_jsx(source, "App.jsx", TransformOptions::default()).unwrap();

        assert!(!result.code.contains("p-4 m-2"));
        assert_eq!(result.class_map.len(), 1);
    }

    #[test]
    fn test_transform_jsx_expression_string() {
        let source = r#"function App() {
    return <div className={"p-4 m-2"}>Hello</div>;
}"#;

        let result = transform_jsx(source, "App.jsx", TransformOptions::default()).unwrap();

        println!("=== Code ===\n{}", result.code);
        assert!(!result.code.contains("p-4 m-2"));
        assert_eq!(result.class_map.len(), 1);
    }

    #[test]
    fn test_transform_jsx_preserves_dynamic() {
        let source = r#"function App({ active }) {
    return <div className={active ? "p-4" : "p-8"}>Hello</div>;
}"#;

        let result = transform_jsx(source, "App.jsx", TransformOptions::default()).unwrap();

        // 动态表达式中的字面量不会被转换（保守策略）
        // 三元表达式整体是一个 CondExpr，不会匹配
        assert!(result.code.contains("active"));
    }

    #[test]
    fn test_transform_jsx_readable_naming() {
        let source = r#"function App() {
    return <div className="p-4 m-2">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.jsx",
            TransformOptions {
                naming_mode: NamingMode::Readable,
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Code ===\n{}", result.code);
        // Readable 模式的类名应该可读
        assert!(result.code.contains("p4_m2"));
    }

    #[test]
    fn test_transform_html_basic() {
        let html = r#"<!DOCTYPE html>
<html>
<body>
    <div class="p-4 text-center">Hello</div>
    <span class="text-red-500 mt-2">World</span>
</body>
</html>"#;

        let result = transform_html(html, TransformOptions::default()).unwrap();

        println!("=== HTML ===\n{}", result.code);
        println!("\n=== CSS ===\n{}", result.css);

        assert!(!result.code.contains("p-4 text-center"));
        assert!(!result.code.contains("text-red-500 mt-2"));
        assert_eq!(result.class_map.len(), 2);
        assert!(result.css.contains("padding"));
    }

    #[test]
    fn test_transform_html_responsive() {
        let html = r#"<div class="p-4 md:p-8 lg:p-12">content</div>"#;

        let result = transform_html(html, TransformOptions::default()).unwrap();

        println!("=== CSS ===\n{}", result.css);

        assert!(result.css.contains("padding: 1rem"));
        assert!(result.css.contains("@media"));
    }

    #[test]
    fn test_same_classes_reuse_name() {
        let source = r#"function App() {
    return (
        <div>
            <p className="p-4 m-2">A</p>
            <p className="p-4 m-2">B</p>
        </div>
    );
}"#;

        let result = transform_jsx(source, "App.jsx", TransformOptions::default()).unwrap();

        // 相同的类组合应该复用同一个生成类名
        assert_eq!(result.class_map.len(), 1);
    }

    // === CSS Modules 模式测试 ===

    #[test]
    fn test_css_modules_basic() {
        let source = r#"export default function App() {
    return <div className="p-4 text-center">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                output_mode: OutputMode::css_modules(),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== CSS Modules Code ===\n{}", result.code);
        println!("\n=== CSS ===\n{}", result.css);

        // 应包含 import 语句
        assert!(result.code.contains("import styles from"));
        assert!(result.code.contains("App.module.css"));
        // 应包含 styles.xxx 引用
        assert!(result.code.contains("styles."));
        // 不应包含原始 Tailwind 类
        assert!(!result.code.contains("p-4 text-center"));
        // CSS 仍然正确生成
        assert!(result.css.contains("padding"));
        assert!(result.css.contains("text-align"));
    }

    #[test]
    fn test_css_modules_multiple_elements() {
        let source = r#"function Page() {
    return (
        <div className="flex flex-col">
            <h1 className="text-2xl font-bold">Title</h1>
            <p className="text-gray-500 mt-4">Content</p>
        </div>
    );
}"#;

        let result = transform_jsx(
            source,
            "Page.tsx",
            TransformOptions {
                output_mode: OutputMode::css_modules(),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Code ===\n{}", result.code);

        // 只应有一个 import 语句
        assert_eq!(result.code.matches("import styles").count(), 1);
        // 应有 3 个 styles.xxx 引用
        assert_eq!(result.class_map.len(), 3);
    }

    #[test]
    fn test_css_modules_custom_binding() {
        let source = r#"function App() {
    return <div className="p-4">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                output_mode: OutputMode::CssModules {
                    binding_name: "css".to_string(),
                    import_path: None,
                    access: CssModulesAccess::Dot,
                },
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Code ===\n{}", result.code);

        assert!(result.code.contains("import css from"));
        assert!(result.code.contains("css."));
    }

    #[test]
    fn test_css_modules_custom_path() {
        let source = r#"function App() {
    return <div className="p-4">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                output_mode: OutputMode::css_modules_with_path("../styles/app.module.css"),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Code ===\n{}", result.code);

        assert!(result.code.contains("../styles/app.module.css"));
    }

    #[test]
    fn test_css_modules_expression_string() {
        let source = r#"function App() {
    return <div className={"p-4 m-2"}>Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.jsx",
            TransformOptions {
                output_mode: OutputMode::css_modules(),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Code ===\n{}", result.code);

        // 花括号内的字符串也应被转换为 styles.xxx
        assert!(result.code.contains("styles."));
        assert!(!result.code.contains("p-4 m-2"));
    }

    #[test]
    fn test_css_modules_same_classes_reuse() {
        let source = r#"function App() {
    return (
        <div>
            <p className="p-4 m-2">A</p>
            <p className="p-4 m-2">B</p>
        </div>
    );
}"#;

        let result = transform_jsx(
            source,
            "App.jsx",
            TransformOptions {
                output_mode: OutputMode::css_modules(),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Code ===\n{}", result.code);

        // 相同类组合应复用同一个 styles.xxx
        assert_eq!(result.class_map.len(), 1);
        // 两处使用同一个 styles 属性
        let class_name = result.class_map.values().next().unwrap();
        let pattern = format!("styles.{}", class_name);
        assert_eq!(result.code.matches(&pattern).count(), 2);
    }

    #[test]
    fn test_css_modules_no_import_when_empty() {
        let source = r#"function App() {
    return <div id="main">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                output_mode: OutputMode::css_modules(),
                ..Default::default()
            },
        )
        .unwrap();

        // 没有 className 时不应注入 import
        assert!(!result.code.contains("import styles"));
    }

    #[test]
    fn test_derive_css_module_path() {
        assert_eq!(derive_css_module_path("App.tsx"), "./App.module.css");
        assert_eq!(
            derive_css_module_path("src/components/Button.jsx"),
            "./Button.module.css"
        );
        assert_eq!(derive_css_module_path("index.ts"), "./index.module.css");
    }

    // === Bracket 访问模式测试 ===

    #[test]
    fn test_css_modules_bracket_basic() {
        let source = r#"function App() {
    return <div className="p-4 text-center">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                output_mode: OutputMode::css_modules_bracket(),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Bracket Code ===\n{}", result.code);

        // 应包含 import
        assert!(result.code.contains("import styles from"));
        // 应使用 styles["xxx"] 而非 styles.xxx
        assert!(result.code.contains("styles[\""));
        assert!(!result.code.contains("p-4 text-center"));
        // CSS 仍然正确
        assert!(result.css.contains("padding"));
    }

    #[test]
    fn test_css_modules_bracket_expression_string() {
        let source = r#"function App() {
    return <div className={"p-4 m-2"}>Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.jsx",
            TransformOptions {
                output_mode: OutputMode::css_modules_bracket(),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Bracket Expr Code ===\n{}", result.code);

        assert!(result.code.contains("styles[\""));
        assert!(!result.code.contains("p-4 m-2"));
    }

    #[test]
    fn test_css_modules_bracket_multiple_elements() {
        let source = r#"function Page() {
    return (
        <div className="flex flex-col">
            <h1 className="text-2xl font-bold">Title</h1>
            <p className="text-gray-500 mt-4">Content</p>
        </div>
    );
}"#;

        let result = transform_jsx(
            source,
            "Page.tsx",
            TransformOptions {
                output_mode: OutputMode::css_modules_bracket(),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Bracket Multi Code ===\n{}", result.code);

        // 只应有一个 import
        assert_eq!(result.code.matches("import styles").count(), 1);
        // 3 个不同类组合 -> 3 个 styles["xxx"]
        assert_eq!(result.class_map.len(), 3);
        assert_eq!(result.code.matches("styles[\"").count(), 3);
    }

    // === CamelCase 命名 + CSS Modules 组合测试 ===

    #[test]
    fn test_camel_case_naming_global() {
        let source = r#"function App() {
    return <div className="p-4 text-center">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.jsx",
            TransformOptions {
                naming_mode: NamingMode::CamelCase,
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== CamelCase Global Code ===\n{}", result.code);

        // CamelCase 类名应为驼峰式
        let class_name = result.class_map.values().next().unwrap();
        // 不应包含 _ 或 -
        assert!(!class_name.contains('_'));
        assert!(!class_name.contains('-'));
        // 应以小写字母开头
        assert!(class_name.chars().next().unwrap().is_lowercase());
    }

    #[test]
    fn test_camel_case_with_css_modules_dot() {
        let source = r#"function App() {
    return <div className="p-4 text-center hover:text-left">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                naming_mode: NamingMode::CamelCase,
                output_mode: OutputMode::css_modules(),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== CamelCase + Dot Code ===\n{}", result.code);
        println!("=== Class Map ===\n{:?}", result.class_map);

        // styles.camelCaseName
        assert!(result.code.contains("styles."));
        let class_name = result.class_map.values().next().unwrap();
        // 驼峰式、无下划线
        assert!(!class_name.contains('_'));
        assert!(!class_name.contains('-'));
        // 代码中应包含 styles.驼峰名
        let pattern = format!("styles.{}", class_name);
        assert!(result.code.contains(&pattern));
    }

    #[test]
    fn test_camel_case_with_css_modules_bracket() {
        let source = r#"function App() {
    return <div className="p-4 text-center">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                naming_mode: NamingMode::CamelCase,
                output_mode: OutputMode::CssModules {
                    binding_name: "styles".to_string(),
                    import_path: None,
                    access: CssModulesAccess::Bracket,
                },
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== CamelCase + Bracket Code ===\n{}", result.code);

        // styles["camelCaseName"]
        assert!(result.code.contains("styles[\""));
        let class_name = result.class_map.values().next().unwrap();
        let pattern = format!("styles[\"{}\"]", class_name);
        assert!(result.code.contains(&pattern));
    }

    #[test]
    fn test_hash_with_css_modules_bracket() {
        let source = r#"function App() {
    return <div className="p-4 m-2">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                naming_mode: NamingMode::Hash,
                output_mode: OutputMode::css_modules_bracket(),
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Hash + Bracket Code ===\n{}", result.code);

        // styles["c_hash"]
        assert!(result.code.contains("styles[\""));
        let class_name = result.class_map.values().next().unwrap();
        assert!(class_name.starts_with("c_"));
        let pattern = format!("styles[\"{}\"]", class_name);
        assert!(result.code.contains(&pattern));
    }

    // === Global mode CSS import injection tests ===

    #[test]
    fn test_global_with_import_path() {
        let source = r#"export default function App() {
    return <div className="p-4 text-center">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                output_mode: OutputMode::Global {
                    import_path: Some("./App.css".to_string()),
                },
                ..Default::default()
            },
        )
        .unwrap();

        println!("=== Global + Import Code ===\n{}", result.code);

        // Should contain side-effect import
        assert!(result.code.contains("import \"./App.css\""));
        // Should still use string class names (not styles.xxx)
        assert!(!result.code.contains("styles."));
        // CSS should still be generated
        assert!(result.css.contains("padding"));
    }

    #[test]
    fn test_global_without_import_path() {
        let source = r#"function App() {
    return <div className="p-4">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                output_mode: OutputMode::Global { import_path: None },
                ..Default::default()
            },
        )
        .unwrap();

        // No import should be injected
        assert!(!result.code.contains("import"));
    }

    #[test]
    fn test_transform_jsx_preserves_empty_lines() {
        let source = "import React from 'react';\n\nfunction App() {\n    const x = 1;\n\n    return (\n        <div className=\"p-4 text-center\">\n\n            <p>Hello</p>\n        </div>\n    );\n}\n\nexport default App;\n";

        let result = transform_jsx(source, "App.tsx", TransformOptions::default()).unwrap();

        println!("=== Source ===\n{}", source);
        println!("=== Code ===\n{}", result.code);

        // Count empty lines in source vs output
        let source_empty = source.lines().filter(|l| l.trim().is_empty()).count();
        let result_empty = result.code.lines().filter(|l| l.trim().is_empty()).count();

        assert_eq!(
            source_empty, result_empty,
            "Empty line count should be preserved (source: {}, result: {})",
            source_empty, result_empty
        );

        // No placeholder markers should remain
        assert!(
            !result.code.contains("__HEADWIND_EMPTY_LINE__"),
            "Placeholder markers should be fully restored"
        );
    }

    #[test]
    fn test_transform_jsx_trailing_empty_lines_no_marker() {
        // 末尾有多行空行时不应产生 marker
        let source = "function App() {\n    return <div className=\"p-4\">Hello</div>;\n}\n\n\n";

        let result = transform_jsx(source, "App.tsx", TransformOptions::default()).unwrap();

        // 不应有任何 marker 残留
        assert!(
            !result.code.contains("__HEADWIND_EMPTY_LINE__"),
            "Trailing empty lines should not produce markers. Got:\n{}",
            result.code
        );
    }

    #[test]
    fn test_global_import_not_injected_when_no_classes() {
        let source = r#"function App() {
    return <div id="main">Hello</div>;
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                output_mode: OutputMode::Global {
                    import_path: Some("./App.css".to_string()),
                },
                ..Default::default()
            },
        )
        .unwrap();

        // No className means no class map → no import
        assert!(!result.code.contains("import"));
    }

    // === Element Tree 测试 ===

    #[test]
    fn test_element_tree_jsx() {
        let source = r#"function App() {
    return (
        <div className="w-full h-20 border">
            <h2 className="text-xl text-red-500">Title</h2>
            <p>some text</p>
            <div>
                <p className="text-lg text-blue-500">
                    <span className="text-sm">inner</span>
                </p>
            </div>
        </div>
    );
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                element_tree: true,
                ..Default::default()
            },
        )
        .unwrap();

        let tree = result.element_tree.as_ref().expect("element_tree should be Some");
        println!("=== Element Tree ===\n{}", tree);

        // 验证组件名和树结构
        assert!(tree.contains("## App"));
        assert!(tree.contains("- div w-full h-20 border [ref=e1]"));
        assert!(tree.contains("  - h2 text-xl text-red-500 \"Title\" [ref=e2]"));
        assert!(tree.contains("  - p: some text [ref=e3]"));
        assert!(tree.contains("  - div [ref=e4]"));
        assert!(tree.contains("    - p text-lg text-blue-500 [ref=e5]"));
        assert!(tree.contains("      - span text-sm \"inner\" [ref=e6]"));

        // 转换结果仍然正常
        assert!(!result.code.contains("w-full h-20 border"));
        assert!(!result.css.is_empty());
    }

    #[test]
    fn test_element_tree_html() {
        let html = r#"<div class="flex flex-col">
    <h1 class="text-2xl font-bold">Title</h1>
    <p class="text-gray-500">Content</p>
</div>"#;

        let result = transform_html(
            html,
            TransformOptions {
                element_tree: true,
                ..Default::default()
            },
        )
        .unwrap();

        let tree = result.element_tree.as_ref().expect("element_tree should be Some");
        println!("=== HTML Element Tree ===\n{}", tree);

        assert!(tree.contains("- div flex flex-col [ref=e1]"));
        assert!(tree.contains("  - h1 text-2xl font-bold"));
        assert!(tree.contains("  - p text-gray-500"));
    }

    #[test]
    fn test_element_tree_multi_component() {
        let source = r#"function Header() {
    return (
        <header className="w-full bg-white shadow">
            <nav className="flex items-center">
                <a className="text-blue-500" href="/">Home</a>
                <a className="text-gray-500" href="/about">About</a>
            </nav>
        </header>
    );
}

function Card({ title, children }) {
    return (
        <div className="rounded-lg border p-4">
            <h3 className="text-lg font-bold">{title}</h3>
            <div className="mt-2">{children}</div>
        </div>
    );
}

export default function App() {
    return (
        <div className="min-h-screen">
            <Header />
            <main className="container mx-auto p-8">
                <Card title="Hello">
                    <p className="text-gray-600">World</p>
                </Card>
            </main>
        </div>
    );
}"#;

        let result = transform_jsx(
            source,
            "App.tsx",
            TransformOptions {
                element_tree: true,
                ..Default::default()
            },
        )
        .unwrap();

        let tree = result.element_tree.as_ref().expect("element_tree should be Some");
        println!("=== Multi-Component Element Tree ===\n{}", tree);

        // 验证每个组件都有标题
        assert!(tree.contains("## Header"));
        assert!(tree.contains("## Card"));
        assert!(tree.contains("## App"));

        // 验证组件内的根元素
        assert!(tree.contains("- header w-full bg-white shadow"));
        assert!(tree.contains("- div rounded-lg border p-4"));
        assert!(tree.contains("- div min-h-screen"));
    }

    #[test]
    fn test_element_tree_disabled_by_default() {
        let source = r#"function App() {
    return <div className="p-4">Hello</div>;
}"#;

        let result = transform_jsx(source, "App.tsx", TransformOptions::default()).unwrap();
        assert!(result.element_tree.is_none());
    }
}
