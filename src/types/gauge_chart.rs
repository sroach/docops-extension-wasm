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
    let subtitle = cfg.get("subtitle").map(String::as_str).unwrap_or("Performance metric");
    let min_val = cfg.get("min").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let max_val = cfg.get("max").and_then(|s| s.parse::<f64>().ok()).unwrap_or(100.0);
    let suffix = cfg.get("suffix").map(String::as_str).unwrap_or("%");
    let direction = cfg.get("direction").map(String::as_str).unwrap_or("normal");

    let val = data.points.first().map(|p| p.1).unwrap_or(0.0);
    
    let range = max_val - min_val;
    let normalized_val = if range > 0.0 {
        ((val - min_val) / range * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let status = cfg.get("status").map(String::as_str).map(|s| s.to_uppercase()).unwrap_or_else(|| {
        if direction == "inverse" {
            if normalized_val <= 33.0 {
                "HEALTHY".to_string()
            } else if normalized_val <= 66.0 {
                "WARNING".to_string()
            } else {
                "CRITICAL".to_string()
            }
        } else {
            if normalized_val >= 66.0 {
                "HEALTHY".to_string()
            } else if normalized_val >= 33.0 {
                "WARNING".to_string()
            } else {
                "CRITICAL".to_string()
            }
        }
    });

    let (s1_color, s3_color) = if direction == "inverse" {
        ("var(--success)", "var(--danger)")
    } else {
        ("var(--danger)", "var(--success)")
    };

    let status_color = if status == "HEALTHY" {
        "var(--success)"
    } else if status == "WARNING" {
        "var(--warning)"
    } else {
        "var(--danger)"
    };

    let chart_id = format!("gauge_{}", Uuid::new_v4().to_string().replace('-', "_"));

    // Gauge center (240, 244), radius 130
    // Needle angle: 180 deg to 0 deg
    let angle_deg = 180.0 - (normalized_val * 1.8);
    let angle_rad = angle_deg.to_radians();
    
    let needle_len = 102.0;
    let nx = 240.0 + needle_len * angle_rad.cos();
    let ny = 244.0 - needle_len * angle_rad.sin();
    
    let sx = 240.0 + (needle_len + 5.0) * angle_rad.cos();
    let sy = 244.0 - (needle_len + 5.0) * angle_rad.sin();

    let extra_class = if use_dark { " dark-mode" } else { "" };

    let tick_positions = [
        (104, 274, min_val),
        (170, 152, min_val + range * 0.25),
        (240, 122, min_val + range * 0.5),
        (310, 152, min_val + range * 0.75),
        (376, 274, max_val),
    ];
    let mut ticks_svg = String::new();
    for (tx, ty, tval) in tick_positions {
        ticks_svg.push_str(&format!(
            r#"  <text x="{tx}" y="{ty}" font-size="12" fill="var(--muted)" text-anchor="middle">{tval}</text>
"#
        ));
    }

    let svg = format!(
        r##"<svg width="480" height="360" viewBox="0 0 480 360" xmlns="http://www.w3.org/2000/svg" id="{chart_id}" class="gauge-container{extra_class}" role="img" aria-labelledby="{chart_id}_title {chart_id}_desc">
  <title id="{chart_id}_title">{title}</title>
  <desc id="{chart_id}_desc">{subtitle}</desc>
  <style>
    #{chart_id} {{
      --bg: {bg};
      --text: {text_color};
      --axis: {axis_color};
      --primary: {primary};
      --border: #E5E7EB;
      --success: #16A34A;
      --warning: #D97706;
      --danger: #DC2626;
      --muted: #6B7280;
    }}
    @media (prefers-color-scheme: dark) {{
      #{chart_id} {{
        --bg: {dark_bg};
        --text: {dark_text};
        --axis: {dark_axis};
        --primary: {dark_primary};
        --border: #334155;
        --muted: #94A3B8;
      }}
    }}
    #{chart_id}.dark-mode {{
      --bg: {dark_bg};
      --text: {dark_text};
      --axis: {dark_axis};
      --primary: {dark_primary};
      --border: #334155;
      --muted: #94A3B8;
    }}
    #{chart_id} text {{ font-family: Inter, system-ui, -apple-system, sans-serif; }}
  </style>

  <!-- Background -->
  <rect x="24" y="24" width="432" height="312" rx="28" fill="var(--bg)"/>
  <rect x="24" y="24" width="432" height="312" rx="28" fill="none" stroke="var(--border)"/>
  
  <text x="240" y="68" font-size="24" font-weight="700" fill="var(--text)" text-anchor="middle">{title}</text>
  <text x="240" y="96" font-size="14" fill="var(--muted)" text-anchor="middle">{subtitle}</text>

  <!-- Gauge segments -->
  <path d="M 110 244 A 130 130 0 0 1 176 131" fill="none" stroke="{s1_color}" stroke-width="24" stroke-linecap="round" />
  <path d="M 176 131 A 130 130 0 0 1 304 131" fill="none" stroke="var(--warning)" stroke-width="24" stroke-linecap="round" />
  <path d="M 304 131 A 130 130 0 0 1 370 244" fill="none" stroke="{s3_color}" stroke-width="24" stroke-linecap="round" />

  <!-- Inner cover -->
  <circle cx="240" cy="244" r="82" fill="var(--bg)"/>

  <!-- Tick labels -->
{ticks}
  <!-- Needle shadow -->
  <line x1="240" y1="244" x2="{sx:.1}" y2="{sy:.1}" stroke="var(--axis)" opacity="0.2" stroke-width="8" stroke-linecap="round" />

  <!-- Needle -->
  <line x1="240" y1="244" x2="{nx:.1}" y2="{ny:.1}" stroke="var(--text)" stroke-width="5" stroke-linecap="round" />

  <!-- Needle center -->
  <circle cx="240" cy="244" r="15" fill="var(--text)"/>
  <circle cx="240" cy="244" r="6" fill="var(--bg)"/>

  <!-- Value text -->
  <text x="240" y="306" font-size="36" font-weight="800" fill="var(--text)" text-anchor="middle">{val}{suffix}</text>
  <text x="240" y="328" font-size="12" font-weight="600" fill="{status_color}" text-anchor="middle">{status}</text>
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
        subtitle = escape(subtitle),
        ticks = ticks_svg,
        sx = sx,
        sy = sy,
        nx = nx,
        ny = ny,
        val = val,
        suffix = escape(suffix),
        extra_class = extra_class,
        status = escape(&status),
        status_color = status_color,
        s1_color = s1_color,
        s3_color = s3_color
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
subtitle=Subtitle Test
min=0
max=200
suffix=units
---
Result | 150
----"#;
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("Values Test"));
        assert!(svg.contains("Subtitle Test"));
        assert!(svg.contains(">150units</text>"));
        assert!(svg.contains(">0</text>"));
        assert!(svg.contains(">100</text>")); // Mid value
        assert!(svg.contains(">200</text>"));
        assert!(svg.contains("width=\"480\""));
        assert!(svg.contains("height=\"360\""));
    }

    #[test]
    fn test_render_gauge_inverse() {
        let body = r#"----
title=Latency
direction=inverse
min=0
max=100
suffix=ms
---
Latency | 10
----"#;
        let svg = render(body, &HashMap::new()).unwrap();
        // In inverse mode, low value (10) should be HEALTHY
        assert!(svg.contains(">HEALTHY</text>"));
        // Segment 1 (low) should be success color
        assert!(svg.contains("stroke=\"var(--success)\""));
        // Segment 3 (high) should be danger color
        assert!(svg.contains("stroke=\"var(--danger)\""));

        let body_high = r#"----
title=Latency
direction=inverse
min=0
max=100
suffix=ms
---
Latency | 90
----"#;
        let svg_high = render(body_high, &HashMap::new()).unwrap();
        // In inverse mode, high value (90) should be CRITICAL
        assert!(svg_high.contains(">CRITICAL</text>"));
    }
}
