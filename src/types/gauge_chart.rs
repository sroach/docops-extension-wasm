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

    let labels_str = cfg.get("labels").map(String::as_str).unwrap_or("LOW,WATCH,HEALTHY");
    let labels: Vec<String> = labels_str.split(',')
        .map(|s| s.trim().to_uppercase())
        .collect();
    
    let l1 = labels.get(0).cloned().unwrap_or_else(|| "LOW".to_string());
    let l2 = labels.get(1).cloned().unwrap_or_else(|| "WATCH".to_string());
    let l3 = labels.get(2).cloned().unwrap_or_else(|| "HEALTHY".to_string());

    let (level_by_val, status_default) = if direction == "inverse" {
        if normalized_val <= 33.0 { (3, l3.clone()) }
        else if normalized_val <= 66.0 { (2, l2.clone()) }
        else { (1, l1.clone()) }
    } else {
        if normalized_val >= 66.0 { (3, l3.clone()) }
        else if normalized_val >= 33.0 { (2, l2.clone()) }
        else { (1, l1.clone()) }
    };

    let status = cfg.get("status").map(String::as_str).map(|s| s.to_uppercase()).unwrap_or(status_default);

    let level = if status == l3 || status == "HEALTHY" {
        3
    } else if status == l2 || status == "WATCH" || status == "WARNING" {
        2
    } else if status == l1 || status == "LOW" || status == "CRITICAL" {
        1
    } else {
        level_by_val
    };

    let (s1_start, s1_stop, s3_start, s3_stop) = if direction == "inverse" {
        ("#86EFAC", "#16A34A", "#FCA5A5", "#DC2626")
    } else {
        ("#FCA5A5", "#DC2626", "#86EFAC", "#16A34A")
    };
    let (s2_start, s2_stop) = ("#FBBF24", "#D97706");
    let s1_color = s1_stop;
    let s2_color = s2_stop;
    let s3_color = s3_stop;

    let (s1_label, s3_label) = if direction == "inverse" {
        (l3.as_str(), l1.as_str())
    } else {
        (l1.as_str(), l3.as_str())
    };
    let s2_label = l2.as_str();

    let (status_bg, status_border, status_text) = if level == 3 {
        ("#F0FDF4", "#DCFCE7", "#16A34A")
    } else if level == 2 {
        ("#FFFBEB", "#FEF3C7", "#D97706")
    } else {
        ("#FEF2F2", "#FEE2E2", "#DC2626")
    };

    let (dark_status_bg, dark_status_border, dark_status_text) = if level == 3 {
        ("#064E3B", "#065F46", "#4ADE80")
    } else if level == 2 {
        ("#78350F", "#92400E", "#FBBF24")
    } else {
        ("#7F1D1D", "#991B1B", "#F87171")
    };

    let chart_id = format!("gauge_{}", Uuid::new_v4().to_string().replace('-', "_"));

    // Gauge center (240, 260), radius 130
    let cy = 260.0;
    let cx = 240.0;
    // Needle angle: -90 (left) to 90 (right), 0 is up
    let rotate = (normalized_val * 1.8) - 90.0;
    
    let extra_class = if use_dark { " dark-mode" } else { "" };

    let tick_positions = [
        (104, 290, min_val),
        (170, 168, min_val + range * 0.25),
        (240, 138, min_val + range * 0.5),
        (310, 168, min_val + range * 0.75),
        (376, 290, max_val),
    ];
    let mut ticks_svg = String::new();
    for (tx, ty, tval) in tick_positions {
        ticks_svg.push_str(&format!(
            r#"  <text x="{tx}" y="{ty}" font-size="12" fill="var(--muted)" text-anchor="middle">{tval}</text>
"#
        ));
    }

    let svg = format!(
        r##"<svg width="480" height="440" viewBox="0 0 480 440" xmlns="http://www.w3.org/2000/svg" id="{chart_id}" class="gauge-container{extra_class}" role="img" aria-labelledby="{chart_id}_title {chart_id}_desc">
  <title id="{chart_id}_title">{title}</title>
  <desc id="{chart_id}_desc">{subtitle}</desc>
  <defs>
    <filter id="{chart_id}_shadow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur in="SourceAlpha" stdDeviation="4"/>
      <feOffset dx="0" dy="4" result="offsetblur"/>
      <feFlood flood-color="#000000" flood-opacity="0.08"/>
      <feComposite in2="offsetblur" operator="in"/>
      <feMerge>
        <feMergeNode/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    <filter id="{chart_id}_softGlow" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur in="SourceGraphic" stdDeviation="2" result="blur"/>
      <feMerge>
        <feMergeNode in="blur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    <linearGradient id="{chart_id}_s1_arc" x1="72" y1="260" x2="176" y2="136" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="{s1_start}"/>
      <stop offset="100%" stop-color="{s1_stop}"/>
    </linearGradient>
    <linearGradient id="{chart_id}_s2_arc" x1="168" y1="136" x2="312" y2="136" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="{s2_start}"/>
      <stop offset="100%" stop-color="{s2_stop}"/>
    </linearGradient>
    <linearGradient id="{chart_id}_s3_arc" x1="304" y1="136" x2="408" y2="260" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="{s3_start}"/>
      <stop offset="100%" stop-color="{s3_stop}"/>
    </linearGradient>
  </defs>
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
      --status-bg: {status_bg};
      --status-border: {status_border};
      --status-text: {status_text};
    }}
    @media (prefers-color-scheme: dark) {{
      #{chart_id} {{
        --bg: {dark_bg};
        --text: {dark_text};
        --axis: {dark_axis};
        --primary: {dark_primary};
        --border: #334155;
        --muted: #94A3B8;
        --status-bg: {dark_status_bg};
        --status-border: {dark_status_border};
        --status-text: {dark_status_text};
      }}
    }}
    #{chart_id}.dark-mode {{
      --bg: {dark_bg};
      --text: {dark_text};
      --axis: {dark_axis};
      --primary: {dark_primary};
      --border: #334155;
      --muted: #94A3B8;
      --status-bg: {dark_status_bg};
      --status-border: {dark_status_border};
      --status-text: {dark_status_text};
    }}
    #{chart_id} text {{ font-family: Inter, system-ui, -apple-system, sans-serif; }}
  </style>

  <!-- Background Card -->
  <g filter="url(#{chart_id}_shadow)">
    <rect x="24" y="24" width="432" height="392" rx="24" fill="var(--bg)"/>
    <rect x="24.5" y="24.5" width="431" height="391" rx="23.5" fill="none" stroke="var(--border)" stroke-width="1"/>
  </g>
  
  <text x="240" y="60" font-size="24" font-weight="700" fill="var(--text)" text-anchor="middle">{title}</text>
  <text x="240" y="88" font-size="14" font-weight="400" fill="var(--muted)" text-anchor="middle">{subtitle}</text>

  <!-- Decorative Dial Circles -->
  <circle cx="240" cy="260" r="150" fill="none" stroke="var(--border)" stroke-width="0.5" opacity="0.4"/>
  <circle cx="240" cy="260" r="120" fill="none" stroke="var(--border)" stroke-width="0.5" opacity="0.4"/>

  <!-- Gauge segments -->
  <path d="M 110 260 A 130 130 0 0 1 370 260" fill="none" stroke="var(--border)" stroke-width="30" stroke-linecap="round" opacity="0.1" />
  <path d="M 110 260 A 130 130 0 0 1 175 147.4" fill="none" stroke="url(#{chart_id}_s1_arc)" stroke-width="30" stroke-linecap="round" />
  <path d="M 175 147.4 A 130 130 0 0 1 305 147.4" fill="none" stroke="url(#{chart_id}_s2_arc)" stroke-width="30" stroke-linecap="round" />
  <path d="M 305 147.4 A 130 130 0 0 1 370 260" fill="none" stroke="url(#{chart_id}_s3_arc)" stroke-width="30" stroke-linecap="round" />

  <!-- Region Labels -->
  <text x="144" y="210" font-size="9" font-weight="800" fill="{s1_color}" text-anchor="middle" letter-spacing="0.05em">{s1_label}</text>
  <text x="240" y="160" font-size="9" font-weight="800" fill="{s2_color}" text-anchor="middle" letter-spacing="0.05em">{s2_label}</text>
  <text x="336" y="210" font-size="9" font-weight="800" fill="{s3_color}" text-anchor="middle" letter-spacing="0.05em">{s3_label}</text>

  <!-- Inner cover -->
  <circle cx="240" cy="260" r="82" fill="var(--bg)"/>

  <!-- Tick labels -->
{ticks}
  <!-- Needle -->
  <g transform="translate({cx}, {cy}) rotate({rotate})">
    <line x1="0" y1="0" x2="0" y2="-105" stroke="var(--text)" opacity="0.08" stroke-width="11" stroke-linecap="round" />
    <line x1="0" y1="0" x2="0" y2="-105" stroke="var(--text)" stroke-width="6" stroke-linecap="round" />
    <g filter="url(#{chart_id}_softGlow)">
      <circle cx="0" cy="-105" r="4.5" fill="#3B82F6"/>
    </g>
  </g>

  <!-- Needle center -->
  <circle cx="{cx}" cy="{cy}" r="21" fill="var(--bg)" stroke="var(--border)" stroke-width="1"/>
  <circle cx="{cx}" cy="{cy}" r="15" fill="var(--text)"/>
  <circle cx="{cx}" cy="{cy}" r="6" fill="var(--bg)"/>

  <!-- Value text -->
  <text x="240" y="350" font-size="36" font-weight="800" fill="var(--text)" text-anchor="middle">{val}{suffix}</text>
  
  <!-- Status Pill -->
  <g transform="translate(180, 370)">
    <rect x="0" y="0" width="120" height="28" rx="14" fill="var(--status-bg)" stroke="var(--status-border)" stroke-width="1"/>
    <circle cx="20" cy="14" r="5" fill="var(--status-text)"/>
    <text x="68" y="19" font-size="12" font-weight="700" fill="var(--status-text)" text-anchor="middle" letter-spacing="0.05em">{status}</text>
  </g>

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
        rotate = rotate,
        cx = cx,
        cy = cy,
        val = val,
        suffix = escape(suffix),
        extra_class = extra_class,
        s1_label = s1_label,
        s2_label = s2_label,
        s3_label = s3_label,
        status = escape(&status),
        status_bg = status_bg,
        status_border = status_border,
        status_text = status_text,
        dark_status_bg = dark_status_bg,
        dark_status_border = dark_status_border,
        dark_status_text = dark_status_text,
        s1_color = s1_color,
        s2_color = s2_color,
        s3_color = s3_color,
        s1_start = s1_start,
        s1_stop = s1_stop,
        s2_start = s2_start,
        s2_stop = s2_stop,
        s3_start = s3_start,
        s3_stop = s3_stop
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
        assert!(svg.contains("height=\"440\""));
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
        // Segment 1 (low) should be success color (gradient)
        assert!(svg.contains("_s1_arc)\""));
        // Segment 3 (high) should be danger color (gradient)
        assert!(svg.contains("_s3_arc)\""));

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
        // In inverse mode, high value (90) should be LOW
        assert!(svg_high.contains(">LOW</text>"));
    }

    #[test]
    fn test_render_gauge_custom_labels() {
        let body = r#"----
title=Custom Labels
labels=POOR,AVERAGE,GREAT
---
Score | 90
----"#;
        let svg = render(body, &HashMap::new()).unwrap();
        // In normal mode, 90 is GREAT (l3)
        assert!(svg.contains(">GREAT</text>"));
        assert!(svg.contains(">POOR</text>"));
        assert!(svg.contains(">AVERAGE</text>"));
    }
}
