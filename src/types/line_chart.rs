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
    let _subtitle = cfg.get("subtitle").map(String::as_str).unwrap_or("Performance over time");
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
    let plot_y = 70.0;
    let plot_w = 565.0;
    let plot_h = 350.0;

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

    let x_step = if x_labels.len() > 1 {
        plot_w / (x_labels.len() - 1) as f64
    } else {
        plot_w
    };

    let mut x_ticks = String::new();
    for (i, label) in x_labels.iter().enumerate() {
        let x = plot_x + i as f64 * x_step;
        x_ticks.push_str(&format!(
            r##"<line x1="{x}" y1="{y_base}" x2="{x}" y2="{tick_y}" class="chart-axis"/><text x="{x}" y="{ty}" text-anchor="middle" class="chart-text tick-label">{label}<title>{label}</title></text>"##,
            x = x, y_base = plot_y + plot_h, tick_y = plot_y + plot_h + 5.0, ty = plot_y + plot_h + 20.0, label = escape(label)
        ));
    }

    let mut areas_svg = String::new();
    let mut series_svg = String::new();
    let mut legend_items = String::new();

    let palette = ["#3B82F6", "#8B5CF6", "#10B981", "#F59E0B", "#E11D48"];

    for (g_idx, group) in groups.iter().enumerate() {
        let color_idx = g_idx % palette.len();
        let color = palette[color_idx];
        let (peak_val, peak_idx) = series_peaks[g_idx];

        let mut path_d = String::new();
        let mut first = true;
        let mut last_x = plot_x;

        let mut points_svg = String::new();

        for (i, label) in x_labels.iter().enumerate() {
            if let Some((_, val)) = group.points.iter().find(|(l, _)| l == label) {
                let x = plot_x + i as f64 * x_step;
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
                let p_fill = if is_peak { color } else { "#FFFFFF" };
                let p_stroke = if is_peak { "#FFFFFF" } else { color };
                let sw = if is_peak { 3.0 } else { 2.5 };

                points_svg.push_str(&format!(
                    r##"<circle class="data-point point-reveal" style="animation-delay:{delay:.2}s" cx="{x}" cy="{y}" r="{r}" fill="{p_fill}" stroke="{p_stroke}" stroke-width="{sw}"><title>{name}: ({label}, {val})</title></circle>"##,
                    delay = 0.15 + (i as f64 * 0.05) + (g_idx as f64 * 0.08),
                    x = x, y = y, r = r, p_fill = p_fill, p_stroke = p_stroke, sw = sw, name = escape(&group.name), label = escape(label), val = val
                ));
            }
        }
        
        let area_d = format!("{path_d} L {last_x},{base} L {first_x},{base} Z", 
            path_d = path_d, last_x = last_x, base = plot_y + plot_h, first_x = plot_x);

        areas_svg.push_str(&format!(
            r##"<path class="area-path area-{g_idx}" style="animation-delay:{delay:.2}s" d="{d}" fill="url(#{id}_area_grad_{g_idx})" fill-opacity="1"/>"##,
            delay = 0.1 + g_idx as f64 * 0.08, d = area_d, id = chart_id, g_idx = g_idx
        ));

        series_svg.push_str(&format!(
            r##"<g class="data-series" id="series-{g_idx}" tabindex="0">
            <path class="line-path line-reveal line-{g_idx}" style="animation-delay:{delay:.2}s" pathLength="1" stroke-dasharray="1" stroke-dashoffset="1" d="{d}" fill="none" stroke="url(#{id}_line_grad_{g_idx})" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>
            {points_svg}
        </g>"##,
            g_idx = g_idx, delay = g_idx as f64 * 0.08, d = path_d, id = chart_id, points_svg = points_svg
        ));

        legend_items.push_str(&format!(
            r##"<g class="legend-item"><circle cx="676" cy="{cy}" r="5.5" fill="{color}"/><text x="690" y="{ty}" dominant-baseline="middle" style="fill: #111827 !important;" class="chart-text legend-label">{name}</text><text x="690" y="{vy}" dominant-baseline="middle" class="chart-text legend-value">Peak {peak_val}</text></g>"##,
            cy = 110 + g_idx * 36, ty = 108 + g_idx * 36, vy = 124 + g_idx * 36, color = color, name = escape(&group.name), peak_val = peak_val
        ));
    }

    let mut series_defs = String::new();
    for (i, color) in palette.iter().enumerate() {
        series_defs.push_str(&format!(
            r##"
        <linearGradient id="{id}_line_grad_{i}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="{color}" stop-opacity="0.8"/>
            <stop offset="100%" stop-color="{color}"/>
        </linearGradient>
        <linearGradient id="{id}_area_grad_{i}" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="{color}" stop-opacity="0.2"/>
            <stop offset="100%" stop-color="{color}" stop-opacity="0"/>
        </linearGradient>
        "##,
            id = chart_id, i = i, color = color
        ));
    }

    let extra_class = if use_dark { " dark-mode" } else { "" };

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
            <stop offset="0%" stop-color="#F7FAFF"/><stop offset="46%" stop-color="#EEF4FF"/><stop offset="100%" stop-color="#E7EEF8"/>
        </linearGradient>
        <radialGradient id="{id}_glow_blue" cx="19%" cy="8%" r="62%">
            <stop offset="0%" stop-color="#7DD3FC" stop-opacity="0.48"/><stop offset="45%" stop-color="#60A5FA" stop-opacity="0.16"/><stop offset="100%" stop-color="#60A5FA" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="{id}_glow_mint" cx="88%" cy="18%" r="56%">
            <stop offset="0%" stop-color="#99F6E4" stop-opacity="0.5"/><stop offset="48%" stop-color="#2DD4BF" stop-opacity="0.13"/><stop offset="100%" stop-color="#2DD4BF" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="{id}_glow_peach" cx="42%" cy="100%" r="62%">
            <stop offset="0%" stop-color="#FDBA74" stop-opacity="0.24"/><stop offset="100%" stop-color="#FDBA74" stop-opacity="0"/>
        </radialGradient>
        <linearGradient id="{id}_plot_glass" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.82"/><stop offset="100%" stop-color="#FFFFFF" stop-opacity="0.48"/>
        </linearGradient>
        <linearGradient id="{id}_plot_stroke" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.86"/><stop offset="100%" stop-color="#94A3B8" stop-opacity="0.22"/>
        </linearGradient>
        <filter id="{id}_apple_card_shadow" x="-20%" y="-20%" width="140%" height="150%">
            <feDropShadow dx="0" dy="22" stdDeviation="28" flood-color="#1E293B" flood-opacity="0.14"/><feDropShadow dx="0" dy="2" stdDeviation="4" flood-color="#FFFFFF" flood-opacity="0.9"/>
        </filter>
        <filter id="{id}_soft_point_shadow" x="-80%" y="-80%" width="260%" height="260%">
            <feDropShadow dx="0" dy="6" stdDeviation="6" flood-color="#0369A1" flood-opacity="0.22"/>
        </filter>
        <filter id="{id}_line_glow" x="-10%" y="-40%" width="120%" height="180%">
            <feGaussianBlur in="SourceGraphic" stdDeviation="3" result="blur"/><feColorMatrix in="blur" type="matrix" values="0 0 0 0 0.03  0 0 0 0 0.45  0 0 0 0 0.85  0 0 0 0 0.28 0" result="glow"/><feMerge><feMergeNode in="glow"/><feMergeNode in="SourceGraphic"/></feMerge>
        </filter>
        <clipPath id="{id}_plot_clip"><rect x="{px}" y="{py}" width="{pw}" height="{ph}" rx="26" ry="26"/></clipPath>
        {series_defs}
        <style>
            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');
            #{id} {{ --text-primary: #111827; --text-secondary: #6B7280; --text-muted: #8A94A6; --grid: #CBD5E1; --axis: #94A3B8; }}
            #{id} .chart-text {{ font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'SF Pro Text', Inter, system-ui, sans-serif; fill: #111827; letter-spacing: -0.01em; }}
            #{id} .chart-background {{ fill: url(#{id}_apple_bg); }}
            #{id} .plot-surface {{ fill: url(#{id}_plot_glass); stroke: url(#{id}_plot_stroke); stroke-opacity: 0.8; stroke-width: 1; }}
            #{id} .chart-grid {{ stroke: var(--grid); stroke-opacity: 0.42; }}
            #{id} .chart-axis {{ stroke: var(--axis); stroke-width: 1.25; stroke-opacity: 0.62; }}
            #{id} .chart-title {{ fill: #111827; font-size: 25px; font-weight: 760; letter-spacing: -0.025em; }}
            #{id} .axis-label {{ fill: #111827; font-size: 14px; font-weight: 600; }}
            #{id} .tick-label {{ fill: var(--text-muted); font-size: 11px; font-weight: 590; }}
            #{id} .legend-box {{ fill: rgba(255,255,255,0.66); stroke: rgba(255,255,255,0.72); stroke-width: 1; }}
            #{id} .legend-label {{ fill: #111827; font-size: 12px; font-weight: 650; }}
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
            #{id}.dark-mode {{ --text-primary: #F9FAFB; --text-secondary: #9CA3AF; --grid: #4B5563; --axis: #6B7280; }}
            #{id}.dark-mode .chart-background {{ fill: #111827; }}
            #{id}.dark-mode .plot-surface {{ fill: rgba(31, 41, 55, 0.48); }}
        </style>
    </defs>
    <rect width="{width}" height="{height}" rx="34" class="chart-background"/>
    <rect width="{width}" height="{height}" rx="34" fill="url(#{id}_glow_blue)"/>
    <rect width="{width}" height="{height}" rx="34" fill="url(#{id}_glow_mint)"/>
    <rect width="{width}" height="{height}" rx="34" fill="url(#{id}_glow_peach)"/>
    <circle cx="720" cy="80" r="80" fill="#FFFFFF" opacity="0.18"/>
    <circle cx="88" cy="425" r="112" fill="#FFFFFF" opacity="0.16"/>
    <g filter="url(#{id}_apple_card_shadow)">
        <rect x="34" y="28" width="732" height="444" rx="32" fill="rgba(255,255,255,0.38)" stroke="rgba(255,255,255,0.72)" stroke-width="1"/>
    </g>
    <text x="400" y="56" text-anchor="middle" class="chart-text chart-title">{title_esc}</text>
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
        title_esc = escape(title),
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
    }

    #[test]
    fn test_render_multi_line() {
        let input = "---- title=Comparison ---\nS1 | Jan | 10\nS1 | Feb | 20\nS2 | Jan | 15\nS2 | Feb | 25\n----";
        let controls = HashMap::new();
        let svg = render(input, &controls).unwrap();
        assert!(svg.contains("Comparison"));
        assert!(svg.contains("line-0"));
        assert!(svg.contains("line-1"));
    }
}
