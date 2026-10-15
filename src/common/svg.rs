use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};

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
            primary: "#3b82f6",
            axis: "#94a3b8",
            text: "#111827",
            background: "#ffffff",
            palette: &[
                "#3b82f6", "#8b5cf6", "#22c55e", "#f59e0b", "#ef4444", "#6b7280",
            ],
        },
        "dark" => ThemeColors {
            primary: "#38bdf8",
            axis: "#475569",
            text: "#e2e8f0",
            background: "#0f172a",
            palette: &[
                "#38bdf8", "#a78bfa", "#f472b6", "#fbbf24", "#34d399", "#22d3ee",
            ],
        },
        "agentic" => ThemeColors {
            primary: "#a855f7",
            axis: "#52525b",
            text: "#f4f4f5",
            background: "#18181b",
            palette: &[
                "#a855f7", "#22d3ee", "#f97316", "#84cc16", "#f43f5e", "#3b82f6",
            ],
        },
        _ => ThemeColors {
            primary: "#3b82f6",
            axis: "#94a3b8",
            text: "#111827",
            background: "#ffffff",
            palette: &[
                "#3b82f6", "#8b5cf6", "#ec4899", "#f59e0b", "#10b981", "#06b6d4",
            ],
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

pub struct Metadata {
    pub creator: String,
    pub rights: String,
    pub source: String,
    pub date: String,
    pub signature: Option<String>,
}

impl Metadata {
    /// Creates metadata from header controls, falling back to project defaults
    pub fn from_controls(controls: &std::collections::HashMap<String, String>) -> Self {
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let signature = if controls.contains_key("sign") || controls.contains_key("privkey") {
            Some("SIGNATURE_PLACEHOLDER".to_string())
        } else {
            None
        };
        Self {
            creator: controls
                .get("creator")
                .cloned()
                .unwrap_or_else(|| "DocOps.io".to_string()),
            rights: controls
                .get("rights")
                .cloned()
                .unwrap_or_else(|| "MIT License".to_string()),
            source: controls
                .get("source")
                .cloned()
                .unwrap_or_else(|| "https://roach.gy".to_string()),
            date,
            signature,
        }
    }

    /// Generates the RDF XML block requested
    pub fn to_rdf_xml(&self) -> String {
        let sig_tag = match &self.signature {
            Some(s) => format!(r#"<dc:signature rdf:resource="sha256-ed25519:{}"/>"#, s),
            None => "".to_string(),
        };
        format!(
            r#"<metadata><rdf:rdf xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:cc="http://creativecommons.org/ns#"><cc:work rdf:about=""><dc:creator>{}</dc:creator><dc:rights>{}</dc:rights><dc:source>{}</dc:source><dc:date>{}</dc:date>{}</cc:work></rdf:rdf></metadata>"#,
            escape(&self.creator),
            escape(&self.rights),
            escape(&self.source),
            escape(&self.date),
            sig_tag
        )
    }
}

pub fn sign_svg(svg: &mut String, private_key_hex: &str) -> Result<(), String> {
    let placeholder = "SIGNATURE_PLACEHOLDER";
    if !svg.contains(placeholder) {
        return Ok(());
    }

    let mut hasher = Sha256::new();
    hasher.update(svg.as_bytes());
    let hash = hasher.finalize();

    let key_bytes = hex::decode(private_key_hex.trim())
        .map_err(|e| format!("Invalid private key hex: {}", e))?;

    let key_arr: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "Private key must be exactly 32 bytes (64 hex characters)")?;

    let signing_key = SigningKey::from_bytes(&key_arr);
    let signature = signing_key.sign(&hash);
    let sig_base64 = BASE64.encode(signature.to_bytes());

    *svg = svg.replace(placeholder, &sig_base64);

    Ok(())
}

pub fn determine_text_color(hex_color: &str) -> &'static str {
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 {
        return "#ffffff";
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let luminance =
        0.2126 * (r as f64 / 255.0) + 0.7152 * (g as f64 / 255.0) + 0.0722 * (b as f64 / 255.0);
    if luminance < 0.5 {
        "#fcfcfc"
    } else {
        "#000000"
    }
}

/// Formats large numbers compactly using K, M, B suffixes (e.g. 1.4K, 4.5M).
pub fn format_abbreviated_number(val: f64) -> String {
    let abs = val.abs();
    let sign = if val < 0.0 { "-" } else { "" };
    if abs < 1e-6 {
        return "0".to_string();
    }
    if abs >= 1_000_000_000.0 {
        let num = abs / 1_000_000_000.0;
        let s = if num >= 100.0 || (num * 10.0).round() % 10.0 == 0.0 {
            format!("{:.0}B", num)
        } else {
            format!("{:.1}B", num)
        };
        format!("{}{}", sign, s)
    } else if abs >= 1_000_000.0 {
        let num = abs / 1_000_000.0;
        let s = if num >= 100.0 || (num * 10.0).round() % 10.0 == 0.0 {
            format!("{:.0}M", num)
        } else {
            format!("{:.1}M", num)
        };
        format!("{}{}", sign, s)
    } else if abs >= 1_000.0 {
        let num = abs / 1_000.0;
        let s = if num >= 100.0 || (num * 10.0).round() % 10.0 == 0.0 {
            format!("{:.0}K", num)
        } else {
            format!("{:.1}K", num)
        };
        format!("{}{}", sign, s)
    } else if abs.fract() == 0.0 || abs >= 10.0 {
        format!("{:.0}", val)
    } else {
        let s = format!("{:.1}", val);
        s.trim_end_matches(".0").to_string()
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

/// Modifies a DSL string so that `useDark` is forced to `false` in the
/// envelope header, ensuring the SVG is generated with light-mode styling.
/// This should be called before `generate_svg` when the output target is PDF,
/// because PDF embedders cannot switch color schemes at render time.
///
/// If the DSL does not contain an envelope header (no `[docops,...]`),
/// it is returned unchanged.
pub fn force_light_mode(dsl: &str) -> String {
    let trimmed = dsl.trim();
    if !trimmed.starts_with('[') {
        return dsl.to_string();
    }

    let close = match trimmed.find(']') {
        Some(pos) => pos,
        None => return dsl.to_string(),
    };
    let header = &trimmed[1..close];
    let body = &trimmed[close + 1..];

    // Parse the header fields
    let mut fields: Vec<&str> = header.split(',').collect();
    let mut result_fields: Vec<String> = Vec::new();
    let mut found_use_dark = false;

    for field in &mut fields {
        let f = field.trim();
        if f.is_empty() {
            continue;
        }

        if let Some((k, v)) = f.split_once('=') {
            let k = k.trim();
            let v = v.trim();
            if k == "useDark" {
                found_use_dark = true;
                result_fields.push("useDark=false".to_string());
            } else {
                result_fields.push(format!("{k}={v}"));
            }
        } else {
            result_fields.push(f.to_string());
        }
    }

    // If useDark wasn't present, add it to ensure light mode
    if !found_use_dark {
        result_fields.push("useDark=false".to_string());
    }

    let new_header = result_fields.join(", ");
    format!("[{new_header}]{body}")
}

/// Post-processes an SVG so that it renders correctly inside Typst / PDF
/// embedders that do not support CSS custom properties (`var(--name)`).
///
/// The algorithm:
/// 1. Extract the `<style>` block content.
/// 2. Parse `--variable: value;` definitions from light-mode sections only,
///    skipping `@media (prefers-color-scheme: dark)` and `.dark-mode` blocks
///    using brace-depth tracking.
/// 3. Replace every `var(--name)` occurrence with its resolved concrete value.
/// 4. Remove the entire `<style>...</style>` block (animations, hover effects,
///    and `@import` font rules don't work in PDF anyway).
/// 5. Return the cleaned SVG.
pub fn resolve_for_pdf(svg: &str) -> String {
    // 1. Find and extract the <style> block
    let style_start = match svg.find("<style>") {
        Some(pos) => pos,
        None => return svg.to_string(), // No style block; nothing to do
    };
    let inner_start = style_start + "<style>".len();
    let style_end = match svg[inner_start..].find("</style>") {
        Some(pos) => inner_start + pos,
        None => return svg.to_string(), // Malformed; bail out
    };
    let style_content = &svg[inner_start..style_end];
    let after_style = &svg[style_end + "</style>".len()..];

    // 2. Parse CSS variables from light-mode blocks only.
    // The style block structure is:
    //   @import ...;
    //   #{id} {
    //       --var1: value1;
    //       --var2: value2;
    //   }
    //   @media (prefers-color-scheme: dark) {
    //       #{id} { --var1: darkValue1; ... }
    //   }
    //   #{id}.dark-mode { --var1: darkValue1; ... }
    //   #{id} .class { property: var(--var1); }
    // We only want the first #{id} { ... } block (light mode variables).
    let mut vars: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut skip_block = false;
    let mut skip_depth = 0i32;
    let mut in_light_block = false;
    let mut light_block_depth = 0i32;

    for line in style_content.lines() {
        let trimmed = line.trim().to_string();

        // Detect @media (prefers-color-scheme: dark) — skip entire block
        if !skip_block
            && trimmed.contains("@media")
            && trimmed.contains("prefers-color-scheme: dark")
        {
            skip_block = true;
            skip_depth = 0;
            // Count braces on this line
            for ch in trimmed.chars() {
                if ch == '{' {
                    skip_depth += 1;
                } else if ch == '}' {
                    skip_depth -= 1;
                }
            }
            if skip_depth <= 0 {
                skip_block = false;
                skip_depth = 0;
            }
            continue;
        }

        // Detect .dark-mode class — skip entire block
        if !skip_block && trimmed.contains(".dark-mode") {
            skip_block = true;
            skip_depth = 0;
            for ch in trimmed.chars() {
                if ch == '{' {
                    skip_depth += 1;
                } else if ch == '}' {
                    skip_depth -= 1;
                }
            }
            if skip_depth <= 0 {
                skip_block = false;
                skip_depth = 0;
            }
            continue;
        }

        // If we're inside a skip block, track brace depth
        if skip_block {
            for ch in trimmed.chars() {
                if ch == '{' {
                    skip_depth += 1;
                } else if ch == '}' {
                    skip_depth -= 1;
                }
            }
            if skip_depth <= 0 {
                skip_block = false;
                skip_depth = 0;
            }
            continue;
        }

        // Detect the start of a light-mode selector block: #{id} {
        // This is a line that starts with # (after trim) and contains '{'
        if !in_light_block
            && trimmed.starts_with('#')
            && !trimmed.contains(".dark-mode")
            && trimmed.contains('{')
        {
            in_light_block = true;
            light_block_depth = 0;
            for ch in trimmed.chars() {
                if ch == '{' {
                    light_block_depth += 1;
                } else if ch == '}' {
                    light_block_depth -= 1;
                }
            }
            continue;
        }

        // Inside the light-mode block, parse all lines for --variable: value; definitions
        if in_light_block {
            parse_var_from_line(&trimmed, &mut vars);
            for ch in trimmed.chars() {
                if ch == '{' {
                    light_block_depth += 1;
                } else if ch == '}' {
                    light_block_depth -= 1;
                }
            }
            if light_block_depth <= 0 {
                in_light_block = false;
                light_block_depth = 0;
            }
            continue;
        }

        // Skip @import, @keyframes, and other at-rules
        if trimmed.starts_with('@') {
            continue;
        }
    }

    // 3. Build result: everything before <style> + everything after </style>
    let before_style = &svg[..style_start];
    let mut result = before_style.to_string();
    result.push_str(after_style);

    // 4. Replace var(--name) with resolved values
    for (name, value) in &vars {
        let var_ref = format!("var(--{})", name);
        result = result.replace(&var_ref, value);
    }

    result
}

/// Helper: extract CSS variable definitions from a line.
/// Handles both single-variable lines (`--foo: #abc;`) and inline variables
/// (`--foo: #abc; --bar: #def;`).
fn parse_var_from_line(line: &str, vars: &mut std::collections::HashMap<String, String>) {
    // Split by semicolons to handle multiple variables on the same line
    for part in line.split(';') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Look for --variable: value pattern
        if let Some(stripped) = trimmed.strip_prefix("--") {
            if let Some(colon_pos) = stripped.find(':') {
                let name = stripped[..colon_pos].trim().to_string();
                let value = stripped[colon_pos + 1..].trim().to_string();
                if !name.is_empty()
                    && !value.is_empty()
                    && name.starts_with(|c: char| c.is_alphabetic() || c == '_')
                {
                    vars.insert(name, value);
                }
            }
        }
    }
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

#[cfg(test)]
mod pdf_tests {
    use super::*;

    #[test]
    fn test_resolve_for_pdf_replaces_vars_and_removes_style() {
        let svg = r#"<svg>
<style>
#chart1 {
    --bg: #F6F8FB;
    --text: #17202A;
}
@media (prefers-color-scheme: dark) {
    #chart1 {
        --bg: #111827;
        --text: #F9FAFB;
    }
}
#chart1.dark-mode {
    --bg: #111827;
    --text: #F9FAFB;
}
#chart1 .title { fill: var(--text); }
</style>
<rect fill="var(--bg)"/>
<text fill="var(--text)">Hello</text>
</svg>"#;

        let result = resolve_for_pdf(svg);

        // Should not contain var() references
        assert!(
            !result.contains("var("),
            "No var() references should remain"
        );
        // Should not contain <style> block
        assert!(!result.contains("<style>"), "Style block should be removed");
        // Should contain resolved values
        assert!(
            result.contains("#F6F8FB"),
            "Light-mode --bg value should be present"
        );
        assert!(
            result.contains("#17202A"),
            "Light-mode --text value should be present"
        );
        // Should NOT contain dark-mode values
        assert!(
            !result.contains("#111827"),
            "Dark-mode --bg should not be used"
        );
        assert!(
            !result.contains("#F9FAFB"),
            "Dark-mode --text should not be used"
        );
    }

    #[test]
    fn test_resolve_for_pdf_no_style_block() {
        let svg = r#"<svg><rect fill="red"/></svg>"#;
        let result = resolve_for_pdf(svg);
        assert_eq!(result, svg);
    }

    #[test]
    fn test_resolve_for_pdf_rgba_values() {
        let svg = r#"<svg>
<style>
#id1 {
    --surface: rgba(255,255,255,0.74);
}
</style>
<rect fill="var(--surface)"/>
</svg>"#;

        let result = resolve_for_pdf(svg);
        assert!(!result.contains("var("));
        assert!(result.contains("rgba(255,255,255,0.74)"));
    }

    #[test]
    fn test_resolve_for_pdf_bar_chart_structure() {
        // Simulate the actual bar chart SVG structure with gradients
        let svg = r#"<svg>
<defs>
    <linearGradient id="chart1__premiumBackground" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="var(--premium-bg-0)"/>
        <stop offset="42%" stop-color="var(--premium-bg-42)"/>
        <stop offset="100%" stop-color="var(--premium-bg-100)"/>
    </linearGradient>
    <linearGradient id="chart1__glassSurface" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="var(--surface)" stop-opacity="var(--glass-op)"/>
        <stop offset="100%" stop-color="var(--surface)" stop-opacity="0.64"/>
    </linearGradient>
</defs>
<style>
    @import url('https://fonts.googleapis.com/css2?family=Inter');
    #chart1 {
        --bg: #F6F8FB;
        --surface: rgba(255,255,255,0.74);
        --text: #17202A;
        --premium-bg-0: #F7FAFF;
        --premium-bg-42: #EEF4FB;
        --premium-bg-100: #F9FAFB;
        --glass-op: 0.88;
    }
    @media (prefers-color-scheme: dark) {
        #chart1 {
            --bg: #111827;
            --surface: rgba(31, 41, 55, 0.74);
            --text: #F9FAFB;
            --premium-bg-0: #0F172A;
            --premium-bg-42: #111827;
            --premium-bg-100: #1F2937;
            --glass-op: 0.2;
        }
    }
    #chart1.dark-mode {
        --bg: #111827;
        --surface: rgba(31, 41, 55, 0.74);
        --text: #F9FAFB;
        --premium-bg-0: #0F172A;
        --premium-bg-42: #111827;
        --premium-bg-100: #1F2937;
        --glass-op: 0.2;
    }
    #chart1 .title { fill: var(--text); }
</style>
<rect width="100%" height="100%" fill="url(#chart1__premiumBackground)"/>
<rect x="36" y="34" width="888" height="492" fill="url(#chart1__glassSurface)"/>
<text class="title">Test</text>
</svg>"#;

        let result = resolve_for_pdf(svg);

        // Should not contain var() references
        assert!(
            !result.contains("var("),
            "No var() references should remain"
        );
        // Should not contain <style> block
        assert!(!result.contains("<style>"), "Style block should be removed");
        // Should contain resolved light-mode values
        assert!(
            result.contains("#F7FAFF"),
            "Light-mode --premium-bg-0 should be present"
        );
        assert!(
            result.contains("#EEF4FB"),
            "Light-mode --premium-bg-42 should be present"
        );
        assert!(
            result.contains("#F9FAFB"),
            "Light-mode --premium-bg-100 should be present"
        );
        assert!(
            result.contains("rgba(255,255,255,0.74)"),
            "Light-mode --surface should be present"
        );
        assert!(
            result.contains("0.88"),
            "Light-mode --glass-op should be present"
        );
        // Should NOT contain dark-mode values
        assert!(
            !result.contains("#0F172A"),
            "Dark-mode --premium-bg-0 should not be used"
        );
        assert!(
            !result.contains("#111827"),
            "Dark-mode --premium-bg-42 should not be used"
        );
        assert!(
            !result.contains("rgba(31, 41, 55, 0.74)"),
            "Dark-mode --surface should not be used"
        );
    }

    #[test]
    fn test_resolve_for_pdf_pie_chart_inline_vars() {
        // Simulate the pie chart style block where variables are on the same line
        let svg = r#"<svg>
<style>
    @import url('https://fonts.googleapis.com/css2?family=Lexend');
    #chart1 { 
        font-family: 'Lexend', ui-sans-serif, system-ui, sans-serif;
        --donut-bg-1: #f8fafc; --donut-bg-2: #eef4f8; --donut-bg-3: #f9f4e8;
        --donut-h1: #ffffff; --donut-ho1: 0.72;
    }
    @media (prefers-color-scheme: dark) {
        #chart1 {
            --donut-bg-1: #08111d; --donut-bg-2: #102033; --donut-bg-3: #11180f;
            --donut-h1: #4ade80; --donut-ho1: 0.18;
        }
    }
    #chart1.dark-mode {
        --donut-bg-1: #08111d; --donut-bg-2: #102033; --donut-bg-3: #11180f;
        --donut-h1: #4ade80; --donut-ho1: 0.18;
    }
    #chart1 .title { fill: var(--donut-h1); }
</style>
<rect fill="var(--donut-bg-1)"/>
<rect fill="var(--donut-bg-2)"/>
<rect fill="var(--donut-bg-3)"/>
<text fill="var(--donut-h1)">Test</text>
<text opacity="var(--donut-ho1)">Opacity</text>
</svg>"#;

        let result = resolve_for_pdf(svg);

        // Should not contain var() references
        assert!(
            !result.contains("var("),
            "No var() references should remain"
        );
        // Should not contain <style> block
        assert!(!result.contains("<style>"), "Style block should be removed");
        // Should contain resolved light-mode values
        assert!(
            result.contains("#f8fafc"),
            "Light-mode --donut-bg-1 should be present"
        );
        assert!(
            result.contains("#eef4f8"),
            "Light-mode --donut-bg-2 should be present"
        );
        assert!(
            result.contains("#f9f4e8"),
            "Light-mode --donut-bg-3 should be present"
        );
        assert!(
            result.contains("#ffffff"),
            "Light-mode --donut-h1 should be present"
        );
        assert!(
            result.contains("0.72"),
            "Light-mode --donut-ho1 should be present"
        );
        // Should NOT contain dark-mode values
        assert!(
            !result.contains("#08111d"),
            "Dark-mode --donut-bg-1 should not be used"
        );
        assert!(
            !result.contains("#102033"),
            "Dark-mode --donut-bg-2 should not be used"
        );
        assert!(
            !result.contains("#4ade80"),
            "Dark-mode --donut-h1 should not be used"
        );
    }

    #[test]
    fn test_resolve_for_pdf_line_chart() {
        // Simulate the actual line chart SVG structure
        let svg = r#"<svg>
<defs>
    <linearGradient id="id_abc_apple_bg" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="var(--bg-0)"/><stop offset="46%" stop-color="var(--bg-46)"/><stop offset="100%" stop-color="var(--bg-100)"/>
    </linearGradient>
    <linearGradient id="id_abc_plot_glass" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="var(--plot-surface-0)" stop-opacity="var(--plot-surface-op-0)"/><stop offset="100%" stop-color="var(--plot-surface-100)" stop-opacity="var(--plot-surface-op-100)"/>
    </linearGradient>
    <linearGradient id="id_abc_line_grad_0" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="var(--line-pal-0-1)" stop-opacity="0.9"/>
        <stop offset="100%" stop-color="var(--line-pal-0-2)"/>
    </linearGradient>
    <linearGradient id="id_abc_area_grad_0" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="var(--line-pal-0-1)" stop-opacity="0.25"/>
        <stop offset="100%" stop-color="var(--line-pal-0-2)" stop-opacity="0.0"/>
    </linearGradient>
    <filter id="id_abc_apple_card_shadow" x="-20%" y="-20%" width="140%" height="150%">
        <feDropShadow dx="0" dy="22" stdDeviation="28" flood-color="var(--shadow-flood)" flood-opacity="var(--shadow-op)"/>
    </filter>
</defs>
<style>
    @import url('https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans');
    #id_abc {
        --bg: #F8FAFC;
        --bg-0: #FFFFFF;
        --bg-46: #F1F5F9;
        --bg-100: #E2E8F0;
        --card-bg: rgba(255, 255, 255, 0.45);
        --plot-surface-0: #FFFFFF;
        --plot-surface-100: #F8FAFC;
        --plot-surface-op-0: 0.85;
        --plot-surface-op-100: 0.50;
        --text: #141414;
        --shadow-flood: #0F172A;
        --shadow-op: 0.10;
        --line-pal-0-1: #4B7BFF;
        --line-pal-0-2: #1856FF;
        --line-pal-0-stroke: #1856FF;
        --point-bg: #FFFFFF;
    }
    @media (prefers-color-scheme: dark) {
        #id_abc {
            --bg: #0B0F19;
            --bg-0: #0B0F19;
            --bg-46: #111827;
            --bg-100: #1E293B;
            --card-bg: rgba(17, 24, 39, 0.65);
            --plot-surface-0: #1E293B;
            --plot-surface-100: #0F172A;
            --plot-surface-op-0: 0.85;
            --plot-surface-op-100: 0.65;
            --text: #F8FAFC;
            --shadow-flood: #000000;
            --shadow-op: 0.40;
            --line-pal-0-1: #60A5FA;
            --line-pal-0-2: #1856FF;
            --line-pal-0-stroke: #4B7BFF;
            --point-bg: #1E293B;
        }
    }
    #id_abc.dark-mode {
        --bg: #0B0F19;
        --bg-0: #0B0F19;
        --bg-46: #111827;
        --bg-100: #1E293B;
        --card-bg: rgba(17, 24, 39, 0.65);
        --plot-surface-0: #1E293B;
        --plot-surface-100: #0F172A;
        --plot-surface-op-0: 0.85;
        --plot-surface-op-100: 0.65;
        --text: #F8FAFC;
        --shadow-flood: #000000;
        --shadow-op: 0.40;
        --line-pal-0-1: #60A5FA;
        --line-pal-0-2: #1856FF;
        --line-pal-0-stroke: #4B7BFF;
        --point-bg: #1E293B;
    }
    #id_abc .chart-text { fill: var(--text); }
    #id_abc .chart-background { fill: url(#id_abc_apple_bg); }
    #id_abc .plot-surface { fill: url(#id_abc_plot_glass); }
</style>
<rect width="100%" height="100%" fill="url(#id_abc_apple_bg)"/>
<rect fill="var(--card-bg)"/>
<rect class="plot-surface"/>
<path stroke="url(#id_abc_line_grad_0)"/>
<path fill="url(#id_abc_area_grad_0)"/>
<circle fill="var(--point-bg)" stroke="var(--line-pal-0-stroke)"/>
<text class="chart-text">Title</text>
</svg>"#;

        let result = resolve_for_pdf(svg);

        // Should not contain var() references
        assert!(
            !result.contains("var("),
            "No var() references should remain. Result:\n{}",
            result
        );
        // Should not contain <style> block
        assert!(!result.contains("<style>"), "Style block should be removed");
        // Should contain resolved light-mode values
        assert!(
            result.contains("#FFFFFF"),
            "Light-mode --bg-0 should be present"
        );
        assert!(
            result.contains("#F1F5F9"),
            "Light-mode --bg-46 should be present"
        );
        assert!(
            result.contains("#E2E8F0"),
            "Light-mode --bg-100 should be present"
        );
        assert!(
            result.contains("#4B7BFF"),
            "Light-mode --line-pal-0-1 should be present"
        );
        assert!(
            result.contains("#1856FF"),
            "Light-mode --line-pal-0-2 should be present"
        );
        assert!(
            result.contains("rgba(255, 255, 255, 0.45)"),
            "Light-mode --card-bg should be present"
        );
        // Should NOT contain dark-mode values
        assert!(
            !result.contains("#0B0F19"),
            "Dark-mode --bg-0 should not be used"
        );
        assert!(
            !result.contains("#1E293B"),
            "Dark-mode --plot-surface-0 should not be used"
        );
    }

    #[test]
    fn test_force_light_mode_replaces_use_dark_true() {
        let dsl = "[docops,bar, useDark=true, controls=false]\n----\ndata\n----";
        let result = force_light_mode(dsl);
        assert!(
            result.contains("useDark=false"),
            "useDark should be set to false"
        );
        assert!(
            result.contains("controls=false"),
            "other controls should be preserved"
        );
        assert!(
            result.contains("[docops, bar"),
            "header structure should be preserved"
        );
    }

    #[test]
    fn test_force_light_mode_adds_use_dark_when_missing() {
        let dsl = "[docops,pie]\n----\ndata\n----";
        let result = force_light_mode(dsl);
        assert!(
            result.contains("useDark=false"),
            "useDark=false should be added"
        );
    }

    #[test]
    fn test_force_light_mode_ignores_non_envelope_input() {
        let dsl = "just plain text without envelope";
        let result = force_light_mode(dsl);
        assert_eq!(result, dsl, "non-envelope input should be unchanged");
    }
}
