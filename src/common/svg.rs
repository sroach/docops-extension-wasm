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
