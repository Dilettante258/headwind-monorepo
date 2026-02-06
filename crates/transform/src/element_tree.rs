use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{Visit, VisitWith};

/// 元素树节点
#[derive(Debug, Clone)]
pub struct ElementNode {
    /// 标签名（如 "div", "h2", "span", "MyComponent"）
    pub tag: String,
    /// Tailwind 类名（原始值）
    pub classes: String,
    /// 直接文本内容
    pub text: String,
    /// 子元素
    pub children: Vec<ElementNode>,
}

/// 按组件分组的元素树
#[derive(Debug, Clone)]
pub struct ComponentTree {
    /// 组件/函数名（如 "Header", "App"）
    pub name: String,
    /// 该组件内的根元素
    pub roots: Vec<ElementNode>,
}

// ── 文本截断（UTF-8 安全）────────────────────────────────────────

/// 按字符数截断文本，超出 max_chars 则直接截断
fn truncate_text(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let mut result = String::new();
    for _ in 0..max_chars {
        match chars.next() {
            Some(c) => result.push(c),
            None => return result,
        }
    }
    result
}

/// 文本显示最大字符数
const MAX_TEXT_CHARS: usize = 10;

// ── 格式化输出 ──────────────────────────────────────────────────

/// 将元素树格式化为缩进文本（HTML 用），每个节点附加 [ref=eN]
pub fn format_element_tree(nodes: &[ElementNode]) -> String {
    let mut output = String::new();
    let mut counter = 0usize;
    for node in nodes {
        format_node(node, 0, &mut output, &mut counter);
    }
    if output.ends_with('\n') {
        output.pop();
    }
    output
}

/// 将按组件分组的元素树格式化为缩进文本（JSX 用）
///
/// 输出示例：
/// ```text
/// ## Header
/// - header w-full bg-white shadow [ref=e1]
///   - nav flex items-center [ref=e2]
///
/// ## App
/// - div min-h-screen [ref=e3]
///   - Header [ref=e4]
/// ```
pub fn format_component_trees(components: &[ComponentTree]) -> String {
    let mut output = String::new();
    let mut counter = 0usize;
    for (i, comp) in components.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        if !comp.name.is_empty() {
            output.push_str("## ");
            output.push_str(&comp.name);
            output.push('\n');
        }
        for node in &comp.roots {
            format_node(node, 0, &mut output, &mut counter);
        }
    }
    if output.ends_with('\n') {
        output.pop();
    }
    output
}

fn format_node(node: &ElementNode, depth: usize, output: &mut String, counter: &mut usize) {
    let indent = "  ".repeat(depth);
    *counter += 1;
    let ref_id = *counter;

    output.push_str(&indent);
    output.push_str("- ");
    output.push_str(&node.tag);

    if !node.classes.is_empty() {
        output.push(' ');
        output.push_str(&node.classes);
    }

    let text = node.text.trim();
    if !text.is_empty() {
        let display_text = truncate_text(text, MAX_TEXT_CHARS);
        if node.classes.is_empty() {
            output.push_str(": ");
            output.push_str(&display_text);
        } else {
            output.push_str(" \"");
            output.push_str(&display_text);
            output.push('"');
        }
    }

    output.push_str(&format!(" [ref=e{}]", ref_id));
    output.push('\n');

    for child in &node.children {
        format_node(child, depth + 1, output, counter);
    }
}

// ── JSX 树构建 ──────────────────────────────────────────────────

/// 从 SWC Module AST 构建按组件分组的元素树
pub fn build_jsx_element_tree(module: &Module) -> Vec<ComponentTree> {
    let mut builder = JsxTreeBuilder {
        components: Vec::new(),
        current_fn: None,
        stack: Vec::new(),
    };
    module.visit_with(&mut builder);
    builder.components
}

struct JsxTreeBuilder {
    components: Vec<ComponentTree>,
    /// 当前所在的函数/组件名
    current_fn: Option<String>,
    /// JSX 元素嵌套栈
    stack: Vec<Vec<ElementNode>>,
}

impl JsxTreeBuilder {
    fn add_root(&mut self, node: ElementNode) {
        let name = self.current_fn.clone().unwrap_or_default();
        if let Some(comp) = self.components.iter_mut().find(|c| c.name == name) {
            comp.roots.push(node);
        } else {
            self.components.push(ComponentTree {
                name,
                roots: vec![node],
            });
        }
    }
}

impl Visit for JsxTreeBuilder {
    // ── 跟踪组件名 ──

    fn visit_fn_decl(&mut self, n: &FnDecl) {
        let prev = self.current_fn.take();
        self.current_fn = Some(n.ident.sym.to_string());
        n.visit_children_with(self);
        self.current_fn = prev;
    }

    fn visit_var_declarator(&mut self, n: &VarDeclarator) {
        let prev = self.current_fn.take();
        if let Some(init) = &n.init {
            if matches!(init.as_ref(), Expr::Arrow(_) | Expr::Fn(_)) {
                if let Pat::Ident(id) = &n.name {
                    self.current_fn = Some(id.id.sym.to_string());
                }
            }
        }
        n.visit_children_with(self);
        self.current_fn = prev;
    }

    fn visit_export_default_decl(&mut self, n: &ExportDefaultDecl) {
        let prev = self.current_fn.take();
        match &n.decl {
            DefaultDecl::Fn(fn_expr) => {
                self.current_fn = Some(
                    fn_expr
                        .ident
                        .as_ref()
                        .map(|id| id.sym.to_string())
                        .unwrap_or_else(|| "default".to_string()),
                );
            }
            DefaultDecl::Class(class_expr) => {
                self.current_fn = Some(
                    class_expr
                        .ident
                        .as_ref()
                        .map(|id| id.sym.to_string())
                        .unwrap_or_else(|| "default".to_string()),
                );
            }
            _ => {}
        }
        n.visit_children_with(self);
        self.current_fn = prev;
    }

    // ── 构建 JSX 树 ──

    fn visit_jsx_element(&mut self, el: &JSXElement) {
        let tag = jsx_tag_name(&el.opening.name);
        let classes = jsx_class_attr(&el.opening.attrs);

        self.stack.push(Vec::new());

        let mut text_parts: Vec<String> = Vec::new();
        for child in &el.children {
            match child {
                JSXElementChild::JSXText(t) => {
                    let trimmed = t.value.trim();
                    if !trimmed.is_empty() {
                        text_parts.push(trimmed.to_string());
                    }
                }
                JSXElementChild::JSXExprContainer(container) => {
                    if let JSXExpr::Expr(expr) = &container.expr {
                        if let Expr::Lit(Lit::Str(s)) = expr.as_ref() {
                            let v = s.value.as_str().unwrap_or_default().trim();
                            if !v.is_empty() {
                                text_parts.push(v.to_string());
                            }
                        }
                    }
                    child.visit_with(self);
                }
                _ => {
                    child.visit_with(self);
                }
            }
        }

        let children = self.stack.pop().unwrap_or_default();
        let text = text_parts.join(" ");

        let node = ElementNode {
            tag,
            classes,
            text,
            children,
        };

        if let Some(parent) = self.stack.last_mut() {
            parent.push(node);
        } else {
            self.add_root(node);
        }
    }

    fn visit_jsx_fragment(&mut self, frag: &JSXFragment) {
        for child in &frag.children {
            child.visit_with(self);
        }
    }
}

fn jsx_tag_name(name: &JSXElementName) -> String {
    match name {
        JSXElementName::Ident(id) => id.sym.to_string(),
        JSXElementName::JSXMemberExpr(m) => jsx_member_expr(m),
        JSXElementName::JSXNamespacedName(ns) => format!("{}:{}", ns.ns.sym, ns.name.sym),
        _ => "unknown".to_string(),
    }
}

fn jsx_member_expr(m: &JSXMemberExpr) -> String {
    let obj = match &m.obj {
        JSXObject::Ident(id) => id.sym.to_string(),
        JSXObject::JSXMemberExpr(inner) => jsx_member_expr(inner),
        _ => "unknown".to_string(),
    };
    format!("{}.{}", obj, m.prop.sym)
}

fn jsx_class_attr(attrs: &[JSXAttrOrSpread]) -> String {
    for attr in attrs {
        if let JSXAttrOrSpread::JSXAttr(a) = attr {
            let is_class = match &a.name {
                JSXAttrName::Ident(id) => {
                    let s: &str = &id.sym;
                    s == "className" || s == "class"
                }
                _ => false,
            };
            if !is_class {
                continue;
            }
            return match &a.value {
                Some(JSXAttrValue::Str(s)) => s.value.as_str().unwrap_or_default().to_string(),
                Some(JSXAttrValue::JSXExprContainer(c)) => match &c.expr {
                    JSXExpr::Expr(expr) => match expr.as_ref() {
                        Expr::Lit(Lit::Str(s)) => s.value.as_str().unwrap_or_default().to_string(),
                        Expr::Tpl(tpl) if tpl.exprs.is_empty() && tpl.quasis.len() == 1 => {
                            let raw: &str = &tpl.quasis[0].raw;
                            raw.to_string()
                        }
                        _ => "{...}".to_string(),
                    },
                    _ => String::new(),
                },
                _ => String::new(),
            };
        }
    }
    String::new()
}

// ── HTML 树构建 ──────────────────────────────────────────────────

/// HTML void 元素（自闭合）
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// 从 HTML 源码构建元素树
pub fn build_html_element_tree(source: &str) -> Vec<ElementNode> {
    let mut parser = HtmlTreeParser::new(source);
    parser.parse();
    parser.roots
}

struct HtmlTreeParser<'a> {
    src: &'a str,
    pos: usize,
    roots: Vec<ElementNode>,
    stack: Vec<ElementNode>,
}

impl<'a> HtmlTreeParser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            pos: 0,
            roots: Vec::new(),
            stack: Vec::new(),
        }
    }

    fn parse(&mut self) {
        let bytes = self.src.as_bytes();
        let len = bytes.len();

        while self.pos < len {
            if bytes[self.pos] == b'<' {
                if self.pos + 4 < len && &bytes[self.pos..self.pos + 4] == b"<!--" {
                    if let Some(end) = self.src[self.pos..].find("-->") {
                        self.pos += end + 3;
                        continue;
                    }
                }
                if self.pos + 2 < len && bytes[self.pos + 1] == b'!' {
                    if let Some(end) = self.src[self.pos..].find('>') {
                        self.pos += end + 1;
                        continue;
                    }
                }
                if self.pos + 2 < len && bytes[self.pos + 1] == b'/' {
                    self.parse_closing_tag();
                    continue;
                }
                self.parse_opening_tag();
            } else {
                let start = self.pos;
                while self.pos < len && bytes[self.pos] != b'<' {
                    self.pos += 1;
                }
                let text = self.src[start..self.pos].trim();
                if !text.is_empty() {
                    if let Some(parent) = self.stack.last_mut() {
                        if !parent.text.is_empty() {
                            parent.text.push(' ');
                        }
                        parent.text.push_str(text);
                    }
                }
            }
        }

        while let Some(node) = self.stack.pop() {
            if let Some(parent) = self.stack.last_mut() {
                parent.children.push(node);
            } else {
                self.roots.push(node);
            }
        }
    }

    fn parse_opening_tag(&mut self) {
        let bytes = self.src.as_bytes();
        let len = bytes.len();

        self.pos += 1;

        let tag_start = self.pos;
        while self.pos < len
            && !bytes[self.pos].is_ascii_whitespace()
            && bytes[self.pos] != b'>'
            && bytes[self.pos] != b'/'
        {
            self.pos += 1;
        }
        let tag = self.src[tag_start..self.pos].to_ascii_lowercase();
        if tag.is_empty() {
            return;
        }

        let mut classes = String::new();
        let mut self_closing = false;

        while self.pos < len && bytes[self.pos] != b'>' {
            if bytes[self.pos] == b'/' {
                self_closing = true;
                self.pos += 1;
                continue;
            }
            if bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
                continue;
            }

            let attr_start = self.pos;
            while self.pos < len
                && !bytes[self.pos].is_ascii_whitespace()
                && bytes[self.pos] != b'='
                && bytes[self.pos] != b'>'
                && bytes[self.pos] != b'/'
            {
                self.pos += 1;
            }
            let attr_name = &self.src[attr_start..self.pos];

            while self.pos < len && bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }

            if self.pos < len && bytes[self.pos] == b'=' {
                self.pos += 1;
                while self.pos < len && bytes[self.pos].is_ascii_whitespace() {
                    self.pos += 1;
                }
                if self.pos < len && (bytes[self.pos] == b'"' || bytes[self.pos] == b'\'') {
                    let quote = bytes[self.pos];
                    self.pos += 1;
                    let val_start = self.pos;
                    while self.pos < len && bytes[self.pos] != quote {
                        self.pos += 1;
                    }
                    let value = &self.src[val_start..self.pos];
                    if self.pos < len {
                        self.pos += 1;
                    }
                    if attr_name == "class" {
                        classes = value.to_string();
                    }
                } else {
                    let val_start = self.pos;
                    while self.pos < len
                        && !bytes[self.pos].is_ascii_whitespace()
                        && bytes[self.pos] != b'>'
                    {
                        self.pos += 1;
                    }
                    let value = &self.src[val_start..self.pos];
                    if attr_name == "class" {
                        classes = value.to_string();
                    }
                }
            }
        }

        if self.pos < len && bytes[self.pos] == b'>' {
            self.pos += 1;
        }

        let is_void = VOID_ELEMENTS.contains(&tag.as_str());
        let node = ElementNode {
            tag,
            classes,
            text: String::new(),
            children: Vec::new(),
        };

        if self_closing || is_void {
            if let Some(parent) = self.stack.last_mut() {
                parent.children.push(node);
            } else {
                self.roots.push(node);
            }
        } else {
            self.stack.push(node);
        }
    }

    fn parse_closing_tag(&mut self) {
        let bytes = self.src.as_bytes();
        let len = bytes.len();

        self.pos += 2;

        let tag_start = self.pos;
        while self.pos < len && bytes[self.pos] != b'>' && !bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
        let tag = self.src[tag_start..self.pos].to_ascii_lowercase();

        while self.pos < len && bytes[self.pos] != b'>' {
            self.pos += 1;
        }
        if self.pos < len {
            self.pos += 1;
        }

        if let Some(node) = self.stack.pop() {
            if node.tag == tag {
                if let Some(parent) = self.stack.last_mut() {
                    parent.children.push(node);
                } else {
                    self.roots.push(node);
                }
            } else {
                let mut unmatched = vec![node];
                let mut found = false;
                while let Some(n) = self.stack.pop() {
                    if n.tag == tag {
                        let mut matched_node = n;
                        for um in unmatched.drain(..).rev() {
                            matched_node.children.push(um);
                        }
                        if let Some(parent) = self.stack.last_mut() {
                            parent.children.push(matched_node);
                        } else {
                            self.roots.push(matched_node);
                        }
                        found = true;
                        break;
                    }
                    unmatched.push(n);
                }
                if !found {
                    for um in unmatched.into_iter().rev() {
                        if let Some(parent) = self.stack.last_mut() {
                            parent.children.push(um);
                        } else {
                            self.roots.push(um);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_tree() {
        let tree = vec![ElementNode {
            tag: "div".to_string(),
            classes: "w-full h-20 border".to_string(),
            text: String::new(),
            children: vec![
                ElementNode {
                    tag: "h2".to_string(),
                    classes: "text-xl text-red-500".to_string(),
                    text: String::new(),
                    children: vec![],
                },
                ElementNode {
                    tag: "p".to_string(),
                    classes: String::new(),
                    text: "xxxx".to_string(),
                    children: vec![],
                },
                ElementNode {
                    tag: "div".to_string(),
                    classes: String::new(),
                    text: "yyyy".to_string(),
                    children: vec![ElementNode {
                        tag: "p".to_string(),
                        classes: "text-lg text-blue-500".to_string(),
                        text: String::new(),
                        children: vec![ElementNode {
                            tag: "span".to_string(),
                            classes: "text-sm".to_string(),
                            text: String::new(),
                            children: vec![],
                        }],
                    }],
                },
            ],
        }];

        let result = format_element_tree(&tree);
        println!("{}", result);

        assert!(result.contains("- div w-full h-20 border [ref=e1]"));
        assert!(result.contains("  - h2 text-xl text-red-500 [ref=e2]"));
        assert!(result.contains("  - p: xxxx [ref=e3]"));
        assert!(result.contains("  - div: yyyy [ref=e4]"));
        assert!(result.contains("    - p text-lg text-blue-500 [ref=e5]"));
        assert!(result.contains("      - span text-sm [ref=e6]"));
    }

    #[test]
    fn test_truncate_text_ascii() {
        assert_eq!(truncate_text("hello", 10), "hello");
        assert_eq!(truncate_text("hello world!", 10), "hello worl");
        assert_eq!(truncate_text("abcdefghij", 10), "abcdefghij");
        assert_eq!(truncate_text("abcdefghijk", 10), "abcdefghij");
    }

    #[test]
    fn test_truncate_text_unicode() {
        // 中文 3 字节 / 字符
        assert_eq!(
            truncate_text("你好世界测试文本超过十个字", 10),
            "你好世界测试文本超过"
        );
        // emoji 4 字节 / 字符
        assert_eq!(
            truncate_text("😀😁😂🤣😃😄😅😆😇😈😉", 10),
            "😀😁😂🤣😃😄😅😆😇😈"
        );
        // 短文本不截断
        assert_eq!(truncate_text("你好", 10), "你好");
    }

    #[test]
    fn test_html_tree_basic() {
        let html =
            r#"<div class="p-4 m-2"><p>Hello</p><span class="text-red-500">World</span></div>"#;
        let tree = build_html_element_tree(html);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].tag, "div");
        assert_eq!(tree[0].classes, "p-4 m-2");
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[0].children[0].tag, "p");
        assert_eq!(tree[0].children[0].text, "Hello");
        assert_eq!(tree[0].children[1].tag, "span");
        assert_eq!(tree[0].children[1].classes, "text-red-500");
        assert_eq!(tree[0].children[1].text, "World");
    }

    #[test]
    fn test_html_tree_nested() {
        let html = r#"<div class="flex"><div class="w-full"><p>text</p></div></div>"#;
        let tree = build_html_element_tree(html);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children.len(), 1);

        let result = format_element_tree(&tree);
        println!("{}", result);
        assert!(result.contains("- div flex [ref=e1]"));
        assert!(result.contains("  - div w-full [ref=e2]"));
        assert!(result.contains("    - p: text [ref=e3]"));
    }

    #[test]
    fn test_html_tree_void_elements() {
        let html = r#"<div><img src="a.png"><br><p>text</p></div>"#;
        let tree = build_html_element_tree(html);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 3);
        assert_eq!(tree[0].children[0].tag, "img");
        assert_eq!(tree[0].children[1].tag, "br");
        assert_eq!(tree[0].children[2].tag, "p");
    }

    #[test]
    fn test_html_tree_with_doctype() {
        let html = r#"<!DOCTYPE html><html><body><div class="p-4">hello</div></body></html>"#;
        let tree = build_html_element_tree(html);

        let result = format_element_tree(&tree);
        println!("{}", result);
        assert!(result.contains("div p-4"));
    }
}
