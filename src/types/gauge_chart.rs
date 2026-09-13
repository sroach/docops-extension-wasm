use crate::common::kv::parse_kv_body;
use crate::common::svg::{escape, theme};
use std::collections::HashMap;
use uuid::Uuid;

/// Renders a Gauge Chart.
/// Syntax:
/// [docops,gauge]
/// ----
/// title=Performance
/// min=0
/// max=100
/// suffix=%
/// ---
/// Score | 68
/// ----
pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let data = parse_kv_body(body)?;
    let cfg = &data.config;

    let use_dark = controls.get("useDark").map(|s| s == "true").unwrap_or(false)
        || cfg.get("theme").map(|s| s.as_str()) == Some("dark");
    
    let theme_name = cfg.get("theme").map(|s| s.as_str()).unwrap_or("default");
    let colors = theme(theme_name);
    let dark_colors = theme("dark");

    let title = cfg.get("title").map(String::as_str).unwrap_or("Gauge Chart");
    let min_val = cfg.get("min").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let max_val = cfg.get("max").and_then(|s| s.parse::<f64>().ok()).unwrap_or(100.0);
    let suffix = cfg.get("suffix").map(String::as_str).unwrap_or("%");

    let val = data.points.first().map(|p| p.1).unwrap_or(0.0);
    
    let range = max_val - min_val;
    let normalized_val = if range > 0.0 {
        ((val - min_val) / range * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let chart_id = format!("gauge_{}", Uuid::new_v4().to_string().replace('-', "_"));

    // Gauge center (200, 180), radius 130
    // Needle angle: 180 deg to 0 deg
    let angle_deg = 180.0 - (normalized_val * 1.8);
    let angle_rad = angle_deg.to_radians();
    
    let needle_len = 100.0;
    let nx = 200.0 + needle_len * angle_rad.cos();
    let ny = 180.0 - needle_len * angle_rad.sin();
    
    let sx = 200.0 + (needle_len + 5.0) * angle_rad.cos();
    let sy = 180.0 - (needle_len + 5.0) * angle_rad.sin();

    let extra_class = if use_dark { " dark-mode" } else { "" };

    let svg = format!(
        r##"<svg width="400" height="260" viewBox="0 0 400 260" xmlns="http://www.w3.org/2000/svg" id="{chart_id}" class="gauge-container{extra_class}">
  <style>
    #{chart_id} {{
      --bg: {bg};
      --text: {text_color};
      --axis: {axis_color};
      --primary: {primary};
    }}
    @media (prefers-color-scheme: dark) {{
      #{chart_id} {{
        --bg: {dark_bg};
        --text: {dark_text};
        --axis: {dark_axis};
        --primary: {dark_primary};
      }}
    }}
    #{chart_id}.dark-mode {{
      --bg: {dark_bg};
      --text: {dark_text};
      --axis: {dark_axis};
      --primary: {dark_primary};
    }}
    #{chart_id} text {{ font-family: system-ui, -apple-system, sans-serif; }}
  </style>

  <!-- Background -->
  <rect width="400" height="260" fill="var(--bg)" rx="12"/>
  
  <text x="200" y="30" font-size="18" font-weight="700" fill="var(--text)" text-anchor="middle">{title}</text>

  <!-- Gauge segments -->
  <path d="M 70 180 A 130 130 0 0 1 156 57" fill="none" stroke="#ef4444" stroke-width="28" stroke-linecap="round" />
  <path d="M 156 57 A 130 130 0 0 1 244 57" fill="none" stroke="#f59e0b" stroke-width="28" stroke-linecap="round" />
  <path d="M 244 57 A 130 130 0 0 1 330 180" fill="none" stroke="#22c55e" stroke-width="28" stroke-linecap="round" />

  <!-- Inner cover -->
  <circle cx="200" cy="180" r="88" fill="var(--bg)"/>

  <!-- Tick labels -->
  <text x="62" y="205" font-size="14" fill="var(--axis)" text-anchor="middle">{min}</text>
  <text x="200" y="45" font-size="14" fill="var(--axis)" text-anchor="middle">{mid}</text>
  <text x="338" y="205" font-size="14" fill="var(--axis)" text-anchor="middle">{max}</text>

  <!-- Needle shadow -->
  <line x1="200" y1="180" x2="{sx:.1}" y2="{sy:.1}" stroke="var(--axis)" opacity="0.2" stroke-width="8" stroke-linecap="round" />

  <!-- Needle -->
  <line x1="200" y1="180" x2="{nx:.1}" y2="{ny:.1}" stroke="var(--primary)" stroke-width="5" stroke-linecap="round" />

  <!-- Needle center -->
  <circle cx="200" cy="180" r="14" fill="var(--primary)"/>
  <circle cx="200" cy="180" r="6" fill="var(--bg)"/>

  <!-- Value text -->
  <text x="200" y="235" font-size="28" font-weight="800" fill="var(--text)" text-anchor="middle">{val}{suffix}</text>
</svg>"##,
        chart_id = chart_id,
        bg = colors.background,
        text_color = colors.text,
        axis_color = colors.axis,
        primary = colors.primary,
        dark_bg = dark_colors.background,
        dark_text = dark_colors.text,
        dark_axis = dark_colors.axis,
        dark_primary = dark_colors.primary,
        title = escape(title),
        min = min_val,
        mid = (min_val + max_val) / 2.0,
        max = max_val,
        sx = sx,
        sy = sy,
        nx = nx,
        ny = ny,
        val = val,
        suffix = escape(suffix),
        extra_class = extra_class
    );

    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_gauge_dark_mode_support() {
        let body = r#"----
title=Dark Mode Test
---
Score | 42
----"#;
        
        // Default mode (should have variables and media query)
        let svg_light = render(body, &HashMap::new()).unwrap();
        assert!(svg_light.contains("gauge_"));
        assert!(svg_light.contains("@media (prefers-color-scheme: dark)"));
        assert!(svg_light.contains("class=\"gauge-container\""));
        assert!(svg_light.contains("var(--bg)"));
        
        // Forced dark mode
        let mut controls = HashMap::new();
        controls.insert("useDark".to_string(), "true".to_string());
        let svg_dark = render(body, &controls).unwrap();
        assert!(svg_dark.contains("class=\"gauge-container dark-mode\""));
    }

    #[test]
    fn test_render_gauge_values() {
        let body = r#"----
title=Values Test
min=0
max=200
suffix=units
---
Result | 150
----"#;
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("Values Test"));
        assert!(svg.contains(">150units</text>"));
        assert!(svg.contains(">0</text>"));
        assert!(svg.contains(">100</text>")); // Mid value
        assert!(svg.contains(">200</text>"));
    }
}
