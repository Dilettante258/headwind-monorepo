//! Variant name → CSS selector/at-rule resolver
//!
//! Tailwind variant names often differ from their CSS equivalents.
//! This module provides a single source of truth for the mapping.

/// Resolves a pseudo-class variant name to its CSS selector fragment (without leading colon).
///
/// # Examples
/// - `"hover"` → `"hover"`
/// - `"first"` → `"first-child"`
/// - `"odd"` → `"nth-child(odd)"`
pub fn pseudo_class_selector(name: &str) -> String {
    match name {
        // Shorthand → full CSS pseudo-class
        "first" => "first-child".to_string(),
        "last" => "last-child".to_string(),
        "only" => "only-child".to_string(),
        "odd" => "nth-child(odd)".to_string(),
        "even" => "nth-child(even)".to_string(),

        // Complex selectors
        "open" => "is([open], :popover-open, :open)".to_string(),
        "inert" => "is([inert], [inert] *)".to_string(),

        // Direct 1:1 mappings (name == CSS pseudo-class)
        _ => name.to_string(),
    }
}

/// Returns an optional at-rule wrapper for a pseudo-class (Tailwind v4 behavior).
///
/// e.g., `"hover"` → `Some("@media (hover: hover)")` so that hover styles
/// only apply on devices that support hover.
pub fn pseudo_class_at_rule(name: &str) -> Option<&'static str> {
    match name {
        "hover" => Some("@media (hover: hover)"),
        _ => None,
    }
}

// ── Responsive breakpoints ──────────────────────────────────────────────────

/// Standard responsive breakpoint values (Tailwind v4: rem-based).
pub fn breakpoint_value(size: &str) -> Option<&'static str> {
    match size {
        "sm" => Some("40rem"),
        "md" => Some("48rem"),
        "lg" => Some("64rem"),
        "xl" => Some("80rem"),
        "2xl" => Some("96rem"),
        _ => None,
    }
}

/// Generates an at-rule for a responsive breakpoint.
///
/// - `"sm"` → `"@media (width >= 40rem)"`
/// - `"max-sm"` → `"@media (width < 40rem)"`
/// - `"min-[800px]"` → `"@media (width >= 800px)"`
/// - `"max-[800px]"` → `"@media (width < 800px)"`
pub fn responsive_at_rule(name: &str) -> Option<String> {
    // max-* (must check before min-* since "max-sm" etc.)
    if let Some(rest) = name.strip_prefix("max-") {
        if let Some(arb) = extract_bracket(rest) {
            return Some(format!("@media (width < {})", arb));
        }
        let bp = breakpoint_value(rest)?;
        return Some(format!("@media (width < {})", bp));
    }

    // min-[...] custom breakpoints
    if let Some(rest) = name.strip_prefix("min-") {
        if let Some(arb) = extract_bracket(rest) {
            return Some(format!("@media (width >= {})", arb));
        }
    }

    // Standard breakpoints
    let bp = breakpoint_value(name)?;
    Some(format!("@media (width >= {})", bp))
}

// ── Container queries ────────────────────────────────────────────────────────

/// Container query breakpoint values.
fn container_breakpoint(name: &str) -> Option<&'static str> {
    match name {
        "3xs" => Some("16rem"),
        "2xs" => Some("18rem"),
        "xs" => Some("20rem"),
        "sm" => Some("24rem"),
        "md" => Some("28rem"),
        "lg" => Some("32rem"),
        "xl" => Some("36rem"),
        "2xl" => Some("42rem"),
        "3xl" => Some("48rem"),
        "4xl" => Some("56rem"),
        "5xl" => Some("64rem"),
        "6xl" => Some("72rem"),
        "7xl" => Some("80rem"),
        _ => None,
    }
}

/// Generates an at-rule for a container query variant.
///
/// - `"@sm"` → `"@container (width >= 24rem)"`
/// - `"@max-sm"` → `"@container (width < 24rem)"`
/// - `"@min-[400px]"` → `"@container (width >= 400px)"`
pub fn container_at_rule(name: &str) -> Option<String> {
    // @max-*
    if let Some(rest) = name.strip_prefix("max-") {
        if let Some(arb) = extract_bracket(rest) {
            return Some(format!("@container (width < {})", arb));
        }
        let bp = container_breakpoint(rest)?;
        return Some(format!("@container (width < {})", bp));
    }

    // @min-[...]
    if let Some(rest) = name.strip_prefix("min-") {
        if let Some(arb) = extract_bracket(rest) {
            return Some(format!("@container (width >= {})", arb));
        }
    }

    // Standard
    let bp = container_breakpoint(name)?;
    Some(format!("@container (width >= {})", bp))
}

// ── Parameterized variants ───────────────────────────────────────────────────

/// Resolves a parameterized pseudo-class variant (with bracket argument).
///
/// - `"has-[.active]"` → `":has(.active)"`
/// - `"not-[.disabled]"` → `":not(.disabled)"`
/// - `"nth-[2n+1]"` → `":nth-child(2n+1)"`
/// - `"aria-[sort=ascending]"` → `"[aria-sort=ascending]"`
/// - `"data-[loading]"` → `"[data-loading]"`
pub fn parameterized_selector(name: &str) -> Option<String> {
    // has-[...] → :has(...)
    if let Some(rest) = name.strip_prefix("has-") {
        let arg = extract_bracket(rest)?;
        return Some(format!(":has({})", unescape_bracket(arg)));
    }

    // not-[...] → :not(...)
    if let Some(rest) = name.strip_prefix("not-") {
        let arg = extract_bracket(rest)?;
        return Some(format!(":not({})", unescape_bracket(arg)));
    }

    // nth-last-of-type-[...] → :nth-last-of-type(...)
    if let Some(rest) = name.strip_prefix("nth-last-of-type-") {
        let arg = extract_bracket(rest)?;
        return Some(format!(":nth-last-of-type({})", unescape_bracket(arg)));
    }

    // nth-of-type-[...] → :nth-of-type(...)
    if let Some(rest) = name.strip_prefix("nth-of-type-") {
        let arg = extract_bracket(rest)?;
        return Some(format!(":nth-of-type({})", unescape_bracket(arg)));
    }

    // nth-last-[...] → :nth-last-child(...)
    if let Some(rest) = name.strip_prefix("nth-last-") {
        let arg = extract_bracket(rest)?;
        return Some(format!(":nth-last-child({})", unescape_bracket(arg)));
    }

    // nth-[...] → :nth-child(...)
    if let Some(rest) = name.strip_prefix("nth-") {
        let arg = extract_bracket(rest)?;
        return Some(format!(":nth-child({})", unescape_bracket(arg)));
    }

    // aria-[...] → [aria-...]  (attribute selector)
    if let Some(rest) = name.strip_prefix("aria-") {
        if let Some(arg) = extract_bracket(rest) {
            return Some(format!("[aria-{}]", unescape_bracket(arg)));
        }
        // Named aria: aria-busy → [aria-busy="true"]
        return Some(format!("[aria-{}=\"true\"]", rest));
    }

    // data-[...] → [data-...]
    if let Some(rest) = name.strip_prefix("data-") {
        let arg = extract_bracket(rest)?;
        return Some(format!("[data-{}]", unescape_bracket(arg)));
    }

    // supports-[...] → @supports (not a selector, handled separately)
    // group-[...] / peer-[...] → handled in resolve_state
    // in-[...] → :where(...) selector
    if let Some(rest) = name.strip_prefix("in-") {
        let arg = extract_bracket(rest)?;
        return Some(format!(":where({})", unescape_bracket(arg)));
    }

    None
}

/// Resolves a `supports-[...]` variant to an @supports at-rule.
pub fn supports_at_rule(name: &str) -> Option<String> {
    let rest = name.strip_prefix("supports-")?;
    let arg = extract_bracket(rest)?;
    let unescaped = unescape_bracket(arg);
    // If the argument contains ':', it's a property:value pair → wrap in parens
    if unescaped.contains(':') {
        Some(format!("@supports ({})", unescaped))
    } else {
        Some(format!("@supports ({})", unescaped))
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Extracts the content of a `[...]` bracket from the start of a string.
fn extract_bracket(s: &str) -> Option<&str> {
    let s = s.strip_prefix('[')?;
    let end = s.find(']')?;
    Some(&s[..end])
}

/// Unescapes Tailwind bracket notation (underscores → spaces).
fn unescape_bracket(s: &str) -> String {
    s.replace('_', " ")
}

/// Resolves a pseudo-element variant name to its CSS selector fragment (without leading `::`)
///
/// # Examples
/// - `"before"` → `"before"`
/// - `"file"` → `"file-selector-button"`
pub fn pseudo_element_selector(name: &str) -> String {
    match name {
        "file" => "file-selector-button".to_string(),
        // Direct 1:1 mappings
        _ => name.to_string(),
    }
}

/// Output of resolving a state variant.
pub enum StateResolution {
    /// A CSS selector string (e.g., `.dark .{class}`)
    Selector(String),
    /// An at-rule wrapper (e.g., `@media (prefers-color-scheme: dark)`)
    /// The declarations should be nested inside it.
    AtRule(String),
}

/// Resolves a state variant to either a selector or an at-rule.
///
/// `class_selector` should include the leading dot, e.g., `.my-class`.
pub fn resolve_state(name: &str, class_selector: &str) -> StateResolution {
    match name {
        // ── Color scheme ──
        "dark" => StateResolution::AtRule("@media (prefers-color-scheme: dark)".to_string()),

        // ── Motion ──
        "motion-safe" => StateResolution::AtRule(
            "@media (prefers-reduced-motion: no-preference)".to_string(),
        ),
        "motion-reduce" => {
            StateResolution::AtRule("@media (prefers-reduced-motion: reduce)".to_string())
        }

        // ── Contrast ──
        "contrast-more" => {
            StateResolution::AtRule("@media (prefers-contrast: more)".to_string())
        }
        "contrast-less" => {
            StateResolution::AtRule("@media (prefers-contrast: less)".to_string())
        }

        // ── Media features ──
        "portrait" => StateResolution::AtRule("@media (orientation: portrait)".to_string()),
        "landscape" => StateResolution::AtRule("@media (orientation: landscape)".to_string()),
        "print" => StateResolution::AtRule("@media print".to_string()),
        "forced-colors" => {
            StateResolution::AtRule("@media (forced-colors: active)".to_string())
        }
        "inverted-colors" => {
            StateResolution::AtRule("@media (inverted-colors: inverted)".to_string())
        }
        "pointer-fine" => StateResolution::AtRule("@media (pointer: fine)".to_string()),
        "pointer-coarse" => StateResolution::AtRule("@media (pointer: coarse)".to_string()),
        "pointer-none" => StateResolution::AtRule("@media (pointer: none)".to_string()),
        "any-pointer-fine" => {
            StateResolution::AtRule("@media (any-pointer: fine)".to_string())
        }
        "any-pointer-coarse" => {
            StateResolution::AtRule("@media (any-pointer: coarse)".to_string())
        }
        "any-pointer-none" => {
            StateResolution::AtRule("@media (any-pointer: none)".to_string())
        }
        "noscript" => StateResolution::AtRule("@media (scripting: none)".to_string()),

        // ── Direction ──
        "rtl" => StateResolution::Selector(format!(
            "{}:where(:dir(rtl), [dir=\"rtl\"], [dir=\"rtl\"] *)",
            class_selector
        )),
        "ltr" => StateResolution::Selector(format!(
            "{}:where(:dir(ltr), [dir=\"ltr\"], [dir=\"ltr\"] *)",
            class_selector
        )),

        // ── Group / Peer ──
        name if name.starts_with("group-") => {
            let pseudo = &name[6..];
            if let Some(param_sel) = parameterized_selector(pseudo) {
                StateResolution::Selector(format!(".group{} {}", param_sel, class_selector))
            } else {
                let css_pseudo = pseudo_class_selector(pseudo);
                StateResolution::Selector(format!(".group:{} {}", css_pseudo, class_selector))
            }
        }
        name if name.starts_with("peer-") => {
            let pseudo = &name[5..];
            if let Some(param_sel) = parameterized_selector(pseudo) {
                StateResolution::Selector(format!(".peer{} ~ {}", param_sel, class_selector))
            } else {
                let css_pseudo = pseudo_class_selector(pseudo);
                StateResolution::Selector(format!(".peer:{} ~ {}", css_pseudo, class_selector))
            }
        }

        // ── Fallback ──
        _ => StateResolution::Selector(class_selector.to_string()),
    }
}

/// Returns the CSS selector suffix for the `marker` pseudo-element.
///
/// `marker` is special: it targets both the element and its children.
/// Returns two selectors: `::marker` and ` *::marker`.
pub fn marker_selectors(class_selector: &str) -> Vec<String> {
    vec![
        format!("{}::marker", class_selector),
        format!("{} *::marker", class_selector),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Pseudo-class ──

    #[test]
    fn test_pseudo_class_shorthand() {
        assert_eq!(pseudo_class_selector("first"), "first-child");
        assert_eq!(pseudo_class_selector("last"), "last-child");
        assert_eq!(pseudo_class_selector("only"), "only-child");
        assert_eq!(pseudo_class_selector("odd"), "nth-child(odd)");
        assert_eq!(pseudo_class_selector("even"), "nth-child(even)");
    }

    #[test]
    fn test_pseudo_class_passthrough() {
        assert_eq!(pseudo_class_selector("hover"), "hover");
        assert_eq!(pseudo_class_selector("focus"), "focus");
        assert_eq!(pseudo_class_selector("focus-within"), "focus-within");
        assert_eq!(pseudo_class_selector("disabled"), "disabled");
    }

    #[test]
    fn test_pseudo_class_at_rule() {
        assert_eq!(pseudo_class_at_rule("hover"), Some("@media (hover: hover)"));
        assert_eq!(pseudo_class_at_rule("focus"), None);
        assert_eq!(pseudo_class_at_rule("active"), None);
    }

    // ── Pseudo-element ──

    #[test]
    fn test_pseudo_element() {
        assert_eq!(pseudo_element_selector("before"), "before");
        assert_eq!(pseudo_element_selector("after"), "after");
        assert_eq!(pseudo_element_selector("file"), "file-selector-button");
        assert_eq!(pseudo_element_selector("placeholder"), "placeholder");
    }

    #[test]
    fn test_marker_selectors() {
        let sels = marker_selectors(".c");
        assert_eq!(sels, vec![".c::marker", ".c *::marker"]);
    }

    // ── State ──

    #[test]
    fn test_state_dark() {
        match resolve_state("dark", ".c") {
            StateResolution::AtRule(rule) => {
                assert_eq!(rule, "@media (prefers-color-scheme: dark)");
            }
            _ => panic!("expected AtRule"),
        }
    }

    #[test]
    fn test_state_motion() {
        match resolve_state("motion-safe", ".c") {
            StateResolution::AtRule(rule) => {
                assert_eq!(rule, "@media (prefers-reduced-motion: no-preference)");
            }
            _ => panic!("expected AtRule"),
        }
    }

    #[test]
    fn test_state_direction() {
        match resolve_state("rtl", ".c") {
            StateResolution::Selector(s) => assert!(s.contains(":dir(rtl)")),
            _ => panic!("expected Selector"),
        }
    }

    #[test]
    fn test_state_group_peer() {
        match resolve_state("group-hover", ".c") {
            StateResolution::Selector(s) => assert_eq!(s, ".group:hover .c"),
            _ => panic!("expected Selector"),
        }
        match resolve_state("peer-focus", ".c") {
            StateResolution::Selector(s) => assert_eq!(s, ".peer:focus ~ .c"),
            _ => panic!("expected Selector"),
        }
    }

    #[test]
    fn test_group_with_shorthand() {
        match resolve_state("group-first", ".c") {
            StateResolution::Selector(s) => assert_eq!(s, ".group:first-child .c"),
            _ => panic!("expected Selector"),
        }
    }

    // ── Responsive breakpoints ──

    #[test]
    fn test_responsive_at_rule() {
        assert_eq!(
            responsive_at_rule("sm").unwrap(),
            "@media (width >= 40rem)"
        );
        assert_eq!(
            responsive_at_rule("2xl").unwrap(),
            "@media (width >= 96rem)"
        );
    }

    #[test]
    fn test_responsive_max() {
        assert_eq!(
            responsive_at_rule("max-sm").unwrap(),
            "@media (width < 40rem)"
        );
        assert_eq!(
            responsive_at_rule("max-lg").unwrap(),
            "@media (width < 64rem)"
        );
    }

    #[test]
    fn test_responsive_custom() {
        assert_eq!(
            responsive_at_rule("min-[800px]").unwrap(),
            "@media (width >= 800px)"
        );
        assert_eq!(
            responsive_at_rule("max-[600px]").unwrap(),
            "@media (width < 600px)"
        );
    }

    // ── Container queries ──

    #[test]
    fn test_container_at_rule() {
        assert_eq!(
            container_at_rule("sm").unwrap(),
            "@container (width >= 24rem)"
        );
        assert_eq!(
            container_at_rule("3xs").unwrap(),
            "@container (width >= 16rem)"
        );
        assert_eq!(
            container_at_rule("7xl").unwrap(),
            "@container (width >= 80rem)"
        );
    }

    #[test]
    fn test_container_max() {
        assert_eq!(
            container_at_rule("max-sm").unwrap(),
            "@container (width < 24rem)"
        );
    }

    #[test]
    fn test_container_custom() {
        assert_eq!(
            container_at_rule("min-[400px]").unwrap(),
            "@container (width >= 400px)"
        );
    }

    // ── Parameterized selectors ──

    #[test]
    fn test_parameterized_has_not() {
        assert_eq!(
            parameterized_selector("has-[.active]").unwrap(),
            ":has(.active)"
        );
        assert_eq!(
            parameterized_selector("not-[.disabled]").unwrap(),
            ":not(.disabled)"
        );
    }

    #[test]
    fn test_parameterized_nth() {
        assert_eq!(
            parameterized_selector("nth-[2n+1]").unwrap(),
            ":nth-child(2n+1)"
        );
        assert_eq!(
            parameterized_selector("nth-last-[3]").unwrap(),
            ":nth-last-child(3)"
        );
        assert_eq!(
            parameterized_selector("nth-of-type-[odd]").unwrap(),
            ":nth-of-type(odd)"
        );
        assert_eq!(
            parameterized_selector("nth-last-of-type-[even]").unwrap(),
            ":nth-last-of-type(even)"
        );
    }

    #[test]
    fn test_parameterized_aria() {
        assert_eq!(
            parameterized_selector("aria-[sort=ascending]").unwrap(),
            "[aria-sort=ascending]"
        );
        assert_eq!(
            parameterized_selector("aria-busy").unwrap(),
            "[aria-busy=\"true\"]"
        );
    }

    #[test]
    fn test_parameterized_data() {
        assert_eq!(
            parameterized_selector("data-[loading]").unwrap(),
            "[data-loading]"
        );
    }

    #[test]
    fn test_parameterized_in() {
        assert_eq!(
            parameterized_selector("in-[.parent]").unwrap(),
            ":where(.parent)"
        );
    }

    #[test]
    fn test_supports_at_rule() {
        assert_eq!(
            supports_at_rule("supports-[display:grid]").unwrap(),
            "@supports (display:grid)"
        );
        // With underscore → space
        assert_eq!(
            supports_at_rule("supports-[display:_grid]").unwrap(),
            "@supports (display: grid)"
        );
    }

    #[test]
    fn test_underscore_unescape() {
        assert_eq!(
            parameterized_selector("has-[.parent_.child]").unwrap(),
            ":has(.parent .child)"
        );
    }
}
