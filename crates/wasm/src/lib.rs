use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

use headwind_transform::{
    transform_jsx as rs_transform_jsx,
    transform_html as rs_transform_html,
    TransformOptions, OutputMode, CssModulesAccess, NamingMode, CssVariableMode, UnknownClassMode,
};

// ── JS 侧 serde 镜像类型 ──────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsTransformOptions {
    #[serde(default)]
    naming_mode: JsNamingMode,
    #[serde(default)]
    output_mode: JsOutputMode,
    #[serde(default)]
    css_variables: JsCssVariableMode,
    #[serde(default)]
    unknown_classes: JsUnknownClassMode,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum JsNamingMode {
    Hash,
    Readable,
    CamelCase,
}

impl Default for JsNamingMode {
    fn default() -> Self {
        JsNamingMode::Hash
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum JsOutputMode {
    #[serde(rename_all = "camelCase")]
    Global {
        #[serde(default)]
        import_path: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    CssModules {
        #[serde(default = "default_binding")]
        binding_name: String,
        #[serde(default)]
        import_path: Option<String>,
        #[serde(default)]
        access: JsCssModulesAccess,
    },
}

impl Default for JsOutputMode {
    fn default() -> Self {
        JsOutputMode::Global { import_path: None }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum JsCssModulesAccess {
    Dot,
    Bracket,
}

impl Default for JsCssModulesAccess {
    fn default() -> Self {
        JsCssModulesAccess::Dot
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum JsCssVariableMode {
    Var,
    Inline,
}

impl Default for JsCssVariableMode {
    fn default() -> Self {
        JsCssVariableMode::Var
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum JsUnknownClassMode {
    Remove,
    Preserve,
}

impl Default for JsUnknownClassMode {
    fn default() -> Self {
        JsUnknownClassMode::Remove
    }
}

fn default_binding() -> String {
    "styles".to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsTransformResult {
    code: String,
    css: String,
    class_map: IndexMap<String, String>,
}

// ── 类型转换 ──────────────────────────────────────────────────

impl From<JsNamingMode> for NamingMode {
    fn from(m: JsNamingMode) -> Self {
        match m {
            JsNamingMode::Hash => NamingMode::Hash,
            JsNamingMode::Readable => NamingMode::Readable,
            JsNamingMode::CamelCase => NamingMode::CamelCase,
        }
    }
}

impl From<JsCssModulesAccess> for CssModulesAccess {
    fn from(a: JsCssModulesAccess) -> Self {
        match a {
            JsCssModulesAccess::Dot => CssModulesAccess::Dot,
            JsCssModulesAccess::Bracket => CssModulesAccess::Bracket,
        }
    }
}

impl From<JsOutputMode> for OutputMode {
    fn from(m: JsOutputMode) -> Self {
        match m {
            JsOutputMode::Global { import_path } => OutputMode::Global { import_path },
            JsOutputMode::CssModules {
                binding_name,
                import_path,
                access,
            } => OutputMode::CssModules {
                binding_name,
                import_path,
                access: access.into(),
            },
        }
    }
}

impl From<JsCssVariableMode> for CssVariableMode {
    fn from(m: JsCssVariableMode) -> Self {
        match m {
            JsCssVariableMode::Var => CssVariableMode::Var,
            JsCssVariableMode::Inline => CssVariableMode::Inline,
        }
    }
}

impl From<JsUnknownClassMode> for UnknownClassMode {
    fn from(m: JsUnknownClassMode) -> Self {
        match m {
            JsUnknownClassMode::Remove => UnknownClassMode::Remove,
            JsUnknownClassMode::Preserve => UnknownClassMode::Preserve,
        }
    }
}

impl From<JsTransformOptions> for TransformOptions {
    fn from(opts: JsTransformOptions) -> Self {
        TransformOptions {
            naming_mode: opts.naming_mode.into(),
            output_mode: opts.output_mode.into(),
            css_variables: opts.css_variables.into(),
            unknown_classes: opts.unknown_classes.into(),
        }
    }
}

fn parse_options(options: JsValue) -> Result<JsTransformOptions, JsError> {
    if options.is_undefined() || options.is_null() {
        Ok(JsTransformOptions {
            naming_mode: JsNamingMode::default(),
            output_mode: JsOutputMode::default(),
            css_variables: JsCssVariableMode::default(),
            unknown_classes: JsUnknownClassMode::default(),
        })
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|e| JsError::new(&format!("Invalid options: {}", e)))
    }
}

fn serialize_result(result: headwind_transform::TransformResult) -> Result<JsValue, JsError> {
    let js_result = JsTransformResult {
        code: result.code,
        css: result.css,
        class_map: result.class_map,
    };
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    js_result.serialize(&serializer)
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

// ── WASM 导出函数 ─────────────────────────────────────────────

/// 初始化 panic hook（自动调用）
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// 转换 JSX/TSX 源码
///
/// @param source   - JSX/TSX 源码字符串
/// @param filename - 文件名（如 "App.tsx"），用于判断语法和推导 CSS Module 路径
/// @param options  - 转换选项，可选
/// @returns `{ code, css, classMap }`
#[wasm_bindgen(js_name = "transformJsx")]
pub fn transform_jsx(
    source: &str,
    filename: &str,
    options: JsValue,
) -> Result<JsValue, JsError> {
    let opts = parse_options(options)?;
    let result = rs_transform_jsx(source, filename, opts.into())
        .map_err(|e| JsError::new(&e))?;
    serialize_result(result)
}

/// 转换 HTML 源码
///
/// @param source  - HTML 源码字符串
/// @param options - 转换选项，可选
/// @returns `{ code, css, classMap }`
#[wasm_bindgen(js_name = "transformHtml")]
pub fn transform_html(source: &str, options: JsValue) -> Result<JsValue, JsError> {
    let opts = parse_options(options)?;
    let result = rs_transform_html(source, opts.into())
        .map_err(|e| JsError::new(&e))?;
    serialize_result(result)
}
