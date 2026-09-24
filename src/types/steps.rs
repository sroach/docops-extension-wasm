use crate::common::svg::escape;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
struct StepItem {
    order: String,
    title: String,
    description: String,
    color: String,
    tag: Option<String>,
}

#[derive(Debug)]
struct StepsSpec {
    title: String,
    subtitle: String,
    footer: Option<String>,
    items: Vec<StepItem>,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let spec = parse_steps(body)?;
    Ok(render_svg(&spec, controls))
}

fn parse_steps(body: &str) -> Result<StepsSpec, String> {
    let trimmed = body.trim();
    if !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("Steps body must be wrapped in '---- ... ----'".into());
    }
    let inner = trimmed[4..trimmed.len() - 4].trim();

    let (config_part, table_part) = if let Some((c, t)) = inner.split_once("---") {
        (c.trim(), t.trim())
    } else {
        ("", inner)
    };

    let mut title = String::new();
    let mut subtitle = String::new();
    let mut footer = None;

    for line in config_part.lines() {
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "title" => title = v.trim().to_string(),
                "subtitle" => subtitle = v.trim().to_string(),
                "footer" => footer = Some(v.trim().to_string()),
                _ => {}
            }
        }
    }

    let mut items = Vec::new();
    for line in table_part.lines() {
        let line = line.trim();
        if line.is_empty() || line.to_lowercase().contains("order |") {
            continue;
        }
        if let Some(item) = parse_line(line) {
            items.push(item);
        }
    }

    Ok(StepsSpec {
        title,
        subtitle,
        footer,
        items,
    })
}

fn parse_line(line: &str) -> Option<StepItem> {
    let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
    if parts.len() >= 3 {
        Some(StepItem {
            order: parts[0].to_string(),
            title: parts[1].to_string(),
            description: parts[2].to_string(),
            color: parts.get(3).unwrap_or(&"#3B82F6").to_string(),
            tag: parts.get(4).map(|s| s.to_string()).filter(|s| !s.is_empty()),
        })
    } else {
        None
    }
}

fn wrap_text(text: &str, max_width_px: i32, font_size_px: i32) -> Vec<String> {
    if text.is_empty() || max_width_px <= 0 {
        return vec![text.to_string()];
    }
    let avg_char_width = font_size_px as f32 * 0.55;
    let max_chars = (max_width_px as f32 / avg_char_width).max(1.0) as usize;

    let words = text.split_whitespace();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else {
            if current_line.len() + 1 + word.len() <= max_chars {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

fn render_svg(spec: &StepsSpec, controls: &HashMap<String, String>) -> String {
    let use_dark = controls.get("useDark").map(|s| s == "true").unwrap_or(false);
    let id = Uuid::new_v4().simple().to_string()[..8].to_string();
    let id_full = format!("steps_{}", id);

    let n = spec.items.len();
    
    // Pre-calculate card heights and wrap descriptions
    let mut card_data = Vec::new();
    for item in &spec.items {
        let desc_lines = wrap_text(&item.description, 300, 14);
        let card_h = (desc_lines.len() as f32 * 22.0 + 85.0).max(120.0);
        card_data.push((desc_lines, card_h));
    }

    let width = ((n as f32 * 320.0) + 500.0).max(1200.0) as i32;
    let height = ((n as f32 * 180.0) + 450.0).max(800.0) as i32;

    let mut sb = String::new();

    sb.push_str(&format!(
        r##"<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" fill="none" xmlns="http://www.w3.org/2000/svg" id="{id_full}" role="graphics-document document" aria-labelledby="{id_full}_title {id_full}_desc">"##,
        width = width,
        height = height,
        id_full = id_full
    ));

    sb.push_str(&format!(r##"<title id="{id_full}_title">{}</title>"##, escape(&spec.title)));
    sb.push_str(&format!(r##"<desc id="{id_full}_desc">{}</desc>"##, escape(&spec.subtitle)));

    sb.push_str(&format!(r##"
<defs>
    <filter id="premiumShadow_{id}" x="-20%" y="-20%" width="140%" height="160%">
        <feDropShadow dx="0" dy="20" stdDeviation="30" flood-color="var(--shadow-color)" flood-opacity="var(--shadow-op)"/>
        <feDropShadow dx="0" dy="4" stdDeviation="8" flood-color="var(--shadow-color)" flood-opacity="var(--shadow-op-2)"/>
    </filter>
    
    <linearGradient id="isoTopGrad_{id}" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="white" stop-opacity="0.5"/>
        <stop offset="100%" stop-color="white" stop-opacity="0.1"/>
    </linearGradient>
    <linearGradient id="isoLeftGrad_{id}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="black" stop-opacity="0.3"/>
        <stop offset="100%" stop-color="black" stop-opacity="0.1"/>
    </linearGradient>
    <linearGradient id="isoRightGrad_{id}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="black" stop-opacity="0.15"/>
        <stop offset="100%" stop-color="black" stop-opacity="0.05"/>
    </linearGradient>

    <style>
        #{id_full} {{
            --bg-color: #F8FAFC;
            --text-primary: #111827;
            --text-secondary: #4B5563;
            --card-bg: rgba(255, 255, 255, 0.85);
            --card-border: rgba(229, 231, 235, 0.8);
            --shadow-color: #0F172A;
            --shadow-op: 0.08;
            --shadow-op-2: 0.04;
            --connector-color: #3B82F6;
            --tag-bg-op: 0.1;
            --indicator-stroke: rgba(255, 255, 255, 0.5);
        }}
        @media (prefers-color-scheme: dark) {{
            #{id_full} {{
                --bg-color: #0F172A;
                --text-primary: #F8FAFC;
                --text-secondary: #94A3B8;
                --card-bg: rgba(30, 41, 59, 0.7);
                --card-border: rgba(255, 255, 255, 0.1);
                --shadow-color: #000;
                --shadow-op: 0.4;
                --shadow-op-2: 0.2;
                --indicator-stroke: rgba(255, 255, 255, 0.2);
            }}
        }}
        {dark_override}

        .title_{id} {{ font-family: Inter, -apple-system, sans-serif; font-size: 48px; font-weight: 800; fill: var(--text-primary); letter-spacing: -0.04em; }}
        .subtitle_{id} {{ font-family: Inter, sans-serif; font-size: 18px; font-weight: 500; fill: var(--text-secondary); }}
        .card-title_{id} {{ font-family: Inter, sans-serif; font-size: 20px; font-weight: 700; fill: var(--text-primary); letter-spacing: -0.02em; }}
        .card-desc_{id} {{ font-family: Inter, sans-serif; font-size: 14px; font-weight: 450; fill: var(--text-secondary); line-height: 1.6; }}
        .step-num_{id} {{ font-family: Inter, sans-serif; font-size: 14px; font-weight: 800; fill: #fff; }}
        .tag-text_{id} {{ font-family: 'JetBrains Mono', monospace; font-size: 12px; font-weight: 700; }}
        .footer-text_{id} {{ font-family: Inter, sans-serif; font-size: 14px; font-weight: 600; fill: var(--text-primary); }}
        .glass_{id} {{ backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px); }}
    </style>
</defs>
"##, id = id, id_full = id_full, 
    dark_override = if use_dark { format!(r##"
        #{id_full} {{
            --bg-color: #0F172A;
            --text-primary: #F8FAFC;
            --text-secondary: #94A3B8;
            --card-bg: rgba(30, 41, 59, 0.7);
            --card-border: rgba(255, 255, 255, 0.1);
            --shadow-color: #000;
            --shadow-op: 0.4;
            --shadow-op-2: 0.2;
            --indicator-stroke: rgba(255, 255, 255, 0.2);
        }}"##, id_full = id_full) } else { "".to_string() }
    ));

    // Background
    sb.push_str(&format!(r##"<rect width="{width}" height="{height}" fill="var(--bg-color)" aria-hidden="true"/>"##, width = width, height = height));

    // Header
    sb.push_str(&format!(r##"
<g transform="translate(80, 100)">
    <text class="title_{id}">{title}</text>
    <text y="45" class="subtitle_{id}">{subtitle}</text>
</g>
"##, id = id, title = escape(&spec.title), subtitle = escape(&spec.subtitle)));

    // Positions
    let mut positions = Vec::new();
    let start_x = 120.0;
    let start_y = height as f32 - 180.0;
    let end_x = width as f32 - 500.0;
    let end_y = 220.0;

    for i in 0..n {
        let t = if n > 1 { i as f32 / (n - 1) as f32 } else { 0.5 };
        let px = start_x + (end_x - start_x) * t;
        let py = start_y + (end_y - start_y) * t;
        positions.push((px, py));
    }

    // Connector Line
    if n > 1 {
        let mut path_d = format!("M {} {}", positions[0].0 + 40.0, positions[0].1 + 20.0);
        for (px, py) in positions.iter().skip(1) {
            path_d.push_str(&format!(" L {} {}", px + 40.0, py + 20.0));
        }
        sb.push_str(&format!(r##"<path d="{}" stroke="var(--connector-color)" stroke-width="2.5" stroke-dasharray="6 6" stroke-linecap="round" opacity="0.4" fill="none" aria-hidden="true"/>"##, path_d));
    }

    // Render Steps
    for (i, item) in spec.items.iter().enumerate() {
        let (x, y) = positions[i];
        let (desc_lines, card_h) = &card_data[i];
        let color = &item.color;
        let order = escape(&item.order);
        let title = escape(&item.title);

        sb.push_str(&format!(r##"
<g transform="translate({x}, {y})" role="group" aria-label="Step {order}: {title}">
    <!-- Isometric Shape -->
    <g role="graphics-symbol" aria-roledescription="step indicator" tabindex="0" aria-label="Step {order}">
        <!-- Left Side -->
        <polygon points="-20,-30 -20,-10 40,20 40,0" fill="{color}"/>
        <polygon points="-20,-30 -20,-10 40,20 40,0" fill="url(#isoLeftGrad_{id})"/>
        
        <!-- Right Side -->
        <polygon points="100,-30 100,-10 40,20 40,0" fill="{color}"/>
        <polygon points="100,-30 100,-10 40,20 40,0" fill="url(#isoRightGrad_{id})"/>
        
        <!-- Top -->
        <polygon points="40,-60 100,-30 40,0 -20,-30" fill="{color}"/>
        <polygon points="40,-60 100,-30 40,0 -20,-30" fill="url(#isoTopGrad_{id})"/>
        
        <!-- Indicator Circle -->
        <circle cx="40" cy="-75" r="18" fill="{color}" filter="url(#premiumShadow_{id})"/>
        <circle cx="40" cy="-75" r="18" stroke="var(--indicator-stroke)" stroke-width="2" fill="none"/>
        <text x="40" y="-70" text-anchor="middle" class="step-num_{id}">{order}</text>
    </g>

    <!-- Details Card -->
    <g transform="translate(120, -100)" filter="url(#premiumShadow_{id})" role="region" aria-label="{title} details" tabindex="0">
        <rect width="360" height="{card_h}" rx="20" fill="var(--card-bg)" class="glass_{id}" stroke="var(--card-border)" stroke-width="1.5"/>
        <rect width="4" height="36" x="20" y="24" rx="2" fill="{color}"/>
        
        <text x="40" y="42" class="card-title_{id}">{title}</text>
        <text x="40" y="68" class="card-desc_{id}">
"##, x=x, y=y, color=color, order=order, title=title, id=id, card_h=card_h));

        for (j, line) in desc_lines.iter().enumerate() {
            sb.push_str(&format!(r##"<tspan x="40" dy="{}">{}</tspan>"##, if j == 0 { "0" } else { "1.6em" }, escape(line)));
        }

        sb.push_str("</text>");

        if let Some(tag) = &item.tag {
            let tag_width = (tag.len() as f32 * 8.5 + 24.0).max(60.0);
            let tag_x = 360.0 - tag_width - 20.0;
            sb.push_str(&format!(r##"
        <g transform="translate({tag_x}, 24)" role="status" aria-label="Tag: {tag}">
            <rect width="{tag_width}" height="28" rx="14" fill="{color}" fill-opacity="var(--tag-bg-op)"/>
            <text x="{tag_mid}" y="19" text-anchor="middle" fill="{color}" class="tag-text_{id}">{tag}</text>
        </g>
"##, tag_x=tag_x, tag_width=tag_width, tag_mid=tag_width/2.0, color=color, tag=escape(tag), id=id));
        }

        sb.push_str("</g></g>");
    }

    // Footer
    if let Some(footer) = &spec.footer {
        let footer_x = width as f32 - 400.0;
        let footer_y = height as f32 - 80.0;
        let footer_esc = escape(footer);
        sb.push_str(&format!(r##"
<g transform="translate({footer_x}, {footer_y})">
    <circle cx="20" cy="20" r="8" fill="#16A34A"/>
    <text x="40" y="25" class="footer-text_{id}">{footer_esc}</text>
</g>
"##, footer_esc = footer_esc, id = id));
    }

    sb.push_str("</svg>");
    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_steps() {
        let body = r##"----
title=Test Journey
subtitle=Test Subtitle
footer=Test Footer
---
Order | Title | Description | Color | Tag
1 | Step 1 | Desc 1 | #FF0000 | +10%
2 | Step 2 | Desc 2 | #00FF00 |
----"##;
        let spec = parse_steps(body).unwrap();
        assert_eq!(spec.title, "Test Journey");
        assert_eq!(spec.subtitle, "Test Subtitle");
        assert_eq!(spec.footer, Some("Test Footer".to_string()));
        assert_eq!(spec.items.len(), 2);
        assert_eq!(spec.items[0].order, "1");
        assert_eq!(spec.items[0].color, "#FF0000");
        assert_eq!(spec.items[0].tag, Some("+10%".to_string()));
        assert_eq!(spec.items[1].tag, None);
    }

    #[test]
    fn test_render_steps_premium() {
        let body = r##"----
title=Enterprise Journey
subtitle=Premium Design
---
1 | Start | Description | #3B82F6 | Tag
----"##;
        let result = render(body, &HashMap::new()).unwrap();
        
        // Check for premium features
        assert!(result.contains("role=\"graphics-document document\""));
        assert!(result.contains("aria-labelledby=\"steps_"));
        assert!(result.contains("--bg-color: #F8FAFC;"));
        assert!(result.contains("prefers-color-scheme: dark"));
        assert!(result.contains("isoTopGrad_"));
        assert!(result.contains("isoLeftGrad_"));
        assert!(result.contains("isoRightGrad_"));
        assert!(result.contains("premiumShadow_"));
        assert!(result.contains("role=\"group\""));
        assert!(result.contains("role=\"region\""));
        assert!(result.contains("role=\"graphics-symbol\""));
        assert!(result.contains("aria-roledescription=\"step indicator\""));
        assert!(result.contains("tabindex=\"0\""));
    }

    #[test]
    fn test_generate_gen_files() {
        let full_sample = r#"[docops,steps]
----
title=Complex Multi-Stage Deployment
subtitle=A very long description of a complex process that involves multiple teams and stakeholders across different timezones and regions.
footer=Deployment successful across 12 clusters
---
Order | Title | Description | Color | Tag
1 | Initial Planning | Define requirements, goals, and key performance indicators for the entire project lifecycle. This involves multiple stakeholders. | #6EAEFF | Start
2 | Architecture Design | Create detailed system architecture, including data flow diagrams, security protocols, and infrastructure scaling plans. | #69DEE5 | 
3 | Security Audit | Perform a comprehensive security review of all proposed systems and processes to ensure compliance with global standards. | #F8BC95 | Critical
4 | Resource Allocation | Assign personnel and budget to various workstreams based on the project roadmap and identified risks. | #D9AEF8 |
5 | Beta Testing | Deploy to a subset of users to gather early feedback and identify potential bugs before full production rollout. | #B0A5FB | 
6 | Global Rollout | Execute the full deployment strategy across all target markets and regions simultaneously. | #3B82F6 | 
7 | Post-Launch Review | Analyze performance data and user feedback to identify areas for improvement in future iterations. | #8B5CF6 |
8 | Maintenance Phase | Continuous monitoring and support to ensure system stability and performance over the long term. | #16A34A | Ongoing
9 | Future Expansion | Plan for next-generation features and additional market entries based on the success of the current deployment. | #D97706 |
10 | Project Closure | Finalize all documentation and transition responsibilities to the operations team for ongoing management. | #10B981 | Done
----"#;
        let generated_svg = crate::generate_svg(full_sample);
        let _ = std::fs::write("gen/enterprise_advantage.svg", generated_svg);
    }
}