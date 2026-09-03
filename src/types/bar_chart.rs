use crate::common::kv::parse_kv_body;
use crate::common::svg::escape;
use std::collections::HashMap;
use uuid::Uuid;

/// Grammar: `[docops,bar] ---- title=... theme=... --- Label | value ... ----`
/// (identical body syntax to pie_chart — see common::kv)
pub fn render(body: &str, _controls: &HashMap<String, String>) -> Result<String, String> {
    let data = parse_kv_body(body)?;
    let cfg = &data.config;

    let title = cfg.get("title").map(String::as_str).unwrap_or("Bar Chart");
    let subtitle = cfg.get("subtitle").map(String::as_str).unwrap_or("Visualized data report");
    let x_label = cfg.get("xLabel").map(String::as_str).unwrap_or("");
    let y_label = cfg.get("yLabel").map(String::as_str).unwrap_or("");

    let width = 960.0;
    let height = 560.0;
    let plot_x = 120.0;
    let plot_y = 150.0;
    let plot_w = 740.0;
    let plot_h = 300.0;

    let n = data.points.len();
    if n == 0 {
        return Ok(crate::common::svg::error_svg("No data points provided"));
    }

    let mut raw_max = 0.0;
    let mut peak_idx = 0;
    for (i, (_, v)) in data.points.iter().enumerate() {
        if *v > raw_max {
            raw_max = *v;
            peak_idx = i;
        }
    }
    let max_val = if raw_max <= 0.0 { 1.0 } else { raw_max * 1.1 };
    let chart_id = format!("id_{}", Uuid::new_v4());

    let mut anim_css = String::new();
    for i in 0..n {
        let idx = i + 1;
        anim_css.push_str(&format!(
            "            #{} .anim-{} {{ animation: growBar 760ms cubic-bezier(.2,.8,.2,1) {}ms both; }}\n",
            chart_id, idx, 100 + i * 90
        ));
        anim_css.push_str(&format!(
            "            #{} .val-{} {{ animation: revealValue 360ms ease {}ms both; }}\n",
            chart_id, idx, 760 + i * 90
        ));
    }

    let mut bars_html = String::new();
    let bar_hit_w = plot_w / (n as f64);
    let bar_inner_w = (bar_hit_w * 0.66).min(80.0);
    let bar_offset = (bar_hit_w - bar_inner_w) / 2.0;

    for (i, (label, value)) in data.points.iter().enumerate() {
        let idx = i + 1;
        let bar_h = (value / max_val) * plot_h;
        let x_hit = plot_x + (i as f64) * bar_hit_w;
        let x_inner = x_hit + bar_offset;
        let y_base = plot_y + plot_h;

        let fill = if i == peak_idx {
            "url(#barPeak)"
        } else if i % 2 == 0 {
            "url(#barBlue)"
        } else {
            "url(#barSteel)"
        };

        let peak_class = if i == peak_idx { " peak-label" } else { "" };

        bars_html.push_str(&format!(
            r##"    <g class="bar-wrap" tabindex="0" aria-label="{label}: {value}">
        <rect class="bar-hit" x="{x_hit:.1}" y="{plot_y:.1}" width="{bar_hit_w:.1}" height="{plot_h:.1}"/>
        <g transform="translate({x_inner:.1} {y_base:.1})">
            <g class="bar-inner anim-{idx}">
                <rect x="0" y="-{bar_h:.1}" width="{bar_inner_w:.1}" height="{bar_h:.1}" rx="{rx:.1}" ry="{rx:.1}" fill="{fill}"/>
                <rect class="bar-top-gloss" x="{gloss_x:.1}" y="-{gloss_y:.1}" width="{gloss_w:.1}" height="18" rx="9" ry="9" fill="#FFFFFF"/>
            </g>
        </g>
        <text class="x-label" x="{cx:.1}" y="{label_y:.1}" text-anchor="middle">{label}</text>
        <text class="value-label{peak_class} val-{idx}" x="{cx:.1}" y="{val_y:.1}" text-anchor="middle">{value}</text>
    </g>
"##,
            label = escape(label),
            value = value,
            x_hit = x_hit,
            plot_y = plot_y,
            bar_hit_w = bar_hit_w,
            plot_h = plot_h,
            x_inner = x_inner,
            y_base = y_base,
            idx = idx,
            bar_h = bar_h,
            bar_inner_w = bar_inner_w,
            rx = bar_inner_w / 2.0,
            fill = fill,
            gloss_x = bar_inner_w * 0.15,
            gloss_y = bar_h - 9.0,
            gloss_w = bar_inner_w * 0.7,
            cx = x_hit + bar_hit_w / 2.0,
            label_y = y_base + 24.0,
            peak_class = peak_class,
            val_y = y_base - bar_h - 12.0,
        ));
    }

    let mut y_ticks = String::new();
    let mut grid_lines = String::new();
    for i in 0..=10 {
        let frac = i as f64 / 10.0;
        let y = plot_y + plot_h - frac * plot_h;
        let val = max_val * frac;
        grid_lines.push_str(&format!(r##"        <line class="grid" x1="{plot_x:.1}" y1="{y:.1}" x2="{plot_rx:.1}" y2="{y:.1}"/>"##,
            plot_x = plot_x, y = y, plot_rx = plot_x + plot_w));
        y_ticks.push_str(&format!(r##"    <text class="tick-text" x="{tick_x:.1}" y="{tick_y:.1}" text-anchor="end">{val:.0}</text>"##,
            tick_x = plot_x - 12.0, tick_y = y + 4.0, val = val));
    }

    Ok(format!(
        r##"<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg" role="img" id="{chart_id}">
    <defs>
        <linearGradient id="premiumBackground" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="#F7FAFF"/>
            <stop offset="42%" stop-color="#EEF4FB"/>
            <stop offset="100%" stop-color="#F9FAFB"/>
        </linearGradient>
        <radialGradient id="ambientBlue" cx="22%" cy="12%" r="52%">
            <stop offset="0%" stop-color="#B8D8FF" stop-opacity="0.72"/>
            <stop offset="48%" stop-color="#DCEBFF" stop-opacity="0.28"/>
            <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="ambientGold" cx="77%" cy="20%" r="44%">
            <stop offset="0%" stop-color="#FFE7B0" stop-opacity="0.56"/>
            <stop offset="56%" stop-color="#FFF4D9" stop-opacity="0.18"/>
            <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0"/>
        </radialGradient>
        <linearGradient id="glassSurface" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.88"/>
            <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0.64"/>
        </linearGradient>
        <linearGradient id="glassStroke" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.95"/>
            <stop offset="100%" stop-color="#C7D2E1" stop-opacity="0.42"/>
        </linearGradient>
        <linearGradient id="barBlue" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="#4F7FAE"/>
            <stop offset="100%" stop-color="#8DB8DD"/>
        </linearGradient>
        <linearGradient id="barSteel" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="#5F86B2"/>
            <stop offset="100%" stop-color="#A4C6E2"/>
        </linearGradient>
        <linearGradient id="barPeak" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="#D98613"/>
            <stop offset="48%" stop-color="#F7B034"/>
            <stop offset="100%" stop-color="#FFE2A4"/>
        </linearGradient>
        <filter id="premiumShadow" x="-20%" y="-20%" width="140%" height="150%">
            <feDropShadow dx="0" dy="18" stdDeviation="28" flood-color="#1B2735" flood-opacity="0.12"/>
            <feDropShadow dx="0" dy="4" stdDeviation="8" flood-color="#1B2735" flood-opacity="0.06"/>
        </filter>
        <filter id="softBarShadow" x="-30%" y="-20%" width="160%" height="150%">
            <feDropShadow dx="0" dy="10" stdDeviation="10" flood-color="#31516F" flood-opacity="0.16"/>
        </filter>
        <clipPath id="chartClip">
            <rect x="{plot_x}" y="{plot_y}" width="{plot_w}" height="{plot_h}" rx="22" ry="22"/>
        </clipPath>
        <style>
            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&amp;display=swap');
            #{chart_id} {{
                --bg: #F6F8FB;
                --surface: rgba(255,255,255,0.74);
                --text: #17202A;
                --text-soft: #657184;
                --grid: #7C8BA1;
                --axis: #9AA8BA;
                --accent: #F5A524;
            }}
            #{chart_id} text {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; }}
            #{chart_id} .title {{ font-size: 31px; font-weight: 800; letter-spacing: -0.02em; fill: var(--text); }}
            #{chart_id} .subtitle {{ font-size: 13px; font-weight: 600; fill: var(--text-soft); }}
            #{chart_id} .eyebrow {{ font-size: 11px; font-weight: 800; letter-spacing: 0.14em; fill: var(--accent); }}
            #{chart_id} .grid {{ stroke: var(--grid); stroke-width: 1; stroke-opacity: 0.13; stroke-dasharray: 1 10; stroke-linecap: round; }}
            #{chart_id} .axis {{ stroke: var(--axis); stroke-width: 1.2; stroke-opacity: 0.3; stroke-linecap: round; }}
            #{chart_id} .tick-text, #{chart_id} .x-label {{ font-size: 12px; font-weight: 600; fill: var(--text-soft); }}
            #{chart_id} .y-label {{ font-size: 13px; font-weight: 700; fill: var(--text-soft); }}
            #{chart_id} .value-label {{ font-size: 12px; font-weight: 800; fill: var(--text); opacity: 0; pointer-events: none; }}
            #{chart_id} .peak-label {{ fill: #8A5200; }}
            #{chart_id} .bar-hit {{ fill: transparent; }}
            #{chart_id} .bar-inner {{ transform-box: fill-box; transform-origin: 50% 100%; filter: url(#softBarShadow); transition: transform 260ms ease, filter 260ms ease, opacity 260ms ease; }}
            #{chart_id} .bar-wrap:hover .bar-inner, #{chart_id} .bar-wrap:focus .bar-inner {{ transform: scaleX(1.07) scaleY(1.025); filter: url(#softBarShadow) saturate(1.16); }}
            #{chart_id} .bar-wrap:hover .value-label, #{chart_id} .bar-wrap:focus .value-label {{ opacity: 1; }}
            #{chart_id} .bar-top-gloss {{ opacity: 0.38; mix-blend-mode: screen; }}
            @keyframes growBar {{ from {{ transform: scaleY(0); opacity: 0.2; }} to {{ transform: scaleY(1); opacity: 1; }} }}
            @keyframes revealValue {{ from {{ opacity: 0; transform: translateY(7px); }} to {{ opacity: 0.72; transform: translateY(0); }} }}
            @keyframes floatCard {{ from {{ opacity: 0; transform: translateY(10px); }} to {{ opacity: 1; transform: translateY(0); }} }}
            #{chart_id} .glass-card {{ animation: floatCard 680ms cubic-bezier(.2,.8,.2,1) both; }}
{anim_css}
        </style>
    </defs>

    <rect width="100%" height="100%" fill="url(#premiumBackground)"/>
    <rect width="100%" height="100%" fill="url(#ambientBlue)"/>
    <rect width="100%" height="100%" fill="url(#ambientGold)"/>

    <g class="glass-card" filter="url(#premiumShadow)">
        <rect x="36" y="34" width="888" height="492" rx="34" ry="34" fill="url(#glassSurface)"/>
        <rect x="36.5" y="34.5" width="887" height="491" rx="33.5" ry="33.5" fill="none" stroke="url(#glassStroke)" stroke-width="1"/>
    </g>

    <g>
        <text class="eyebrow" x="78" y="80">PREMIUM METRICS</text>
        <text class="title" x="78" y="116">{title}</text>
        <text class="subtitle" x="78" y="140">{subtitle}</text>
    </g>

    <g clip-path="url(#chartClip)">
{grid_lines}
    </g>

    <line class="axis" x1="{plot_x}" y1="{plot_y}" x2="{plot_x}" y2="{plot_by}"/>
    <line class="axis" x1="{plot_x}" y1="{plot_by}" x2="{plot_rx}" y2="{plot_by}"/>

{y_ticks}

    <text class="y-label" x="50" y="{y_label_y}" text-anchor="middle" transform="rotate(-90 50 {y_label_y})">{y_label}</text>
    <text class="x-label" x="{width_half}" y="498" text-anchor="middle">{x_label}</text>

{bars_html}
</svg>"##,
        width = width,
        height = height,
        title = escape(title),
        subtitle = escape(subtitle),
        plot_x = plot_x,
        plot_y = plot_y,
        plot_w = plot_w,
        plot_h = plot_h,
        plot_rx = plot_x + plot_w,
        plot_by = plot_y + plot_h,
        anim_css = anim_css,
        grid_lines = grid_lines,
        y_ticks = y_ticks,
        y_label = escape(y_label),
        y_label_y = plot_y + plot_h / 2.0,
        x_label = escape(x_label),
        width_half = width / 2.0,
        bars_html = bars_html,
        chart_id = chart_id,
    ))
}