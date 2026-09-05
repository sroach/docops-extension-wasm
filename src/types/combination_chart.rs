use crate::common::kv::parse_kv_header;
use crate::common::svg::escape;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct ComboPoint {
    series: String,
    chart_type: String, // BAR or LINE
    x_label: String,
    value: f64,
    #[allow(dead_code)]
    extra: String,
    axis: String, // PRIMARY or SECONDARY
}

struct CombinationContext {
    chart_id: String,
    plot_x: f64,
    plot_y: f64,
    plot_w: f64,
    plot_h: f64,
    max_primary: f64,
    max_secondary: f64,
    x_labels: Vec<String>,
    // series_name -> (type, axis, values_mapped_to_x_labels)
    series_map: Vec<SeriesData>,
    dual_y_axis: bool,
}

struct SeriesData {
    name: String,
    chart_type: String,
    axis: String,
    values: Vec<Option<f64>>,
}

fn parse_combo_body(body: &str) -> Result<(HashMap<String, String>, Vec<ComboPoint>), String> {
    let trimmed = body.trim();
    if !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("body must start and end with '----'".into());
    }
    let inner = &trimmed[4..trimmed.len() - 4];
    let parts: Vec<&str> = inner.splitn(2, "---").collect();
    if parts.len() != 2 {
        return Err("missing '---' separator".into());
    }
    let config = parse_kv_header(parts[0]);
    let mut points = Vec::new();
    for line in parts[1].lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if cols.len() < 4 {
            continue;
        }
        let series = cols[0].to_string();
        let chart_type = cols[1].to_uppercase();
        let x_label = cols[2].to_string();
        let value: f64 = cols[3]
            .parse()
            .map_err(|_| format!("Invalid value: {}", cols[3]))?;
        let extra = cols.get(4).unwrap_or(&"").to_string();
        let axis = cols.get(5).unwrap_or(&"PRIMARY").to_uppercase();
        points.push(ComboPoint {
            series,
            chart_type,
            x_label,
            value,
            extra,
            axis,
        });
    }
    Ok((config, points))
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let (cfg, points) = parse_combo_body(body)?;

    let use_dark = controls.get("useDark").map(|s| s == "true").unwrap_or(false)
        || cfg.get("theme").map(|s| s.as_str()) == Some("dark");

    let title = cfg.get("title").map(String::as_str).unwrap_or("Combination Chart");
    let subtitle = cfg.get("subtitle").map(String::as_str).unwrap_or("Visualized data report");
    let x_axis_label = cfg.get("xLabel").map(String::as_str).unwrap_or("");
    let y_axis_label = cfg.get("yLabel").map(String::as_str).unwrap_or("");
    let y_axis_secondary_label = cfg.get("yLabelSecondary").map(String::as_str).unwrap_or("");
    let dual_y_axis = cfg.get("dualYAxis").map(|s| s == "true").unwrap_or(false);

    let mut x_labels_set = HashSet::new();
    let mut x_labels_order = Vec::new();
    for p in &points {
        if x_labels_set.insert(p.x_label.clone()) {
            x_labels_order.push(p.x_label.clone());
        }
    }

    let mut series_names = Vec::new();
    let mut series_info = HashMap::new(); // name -> (type, axis)
    for p in &points {
        if !series_info.contains_key(&p.series) {
            series_names.push(p.series.clone());
            series_info.insert(p.series.clone(), (p.chart_type.clone(), p.axis.clone()));
        }
    }

    let mut series_map = Vec::new();
    let mut max_primary = 0.0;
    let mut max_secondary = 0.0;

    for name in series_names {
        let (ctype, axis) = series_info.get(&name).unwrap();
        let mut values = vec![None; x_labels_order.len()];
        for p in &points {
            if p.series == name {
                if let Some(idx) = x_labels_order.iter().position(|l| l == &p.x_label) {
                    values[idx] = Some(p.value);
                    if axis == "SECONDARY" {
                        if p.value > max_secondary {
                            max_secondary = p.value;
                        }
                    } else {
                        if p.value > max_primary {
                            max_primary = p.value;
                        }
                    }
                }
            }
        }
        series_map.push(SeriesData {
            name,
            chart_type: ctype.clone(),
            axis: axis.clone(),
            values,
        });
    }

    if max_primary <= 0.0 {
        max_primary = 1.0;
    }
    if max_secondary <= 0.0 {
        max_secondary = 1.0;
    }
    // Round up max values to nice numbers
    max_primary = (max_primary / 10.0).ceil() * 10.0;
    if dual_y_axis {
        max_secondary = (max_secondary / 5.0).ceil() * 5.0;
    }

    let width = 960.0;
    let height = 600.0;
    let plot_x = 100.0;
    let plot_y = 120.0;
    let plot_w = 760.0;
    let plot_h = 360.0;

    let chart_id = format!("id_{}", Uuid::new_v4());

    let ctx = CombinationContext {
        chart_id: chart_id.clone(),
        plot_x,
        plot_y,
        plot_w,
        plot_h,
        max_primary,
        max_secondary,
        x_labels: x_labels_order,
        series_map,
        dual_y_axis,
    };

    let mut content = String::new();
    render_grid(&ctx, &mut content);
    render_axes(&ctx, &mut content, y_axis_label, y_axis_secondary_label, x_axis_label);
    render_series(&ctx, &mut content);
    render_legend(&ctx, &mut content);

    let extra_class = if use_dark { " dark-mode" } else { "" };

    Ok(format!(
        r##"<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg" role="img" id="{chart_id}" class="combo-chart-container{extra_class}" preserveAspectRatio="xMidYMid meet">
    <defs>
        <filter id="{chart_id}__premiumShadow" x="-20%" y="-20%" width="140%" height="150%">
            <feDropShadow dx="0" dy="12" stdDeviation="20" flood-color="#1B2735" flood-opacity="0.12"/>
        </filter>
        <linearGradient id="{chart_id}__barGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#4F7FAE"/>
            <stop offset="100%" stop-color="#8DB8DD"/>
        </linearGradient>
    </defs>
    <style>
        #{chart_id} {{
            --bg: #F6F8FB;
            --text: #17202A;
            --text-soft: #657184;
            --grid: #E5EAF0;
            --axis: #9AA8BA;
            --primary: #4F7FAE;
            --secondary: #E11D48;
        }}
        #{chart_id}.dark-mode {{
            --bg: #111827;
            --text: #F9FAFB;
            --text-soft: #9CA3AF;
            --grid: #374151;
            --axis: #4B5563;
            --primary: #60A5FA;
            --secondary: #FB7185;
        }}
        #{chart_id} text {{ font-family: 'Inter', system-ui, sans-serif; }}
        #{chart_id} .title {{ font-size: 24px; font-weight: 700; fill: var(--text); }}
        #{chart_id} .subtitle {{ font-size: 14px; fill: var(--text-soft); }}
        #{chart_id} .axis-label {{ font-size: 12px; font-weight: 600; fill: var(--text-soft); }}
        #{chart_id} .tick-text {{ font-size: 11px; fill: var(--text-soft); }}
        #{chart_id} .grid {{ stroke: var(--grid); stroke-width: 1; }}
        #{chart_id} .axis-line {{ stroke: var(--axis); stroke-width: 1.5; }}
        #{chart_id} .bar {{ filter: url(#{chart_id}__premiumShadow); transition: all 0.3s; }}
        #{chart_id} .line {{ fill: none; stroke: var(--secondary); stroke-width: 3; stroke-linecap: round; stroke-linejoin: round; }}
        #{chart_id} .point {{ fill: var(--secondary); stroke: #FFF; stroke-width: 2; }}
    </style>
    
    <rect width="{width}" height="{height}" fill="var(--bg)" rx="16"/>
    
    <text x="40" y="50" class="title">{title}</text>
    <text x="40" y="75" class="subtitle">{subtitle}</text>
    
    {content}
</svg>"##,
        width = width,
        height = height,
        chart_id = chart_id,
        title = escape(title),
        subtitle = escape(subtitle),
        extra_class = extra_class,
        content = content
    ))
}

fn render_grid(ctx: &CombinationContext, content: &mut String) {
    for i in 0..=5 {
        let frac = i as f64 / 5.0;
        let y = ctx.plot_y + ctx.plot_h - frac * ctx.plot_h;
        content.push_str(&format!(
            r##"<line class="grid" x1="{x1}" y1="{y}" x2="{x2}" y2="{y}"/>"##,
            x1 = ctx.plot_x,
            y = y,
            x2 = ctx.plot_x + ctx.plot_w
        ));
    }
}

fn render_axes(
    ctx: &CombinationContext,
    content: &mut String,
    y_label: &str,
    y_label_sec: &str,
    x_label: &str,
) {
    // Primary Y Axis
    for i in 0..=5 {
        let frac = i as f64 / 5.0;
        let y = ctx.plot_y + ctx.plot_h - frac * ctx.plot_h;
        let val = ctx.max_primary * frac;
        content.push_str(&format!(
            r##"<text class="tick-text" x="{x}" y="{y}" text-anchor="end">{val:.0}</text>"##,
            x = ctx.plot_x - 10.0,
            y = y + 4.0,
            val = val
        ));
    }
    content.push_str(&format!(
        r##"<text class="axis-label" x="{x}" y="{y}" transform="rotate(-90, {x}, {y})" text-anchor="middle">{label}</text>"##,
        x = ctx.plot_x - 60.0,
        y = ctx.plot_y + ctx.plot_h / 2.0,
        label = escape(y_label)
    ));

    // Secondary Y Axis
    if ctx.dual_y_axis {
        for i in 0..=5 {
            let frac = i as f64 / 5.0;
            let y = ctx.plot_y + ctx.plot_h - frac * ctx.plot_h;
            let val = ctx.max_secondary * frac;
            content.push_str(&format!(
                r##"<text class="tick-text" x="{x}" y="{y}" text-anchor="start">{val:.1}</text>"##,
                x = ctx.plot_x + ctx.plot_w + 10.0,
                y = y + 4.0,
                val = val
            ));
        }
        content.push_str(&format!(
            r##"<text class="axis-label" x="{x}" y="{y}" transform="rotate(90, {x}, {y})" text-anchor="middle">{label}</text>"##,
            x = ctx.plot_x + ctx.plot_w + 65.0,
            y = ctx.plot_y + ctx.plot_h / 2.0,
            label = escape(y_label_sec)
        ));
    }

    // X Axis Labels
    let x_step = ctx.plot_w / ctx.x_labels.len() as f64;
    for (i, label) in ctx.x_labels.iter().enumerate() {
        let x = ctx.plot_x + (i as f64 + 0.5) * x_step;
        content.push_str(&format!(
            r##"<text class="tick-text" x="{x}" y="{y}" text-anchor="middle">{label}</text>"##,
            x = x,
            y = ctx.plot_y + ctx.plot_h + 25.0,
            label = escape(label)
        ));
    }
    content.push_str(&format!(
        r##"<text class="axis-label" x="{x}" y="{y}" text-anchor="middle">{label}</text>"##,
        x = ctx.plot_x + ctx.plot_w / 2.0,
        y = ctx.plot_y + ctx.plot_h + 50.0,
        label = escape(x_label)
    ));
}

fn render_series(ctx: &CombinationContext, content: &mut String) {
    let x_step = ctx.plot_w / ctx.x_labels.len() as f64;

    // First pass: Draw bars
    for series in &ctx.series_map {
        if series.chart_type == "BAR" {
            let bar_width = x_step * 0.6;
            for (i, val_opt) in series.values.iter().enumerate() {
                if let Some(val) = val_opt {
                    let max = if series.axis == "SECONDARY" { ctx.max_secondary } else { ctx.max_primary };
                    let h = (val / max) * ctx.plot_h;
                    let x = ctx.plot_x + (i as f64 + 0.5) * x_step - bar_width / 2.0;
                    let y = ctx.plot_y + ctx.plot_h - h;
                    content.push_str(&format!(
                        r##"<rect class="bar" x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="url(#{id}__barGradient)"/>"##,
                        x = x,
                        y = y,
                        w = bar_width,
                        h = h,
                        id = ctx.chart_id
                    ));
                }
            }
        }
    }

    // Second pass: Draw lines
    for series in &ctx.series_map {
        if series.chart_type == "LINE" {
            let mut path_data = String::new();
            let mut points_svg = String::new();
            let mut first = true;

            for (i, val_opt) in series.values.iter().enumerate() {
                if let Some(val) = val_opt {
                    let max = if series.axis == "SECONDARY" { ctx.max_secondary } else { ctx.max_primary };
                    let h = (val / max) * ctx.plot_h;
                    let x = ctx.plot_x + (i as f64 + 0.5) * x_step;
                    let y = ctx.plot_y + ctx.plot_h - h;

                    if first {
                        path_data.push_str(&format!("M {} {}", x, y));
                        first = false;
                    } else {
                        path_data.push_str(&format!(" L {} {}", x, y));
                    }
                    points_svg.push_str(&format!(
                        r##"<circle class="point" cx="{x}" cy="{y}" r="5"/>"##,
                        x = x,
                        y = y
                    ));
                }
            }
            content.push_str(&format!(
                r##"<path class="line" d="{d}"/>"##,
                d = path_data
            ));
            content.push_str(&points_svg);
        }
    }
}

fn render_legend(ctx: &CombinationContext, content: &mut String) {
    let mut x = ctx.plot_x;
    let y = 95.0;
    
    for series in &ctx.series_map {
        let color = if series.chart_type == "BAR" { "var(--primary)" } else { "var(--secondary)" };
        if series.chart_type == "BAR" {
            content.push_str(&format!(
                r##"<rect x="{x}" y="{y}" width="12" height="12" rx="3" fill="{color}"/>"##,
                x = x,
                y = y - 10.0,
                color = color
            ));
        } else {
            content.push_str(&format!(
                r##"<line x1="{x}" y1="{y}" x2="{x2}" y2="{y}" stroke="{color}" stroke-width="3"/>"##,
                x = x,
                y = y - 4.0,
                x2 = x + 12.0,
                color = color
            ));
            content.push_str(&format!(
                r##"<circle cx="{x}" cy="{y}" r="3" fill="{color}"/>"##,
                x = x + 6.0,
                y = y - 4.0,
                color = color
            ));
        }
        content.push_str(&format!(
            r##"<text x="{tx}" y="{y}" class="tick-text" style="font-weight: 600;">{name}</text>"##,
            tx = x + 18.0,
            y = y,
            name = escape(&series.name)
        ));
        
        // Approximate width of legend item
        x += 25.0 + series.name.len() as f64 * 7.0 + 20.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_combo() {
        let input = r##"----
title=Sales Volume vs Profit Margin
xLabel=Quarter
yLabel=Units Sold
yLabelSecondary=Margin (%)
dualYAxis=true
theme=premium
---
Units Sold | BAR | Q1 | 1200 |  | PRIMARY
Units Sold | BAR | Q2 | 1450 |  | PRIMARY
Units Sold | BAR | Q3 | 1680 |  | PRIMARY
Units Sold | BAR | Q4 | 1920 |  | PRIMARY
Profit Margin | LINE | Q1 | 22.5 |  | SECONDARY
Profit Margin | LINE | Q2 | 24.8 |  | SECONDARY
Profit Margin | LINE | Q3 | 26.2 |  | SECONDARY
Profit Margin | LINE | Q4 | 28.1 |  | SECONDARY
----"##;
        let (cfg, points) = parse_combo_body(input).unwrap();
        assert_eq!(cfg.get("title").unwrap(), "Sales Volume vs Profit Margin");
        assert_eq!(points.len(), 8);
        assert_eq!(points[0].series, "Units Sold");
        assert_eq!(points[0].chart_type, "BAR");
        assert_eq!(points[4].series, "Profit Margin");
        assert_eq!(points[4].chart_type, "LINE");
        assert_eq!(points[4].axis, "SECONDARY");
    }

    #[test]
    fn test_render_combo() {
        let input = r##"----
title=Sales Volume vs Profit Margin
xLabel=Quarter
yLabel=Units Sold
yLabelSecondary=Margin (%)
dualYAxis=true
theme=premium
---
Units Sold | BAR | Q1 | 1200 |  | PRIMARY
Units Sold | BAR | Q2 | 1450 |  | PRIMARY
Units Sold | BAR | Q3 | 1680 |  | PRIMARY
Units Sold | BAR | Q4 | 1920 |  | PRIMARY
Profit Margin | LINE | Q1 | 22.5 |  | SECONDARY
Profit Margin | LINE | Q2 | 24.8 |  | SECONDARY
Profit Margin | LINE | Q3 | 26.2 |  | SECONDARY
Profit Margin | LINE | Q4 | 28.1 |  | SECONDARY
----"##;
        let controls = HashMap::new();
        let svg = render(input, &controls).unwrap();
        assert!(svg.contains("Sales Volume vs Profit Margin"));
        assert!(svg.contains("combo-chart-container"));
        assert!(svg.contains("class=\"bar\""));
        assert!(svg.contains("class=\"line\""));
        assert!(svg.contains("Margin (%)"));
    }
}
