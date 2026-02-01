use crate::collector::ClassCollector;
use crate::CssModulesAccess;
use swc_core::common::{Span, DUMMY_SP};
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{VisitMut, VisitMutWith};

/// JSX/TSX 遍历器 —— 使用 SWC VisitMut 遍历 AST，
/// 找到 className / class 属性中的字符串字面量，
/// 替换为生成的类名。
///
/// 支持三种输出风格：
/// - Global:              `className="c_hash123"`
/// - CssModules + Dot:    `className={styles.textCenterP4}`
/// - CssModules + Bracket:`className={styles["c_hash123"]}`
pub struct JsxClassVisitor<'a> {
    collector: &'a mut ClassCollector,
    /// CSS Modules 配置。None = Global 模式
    css_modules: Option<CssModulesConfig>,
}

struct CssModulesConfig {
    binding_name: String,
    access: CssModulesAccess,
}

impl<'a> JsxClassVisitor<'a> {
    pub fn new(
        collector: &'a mut ClassCollector,
        css_modules: Option<(&str, CssModulesAccess)>,
    ) -> Self {
        Self {
            collector,
            css_modules: css_modules.map(|(b, a)| CssModulesConfig {
                binding_name: b.to_string(),
                access: a,
            }),
        }
    }

    /// 判断 JSX 属性名是否为 class 相关属性
    fn is_class_attr(name: &JSXAttrName) -> bool {
        #[allow(unreachable_patterns)]
        match name {
            JSXAttrName::Ident(ident) => {
                let s: &str = &ident.sym;
                s == "className" || s == "class"
            }
            JSXAttrName::JSXNamespacedName(_) => false,
            _ => false,
        }
    }

    /// 从 Str 节点提取字符串值
    fn str_value(s: &Str) -> String {
        s.value.as_str().unwrap_or_default().to_string()
    }

    /// 构建属性值：根据模式生成字符串字面量或 styles 访问表达式
    fn build_attr_value(&self, new_class: &str, span: Span) -> JSXAttrValue {
        match &self.css_modules {
            Some(config) => {
                let expr = create_access_expr(
                    &config.binding_name,
                    new_class,
                    config.access,
                );
                JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span,
                    expr: JSXExpr::Expr(Box::new(expr)),
                })
            }
            None => {
                // Global: "xxx"
                JSXAttrValue::Str(Str {
                    span,
                    value: new_class.into(),
                    raw: None,
                })
            }
        }
    }
}

impl<'a> VisitMut for JsxClassVisitor<'a> {
    fn visit_mut_jsx_attr(&mut self, attr: &mut JSXAttr) {
        if !Self::is_class_attr(&attr.name) {
            attr.visit_mut_children_with(self);
            return;
        }

        match &mut attr.value {
            // className="p-4 m-2"
            Some(JSXAttrValue::Str(str_lit)) => {
                let original = Self::str_value(str_lit);
                if !original.trim().is_empty() {
                    let new_class = self.collector.process_classes(&original);
                    let span = str_lit.span;
                    attr.value = Some(self.build_attr_value(&new_class, span));
                }
            }
            // className={"p-4 m-2"} 或 className={`p-4 m-2`}
            Some(JSXAttrValue::JSXExprContainer(container)) => {
                if let JSXExpr::Expr(expr) = &mut container.expr {
                    self.visit_class_expr(expr, container.span);
                    // CSS Modules 模式下，如果内部已转为 member expr，
                    // 上层 container 保持不变即可（已经是 JSXExprContainer）
                }
            }
            _ => {}
        }

        attr.visit_mut_children_with(self);
    }
}

impl<'a> JsxClassVisitor<'a> {
    /// 处理花括号内的表达式
    fn visit_class_expr(&mut self, expr: &mut Box<Expr>, _container_span: Span) {
        match expr.as_mut() {
            // className={"p-4 m-2"}
            Expr::Lit(Lit::Str(str_lit)) => {
                let original = Self::str_value(str_lit);
                if !original.trim().is_empty() {
                    let new_class = self.collector.process_classes(&original);
                    match &self.css_modules {
                        Some(config) => {
                            **expr = create_access_expr(
                                &config.binding_name,
                                &new_class,
                                config.access,
                            );
                        }
                        None => {
                            str_lit.value = new_class.into();
                            str_lit.raw = None;
                        }
                    }
                }
            }
            // className={`p-4 m-2`} — 无插值模板字面量
            Expr::Tpl(tpl) if tpl.exprs.is_empty() && tpl.quasis.len() == 1 => {
                if let Some(quasi) = tpl.quasis.first() {
                    let original: &str = &quasi.raw;
                    if !original.trim().is_empty() {
                        let new_class = self.collector.process_classes(original);
                        match &self.css_modules {
                            Some(config) => {
                                **expr = create_access_expr(
                                    &config.binding_name,
                                    &new_class,
                                    config.access,
                                );
                            }
                            None => {
                                **expr = Expr::Lit(Lit::Str(Str {
                                    span: tpl.span,
                                    value: new_class.into(),
                                    raw: None,
                                }));
                            }
                        }
                    }
                }
            }
            _ => {
                // 动态表达式暂不处理
            }
        }
    }
}

/// 根据 access 模式创建 `binding.prop` 或 `binding["prop"]` 表达式
fn create_access_expr(binding: &str, prop: &str, access: CssModulesAccess) -> Expr {
    let obj = Box::new(Expr::Ident(Ident {
        span: DUMMY_SP,
        ctxt: Default::default(),
        sym: binding.into(),
        optional: false,
    }));

    let member_prop = match access {
        CssModulesAccess::Dot => {
            // styles.textCenterP4
            MemberProp::Ident(IdentName {
                span: DUMMY_SP,
                sym: prop.into(),
            })
        }
        CssModulesAccess::Bracket => {
            // styles["c_hash123"]
            MemberProp::Computed(ComputedPropName {
                span: DUMMY_SP,
                expr: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: prop.into(),
                    raw: None,
                }))),
            })
        }
    };

    Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj,
        prop: member_prop,
    })
}
