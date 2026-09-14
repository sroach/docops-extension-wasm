use crate::common::kv::parse_kv_header;
use crate::common::svg::escape;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct MetricsCard {
    pub title: String,
    pub metrics: Vec<MetricItem>,
    pub theme: String,
    pub use_glass: bool,
}

#[derive(Debug, Default)]
pub struct MetricItem {
    pub label: String,
    pub value: String,
    pub sublabel: String,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let metrics_card = parse_metrics_card(body)?;
    let use_dark = controls
        .get("useDark")
        .map(|s| s == "true")
        .unwrap_or(false)
        || metrics_card.theme == "dark";
    Ok(render_svg(&metrics_card, use_dark))
}

fn parse_metrics_card(body: &str) -> Result<MetricsCard, String> {
    let mut trimmed = body.trim();
    if trimmed.starts_with("----") {
        trimmed = trimmed[4..].trim();
    }
    if trimmed.ends_with("----") {
        trimmed = trimmed[..trimmed.len() - 4].trim();
    }

    let parts: Vec<&str> = trimmed.split("---").map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err("Empty metrics card body".into());
    }

    let mut card = MetricsCard::default();

    // Part 0: Header
    let header = parse_kv_header(parts[0]);
    card.title = header.get("title").cloned().unwrap_or_default();
    card.theme = header.get("theme").cloned().unwrap_or_default();
    card.use_glass = header.get("useGlass").map(|s| s == "true").unwrap_or(false);

    if parts.len() > 1 {
        let data_part = parts[1];
        let lines: Vec<&str> = data_part
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        let mut start_idx = 0;
        if let Some(first_line) = lines.first() {
            if first_line.to_lowercase().contains("metric")
                && first_line.to_lowercase().contains("value")
            {
                start_idx = 1;
            }
        }

        for line in &lines[start_idx..] {
            let cols: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if cols.len() >= 2 {
                card.metrics.push(MetricItem {
                    label: cols[0].to_string(),
                    value: cols[1].to_string(),
                    sublabel: cols.get(2).cloned().unwrap_or_default().to_string(),
                });
            }
        }
    }

    Ok(card)
}

fn render_svg(card: &MetricsCard, use_dark: bool) -> String {
    let id = format!("id_{}", Uuid::new_v4().to_string().replace('-', "_"));

    let card_width = 200;
    let card_height = 160;
    let gap = 20;
    let padding = 40;

    let n = card.metrics.len();
    let total_width = (padding * 2) + (n as i32 * card_width) + ((n as i32 - 1).max(0) * gap);
    let total_height = 336;

    let extra_class = if use_dark { " dark-mode" } else { "" };
    let title_esc = escape(&card.title);
    let desc_text = format!("Metrics card showing {} metrics", card.metrics.len());
    let desc_esc = escape(&desc_text);

    let mut svg = format!(
        r##"<svg id="{id}" width="{total_width}" height="{total_height}" viewBox="0 0 {total_width} {total_height}" xmlns="http://www.w3.org/2000/svg" class="metrics-card-container premium{extra_class}" role="region" aria-labelledby="{id}_title {id}_desc">
    <title id="{id}_title">{title_esc}</title>
    <desc id="{id}_desc">{desc_esc}</desc>
    <metadata><rdf:rdf xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:cc="http://creativecommons.org/ns#"><cc:work rdf:about=""><dc:creator>DocOps.io</dc:creator><dc:rights>MIT License</dc:rights><dc:source>https://docops.io</dc:source><dc:date>2026-09-11</dc:date></cc:work></rdf:rdf></metadata>
    <defs>
        <style>
            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;850;900&amp;display=swap');
            #{id} {{
                --bg-start: #f8fafc;
                --bg-end: #eef2ff;
                --panel-bg: rgba(255, 255, 255, 0.76);
                --panel-stroke: rgba(148, 163, 184, 0.28);
                --card-bg: rgba(255, 255, 255, 0.88);
                --card-stroke: rgba(148, 163, 184, 0.28);
                --text: #0f172a;
                --muted: #64748b;
                --accent-primary: #6366f1;
                --shadow-opacity: 0.16;
            }}

            @media (prefers-color-scheme: dark) {{
                #{id} {{
                    --bg-start: #0f172a;
                    --bg-end: #020617;
                    --panel-bg: rgba(30, 41, 59, 0.7);
                    --panel-stroke: rgba(51, 65, 85, 0.4);
                    --card-bg: rgba(30, 41, 59, 0.8);
                    --card-stroke: rgba(51, 65, 85, 0.5);
                    --text: #f8fafc;
                    --muted: #94a3b8;
                    --shadow-opacity: 0.4;
                }}
            }}

            #{id}.dark-mode {{
                --bg-start: #0f172a;
                --bg-end: #020617;
                --panel-bg: rgba(30, 41, 59, 0.7);
                --panel-stroke: rgba(51, 65, 85, 0.4);
                --card-bg: rgba(30, 41, 59, 0.8);
                --card-stroke: rgba(51, 65, 85, 0.5);
                --text: #f8fafc;
                --muted: #94a3b8;
                --shadow-opacity: 0.4;
            }}

            .title-text_{id} {{ font-family: 'Inter', sans-serif; font-weight: 850; font-size: 32px; fill: var(--text); letter-spacing: -0.035em; }}
            .eyebrow_{id} {{ font-family: 'Inter', sans-serif; font-weight: 800; font-size: 11px; fill: var(--muted); text-transform: uppercase; letter-spacing: 0.09em; }}
            .metric-value_{id} {{ font-family: 'Inter', sans-serif; font-weight: 850; font-size: 38px; fill: var(--text); letter-spacing: -0.04em; }}
            .metric-label_{id} {{ font-family: 'Inter', sans-serif; font-weight: 800; font-size: 11px; fill: var(--muted); text-transform: uppercase; letter-spacing: 0.09em; }}
            .metric-sub_{id} {{ font-family: 'Inter', sans-serif; font-weight: 650; font-size: 12px; fill: var(--muted); }}
        </style>
        
        <filter id="card-shadow_{id}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="12" stdDeviation="15" flood-color="#0F172A" flood-opacity="var(--shadow-opacity)"/>
        </filter>

        <linearGradient id="bg-grad_{id}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--bg-start)"/>
            <stop offset="100%" stop-color="var(--bg-end)"/>
        </linearGradient>

        <linearGradient id="card-grad_{id}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="#ffffff" stop-opacity="0.96"/>
            <stop offset="100%" stop-color="#f8fafc" stop-opacity="0.86"/>
        </linearGradient>
        
        <radialGradient id="shine_{id}" cx="35%" cy="20%" r="80%">
            <stop offset="0%" stop-color="#ffffff" stop-opacity="0.72"/>
            <stop offset="100%" stop-color="#ffffff" stop-opacity="0"/>
        </radialGradient>

        <linearGradient id="accent-0_{id}" x1="0" y1="0" x2="1" y2="0"><stop offset="0%" stop-color="#6366f1"/><stop offset="100%" stop-color="#8b5cf6"/></linearGradient>
        <linearGradient id="accent-1_{id}" x1="0" y1="0" x2="1" y2="0"><stop offset="0%" stop-color="#8b5cf6"/><stop offset="100%" stop-color="#d946ef"/></linearGradient>
        <linearGradient id="accent-2_{id}" x1="0" y1="0" x2="1" y2="0"><stop offset="0%" stop-color="#10b981"/><stop offset="100%" stop-color="#3b82f6"/></linearGradient>
        <linearGradient id="accent-3_{id}" x1="0" y1="0" x2="1" y2="0"><stop offset="0%" stop-color="#f59e0b"/><stop offset="100%" stop-color="#ef4444"/></linearGradient>
    </defs>

    <rect width="100%" height="100%" fill="url(#bg-grad_{id})" rx="24" aria-hidden="true"/>
    
    <g transform="translate({padding}, 60)" aria-hidden="true">
        <text x="0" y="0" class="eyebrow_{id}">Executive Dashboard</text>
        <text x="0" y="36" class="title-text_{id}">{title_esc}</text>
    </g>
    
    <g transform="translate({padding}, 136)" role="list" aria-label="Key Metrics">
"##,
        id = id,
        total_width = total_width,
        total_height = total_height,
        padding = padding,
        title_esc = title_esc,
        desc_esc = desc_esc,
        extra_class = extra_class
    );

    for (i, metric) in card.metrics.iter().enumerate() {
        let x = i as i32 * (card_width + gap);
        let accent_idx = i % 4;

        let sublabel = escape(&metric.sublabel);
        let display_sublabel = if sublabel.contains('%')
            || sublabel.contains("Growth")
            || sublabel.contains("Improvement")
        {
            format!("↗ {}", sublabel)
        } else if sublabel.contains("Leading") || sublabel.contains("Best") {
            format!("★ {}", sublabel)
        } else {
            sublabel
        };

        let val_esc = escape(&metric.value);
        let label_esc = escape(&metric.label);

        svg.push_str(&format!(
            r##"        <g transform="translate({x}, 0)" filter="url(#card-shadow_{id})" role="listitem" aria-label="{label_esc}: {val_esc} - {sublabel}">
            <rect width="{card_width}" height="{card_height}" rx="24" fill="var(--card-bg)" stroke="var(--card-stroke)" stroke-width="1.2" aria-hidden="true"/>
            <rect x="20" y="20" width="50" height="6" rx="3" fill="url(#accent-{accent_idx}_{id})" aria-hidden="true"/>
            <circle cx="{circle_x}" cy="36" r="24" fill="url(#accent-{accent_idx}_{id})" opacity="0.12" aria-hidden="true"/>
            <circle cx="{circle_x}" cy="36" r="24" fill="url(#shine_{id})" aria-hidden="true"/>
            <text x="20" y="76" class="metric-value_{id}">{val_esc}</text>
            <text x="20" y="104" class="metric-label_{id}">{label_esc}</text>
            <text x="20" y="132" class="metric-sub_{id}">{sublabel}</text>
        </g>
"##,
            x = x,
            card_width = card_width,
            card_height = card_height,
            id = id,
            accent_idx = accent_idx,
            val_esc = val_esc,
            label_esc = label_esc,
            sublabel = display_sublabel,
            circle_x = card_width - 36
        ));
    }

    svg.push_str("    </g>\n</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metrics_card() {
        let dsl = r#"----
title= Q2 2024 Business Metrics
---
Metric | Value | Sublabel
Revenue | $4.2M | 18% YoY Growth
New Customers | 156 | 42 Enterprise
----"#;

        let card = parse_metrics_card(dsl).unwrap();
        assert_eq!(card.title, "Q2 2024 Business Metrics");
        assert_eq!(card.metrics.len(), 2);
        assert_eq!(card.metrics[0].label, "Revenue");
        assert_eq!(card.metrics[0].value, "$4.2M");
        assert_eq!(card.metrics[0].sublabel, "18% YoY Growth");
    }

    #[test]
    fn test_render_metrics_card() {
        let dsl = r#"----
title= Q2 2024 Business Metrics
---
Metric | Value | Sublabel
Revenue | $4.2M | 18% YoY Growth
New Customers | 156 | 42 Enterprise
Customer Retention | 94% | 2% Improvement
NPS Score | 72 | Industry Leading
----"#;

        let result = render(dsl, &HashMap::new());
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("Q2 2024 Business Metrics"));
        assert!(svg.contains("Revenue"));
        assert!(svg.contains("$4.2M"));
        assert!(svg.contains("18% YoY Growth"));
        assert!(svg.contains("NPS Score"));
        assert!(svg.contains("72"));
        assert!(svg.contains("premium"));
        assert!(svg.contains("prefers-color-scheme: dark"));
    }
}
