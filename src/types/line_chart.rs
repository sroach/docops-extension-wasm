use crate::common::kv::parse_kv_body;
use crate::common::svg::escape;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct LineGroup {
    name: String,
    points: Vec<(String, f64)>,
}

fn get_series(points: &Vec<(String, f64)>) -> (Vec<String>, Vec<LineGroup>) {
    let mut x_labels = Vec::new();
    let mut x_map = HashMap::new();
    let mut groups: Vec<LineGroup> = Vec::new();
    let mut group_map = HashMap::new();

    for (label, value) in points {
        let parts: Vec<&str> = label.split('|').map(|s| s.trim()).collect();
        let (series_name, x_label) = if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("Series 1".to_string(), label.clone())
        };

        if !x_map.contains_key(&x_label) {
            x_map.insert(x_label.clone(), x_labels.len());
            x_labels.push(x_label.clone());
        }

        if !group_map.contains_key(&series_name) {
            group_map.insert(series_name.clone(), groups.len());
            groups.push(LineGroup {
                name: series_name.clone(),
                points: Vec::new(),
            });
        }
        
        let g_idx = group_map[&series_name];
        groups[g_idx].points.push((x_label, *value));
    }
    
    (x_labels, groups)
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let data = parse_kv_body(body)?;
    let cfg = &data.config;

    let use_dark = controls.get("useDark").map(|s| s == "true").unwrap_or(false)
        || cfg.get("theme").map(|s| s.as_str()) == Some("dark");

    let title = cfg.get("title").map(String::as_str).unwrap_or("Line Chart");
    let subtitle = cfg.get("subtitle").map(String::as_str).unwrap_or("");
    let x_label_text = cfg.get("xLabel").map(String::as_str).unwrap_or("");
    let y_label_text = cfg.get("yLabel").map(String::as_str).unwrap_or("");

    let (x_labels, groups) = get_series(&data.points);
    
    if x_labels.is_empty() {
        return Ok(crate::common::svg::error_svg("No data points provided"));
    }

    let mut series_peaks = Vec::new();
    let mut raw_max = 0.0;
    for g in &groups {
        let mut p_val = 0.0;
        let mut p_idx = 0;
        for (i, (_, v)) in g.points.iter().enumerate() {
            if *v > raw_max { raw_max = *v; }
            if *v > p_val {
                p_val = *v;
                p_idx = i;
            }
        }
        series_peaks.push((p_val, p_idx));
    }
    let max_val = if raw_max <= 0.0 { 1.0 } else { (raw_max * 1.1 / 10.0).ceil() * 10.0 };
    
    let width = 800.0;
    let height = 500.0;
    let plot_x = 85.0;
    let plot_y = 75.0;
    let plot_w = 565.0;
    let plot_h = 345.0;

    let chart_id = format!("id_{}", Uuid::new_v4());
    
    let mut grid_svg = String::new();
    let mut y_ticks = String::new();
    let steps = 8;
    for i in 0..=steps {
        let frac = i as f64 / steps as f64;
        let y = plot_y + plot_h - frac * plot_h;
        let val = max_val * frac;
        grid_svg.push_str(&format!(
            r##"<line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}"/>"##,
            x1 = plot_x, y = y, x2 = plot_x + plot_w
        ));
        y_ticks.push_str(&format!(
            r##"<line x1="{plot_x}" y1="{y}" x2="{tick_x}" y2="{y}" class="chart-axis"/><text x="{tx}" y="{y}" text-anchor="end" dominant-baseline="middle" class="chart-text tick-label">{val:.0}</text>"##,
            plot_x = plot_x, tick_x = plot_x - 5.0, tx = plot_x - 10.0, y = y, val = val
        ));
    }

    let x_padding = 25.0;
    let x_step = if x_labels.len() > 1 {
        (plot_w - 2.0 * x_padding) / (x_labels.len() - 1) as f64
    } else {
        0.0
    };

    let mut x_ticks = String::new();
    for (i, label) in x_labels.iter().enumerate() {
        let x = plot_x + x_padding + i as f64 * x_step;
        grid_svg.push_str(&format!(
            r##"<line x1="{x}" y1="{y1}" x2="{x}" y2="{y2}"/>"##,
            x = x, y1 = plot_y, y2 = plot_y + plot_h
        ));
        x_ticks.push_str(&format!(
            r##"<line x1="{x}" y1="{y_base}" x2="{x}" y2="{tick_y}" class="chart-axis"/><text x="{x}" y="{ty}" text-anchor="middle" class="chart-text tick-label">{label}<title>{label}</title></text>"##,
            x = x, y_base = plot_y + plot_h, tick_y = plot_y + plot_h + 5.0, ty = plot_y + plot_h + 20.0, label = escape(label)
        ));
    }

    let mut areas_svg = String::new();
    let mut series_svg = String::new();
    let mut legend_items = String::new();

    let palette_size = 5;

    for (g_idx, group) in groups.iter().enumerate() {
        let pal_idx = g_idx % palette_size;
        let (peak_val, peak_idx) = series_peaks[g_idx];

        let mut path_d = String::new();
        let mut first = true;
        let mut last_x = plot_x;

        let mut points_svg = String::new();

        for (i, label) in x_labels.iter().enumerate() {
            if let Some((_, val)) = group.points.iter().find(|(l, _)| l == label) {
                let x = plot_x + x_padding + i as f64 * x_step;
                let y = plot_y + plot_h - (val / max_val) * plot_h;

                if first {
                    path_d.push_str(&format!("M {x},{y}"));
                    first = false;
                } else {
                    path_d.push_str(&format!(" L {x},{y}"));
                }
                last_x = x;

                let is_peak = i == peak_idx;
                let r = if is_peak { 7.5 } else { 6.5 };
                let p_fill = if is_peak {
                    format!("var(--line-pal-{pal_idx}-stroke)")
                } else {
                    "var(--point-bg)".to_string()
                };
                let p_stroke = if is_peak {
                    "var(--point-peak-stroke)".to_string()
                } else {
                    format!("var(--line-pal-{pal_idx}-stroke)")
                };
                let sw = if is_peak { 3.0 } else { 2.5 };

                points_svg.push_str(&format!(
                    r##"<circle class="data-point point-reveal" style="animation-delay:{delay:.2}s" cx="{x}" cy="{y}" r="{r}" fill="{p_fill}" stroke="{p_stroke}" stroke-width="{sw}"><title>{name}: ({label}, {val})</title></circle>"##,
                    delay = 0.15 + (i as f64 * 0.05) + (g_idx as f64 * 0.08),
                    x = x, y = y, r = r, p_fill = p_fill, p_stroke = p_stroke, sw = sw, name = escape(&group.name), label = escape(label), val = val
                ));
            }
        }
        
        let area_d = format!("{path_d} L {last_x},{base} L {first_x},{base} Z", 
            path_d = path_d, last_x = last_x, base = plot_y + plot_h, first_x = plot_x + x_padding);

        areas_svg.push_str(&format!(
            r##"<path class="area-path area-{g_idx}" style="animation-delay:{delay:.2}s" d="{d}" fill="url(#{id}_area_grad_{pal_idx})" fill-opacity="1"/>"##,
            delay = 0.1 + g_idx as f64 * 0.08, d = area_d, id = chart_id, g_idx = g_idx, pal_idx = pal_idx
        ));

        series_svg.push_str(&format!(
            r##"<g class="data-series" id="series-{g_idx}" tabindex="0">
            <path class="line-path line-reveal line-{g_idx}" style="animation-delay:{delay:.2}s" pathLength="1" stroke-dasharray="1" stroke-dashoffset="1" d="{d}" fill="none" stroke="url(#{id}_line_grad_{pal_idx})" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>
            {points_svg}
        </g>"##,
            g_idx = g_idx, pal_idx = pal_idx, delay = g_idx as f64 * 0.08, d = path_d, id = chart_id, points_svg = points_svg
        ));

        legend_items.push_str(&format!(
            r##"<g class="legend-item"><circle cx="676" cy="{cy}" r="5.5" fill="var(--line-pal-{pal_idx}-stroke)"/><text x="690" y="{ty}" dominant-baseline="middle" class="chart-text legend-label">{name}</text><text x="690" y="{vy}" dominant-baseline="middle" class="chart-text legend-value">Peak {peak_val}</text></g>"##,
            cy = 110 + g_idx * 36, ty = 108 + g_idx * 36, vy = 124 + g_idx * 36, pal_idx = pal_idx, name = escape(&group.name), peak_val = peak_val
        ));
    }

    let mut series_defs = String::new();
    for i in 0..palette_size {
        series_defs.push_str(&format!(
            r##"
        <linearGradient id="{id}_line_grad_{i}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--line-pal-{i}-1)" stop-opacity="0.9"/>
            <stop offset="100%" stop-color="var(--line-pal-{i}-2)"/>
        </linearGradient>
        <linearGradient id="{id}_area_grad_{i}" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--line-pal-{i}-1)" stop-opacity="0.25"/>
            <stop offset="100%" stop-color="var(--line-pal-{i}-2)" stop-opacity="0.0"/>
        </linearGradient>
        "##,
            id = chart_id, i = i
        ));
    }

    let extra_class = if use_dark { " dark-mode" } else { "" };

    let header_svg = if !subtitle.is_empty() {
        format!(
            r##"<text x="400" y="44" text-anchor="middle" class="chart-text chart-title">{title}</text>
    <text x="400" y="62" text-anchor="middle" class="chart-text chart-subtitle">{subtitle}</text>"##,
            title = escape(title),
            subtitle = escape(subtitle)
        )
    } else {
        format!(
            r##"<text x="400" y="52" text-anchor="middle" class="chart-text chart-title">{title}</text>"##,
            title = escape(title)
        )
    };

    Ok(format!(
        r##"<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg" id="{id}" preserveAspectRatio="xMidYMid meet" viewBox="0 0 {width} {height}" class="line-chart-container{extra_class}">
    <metadata>
        <rdf:rdf xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:cc="http://creativecommons.org/ns#">
            <cc:work rdf:about="">
                <dc:creator>DocOps.io</dc:creator>
                <dc:rights>MIT License</dc:rights>
                <dc:source>https://docops.io</dc:source>
            </cc:work>
        </rdf:rdf>
    </metadata>
    <defs>
        <linearGradient id="{id}_apple_bg" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--bg-0)"/><stop offset="46%" stop-color="var(--bg-46)"/><stop offset="100%" stop-color="var(--bg-100)"/>
        </linearGradient>
        <radialGradient id="{id}_glow_blue" cx="19%" cy="8%" r="62%">
            <stop offset="0%" stop-color="var(--glow-blue-color)" stop-opacity="var(--glow-blue-op-0)"/><stop offset="45%" stop-color="var(--glow-blue-color)" stop-opacity="var(--glow-blue-op-45)"/><stop offset="100%" stop-color="var(--glow-blue-color)" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="{id}_glow_mint" cx="88%" cy="18%" r="56%">
            <stop offset="0%" stop-color="var(--glow-mint-color)" stop-opacity="var(--glow-mint-op-0)"/><stop offset="48%" stop-color="var(--glow-mint-color)" stop-opacity="var(--glow-mint-op-48)"/><stop offset="100%" stop-color="var(--glow-mint-color)" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="{id}_glow_peach" cx="42%" cy="100%" r="62%">
            <stop offset="0%" stop-color="var(--glow-peach-color)" stop-opacity="var(--glow-peach-op)"/><stop offset="100%" stop-color="var(--glow-peach-color)" stop-opacity="0"/>
        </radialGradient>
        <linearGradient id="{id}_plot_glass" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--plot-surface-0)" stop-opacity="var(--plot-surface-op-0)"/><stop offset="100%" stop-color="var(--plot-surface-100)" stop-opacity="var(--plot-surface-op-100)"/>
        </linearGradient>
        <linearGradient id="{id}_plot_stroke" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--plot-stroke-0)" stop-opacity="var(--plot-stroke-op-0)"/><stop offset="100%" stop-color="var(--plot-stroke-100)" stop-opacity="var(--plot-stroke-op-100)"/>
        </linearGradient>
        <filter id="{id}_apple_card_shadow" x="-20%" y="-20%" width="140%" height="150%">
            <feDropShadow dx="0" dy="22" stdDeviation="28" flood-color="var(--shadow-flood)" flood-opacity="var(--shadow-op)"/><feDropShadow dx="0" dy="2" stdDeviation="4" flood-color="var(--shadow-flood-sub)" flood-opacity="var(--shadow-op-sub)"/>
        </filter>
        <filter id="{id}_soft_point_shadow" x="-80%" y="-80%" width="260%" height="260%">
            <feDropShadow dx="0" dy="6" stdDeviation="6" flood-color="var(--point-shadow-color)" flood-opacity="var(--point-shadow-op)"/>
        </filter>
        <filter id="{id}_line_glow" x="-10%" y="-40%" width="120%" height="180%">
            <feGaussianBlur in="SourceGraphic" stdDeviation="3" result="blur"/><feColorMatrix in="blur" type="matrix" values="0 0 0 0 0.03  0 0 0 0 0.45  0 0 0 0 0.85  0 0 0 0 0.28 0" result="glow"/><feMerge><feMergeNode in="glow"/><feMergeNode in="SourceGraphic"/></feMerge>
        </filter>
        <clipPath id="{id}_plot_clip"><rect x="{px}" y="{py}" width="{pw}" height="{ph}" rx="26" ry="26"/></clipPath>
        {series_defs}
        <style>
            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&amp;display=swap');
            #{id} {{
                --bg: #F6F8FB;
                --bg-0: #F7FAFF;
                --bg-46: #EEF4FF;
                --bg-100: #E7EEF8;
                --card-bg: rgba(255, 255, 255, 0.38);
                --card-stroke: rgba(255, 255, 255, 0.72);
                --plot-surface-0: #FFFFFF;
                --plot-surface-100: #FFFFFF;
                --plot-surface-op-0: 0.82;
                --plot-surface-op-100: 0.48;
                --plot-stroke-0: #FFFFFF;
                --plot-stroke-100: #94A3B8;
                --plot-stroke-op-0: 0.86;
                --plot-stroke-op-100: 0.22;
                --glow-blue-color: #60A5FA;
                --glow-blue-op-0: 0.48;
                --glow-blue-op-45: 0.16;
                --glow-mint-color: #2DD4BF;
                --glow-mint-op-0: 0.50;
                --glow-mint-op-48: 0.13;
                --glow-peach-color: #FDBA74;
                --glow-peach-op: 0.24;
                --deco-circle-1: #FFFFFF;
                --deco-circle-2: #FFFFFF;
                --deco-circle-op: 0.18;
                --text: #111827;
                --text-soft: #6B7280;
                --text-muted: #8A94A6;
                --grid: #CBD5E1;
                --grid-op: 0.42;
                --axis: #94A3B8;
                --legend-box-bg: rgba(255, 255, 255, 0.66);
                --legend-box-stroke: rgba(255, 255, 255, 0.72);
                --point-bg: #FFFFFF;
                --point-peak-stroke: #FFFFFF;
                --shadow-flood: #1E293B;
                --shadow-op: 0.14;
                --shadow-flood-sub: #FFFFFF;
                --shadow-op-sub: 0.90;
                --point-shadow-color: #0369A1;
                --point-shadow-op: 0.22;
                --line-pal-0-1: #60A5FA;
                --line-pal-0-2: #3B82F6;
                --line-pal-0-stroke: #3B82F6;
                --line-pal-1-1: #A78BFA;
                --line-pal-1-2: #8B5CF6;
                --line-pal-1-stroke: #8B5CF6;
                --line-pal-2-1: #34D399;
                --line-pal-2-2: #10B981;
                --line-pal-2-stroke: #10B981;
                --line-pal-3-1: #FBBF24;
                --line-pal-3-2: #F59E0B;
                --line-pal-3-stroke: #F59E0B;
                --line-pal-4-1: #FB7185;
                --line-pal-4-2: #E11D48;
                --line-pal-4-stroke: #E11D48;
            }}

            @media (prefers-color-scheme: dark) {{
                #{id} {{
                    --bg: #111827;
                    --bg-0: #0F172A;
                    --bg-46: #111827;
                    --bg-100: #1E293B;
                    --card-bg: rgba(30, 41, 59, 0.55);
                    --card-stroke: rgba(255, 255, 255, 0.12);
                    --plot-surface-0: #1E293B;
                    --plot-surface-100: #0F172A;
                    --plot-surface-op-0: 0.85;
                    --plot-surface-op-100: 0.65;
                    --plot-stroke-0: #475569;
                    --plot-stroke-100: #334155;
                    --plot-stroke-op-0: 0.60;
                    --plot-stroke-op-100: 0.30;
                    --glow-blue-color: #3B82F6;
                    --glow-blue-op-0: 0.25;
                    --glow-blue-op-45: 0.08;
                    --glow-mint-color: #0D9488;
                    --glow-mint-op-0: 0.20;
                    --glow-mint-op-48: 0.06;
                    --glow-peach-color: #C2410C;
                    --glow-peach-op: 0.12;
                    --deco-circle-1: #38BDF8;
                    --deco-circle-2: #818CF8;
                    --deco-circle-op: 0.05;
                    --text: #F9FAFB;
                    --text-soft: #9CA3AF;
                    --text-muted: #6B7280;
                    --grid: #374151;
                    --grid-op: 0.50;
                    --axis: #4B5563;
                    --legend-box-bg: rgba(17, 24, 39, 0.75);
                    --legend-box-stroke: rgba(255, 255, 255, 0.10);
                    --point-bg: #1E293B;
                    --point-peak-stroke: #FFFFFF;
                    --shadow-flood: #000000;
                    --shadow-op: 0.40;
                    --shadow-flood-sub: #000000;
                    --shadow-op-sub: 0.0;
                    --point-shadow-color: #000000;
                    --point-shadow-op: 0.40;
                    --line-pal-0-1: #93C5FD;
                    --line-pal-0-2: #3B82F6;
                    --line-pal-0-stroke: #60A5FA;
                    --line-pal-1-1: #C4B5FD;
                    --line-pal-1-2: #8B5CF6;
                    --line-pal-1-stroke: #A78BFA;
                    --line-pal-2-1: #6EE7B7;
                    --line-pal-2-2: #10B981;
                    --line-pal-2-stroke: #34D399;
                    --line-pal-3-1: #FDE68A;
                    --line-pal-3-2: #F59E0B;
                    --line-pal-3-stroke: #FBBF24;
                    --line-pal-4-1: #FDA4AF;
                    --line-pal-4-2: #E11D48;
                    --line-pal-4-stroke: #FB7185;
                }}
            }}

            #{id}.dark-mode {{
                --bg: #111827;
                --bg-0: #0F172A;
                --bg-46: #111827;
                --bg-100: #1E293B;
                --card-bg: rgba(30, 41, 59, 0.55);
                --card-stroke: rgba(255, 255, 255, 0.12);
                --plot-surface-0: #1E293B;
                --plot-surface-100: #0F172A;
                --plot-surface-op-0: 0.85;
                --plot-surface-op-100: 0.65;
                --plot-stroke-0: #475569;
                --plot-stroke-100: #334155;
                --plot-stroke-op-0: 0.60;
                --plot-stroke-op-100: 0.30;
                --glow-blue-color: #3B82F6;
                --glow-blue-op-0: 0.25;
                --glow-blue-op-45: 0.08;
                --glow-mint-color: #0D9488;
                --glow-mint-op-0: 0.20;
                --glow-mint-op-48: 0.06;
                --glow-peach-color: #C2410C;
                --glow-peach-op: 0.12;
                --deco-circle-1: #38BDF8;
                --deco-circle-2: #818CF8;
                --deco-circle-op: 0.05;
                --text: #F9FAFB;
                --text-soft: #9CA3AF;
                --text-muted: #6B7280;
                --grid: #374151;
                --grid-op: 0.50;
                --axis: #4B5563;
                --legend-box-bg: rgba(17, 24, 39, 0.75);
                --legend-box-stroke: rgba(255, 255, 255, 0.10);
                --point-bg: #1E293B;
                --point-peak-stroke: #FFFFFF;
                --shadow-flood: #000000;
                --shadow-op: 0.40;
                --shadow-flood-sub: #000000;
                --shadow-op-sub: 0.0;
                --point-shadow-color: #000000;
                --point-shadow-op: 0.40;
                --line-pal-0-1: #93C5FD;
                --line-pal-0-2: #3B82F6;
                --line-pal-0-stroke: #60A5FA;
                --line-pal-1-1: #C4B5FD;
                --line-pal-1-2: #8B5CF6;
                --line-pal-1-stroke: #A78BFA;
                --line-pal-2-1: #6EE7B7;
                --line-pal-2-2: #10B981;
                --line-pal-2-stroke: #34D399;
                --line-pal-3-1: #FDE68A;
                --line-pal-3-2: #F59E0B;
                --line-pal-3-stroke: #FBBF24;
                --line-pal-4-1: #FDA4AF;
                --line-pal-4-2: #E11D48;
                --line-pal-4-stroke: #FB7185;
            }}

            #{id} .chart-text {{ font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'SF Pro Text', Inter, system-ui, sans-serif; fill: var(--text); letter-spacing: -0.01em; }}
            #{id} .chart-background {{ fill: url(#{id}_apple_bg); }}
            #{id} .plot-surface {{ fill: url(#{id}_plot_glass); stroke: url(#{id}_plot_stroke); stroke-width: 1; }}
            #{id} .chart-grid {{ stroke: var(--grid); stroke-opacity: var(--grid-op); }}
            #{id} .chart-axis {{ stroke: var(--axis); stroke-width: 1.25; stroke-opacity: 0.62; }}
            #{id} .chart-title {{ fill: var(--text); font-size: 22px; font-weight: 760; letter-spacing: -0.025em; }}
            #{id} .chart-subtitle {{ fill: var(--text-soft); font-size: 12px; font-weight: 500; }}
            #{id} .axis-label {{ fill: var(--text-soft); font-size: 13px; font-weight: 600; }}
            #{id} .tick-label {{ fill: var(--text-muted); font-size: 11px; font-weight: 590; }}
            #{id} .legend-box {{ fill: var(--legend-box-bg); stroke: var(--legend-box-stroke); stroke-width: 1; }}
            #{id} .legend-label {{ fill: var(--text); font-size: 12px; font-weight: 650; }}
            #{id} .legend-value {{ fill: var(--text-muted); font-size: 10px; font-weight: 590; }}
            #{id} .line-path {{ filter: url(#{id}_line_glow); transition: stroke-width 180ms ease; }}
            #{id} .area-path {{ opacity: 0; animation: areaBloom_{id} 720ms cubic-bezier(0.16, 1, 0.3, 1) forwards; }}
            #{id} .line-reveal {{ opacity: 0; animation: lineReveal_{id} 900ms cubic-bezier(.2,.85,.2,1) forwards; }}
            #{id} .point-reveal {{ opacity: 0; transform-box: fill-box; transform-origin: center; animation: pointReveal_{id} 460ms cubic-bezier(0.2, 1.2, 0.2, 1) forwards; }}
            @keyframes lineReveal_{id} {{ from {{ opacity: 0; stroke-dashoffset: 1; }} to {{ opacity: 1; stroke-dashoffset: 0; }} }}
            @keyframes pointReveal_{id} {{ from {{ opacity: 0; transform: scale(0.35); }} to {{ opacity: 1; transform: scale(1); }} }}
            @keyframes areaBloom_{id} {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}
            #{id} .plot .data-series {{ opacity: 0.95; transition: opacity 140ms ease; }}
            #{id} .plot:hover .data-series {{ opacity: 0.36; }}
            #{id} .plot:hover .data-series:hover {{ opacity: 1; }}
            #{id} .data-point {{ transition: all 160ms ease; filter: url(#{id}_soft_point_shadow); }}
            #{id} .data-series:hover .line-path {{ stroke-width: 6; }}
            #{id} .data-series:hover .data-point {{ r: 8; stroke-width: 3; }}
            @media (prefers-reduced-motion: reduce) {{ #{id} * {{ transition: none !important; animation: none !important; }} }}
        </style>
    </defs>
    <rect width="{width}" height="{height}" rx="34" class="chart-background"/>
    <rect width="{width}" height="{height}" rx="34" fill="url(#{id}_glow_blue)"/>
    <rect width="{width}" height="{height}" rx="34" fill="url(#{id}_glow_mint)"/>
    <rect width="{width}" height="{height}" rx="34" fill="url(#{id}_glow_peach)"/>
    <circle cx="720" cy="80" r="80" fill="var(--deco-circle-1)" opacity="var(--deco-circle-op)"/>
    <circle cx="88" cy="425" r="112" fill="var(--deco-circle-2)" opacity="var(--deco-circle-op)"/>
    <g filter="url(#{id}_apple_card_shadow)">
        <rect x="34" y="24" width="732" height="452" rx="32" fill="var(--card-bg)" stroke="var(--card-stroke)" stroke-width="1"/>
    </g>
    {header_svg}
    <rect x="{px}" y="{py}" width="{pw}" height="{ph}" rx="26" class="plot-surface"/>
    <g clip-path="url(#{id}_plot_clip)">
        <g class="chart-grid">{grid_svg}</g>
    </g>
    <g class="axes">
        <line x1="{px}" y1="{pb}" x2="{pr}" y2="{pb}" class="chart-axis"/>
        <line x1="{px}" y1="{py}" x2="{px}" y2="{pb}" class="chart-axis"/>
        {x_ticks}
        {y_ticks}
        <text x="{x_label_x}" y="{x_label_y}" class="chart-text axis-label" text-anchor="middle">{x_label_esc}</text>
        <text x="{y_label_x}" y="{y_label_y}" class="chart-text axis-label" text-anchor="middle" transform="rotate(-90, {y_label_x}, {y_label_y})">{y_label_esc}</text>
    </g>
    <g clip-path="url(#{id}_plot_clip)">{areas_svg}</g>
    <g class="plot" clip-path="url(#{id}_plot_clip)">{series_svg}</g>
    <g class="legend">
        <rect x="660" y="88" width="115" height="{lh}" rx="22" class="legend-box"/>
        {legend_items}
    </g>
</svg>"##,
        width = width, height = height, id = chart_id,
        header_svg = header_svg,
        px = plot_x, py = plot_y, pw = plot_w, ph = plot_h, pb = plot_y + plot_h, pr = plot_x + plot_w,
        grid_svg = grid_svg, x_ticks = x_ticks, y_ticks = y_ticks,
        x_label_x = plot_x + plot_w / 2.0, x_label_y = plot_y + plot_h + 45.0, x_label_esc = escape(x_label_text),
        y_label_x = plot_x - 65.0, y_label_y = plot_y + plot_h / 2.0, y_label_esc = escape(y_label_text),
        areas_svg = areas_svg, series_svg = series_svg,
        legend_items = legend_items, lh = 20 + groups.len() * 36
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_line() {
        let input = "---- title=Growth ---\n2021 | 25\n2022 | 55\n2023 | 85\n----";
        let controls = HashMap::new();
        let svg = render(input, &controls).unwrap();
        assert!(svg.contains("Growth"));
        assert!(svg.contains("line-chart-container"));
        assert!(svg.contains("line-0"));
        assert!(svg.contains("prefers-color-scheme: dark"));
        assert!(svg.contains("--line-pal-0-1"));
    }

    #[test]
    fn test_render_multi_line() {
        let input = "---- title=Comparison\nsubtitle=Monthly comparison ---\nS1 | Jan | 10\nS1 | Feb | 20\nS2 | Jan | 15\nS2 | Feb | 25\n----";
        let controls = HashMap::new();
        let svg = render(input, &controls).unwrap();
        assert!(svg.contains("Comparison"));
        assert!(svg.contains("Monthly comparison"));
        assert!(svg.contains("line-0"));
        assert!(svg.contains("line-1"));
    }

    #[test]
    fn test_render_dark_mode_theme() {
        let input = "---- title=DarkLine\ntheme=dark ---\nJan | 10\nFeb | 20\n----";
        let controls = HashMap::new();
        let svg = render(input, &controls).unwrap();
        assert!(svg.contains("dark-mode"));
        assert!(svg.contains(".dark-mode"));
    }

    #[test]
    fn test_render_dark_mode_controls() {
        let input = "---- title=ControlsDark ---\nJan | 10\nFeb | 20\n----";
        let mut controls = HashMap::new();
        controls.insert("useDark".to_string(), "true".to_string());
        let svg = render(input, &controls).unwrap();
        assert!(svg.contains("dark-mode"));
    }
}
