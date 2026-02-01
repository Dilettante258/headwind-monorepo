pub mod collector;
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
pub use headwind_core::{CssVariableMode, NamingMode, UnknownClassMode};

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
    Global,
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
        OutputMode::Global
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
}

impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            naming_mode: NamingMode::Hash,
            output_mode: OutputMode::Global,
            css_variables: CssVariableMode::Var,
            unknown_classes: UnknownClassMode::Remove,
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

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom(filename.to_string()).into(),
        source.to_string(),
    );

    // 解析（保留注释）
    let comments = SingleThreadedComments::default();
    let mut errors = vec![];
    let mut module = parse_file_as_module(&fm, syntax, EsVersion::latest(), Some(&comments), &mut errors)
        .map_err(|e| format!("解析错误: {:?}", e))?;

    if !errors.is_empty() {
        return Err(format!("解析警告: {:?}", errors));
    }

    // 遍历并替换
    let mut collector = ClassCollector::new(options.naming_mode, options.css_variables, options.unknown_classes);
    let css_modules_config = match &options.output_mode {
        OutputMode::CssModules {
            binding_name,
            access,
            ..
        } => Some((binding_name.clone(), *access)),
        OutputMode::Global => None,
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

    // CSS Modules 模式：注入 import 语句
    if let OutputMode::CssModules {
        binding_name,
        import_path,
        ..
    } = &options.output_mode
    {
        // 只有实际产生了类名映射才注入 import
        if !collector.class_map().is_empty() {
            let path = import_path
                .clone()
                .unwrap_or_else(|| derive_css_module_path(filename));
            let import = create_css_module_import(binding_name, &path);
            module.body.insert(0, import);
        }
    }

    // 输出代码（携带注释）
    let code = GLOBALS.set(&Globals::new(), || emit_module(&cm, &module, Some(&comments)))?;

    Ok(TransformResult {
        code,
        css: collector.combined_css(),
        class_map: collector.into_class_map(),
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
    let mut collector = ClassCollector::new(options.naming_mode, options.css_variables, options.unknown_classes);
    let code = html::transform_html_source(source, &mut collector);

    Ok(TransformResult {
        code,
        css: collector.combined_css(),
        class_map: collector.into_class_map(),
    })
}

/// 从文件名推导 CSS Module 的 import 路径
/// `App.tsx` → `./App.module.css`
fn derive_css_module_path(filename: &str) -> String {
    let base = filename.rsplit('/').next().unwrap_or(filename);
    let stem = base.rsplit_once('.').map(|(name, _)| name).unwrap_or(base);
    format!("./{}.module.css", stem)
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
                css_variables: CssVariableMode::Var,
                unknown_classes: UnknownClassMode::Remove,
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
                css_variables: CssVariableMode::Var,
                unknown_classes: UnknownClassMode::Remove,
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
                css_variables: CssVariableMode::Var,
                unknown_classes: UnknownClassMode::Remove,
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
}
