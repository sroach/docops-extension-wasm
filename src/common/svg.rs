/// Colors shared across visualization types. Any type can call `theme(name)`
/// instead of reinventing its own palette lookup.
pub struct ThemeColors {
    pub primary: &'static str,
    pub axis: &'static str,
    pub text: &'static str,
    pub background: &'static str,
    /// Cycled through by index for multi-series visuals (pie slices, etc.)
    pub palette: &'static [&'static str],
}

pub fn theme(name: &str) -> ThemeColors {
    match name {
        "premium" => ThemeColors {
            primary: "#6366f1",
            axis: "#94a3b8",
            text: "#1e293b",
            background: "#f8fafc",
            palette: &["#6366f1", "#8b5cf6", "#ec4899", "#f59e0b", "#10b981", "#06b6d4"],
        },
        "dark" => ThemeColors {
            primary: "#38bdf8",
            axis: "#475569",
            text: "#e2e8f0",
            background: "#0f172a",
            palette: &["#38bdf8", "#a78bfa", "#f472b6", "#fbbf24", "#34d399", "#22d3ee"],
        },
        "agentic" => ThemeColors {
            primary: "#a855f7",
            axis: "#52525b",
            text: "#f4f4f5",
            background: "#18181b",
            palette: &["#a855f7", "#22d3ee", "#f97316", "#84cc16", "#f43f5e", "#3b82f6"],
        },
        _ => ThemeColors {
            primary: "#3b82f6",
            axis: "#94a3b8",
            text: "#111827",
            background: "#ffffff",
            palette: &["#3b82f6", "#8b5cf6", "#ec4899", "#f59e0b", "#10b981", "#06b6d4"],
        },
    }
}

/// Escapes text before it goes inside SVG markup. Every type MUST run
/// user-provided text (labels, titles, messages) through this before
/// interpolating it into a format! template — otherwise a label containing
/// `<` or `&` breaks the SVG or opens an injection hole.
pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The fallback rendered whenever any type's parser returns Err. Uses
/// double-hash raw string delimiters since the literal hex colors below
/// contain '#' immediately after a quote (see: the whole saga earlier in
/// this conversation about r#"..."# terminating early).
pub fn error_svg(msg: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 500 100">
  <rect width="500" height="100" fill="#fef2f2"/>
  <text x="16" y="40" font-size="14" fill="#dc2626" font-family="monospace">Parse error:</text>
  <text x="16" y="62" font-size="12" fill="#991b1b" font-family="monospace">{msg}</text>
</svg>"##,
        msg = escape(msg)
    )
}