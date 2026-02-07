use crate::plugin_map::get_plugin_properties;
use crate::theme_values;
use crate::value_map::{get_color_value, get_spacing_value, infer_value};
use headwind_core::Declaration;
use headwind_tw_parse::ParsedClass;

use super::arbitrary::extract_bracket_value;
use super::Converter;

impl Converter {
    /// 为标准值构建 CSS 声明
    pub(super) fn build_standard_declarations(&self, parsed: &ParsedClass, value: &str) -> Option<Vec<Declaration>> {
        if let Some(decls) = self.build_complex_standard(parsed, value) {
            return Some(decls);
        }

        let properties = get_plugin_properties(&parsed.plugin)?;
        let mut css_value = infer_value(&parsed.plugin, value, self.color_mode)?;

        if parsed.negative {
            css_value = format!("-{}", css_value);
        }

        let declarations = properties
            .into_iter()
            .map(|property| Declaration::new(property, css_value.clone()))
            .collect();

        Some(declarations)
    }

    /// 处理复杂标准值插件（语义重载，不同值映射到不同 CSS 属性）
    fn build_complex_standard(&self, parsed: &ParsedClass, value: &str) -> Option<Vec<Declaration>> {
        match parsed.plugin.as_str() {
            // ── text: text-align / text-wrap / font-size / color ─────
            "text" => match value {
                "left" | "center" | "right" | "justify" | "start" | "end" => {
                    Some(vec![Declaration::new("text-align", value.to_string())])
                }
                "nowrap" | "wrap" | "balance" | "pretty" => {
                    Some(vec![Declaration::new("text-wrap", value.to_string())])
                }
                "xs" | "sm" | "base" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "5xl" | "6xl"
                | "7xl" | "8xl" | "9xl" => {
                    let font_size = if self.use_variables {
                        format!("var(--text-{})", value)
                    } else {
                        theme_values::TEXT_SIZE.get(value)?.to_string()
                    };

                    // alpha 修饰符覆盖行高：text-base/6, text-base/[1.5rem], text-base/(--lh)
                    let line_height = if let Some(ref alpha) = parsed.alpha {
                        if alpha.starts_with('[') && alpha.ends_with(']') {
                            // 任意值：text-base/[1.5rem] → line-height: 1.5rem
                            let inner = &alpha[1..alpha.len() - 1];
                            inner.replace('_', " ")
                        } else if alpha.starts_with('(') && alpha.ends_with(')') {
                            // CSS 变量：text-base/(--lh) → line-height: var(--lh)
                            let inner = alpha.strip_prefix('(').and_then(|s| s.strip_suffix(')')).unwrap_or(alpha);
                            format!("var({})", inner)
                        } else if alpha.chars().all(|c| c.is_ascii_digit()) {
                            // 数字：text-base/6 → line-height: calc(var(--spacing) * 6)
                            format!("calc(var(--spacing) * {})", alpha)
                        } else {
                            alpha.clone()
                        }
                    } else if self.use_variables {
                        format!("var(--text-{}--line-height)", value)
                    } else {
                        theme_values::TEXT_LINE_HEIGHT.get(value)?.to_string()
                    };

                    Some(vec![
                        Declaration::new("font-size", font_size),
                        Declaration::new("line-height", line_height),
                    ])
                }
            _ => {
                let css_value = infer_value(&parsed.plugin, value, self.color_mode)?;
                Some(vec![Declaration::new("color", css_value)])
            }
        },

        // ── bg: size / position / clip / origin / blend / repeat / gradient / attachment ──
        "bg" => match value {
            // Background size
            "auto" | "contain" | "cover" => {
                Some(vec![Declaration::new("background-size", value)])
            }
            // Background attachment
            "fixed" | "local" | "scroll" => {
                Some(vec![Declaration::new("background-attachment", value)])
            }
            // Background repeat
            "repeat" | "no-repeat" | "repeat-x" | "repeat-y" => {
                Some(vec![Declaration::new("background-repeat", value)])
            }
            "repeat-round" => Some(vec![Declaration::new("background-repeat", "round")]),
            "repeat-space" => Some(vec![Declaration::new("background-repeat", "space")]),
            // Background position
            "top" | "bottom" | "left" | "right" | "center" => {
                Some(vec![Declaration::new("background-position", value)])
            }
            "top-left" => Some(vec![Declaration::new("background-position", "top left")]),
            "top-right" => Some(vec![Declaration::new("background-position", "top right")]),
            "bottom-left" => Some(vec![Declaration::new("background-position", "bottom left")]),
            "bottom-right" => {
                Some(vec![Declaration::new("background-position", "bottom right")])
            }
            // Background clip
            "clip-border" => Some(vec![Declaration::new("background-clip", "border-box")]),
            "clip-content" => Some(vec![Declaration::new("background-clip", "content-box")]),
            "clip-padding" => Some(vec![Declaration::new("background-clip", "padding-box")]),
            "clip-text" => Some(vec![Declaration::new("background-clip", "text")]),
            // Background origin
            "origin-border" => Some(vec![Declaration::new("background-origin", "border-box")]),
            "origin-content" => {
                Some(vec![Declaration::new("background-origin", "content-box")])
            }
            "origin-padding" => {
                Some(vec![Declaration::new("background-origin", "padding-box")])
            }
            // Gradient
            "none" => Some(vec![Declaration::new("background-image", "none")]),
            "radial" => Some(vec![Declaration::new(
                "background-image",
                "radial-gradient(in oklab, var(--tw-gradient-stops))",
            )]),
            "conic" => Some(vec![Declaration::new(
                "background-image",
                "conic-gradient(in oklab, var(--tw-gradient-stops))",
            )]),
            _ => {
                // blend-* → background-blend-mode
                if let Some(mode) = value.strip_prefix("blend-") {
                    return Some(vec![Declaration::new(
                        "background-blend-mode",
                        mode.to_string(),
                    )]);
                }
                // linear-to-* / gradient-to-* (v3 compat) → background-image: linear-gradient(...)
                if let Some(dir) = value
                    .strip_prefix("linear-to-")
                    .or_else(|| value.strip_prefix("gradient-to-"))
                {
                    let direction = match dir {
                        "t" => "to top",
                        "b" => "to bottom",
                        "l" => "to left",
                        "r" => "to right",
                        "tl" => "to top left",
                        "tr" => "to top right",
                        "bl" => "to bottom left",
                        "br" => "to bottom right",
                        _ => return None,
                    };
                    return Some(vec![Declaration::new(
                        "background-image",
                        format!("linear-gradient({}, var(--tw-gradient-stops))", direction),
                    )]);
                }
                // linear-<angle> 或 linear-[<value>]
                if let Some(rest) = value.strip_prefix("linear-") {
                    if let Some(arb) = extract_bracket_value(rest) {
                        return Some(vec![Declaration::new(
                            "background-image",
                            format!("linear-gradient(var(--tw-gradient-stops, {}))", arb),
                        )]);
                    }
                    if let Ok(n) = rest.parse::<f64>() {
                        let deg = if parsed.negative {
                            format!("-{}deg", n)
                        } else {
                            format!("{}deg", n)
                        };
                        return Some(vec![Declaration::new(
                            "background-image",
                            format!(
                                "linear-gradient({} in oklab, var(--tw-gradient-stops))",
                                deg
                            ),
                        )]);
                    }
                }
                // conic-<angle> 或 conic-[<value>]
                if let Some(rest) = value.strip_prefix("conic-") {
                    // conic-[<value>] → 直接作为 background-image
                    if let Some(arb) = extract_bracket_value(rest) {
                        return Some(vec![Declaration::new(
                            "background-image",
                            arb.to_string(),
                        )]);
                    }
                    if let Ok(n) = rest.parse::<f64>() {
                        let deg = if parsed.negative {
                            format!("-{}deg", n)
                        } else {
                            format!("{}deg", n)
                        };
                        return Some(vec![Declaration::new(
                            "background-image",
                            format!(
                                "conic-gradient(from {} in oklab, var(--tw-gradient-stops))",
                                deg
                            ),
                        )]);
                    }
                }
                // radial-[<value>]
                if let Some(rest) = value.strip_prefix("radial-") {
                    if let Some(arb) = extract_bracket_value(rest) {
                        return Some(vec![Declaration::new(
                            "background-image",
                            format!("radial-gradient(var(--tw-gradient-stops, {}))", arb),
                        )]);
                    }
                }
                None // fall through to standard path for colors
            }
        },

        // ── font: weight / family / stretch ──────────────────────
        "font" => match value {
            "sans" | "serif" | "mono" => {
                if self.use_variables {
                    Some(vec![Declaration::new(
                        "font-family",
                        format!("var(--font-{})", value),
                    )])
                } else {
                    let family = theme_values::FONT_FAMILY.get(value)?;
                    Some(vec![Declaration::new("font-family", family.to_string())])
                }
            }
            "thin" => Some(vec![Declaration::new("font-weight", "100")]),
            "extralight" => Some(vec![Declaration::new("font-weight", "200")]),
            "light" => Some(vec![Declaration::new("font-weight", "300")]),
            "normal" => Some(vec![Declaration::new("font-weight", "400")]),
            "medium" => Some(vec![Declaration::new("font-weight", "500")]),
            "semibold" => Some(vec![Declaration::new("font-weight", "600")]),
            "bold" => Some(vec![Declaration::new("font-weight", "700")]),
            "extrabold" => Some(vec![Declaration::new("font-weight", "800")]),
            "black" => Some(vec![Declaration::new("font-weight", "900")]),
            _ => {
                if let Some(stretch) = value.strip_prefix("stretch-") {
                    return Some(vec![Declaration::new(
                        "font-stretch",
                        stretch.to_string(),
                    )]);
                }
                None
            }
        },

        // ── content: align-content vs content property ───────────
        "content" => match value {
            "none" => Some(vec![Declaration::new("content", "none")]),
            "start" => Some(vec![Declaration::new("align-content", "flex-start")]),
            "end" => Some(vec![Declaration::new("align-content", "flex-end")]),
            "around" => Some(vec![Declaration::new("align-content", "space-around")]),
            "between" => Some(vec![Declaration::new("align-content", "space-between")]),
            "evenly" => Some(vec![Declaration::new("align-content", "space-evenly")]),
            _ => Some(vec![Declaration::new("align-content", value.to_string())]),
        },

        // ── border: style / collapse / color ─────────────────────
        "border" => match value {
            "solid" | "dashed" | "dotted" | "double" | "hidden" | "none" => {
                Some(vec![Declaration::new("border-style", value)])
            }
            "collapse" | "separate" => {
                Some(vec![Declaration::new("border-collapse", value)])
            }
            _ => {
                if let Some(color) = get_color_value(value, self.color_mode) {
                    Some(vec![Declaration::new("border-color", color)])
                } else if let Ok(n) = value.parse::<f64>() {
                    // border-<number> → border-width: <number>px
                    let px = if parsed.negative { -n } else { n };
                    Some(vec![Declaration::new("border-width", format!("{}px", px))])
                } else {
                    None // fall through for width
                }
            }
        },

        // ── decoration: style / thickness / color ────────────────
        "decoration" => match value {
            "solid" | "dashed" | "dotted" | "double" | "wavy" => {
                Some(vec![Declaration::new("text-decoration-style", value)])
            }
            "auto" | "from-font" => {
                Some(vec![Declaration::new("text-decoration-thickness", value)])
            }
            _ => {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("text-decoration-color", color)])
            }
        },

        // ── outline: style / hidden / color / width ──────────────
        "outline" => match value {
            "solid" | "dashed" | "dotted" | "double" | "none" => {
                Some(vec![Declaration::new("outline-style", value)])
            }
            "hidden" => Some(vec![
                Declaration::new("outline", "2px solid transparent"),
                Declaration::new("outline-offset", "2px"),
            ]),
            _ => {
                if let Some(color) = get_color_value(value, self.color_mode) {
                    Some(vec![Declaration::new("outline-color", color)])
                } else if let Ok(n) = value.parse::<u32>() {
                    Some(vec![Declaration::new("outline-width", format!("{}px", n))])
                } else {
                    None
                }
            }
        },

        // ── stroke: color / width ────────────────────────────────
        "stroke" => {
            if let Some(color) = get_color_value(value, self.color_mode) {
                Some(vec![Declaration::new("stroke", color)])
            } else if let Ok(n) = value.parse::<u32>() {
                Some(vec![Declaration::new("stroke-width", n.to_string())])
            } else {
                None
            }
        }

        // ── shadow: named size / none / color ─────────────────────
        "shadow" => match value {
            "2xs" | "xs" | "sm" | "md" | "lg" | "xl" | "2xl" => {
                Some(vec![Declaration::new("box-shadow", format!("var(--shadow-{})", value))])
            }
            "none" => Some(vec![Declaration::new("box-shadow", "0 0 #0000")]),
            _ => {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("--tw-shadow-color", color)])
            }
        },

        // ── inset-shadow: named size / none / color ──────────────
        "inset-shadow" => match value {
            "2xs" | "xs" | "sm" => {
                Some(vec![Declaration::new("box-shadow", format!("var(--inset-shadow-{})", value))])
            }
            "none" => Some(vec![Declaration::new("box-shadow", "inset 0 0 #0000")]),
            _ => {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("--tw-inset-shadow-color", color)])
            }
        },

        // ── ring: number width / color ───────────────────────────
        "ring" => {
            if let Ok(n) = value.parse::<u32>() {
                Some(vec![Declaration::new("--tw-ring-shadow", format!("0 0 0 {}px", n))])
            } else {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("--tw-ring-color", color)])
            }
        }

        // ── inset-ring: number width / color ─────────────────────
        "inset-ring" => {
            if let Ok(n) = value.parse::<u32>() {
                Some(vec![Declaration::new("--tw-inset-ring-shadow", format!("inset 0 0 0 {}px", n))])
            } else {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("--tw-inset-ring-color", color)])
            }
        }

        // ── list: type / position / image ────────────────────────
        "list" => match value {
            "disc" | "decimal" | "none" => {
                Some(vec![Declaration::new("list-style-type", value)])
            }
            "inside" | "outside" => {
                Some(vec![Declaration::new("list-style-position", value)])
            }
            "image-none" => Some(vec![Declaration::new("list-style-image", "none")]),
            _ => None,
        },

        // ── object: fit vs position ──────────────────────────────
        "object" => match value {
            "contain" | "cover" | "fill" | "none" | "scale-down" => {
                Some(vec![Declaration::new("object-fit", value)])
            }
            _ => Some(vec![Declaration::new(
                "object-position",
                value.replace('-', " "),
            )]),
        },

        // ── mix: blend mode ──────────────────────────────────────
        "mix" => {
            if let Some(mode) = value.strip_prefix("blend-") {
                Some(vec![Declaration::new(
                    "mix-blend-mode",
                    mode.to_string(),
                )])
            } else {
                None
            }
        }

        // ── perspective: perspective vs perspective-origin ────────
        "perspective" => {
            if let Some(pos) = value.strip_prefix("origin-") {
                Some(vec![Declaration::new(
                    "perspective-origin",
                    pos.replace('-', " "),
                )])
            } else {
                None // fall through to standard path (infer_value handles named values)
            }
        }

        // ── snap: type / align / stop / strictness ───────────────
        "snap" => match value {
            "none" => Some(vec![Declaration::new("scroll-snap-type", "none")]),
            "x" => Some(vec![Declaration::new(
                "scroll-snap-type",
                "x var(--tw-scroll-snap-strictness)",
            )]),
            "y" => Some(vec![Declaration::new(
                "scroll-snap-type",
                "y var(--tw-scroll-snap-strictness)",
            )]),
            "both" => Some(vec![Declaration::new(
                "scroll-snap-type",
                "both var(--tw-scroll-snap-strictness)",
            )]),
            "start" | "end" | "center" => {
                Some(vec![Declaration::new("scroll-snap-align", value)])
            }
            "align-none" => Some(vec![Declaration::new("scroll-snap-align", "none")]),
            "normal" | "always" => {
                Some(vec![Declaration::new("scroll-snap-stop", value)])
            }
            "mandatory" | "proximity" => Some(vec![Declaration::new(
                "--tw-scroll-snap-strictness",
                value,
            )]),
            _ => None,
        },

        // ── mask: size / position / clip / origin / repeat / composite / mode / type ──
        "mask" => match value {
            "auto" | "contain" | "cover" => {
                Some(vec![Declaration::new("mask-size", value)])
            }
            "top" | "bottom" | "left" | "right" | "center" => {
                Some(vec![Declaration::new("mask-position", value)])
            }
            "top-left" => Some(vec![Declaration::new("mask-position", "top left")]),
            "top-right" => Some(vec![Declaration::new("mask-position", "top right")]),
            "bottom-left" => Some(vec![Declaration::new("mask-position", "bottom left")]),
            "bottom-right" => Some(vec![Declaration::new("mask-position", "bottom right")]),
            "repeat" | "no-repeat" | "repeat-x" | "repeat-y" => {
                Some(vec![Declaration::new("mask-repeat", value)])
            }
            "repeat-round" => Some(vec![Declaration::new("mask-repeat", "round")]),
            "repeat-space" => Some(vec![Declaration::new("mask-repeat", "space")]),
            "add" | "subtract" | "intersect" | "exclude" => {
                Some(vec![Declaration::new("mask-composite", value)])
            }
            "alpha" | "luminance" => {
                Some(vec![Declaration::new("mask-mode", value)])
            }
            "match" => Some(vec![Declaration::new("mask-mode", "match-source")]),
            "no-clip" => Some(vec![Declaration::new("mask-clip", "no-clip")]),
            _ => {
                if let Some(clip) = value.strip_prefix("clip-") {
                    let css = match clip {
                        "border" => "border-box",
                        "content" => "content-box",
                        "padding" => "padding-box",
                        "fill" => "fill-box",
                        "stroke" => "stroke-box",
                        "view" => "view-box",
                        _ => return None,
                    };
                    return Some(vec![Declaration::new("mask-clip", css)]);
                }
                if let Some(origin) = value.strip_prefix("origin-") {
                    let css = match origin {
                        "border" => "border-box",
                        "content" => "content-box",
                        "padding" => "padding-box",
                        "fill" => "fill-box",
                        "stroke" => "stroke-box",
                        "view" => "view-box",
                        _ => return None,
                    };
                    return Some(vec![Declaration::new("mask-origin", css)]);
                }
                if let Some(typ) = value.strip_prefix("type-") {
                    return Some(vec![Declaration::new("mask-type", typ.to_string())]);
                }
                None
            }
        },

        // ── translate: complex CSS variable composition ──────────
        "translate" | "translate-x" | "translate-y" | "translate-z" => {
            if value == "none" {
                return Some(vec![Declaration::new("translate", "none")]);
            }
            let css_val = get_spacing_value(value)?;
            let final_val = if parsed.negative {
                format!("-{}", css_val)
            } else {
                css_val
            };
            let result = match parsed.plugin.as_str() {
                "translate" => format!("{0} {0}", final_val),
                "translate-x" => format!("{} var(--tw-translate-y)", final_val),
                "translate-y" => format!("var(--tw-translate-x) {}", final_val),
                "translate-z" => {
                    format!("var(--tw-translate-x) var(--tw-translate-y) {}", final_val)
                }
                _ => unreachable!(),
            };
            Some(vec![Declaration::new("translate", result)])
        }

        // ── scale: named values ──────────────────────────────────
        "scale" => match value {
            "none" => Some(vec![Declaration::new("scale", "none")]),
            "3d" => Some(vec![Declaration::new(
                "scale",
                "var(--tw-scale-x) var(--tw-scale-y) var(--tw-scale-z)",
            )]),
            _ => None,
        },

        // ── transform: mode / style ──────────────────────────────
        "transform" => match value {
            "none" => Some(vec![Declaration::new("transform", "none")]),
            "gpu" => Some(vec![Declaration::new(
                "transform",
                "translateZ(0) var(--tw-rotate-x) var(--tw-rotate-y) var(--tw-rotate-z) var(--tw-skew-x) var(--tw-skew-y)",
            )]),
            "cpu" => Some(vec![Declaration::new(
                "transform",
                "var(--tw-rotate-x) var(--tw-rotate-y) var(--tw-rotate-z) var(--tw-skew-x) var(--tw-skew-y)",
            )]),
            "flat" => Some(vec![Declaration::new("transform-style", "flat")]),
            "3d" => Some(vec![Declaration::new("transform-style", "preserve-3d")]),
            _ => None,
        },

        // ── blur: filter with var() ──────────────────────────────
        "blur" => {
            if self.use_variables {
                Some(vec![Declaration::new(
                    "filter",
                    format!("blur(var(--blur-{}))", value),
                )])
            } else {
                let size = theme_values::BLUR_SIZE.get(value)?;
                Some(vec![Declaration::new(
                    "filter",
                    format!("blur({})", size),
                )])
            }
        }

        // ── backdrop-blur: backdrop-filter with var() ────────────
        "backdrop-blur" => {
            if self.use_variables {
                Some(vec![Declaration::new(
                    "backdrop-filter",
                    format!("blur(var(--blur-{}))", value),
                )])
            } else {
                let size = theme_values::BLUR_SIZE.get(value)?;
                Some(vec![Declaration::new(
                    "backdrop-filter",
                    format!("blur({})", size),
                )])
            }
        }

        // ── backdrop: filter-none ────────────────────────────────
        "backdrop" => match value {
            "filter-none" => Some(vec![Declaration::new("backdrop-filter", "none")]),
            _ => None,
        },

        // ── filter: none ─────────────────────────────────────────
        "filter" => match value {
            "none" => Some(vec![Declaration::new("filter", "none")]),
            _ => None,
        },

        // ── underline: offset ────────────────────────────────────
        "underline" => match value {
            "offset-auto" => Some(vec![Declaration::new("text-underline-offset", "auto")]),
            _ => None,
        },

        // ── line-clamp ───────────────────────────────────────────
        "line-clamp" => match value {
            "none" => Some(vec![
                Declaration::new("overflow", "visible"),
                Declaration::new("display", "block"),
                Declaration::new("-webkit-box-orient", "horizontal"),
                Declaration::new("-webkit-line-clamp", "unset"),
            ]),
            _ => None,
        },

        // ── break (word-break) ───────────────────────────────────
        "break" => match value {
            "all" => Some(vec![Declaration::new("word-break", "break-all")]),
            "keep" => Some(vec![Declaration::new("word-break", "keep-all")]),
            "normal" => Some(vec![Declaration::new("word-break", "normal")]),
            _ => None,
        },

        // ── aspect ratio ─────────────────────────────────────────
        "aspect" => match value {
            "auto" => Some(vec![Declaration::new("aspect-ratio", "auto")]),
            "square" => Some(vec![Declaration::new("aspect-ratio", "1 / 1")]),
            "video" => {
                if self.use_variables {
                    Some(vec![Declaration::new(
                        "aspect-ratio",
                        "var(--aspect-video)",
                    )])
                } else {
                    Some(vec![Declaration::new("aspect-ratio", "16 / 9")])
                }
            }
            _ => None,
        },

        // ── rotate ───────────────────────────────────────────────
        "rotate" => match value {
            "none" => Some(vec![Declaration::new("rotate", "none")]),
            _ => None,
        },

        // ── divide: border-style with child combinator ──────────
        "divide" => match value {
            "solid" | "dashed" | "dotted" | "double" | "hidden" | "none" => {
                Some(vec![Declaration::new("border-style", value)])
            }
            _ => None,
        },

        // ── leading: line-height ────────────────────────────────
        "leading" => match value {
            "none" => Some(vec![Declaration::new("line-height", "1")]),
            _ => {
                if let Ok(n) = value.parse::<u32>() {
                    Some(vec![Declaration::new(
                        "line-height",
                        format!("calc(var(--spacing) * {})", n),
                    )])
                } else {
                    None // fall through to standard path (infer_value handles named values)
                }
            }
        },

        // ── from / via / to: gradient color stops ────────────────
        "from" => {
            get_color_value(value, self.color_mode)
                .map(|color| vec![Declaration::new("--tw-gradient-from", color)])
        }
        "via" => {
            get_color_value(value, self.color_mode)
                .map(|color| vec![Declaration::new("--tw-gradient-via", color)])
        }
        "to" => {
            get_color_value(value, self.color_mode)
                .map(|color| vec![Declaration::new("--tw-gradient-to", color)])
        }

        _ => None,
    }
}
}
