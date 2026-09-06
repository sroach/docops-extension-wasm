/// Colors shared across visualization types. Any type can call `theme(name)`
/// instead of reinventing its own palette lookup.
pub struct ThemeColors {
    #[allow(dead_code)]
    pub primary: &'static str,
    #[allow(dead_code)]
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

pub fn determine_text_color(hex_color: &str) -> &'static str {
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 {
        return "#ffffff";
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let luminance = 0.2126 * (r as f64 / 255.0) + 0.7152 * (g as f64 / 255.0) + 0.0722 * (b as f64 / 255.0);
    if luminance < 0.5 {
        "#fcfcfc"
    } else {
        "#000000"
    }
}

pub fn darken_color(hex_color: &str, factor: f64) -> String {
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 {
        return hex_color.to_string();
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let r = (r as f64 * (1.0 - factor)).clamp(0.0, 255.0) as u8;
    let g = (g as f64 * (1.0 - factor)).clamp(0.0, 255.0) as u8;
    let b = (b as f64 * (1.0 - factor)).clamp(0.0, 255.0) as u8;

    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

pub fn brighten_color(hex_color: &str, factor: f64) -> String {
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 {
        return hex_color.to_string();
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let r = (r as f64 + (255.0 - r as f64) * factor).clamp(0.0, 255.0) as u8;
    let g = (g as f64 + (255.0 - g as f64) * factor).clamp(0.0, 255.0) as u8;
    let b = (b as f64 + (255.0 - b as f64) * factor).clamp(0.0, 255.0) as u8;

    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

pub fn get_rgb(hex_color: &str) -> (f64, f64, f64) {
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 {
        return (1.0, 1.0, 1.0);
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f64 / 255.0;

    (r, g, b)
}

pub fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= max_chars {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
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