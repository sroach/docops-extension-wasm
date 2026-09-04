use crate::common::kv::{parse_kv_body, KvBody};
use crate::common::svg::escape;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
enum BarMode {
    Simple,
    Grouped,
    Stacked,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BarShape {
    Rect,
    Rounded,
    Cylinder,
}

struct BarGroup {
    name: String,
    points: Vec<(String, f64)>,
    total: f64,
}

struct BarContext<'a> {
    chart_id: String,
    plot_x: f64,
    plot_y: f64,
    plot_w: f64,
    plot_h: f64,
    max_val: f64,
    peak_idx: usize,
    shape: BarShape,
    data: &'a KvBody,
}

fn group_points(points: &Vec<(String, f64)>) -> Vec<BarGroup> {
    let mut groups: Vec<BarGroup> = Vec::new();
    let mut group_map: HashMap<String, usize> = HashMap::new();

    for (label, value) in points {
        let parts: Vec<&str> = label.split('|').map(|s| s.trim()).collect();
        let (group_name, sub_label) = if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            (label.clone(), "".to_string())
        };

        if let Some(&idx) = group_map.get(&group_name) {
            groups[idx].points.push((sub_label, *value));
            groups[idx].total += *value;
        } else {
            group_map.insert(group_name.clone(), groups.len());
            groups.push(BarGroup {
                name: group_name,
                points: vec![(sub_label, *value)],
                total: *value,
            });
        }
    }
    groups
}

/// Grammar: `[docops,bar] ---- title=... theme=... --- Label | value ... ----`
/// (identical body syntax to pie_chart — see common::kv)
pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let data = parse_kv_body(body)?;
    let cfg = &data.config;

    let use_dark = controls.get("useDark").map(|s| s == "true").unwrap_or(false)
        || cfg.get("theme").map(|s| s.as_str()) == Some("dark");

    let title = cfg.get("title").map(String::as_str).unwrap_or("Bar Chart");
    let subtitle = cfg.get("subtitle").map(String::as_str).unwrap_or("Visualized data report");
    let x_label = cfg.get("xLabel").map(String::as_str).unwrap_or("");
    let y_label = cfg.get("yLabel").map(String::as_str).unwrap_or("");

    let mode = match cfg.get("mode").map(|s| s.to_lowercase()).as_deref() {
        Some("grouped") => BarMode::Grouped,
        Some("stacked") => BarMode::Stacked,
        _ => BarMode::Simple,
    };
    let shape = match cfg.get("shape").map(|s| s.to_lowercase()).as_deref() {
        Some("rect") => BarShape::Rect,
        Some("cylinder") => BarShape::Cylinder,
        _ => BarShape::Rounded,
    };

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

    let groups = group_points(&data.points);
    
    let mut raw_max = 0.0;
    let mut peak_idx = 0;
    
    match mode {
        BarMode::Stacked => {
            for (i, g) in groups.iter().enumerate() {
                if g.total > raw_max {
                    raw_max = g.total;
                    peak_idx = i;
                }
            }
        },
        _ => {
            for (i, (_, v)) in data.points.iter().enumerate() {
                if *v > raw_max {
                    raw_max = *v;
                    peak_idx = i;
                }
            }
        }
    }
    
    let max_val = if raw_max <= 0.0 { 1.0 } else { raw_max * 1.1 };
    let chart_id = format!("id_{}", Uuid::new_v4());

    let ctx = BarContext {
        chart_id: chart_id.clone(),
        plot_x,
        plot_y,
        plot_w,
        plot_h,
        max_val,
        peak_idx,
        shape,
        data: &data,
    };

    let mut bars_html = String::new();
    let mut anim_css = String::new();

    match mode {
        BarMode::Simple => render_simple_bar(&ctx, &mut bars_html, &mut anim_css),
        BarMode::Grouped => render_grouped_bar(&ctx, &mut bars_html, &mut anim_css),
        BarMode::Stacked => render_stacked_bar(&ctx, &mut bars_html, &mut anim_css),
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

    let extra_class = if use_dark { " dark-mode" } else { "" };

    Ok(format!(
        r##"<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg" role="img" id="{chart_id}" class="bar-chart-container{extra_class}" preserveAspectRatio="xMidYMid meet">
    <defs>
        <linearGradient id="{chart_id}__premiumBackground" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--premium-bg-0)"/>
            <stop offset="42%" stop-color="var(--premium-bg-42)"/>
            <stop offset="100%" stop-color="var(--premium-bg-100)"/>
        </linearGradient>
        <radialGradient id="{chart_id}__ambientBlue" cx="22%" cy="12%" r="52%">
            <stop offset="0%" stop-color="#B8D8FF" stop-opacity="var(--ambient-blue-op)"/>
            <stop offset="48%" stop-color="#DCEBFF" stop-opacity="0.28"/>
            <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="{chart_id}__ambientGold" cx="77%" cy="20%" r="44%">
            <stop offset="0%" stop-color="#FFE7B0" stop-opacity="var(--ambient-gold-op)"/>
            <stop offset="56%" stop-color="#FFF4D9" stop-opacity="0.18"/>
            <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0"/>
        </radialGradient>
        <linearGradient id="{chart_id}__glassSurface" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--surface)" stop-opacity="var(--glass-op)"/>
            <stop offset="100%" stop-color="var(--surface)" stop-opacity="0.64"/>
        </linearGradient>
        <linearGradient id="{chart_id}__glassStroke" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--glass-stroke-0)" stop-opacity="0.95"/>
            <stop offset="100%" stop-color="var(--glass-stroke-100)" stop-opacity="0.42"/>
        </linearGradient>
        <linearGradient id="{chart_id}__barBlue" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="var(--bar-blue-0)"/>
            <stop offset="100%" stop-color="var(--bar-blue-100)"/>
        </linearGradient>
        <linearGradient id="{chart_id}__barSteel" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="var(--bar-steel-0)"/>
            <stop offset="100%" stop-color="var(--bar-steel-100)"/>
        </linearGradient>
        <linearGradient id="{chart_id}__barPeak" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="var(--bar-peak-0)"/>
            <stop offset="48%" stop-color="var(--bar-peak-48)"/>
            <stop offset="100%" stop-color="var(--bar-peak-100)"/>
        </linearGradient>
        <linearGradient id="{chart_id}__barPal_0" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="var(--bar-blue-0)"/>
            <stop offset="100%" stop-color="var(--bar-blue-100)"/>
        </linearGradient>
        <linearGradient id="{chart_id}__barPal_1" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="var(--bar-purp-0)"/>
            <stop offset="100%" stop-color="var(--bar-purp-100)"/>
        </linearGradient>
        <linearGradient id="{chart_id}__barPal_2" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="var(--bar-green-0)"/>
            <stop offset="100%" stop-color="var(--bar-green-100)"/>
        </linearGradient>
        <linearGradient id="{chart_id}__barPal_3" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="var(--bar-steel-0)"/>
            <stop offset="100%" stop-color="var(--bar-steel-100)"/>
        </linearGradient>
        <linearGradient id="{chart_id}__barPal_4" x1="0" y1="1" x2="0" y2="0">
            <stop offset="0%" stop-color="var(--bar-rose-0)"/>
            <stop offset="100%" stop-color="var(--bar-rose-100)"/>
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
                --ambient-blue-op: 0.72;
                --ambient-gold-op: 0.56;
                --glass-op: 0.88;
                --premium-bg-0: #F7FAFF;
                --premium-bg-42: #EEF4FB;
                --premium-bg-100: #F9FAFB;
                --glass-stroke-0: #FFFFFF;
                --glass-stroke-100: #C7D2E1;
                --bar-blue-0: #4F7FAE;
                --bar-blue-100: #8DB8DD;
                --bar-steel-0: #5F86B2;
                --bar-steel-100: #A4C6E2;
                --bar-peak-0: #D98613;
                --bar-peak-48: #F7B034;
                --bar-rose-100: #FFE2A4;
                --peak-label: #8A5200;
                --bar-purp-0: #7C3AED;
                --bar-purp-100: #A78BFA;
                --bar-green-0: #059669;
                --bar-green-100: #34D399;
                --bar-rose-0: #E11D48;
                --bar-rose-100: #FB7185;
            }}

            @media (prefers-color-scheme: dark) {{
                #{chart_id} {{
                    --bg: #111827;
                    --surface: rgba(31, 41, 55, 0.74);
                    --text: #F9FAFB;
                    --text-soft: #9CA3AF;
                    --grid: #4B5563;
                    --axis: #6B7280;
                    --accent: #F5A524;
                    --ambient-blue-op: 0.4;
                    --ambient-gold-op: 0.3;
                    --glass-op: 0.2;
                    --premium-bg-0: #0F172A;
                    --premium-bg-42: #111827;
                    --premium-bg-100: #1F2937;
                    --glass-stroke-0: #374151;
                    --glass-stroke-100: #111827;
                    --bar-blue-0: #3B82F6;
                    --bar-blue-100: #60A5FA;
                    --bar-steel-0: #4B5563;
                    --bar-steel-100: #9CA3AF;
                    --bar-peak-0: #F59E0B;
                    --bar-peak-48: #FBBF24;
                    --bar-peak-100: #FDE68A;
                    --peak-label: #FDE68A;
                    --bar-purp-0: #8B5CF6;
                    --bar-purp-100: #C4B5FD;
                    --bar-green-0: #10B981;
                    --bar-green-100: #6EE7B7;
                    --bar-rose-0: #F43F5E;
                    --bar-rose-100: #FDA4AF;
                }}
            }}

            #{chart_id}.dark-mode {{
                --bg: #111827;
                --surface: rgba(31, 41, 55, 0.74);
                --text: #F9FAFB;
                --text-soft: #9CA3AF;
                --grid: #4B5563;
                --axis: #6B7280;
                --accent: #F5A524;
                --ambient-blue-op: 0.4;
                --ambient-gold-op: 0.3;
                --glass-op: 0.2;
                --premium-bg-0: #0F172A;
                --premium-bg-42: #111827;
                --premium-bg-100: #1F2937;
                --glass-stroke-0: #374151;
                --glass-stroke-100: #111827;
                --bar-blue-0: #3B82F6;
                --bar-blue-100: #60A5FA;
                --bar-steel-0: #4B5563;
                --bar-steel-100: #9CA3AF;
                --bar-peak-0: #F59E0B;
                --bar-peak-48: #FBBF24;
                --bar-peak-100: #FDE68A;
                --peak-label: #FDE68A;
                --bar-purp-0: #8B5CF6;
                --bar-purp-100: #C4B5FD;
                --bar-green-0: #10B981;
                --bar-green-100: #6EE7B7;
                --bar-rose-0: #F43F5E;
                --bar-rose-100: #FDA4AF;
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
            #{chart_id} .peak-label {{ fill: var(--peak-label); }}
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

    <rect width="100%" height="100%" fill="url(#{chart_id}__premiumBackground)"/>
    <rect width="100%" height="100%" fill="url(#{chart_id}__ambientBlue)"/>
    <rect width="100%" height="100%" fill="url(#{chart_id}__ambientGold)"/>

    <g class="glass-card" filter="url(#premiumShadow)">
        <rect x="36" y="34" width="888" height="492" rx="34" ry="34" fill="url(#{chart_id}__glassSurface)"/>
        <rect x="36.5" y="34.5" width="887" height="491" rx="33.5" ry="33.5" fill="none" stroke="url(#{chart_id}__glassStroke)" stroke-width="1"/>
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
        extra_class = extra_class,
        chart_id = chart_id,
    ))
}

fn render_simple_bar(ctx: &BarContext, bars_html: &mut String, anim_css: &mut String) {
    let n = ctx.data.points.len();
    let bar_hit_w = ctx.plot_w / (n as f64);
    let bar_inner_w = (bar_hit_w * 0.66).min(80.0);
    let bar_offset = (bar_hit_w - bar_inner_w) / 2.0;

    for (i, (label, value)) in ctx.data.points.iter().enumerate() {
        let idx = i + 1;
        let bar_h = (value / ctx.max_val) * ctx.plot_h;
        let x_hit = ctx.plot_x + (i as f64) * bar_hit_w;
        let x_inner = x_hit + bar_offset;
        let y_base = ctx.plot_y + ctx.plot_h;

        let fill = if i == ctx.peak_idx {
            format!("url(#{}__barPeak)", ctx.chart_id)
        } else if i % 2 == 0 {
            format!("url(#{}__barBlue)", ctx.chart_id)
        } else {
            format!("url(#{}__barSteel)", ctx.chart_id)
        };

        let peak_class = if i == ctx.peak_idx { " peak-label" } else { "" };
        
        anim_css.push_str(&format!(
            "            #{} .anim-{} {{ animation: growBar 760ms cubic-bezier(.2,.8,.2,1) {}ms both; }}\n",
            ctx.chart_id, idx, 100 + i * 90
        ));
        anim_css.push_str(&format!(
            "            #{} .val-{} {{ animation: revealValue 360ms ease {}ms both; }}\n",
            ctx.chart_id, idx, 760 + i * 90
        ));

        let shape_svg = match ctx.shape {
            BarShape::Rect => render_rect_bar(0.0, -bar_h, bar_inner_w, bar_h, &fill),
            BarShape::Rounded => render_rounded_bar(0.0, -bar_h, bar_inner_w, bar_h, &fill),
            BarShape::Cylinder => render_cylinder_bar(0.0, -bar_h, bar_inner_w, bar_h, &fill),
        };

        bars_html.push_str(&format!(
            r##"    <g class="bar-wrap" tabindex="0" aria-label="{label}: {value}">
        <rect class="bar-hit" x="{x_hit:.1}" y="{plot_y:.1}" width="{bar_hit_w:.1}" height="{plot_h:.1}"/>
        <g transform="translate({x_inner:.1} {y_base:.1})">
            <g class="bar-inner anim-{idx}">
                {shape_svg}
            </g>
        </g>
        <text class="x-label" x="{cx:.1}" y="{label_y:.1}" text-anchor="middle">{label}</text>
        <text class="value-label{peak_class} val-{idx}" x="{cx:.1}" y="{val_y:.1}" text-anchor="middle">{value}</text>
    </g>
"##,
            label = escape(label),
            value = value,
            x_hit = x_hit,
            plot_y = ctx.plot_y,
            bar_hit_w = bar_hit_w,
            plot_h = ctx.plot_h,
            x_inner = x_inner,
            y_base = y_base,
            idx = idx,
            shape_svg = shape_svg,
            cx = x_hit + bar_hit_w / 2.0,
            label_y = y_base + 24.0,
            peak_class = peak_class,
            val_y = y_base - bar_h - 12.0,
        ));
    }
}

fn render_grouped_bar(ctx: &BarContext, bars_html: &mut String, anim_css: &mut String) {
    let groups = group_points(&ctx.data.points);
    if groups.is_empty() { return; }

    let num_groups = groups.len();
    let group_hit_w = ctx.plot_w / (num_groups as f64);
    
    let max_bars_per_group = groups.iter().map(|g| g.points.len()).max().unwrap_or(1);
    let bar_inner_w = (group_hit_w * 0.7 / (max_bars_per_group as f64)).min(60.0);
    let group_padding = group_hit_w * 0.15;
    
    let mut global_idx = 0;
    for (g_idx, group) in groups.iter().enumerate() {
        let x_group_start = ctx.plot_x + (g_idx as f64) * group_hit_w + group_padding;
        let y_base = ctx.plot_y + ctx.plot_h;

        for (p_idx, (sub_label, value)) in group.points.iter().enumerate() {
            global_idx += 1;
            let bar_h = (value / ctx.max_val) * ctx.plot_h;
            let x_bar = x_group_start + (p_idx as f64) * bar_inner_w;
            
            let fill = format!("url(#{}__barPal_{})", ctx.chart_id, p_idx % 5);
            
            anim_css.push_str(&format!(
                "            #{} .anim-{} {{ animation: growBar 760ms cubic-bezier(.2,.8,.2,1) {}ms both; }}\n",
                ctx.chart_id, global_idx, 100 + global_idx * 50
            ));
            anim_css.push_str(&format!(
                "            #{} .val-{} {{ animation: revealValue 360ms ease {}ms both; }}\n",
                ctx.chart_id, global_idx, 760 + global_idx * 50
            ));

            let shape_svg = match ctx.shape {
                BarShape::Rect => render_rect_bar(0.0, -bar_h, bar_inner_w * 0.9, bar_h, &fill),
                BarShape::Rounded => render_rounded_bar(0.0, -bar_h, bar_inner_w * 0.9, bar_h, &fill),
                BarShape::Cylinder => render_cylinder_bar(0.0, -bar_h, bar_inner_w * 0.9, bar_h, &fill),
            };

            let display_label = if sub_label.is_empty() { group.name.clone() } else { format!("{} ({})", group.name, sub_label) };

            bars_html.push_str(&format!(
                r##"    <g class="bar-wrap" tabindex="0" aria-label="{display_label}: {value}">
        <g transform="translate({x_bar:.1} {y_base:.1})">
            <g class="bar-inner anim-{idx}">
                {shape_svg}
            </g>
        </g>
        <text class="value-label val-{idx}" x="{cx:.1}" y="{val_y:.1}" text-anchor="middle">{value}</text>
    </g>
"##,
                display_label = escape(&display_label),
                value = value,
                x_bar = x_bar,
                y_base = y_base,
                idx = global_idx,
                shape_svg = shape_svg,
                cx = x_bar + bar_inner_w / 2.0,
                val_y = y_base - bar_h - 12.0,
            ));
        }
        
        // Group label
        let group_cx = ctx.plot_x + (g_idx as f64) * group_hit_w + group_hit_w / 2.0;
        bars_html.push_str(&format!(
            r##"    <text class="x-label" x="{group_cx:.1}" y="{label_y:.1}" text-anchor="middle" font-weight="bold">{group_name}</text>
"##,
            group_cx = group_cx,
            label_y = ctx.plot_y + ctx.plot_h + 24.0,
            group_name = escape(&group.name),
        ));
    }
}

fn render_stacked_bar(ctx: &BarContext, bars_html: &mut String, anim_css: &mut String) {
    let groups = group_points(&ctx.data.points);
    if groups.is_empty() { return; }

    let num_groups = groups.len();
    let bar_hit_w = ctx.plot_w / (num_groups as f64);
    let bar_inner_w = (bar_hit_w * 0.6).min(80.0);
    let bar_offset = (bar_hit_w - bar_inner_w) / 2.0;
    
    let mut global_idx = 0;
    for (g_idx, group) in groups.iter().enumerate() {
        let x_hit = ctx.plot_x + (g_idx as f64) * bar_hit_w;
        let x_inner = x_hit + bar_offset;
        let mut current_y = ctx.plot_y + ctx.plot_h;
        let y_base = current_y;

        for (p_idx, (sub_label, value)) in group.points.iter().enumerate() {
            global_idx += 1;
            let segment_h = (value / ctx.max_val) * ctx.plot_h;
            let fill = format!("url(#{}__barPal_{})", ctx.chart_id, p_idx % 5);
            
            anim_css.push_str(&format!(
                "            #{} .anim-{} {{ animation: growBar 760ms cubic-bezier(.2,.8,.2,1) {}ms both; }}\n",
                ctx.chart_id, global_idx, 100 + g_idx * 90 + p_idx * 30
            ));

            // For stacked bars, we use Rect for all if shape isn't specifically handled, 
            // but let's try to respect shape. Rounded stacked bars look weird though.
            let shape_svg = match ctx.shape {
                BarShape::Rect => render_rect_bar(0.0, -segment_h, bar_inner_w, segment_h, &fill),
                BarShape::Cylinder => render_cylinder_bar(0.0, -segment_h, bar_inner_w, segment_h, &fill),
                BarShape::Rounded => {
                    // Only round top and bottom of the whole stack? Hard with current structure.
                    // Just use rect for intermediate segments.
                    render_rect_bar(0.0, -segment_h, bar_inner_w, segment_h, &fill)
                }
            };

            bars_html.push_str(&format!(
                r##"    <g class="bar-wrap" tabindex="0" aria-label="{group_name} {sub_label}: {value}">
        <g transform="translate({x_inner:.1} {current_y:.1})">
            <g class="bar-inner anim-{idx}">
                {shape_svg}
            </g>
        </g>
    </g>
"##,
                group_name = escape(&group.name),
                sub_label = escape(sub_label),
                value = value,
                x_inner = x_inner,
                current_y = current_y,
                idx = global_idx,
                shape_svg = shape_svg,
            ));
            
            current_y -= segment_h;
        }
        
        // Group label and total value
        let cx = x_hit + bar_hit_w / 2.0;
        bars_html.push_str(&format!(
            r##"    <text class="x-label" x="{cx:.1}" y="{label_y:.1}" text-anchor="middle">{group_name}</text>
    <text class="value-label val-stack-{g_idx}" x="{cx:.1}" y="{val_y:.1}" text-anchor="middle" style="opacity: 1;">{total:.0}</text>
"##,
            cx = cx,
            label_y = y_base + 24.0,
            group_name = escape(&group.name),
            val_y = current_y - 12.0,
            total = group.total,
            g_idx = g_idx,
        ));
    }
}

fn render_rect_bar(x: f64, y: f64, w: f64, h: f64, fill: &str) -> String {
    format!(r##"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{fill}"/>"##,
        x=x, y=y, w=w, h=h, fill=fill)
}

fn render_rounded_bar(x: f64, y: f64, w: f64, h: f64, fill: &str) -> String {
    let rx = w / 2.0;
    let gloss_x = w * 0.15;
    let gx = x + gloss_x;
    let gy = y + 9.0;
    let gw = w * 0.7;
    format!(
        r##"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{rx:.1}" ry="{rx:.1}" fill="{fill}"/>
<rect class="bar-top-gloss" x="{gx:.1}" y="{gy:.1}" width="{gw:.1}" height="18" rx="9" ry="9" fill="#FFFFFF"/>"##,
        x=x, y=y, w=w, h=h, rx=rx, fill=fill, gx=gx, gy=gy, gw=gw
    )
}

fn render_cylinder_bar(x: f64, y: f64, w: f64, h: f64, fill: &str) -> String {
    let rx = w / 2.0;
    let ry = (w * 0.15).min(10.0);
    let cx = x + rx;
    format!(
        r##"<ellipse cx="{cx:.1}" cy="{y_bottom:.1}" rx="{rx:.1}" ry="{ry:.1}" fill="{fill}" opacity="0.8"/>
<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{fill}"/>
<ellipse cx="{cx:.1}" cy="{y:.1}" rx="{rx:.1}" ry="{ry:.1}" fill="{fill}"/>
<ellipse cx="{cx:.1}" cy="{y:.1}" rx="{rx:.1}" ry="{ry:.1}" fill="#FFFFFF" fill-opacity="0.3"/>"##,
        x=x, y=y, w=w, h=h, rx=rx, ry=ry, fill=fill, cx=cx, y_bottom=y+h
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_bar_shapes() {
        let body = "---- title=Test Bar shape=rect --- A | 10.0 ----";
        let result = render(body, &HashMap::new());
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("__barPeak)"));
        
        let rect_svg = render_rect_bar(0.0, -100.0, 50.0, 100.0, "blue");
        assert!(!rect_svg.contains("rx="));

        let body_cylinder = "---- title=Test Bar shape=cylinder --- A | 10.0 ----";
        let result_cylinder = render(body_cylinder, &HashMap::new());
        assert!(result_cylinder.is_ok());
        let svg_cylinder = result_cylinder.unwrap();
        assert!(svg_cylinder.contains("<ellipse"));
    }

    #[test]
    fn test_render_bar_modes() {
        let body = "---- title=Test Bar mode=grouped --- A | 10.0 ----";
        let result = render(body, &HashMap::new());
        assert!(result.is_ok());

        let body_multi = "---- mode=grouped --- P1 | Q1 | 100 P1 | Q2 | 200 P2 | Q1 | 150 ----";
        let result_multi = render(body_multi, &HashMap::new());
        assert!(result_multi.is_ok(), "Error: {:?}", result_multi.err());
        let svg = result_multi.unwrap();
        assert!(svg.contains("P1"));
        assert!(svg.contains("P2"));
        assert!(svg.contains("Q1")); 
    }

    #[test]
    fn test_render_bar_stacked() {
        let body = "---- mode=stacked --- A | B | 10 A | C | 20 ----";
        let result = render(body, &HashMap::new());
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("30")); // Total value label
    }

    #[test]
    fn test_bar_dark_mode() {
        let body = "---- title=Test --- A | 10 ----";
        
        // Default mode
        let svg_light = render(body, &HashMap::new()).unwrap();
        assert!(svg_light.contains("--bg: #F6F8FB"));
        assert!(svg_light.contains("@media (prefers-color-scheme: dark)"));
        
        // Forced dark mode
        let mut controls = HashMap::new();
        controls.insert("useDark".to_string(), "true".to_string());
        let svg_dark = render(body, &controls).unwrap();
        assert!(svg_dark.contains("class=\"bar-chart-container dark-mode\""));
    }
}