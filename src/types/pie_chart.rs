use crate::common::kv::parse_kv_body;
use crate::common::svg::{escape, theme};
use std::collections::HashMap;
use std::f64::consts::PI;
use uuid::Uuid;

/// Grammar: `[docops,pie] ---- title=... theme=... --- Label | value ... ----`
pub fn render(body: &str, _controls: &HashMap<String, String>) -> Result<String, String> {
    let data = parse_kv_body(body)?;

    if data.points.iter().any(|(_, v)| *v < 0.0) {
        return Err("pie chart values must be non-negative".into());
    }
    let total: f64 = data.points.iter().map(|(_, v)| v).sum();
    if total <= 0.0 {
        return Err("pie chart values must sum to more than zero".into());
    }

    let cfg = &data.config;
    let visual_version = cfg.get("visualVersion").map(|s| s.as_str()).unwrap_or("0");
    let shape = cfg.get("shape").map(|s| s.as_str()).unwrap_or("pie");

    if shape == "donut" {
        return render_donut(&data, total, _controls);
    }

    if visual_version == "1" {
        return render_v1(&data, total, _controls);
    }

    // Fallback to V0 (the original implementation)
    let title = cfg.get("title").map(String::as_str).unwrap_or("");
    let theme_name = cfg.get("theme").map(String::as_str).unwrap_or("default");
    let t = theme(theme_name);

    let cx = 200.0;
    let cy = 220.0;
    let r = 140.0;

    let mut slices = String::new();
    let mut legend = String::new();
    let mut angle = -PI / 2.0; // start at 12 o'clock

    for (i, (label, value)) in data.points.iter().enumerate() {
        let frac = value / total;
        let sweep = frac * 2.0 * PI;
        let end = angle + sweep;

        let x1 = cx + r * angle.cos();
        let y1 = cy + r * angle.sin();
        let x2 = cx + r * end.cos();
        let y2 = cy + r * end.sin();
        let large_arc = if sweep > PI { 1 } else { 0 };
        let color = t.palette[i % t.palette.len()];

        slices.push_str(&format!(
            r##"<path d="M {cx:.1},{cy:.1} L {x1:.2},{y1:.2} A {r:.1},{r:.1} 0 {large_arc} 1 {x2:.2},{y2:.2} Z" fill="{color}" stroke="{bg}" stroke-width="1"><title>{label}: {value} ({pct:.1}%)</title></path>"##,
            cx = cx, cy = cy, r = r, large_arc = large_arc, color = color, bg = t.background,
            label = escape(label), value = value, pct = frac * 100.0,
        ));

        let ly = 40.0 + (i as f64) * 22.0;
        legend.push_str(&format!(
            r##"<rect x="380" y="{ly:.1}" width="14" height="14" fill="{color}"/><text x="400" y="{ty:.1}" font-size="12" fill="{text}">{label} ({pct:.0}%)</text>"##,
            ly = ly, ty = ly + 11.0, color = color, text = t.text,
            label = escape(label), pct = frac * 100.0,
        ));

        angle = end;
    }

    Ok(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 400" font-family="system-ui, sans-serif">
  <rect width="600" height="400" fill="{bg}"/>
  <text x="300" y="28" text-anchor="middle" font-size="18" font-weight="600" fill="{text}">{title}</text>
  {slices}
  {legend}
</svg>"##,
        bg = t.background,
        text = t.text,
        title = escape(title),
        slices = slices,
        legend = legend,
    ))
}

fn render_donut(data: &crate::common::kv::KvBody, total: f64, _controls: &HashMap<String, String>) -> Result<String, String> {
    let chart_id = format!("id_{}", Uuid::new_v4());
    let cfg = &data.config;
    let title = cfg.get("title").map(String::as_str).unwrap_or("Donut Chart");
    let subtitle = cfg.get("subtitle").map(String::as_str).unwrap_or("Rounded donut · bottom legend");

    let is_dark = _controls.get("useDark").map(|s| s.as_str()) == Some("true")
        || cfg.get("theme").map(|s| s.as_str()) == Some("dark");

    // Colors & Styles from design
    let bg_colors = if is_dark { ("#08111d", "#102033", "#11180f") } else { ("#f8fafc", "#eef4f8", "#f9f4e8") };
    let halo_colors = if is_dark { ("#4ade80", "0.18", "#14b8a6", "0.10", "#08111d") } else { ("#ffffff", "0.72", "#d9f99d", "0.18", "#f8fafc") };
    let tick_color = if is_dark { "#334155" } else { "#cbd5e1" };
    let tick_op = if is_dark { "0.38" } else { "0.32" };
    let header_prefix = if is_dark { "#5eead4" } else { "#0f766e" };
    let header_title = if is_dark { "#f8fafc" } else { "#111827" };
    let header_sub = if is_dark { "#cbd5e1" } else { "#475569" };
    let total_box_bg = if is_dark { "#1d3550" } else { "#eaf2ff" };
    let total_box_stroke = if is_dark { "#3b5f83" } else { "#c7d9f4" };
    let total_prefix = if is_dark { "#bae6fd" } else { "#0f766e" };
    let total_val = if is_dark { "#ffffff" } else { "#111827" };
    let center_label = if is_dark { "#cbd5e1" } else { "#64748b" };
    let inner_fill = if is_dark { "#0b1220" } else { "#ffffff" };
    let center_text = if is_dark { "#ffffff" } else { "#111827" };
    let label_line = if is_dark { "#5eead4" } else { "#0f766e" };
    let badge_bg = if is_dark { "#06191E" } else { "#172033" };
    let badge_text = if is_dark { "#F7FBFF" } else { "#FFFFFF" };
    let legend_surface = if is_dark { ("#132032", "#0d1726") } else { ("#ffffff", "#f8fafc") };
    let legend_stroke = if is_dark { ("#4ade80", "0.30", "#14b8a6", "0.16") } else { ("#94a3b8", "0.30", "#cbd5e1", "0.16") };
    let legend_val_header = if is_dark { "#67e8f9" } else { "#0f766e" };
    let legend_item_text = if is_dark { "#ffffff" } else { "#111827" };
    let legend_item_val = if is_dark { "#d1d5db" } else { "#475569" };
    let legend_line = if is_dark { "#475569" } else { "#cbd5e1" };
    let badge_anim_x = if is_dark { "-10px" } else { "10px" };

    let palette = if is_dark {
        vec![
            ("#ea2d69", "#bc003b"),
            ("#36c1ff", "#0993d1"),
            ("#6feca4", "#41bf76"),
            ("#eadf2d", "#bcb100"),
            ("#ad42f6", "#7f14c8"),
        ]
    } else {
        vec![
            ("#f17f7f", "#d86565"),
            ("#fa9146", "#e1772d"),
            ("#ac87d1", "#926db7"),
            ("#92ba19", "#78a100"),
            ("#5dc5a3", "#44ab89"),
        ]
    };

    let mut defs = format!(r##"
        <linearGradient id="donut_bg_{chart_id}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="{bg1}"/><stop offset="48%" stop-color="{bg2}"/><stop offset="100%" stop-color="{bg3}"/>
        </linearGradient>
        <radialGradient id="donut_halo_{chart_id}" cx="54%" cy="24%" r="68%">
            <stop offset="0%" stop-color="{h1}" stop-opacity="{ho1}"/><stop offset="48%" stop-color="{h2}" stop-opacity="{ho2}"/><stop offset="100%" stop-color="{h3}" stop-opacity="0"/>
        </radialGradient>
        <pattern id="donut_ticks_{chart_id}" width="26" height="26" patternUnits="userSpaceOnUse">
            <path d="M 26 0 L 0 0 0 26" fill="none" stroke="{tick_color}" stroke-width="0.8" opacity="{tick_op}"/>
        </pattern>
        <linearGradient id="legend_surface_{chart_id}" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="{ls1}" stop-opacity="0.92"/><stop offset="100%" stop-color="{ls2}" stop-opacity="0.92"/>
        </linearGradient>
        <linearGradient id="legend_stroke_{chart_id}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="{lst1}" stop-opacity="{lsto1}"/><stop offset="100%" stop-color="{lst2}" stop-opacity="{lsto2}"/>
        </linearGradient>
        <filter id="donut_lift_{chart_id}" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur in="SourceAlpha" stdDeviation="5" result="blur"/><feOffset in="blur" dx="0" dy="10" result="offset"/>
            <feComponentTransfer in="offset"><feFuncA type="linear" slope="0.20"/></feComponentTransfer>
            <feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge>
        </filter>
"##,
        chart_id=chart_id, bg1=bg_colors.0, bg2=bg_colors.1, bg3=bg_colors.2,
        h1=halo_colors.0, ho1=halo_colors.1, h2=halo_colors.2, ho2=halo_colors.3, h3=halo_colors.4,
        tick_color=tick_color, tick_op=tick_op,
        ls1=legend_surface.0, ls2=legend_surface.1,
        lst1=legend_stroke.0, lsto1=legend_stroke.1, lst2=legend_stroke.2, lsto2=legend_stroke.3
    );

    let mut segments_html = String::new();
    let mut legend_html = String::new();
    let mut external_labels = String::new();
    let mut angle = -PI / 2.0;
    let cx = 380.0;
    let cy = 310.0;
    let r = 128.0;

    for (i, (label, value)) in data.points.iter().enumerate() {
        let (c1, c2) = palette[i % palette.len()];
        let frac = value / total;
        let sweep = frac * 2.0 * PI;
        let end_angle = angle + sweep;

        defs.push_str(&format!(
            r##"        <linearGradient id="orbit_seg_{chart_id}_{i}" x1="80" y1="80" x2="280" y2="280" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stop-color="{c1}"/><stop offset="100%" stop-color="{c2}"/>
        </linearGradient>
"##,
            chart_id=chart_id, i=i, c1=c1, c2=c2
        ));

        let x1 = angle.cos() * r;
        let y1 = angle.sin() * r;
        let x2 = end_angle.cos() * r;
        let y2 = end_angle.sin() * r;
        let large_arc = if sweep > PI { 1 } else { 0 };

        let delay = (i as f64) * 90.0;
        segments_html.push_str(&format!(
            r##"        <g class="reveal" style="animation-delay: {delay}ms">
            <g class="slice-shell">
                <path d="M {x1:.2} {y1:.2} A {r:.2} {r:.2} 0 {large_arc} 1 {x2:.2} {y2:.2}" fill="none" stroke="url(#orbit_seg_{chart_id}_{i})" stroke-width="52.0" stroke-linecap="round">
                    <title>{label}: {value} ({pct:.1}%)</title>
                </path>
            </g>
        </g>
"##,
            delay=delay, x1=x1, y1=y1, r=r, large_arc=large_arc, x2=x2, y2=y2, chart_id=chart_id, i=i, label=escape(label), value=value, pct=frac*100.0
        ));

        // Legend logic (two columns)
        let row = (i / 2) as f64;
        let col = (i % 2) as f64;
        let lx = 32.0 + col * 300.0;
        let ly = 58.0 + row * 30.0;
        legend_html.push_str(&format!(
            r##"        <g class="legend-item" transform="translate({lx}, {ly})">
            <rect width="14" height="14" rx="4" fill="url(#orbit_seg_{chart_id}_{i})"/>
            <text x="24" y="11" fill="{item_text}" font-size="13" font-weight="850">{label}</text>
            <text x="260" y="11" text-anchor="end" fill="{item_val}" font-size="12" font-weight="800">{value} · {pct:.0}%</text>
        </g>
"##,
            lx=lx, ly=ly, chart_id=chart_id, i=i, item_text=legend_item_text, label=escape(label), item_val=legend_item_val, value=value, pct=frac*100.0
        ));

        // External labels logic
        let mid_angle = angle + sweep / 2.0;
        let ex1 = cx + (r + 15.0) * mid_angle.cos();
        let ey1 = cy + (r + 15.0) * mid_angle.sin();
        let ex2 = cx + (r + 45.0) * mid_angle.cos();
        let ey2 = cy + (r + 45.0) * mid_angle.sin();
        let is_right = mid_angle.cos() > 0.0;
        let ex3 = if is_right { ex2 + 25.0 } else { ex2 - 25.0 };

        let line_delay = 800.0 + (i as f64) * 100.0;
        let badge_delay = 900.0 + (i as f64) * 100.0;
        let badge_x = if is_right { ex3 + 4.0 } else { ex3 - 4.0 - 42.0 };
        let badge_y = ey2 - 10.0;
        let text_anchor = if is_right { "start" } else { "end" };
        let label_text_x = if is_right { badge_x + 50.0 } else { badge_x - 10.0 };

        external_labels.push_str(&format!(
            r##"        <path class="label-line" d="M {ex1:.1} {ey1:.1} L {ex2:.1} {ey2:.1} L {ex3:.1} {ey2:.1}" fill="none" stroke="{label_line}" stroke-width="1.2" stroke-opacity="0.4" style="animation-delay: {line_delay}ms;"/>
        <g class="label-badge" style="animation-delay: {badge_delay}ms;">
            <rect x="{badge_x:.1}" y="{badge_y:.1}" width="42.00" height="20" rx="10" fill="{badge_bg}" opacity="0.9"/>
            <text x="{badge_tx:.1}" y="{ey2:.1}" text-anchor="middle" dominant-baseline="middle" fill="{badge_text}" font-size="11" font-weight="900">{pct:.0}%</text>
            <text x="{label_tx:.1}" y="{ey2:.1}" text-anchor="{anchor}" dominant-baseline="middle" fill="{header_sub}" font-size="12" font-weight="700">{label}</text>
        </g>
"##,
            ex1=ex1, ey1=ey1, ex2=ex2, ey2=ey2, ex3=ex3, label_line=label_line, line_delay=line_delay, badge_delay=badge_delay,
            badge_x=badge_x, badge_y=badge_y, badge_bg=badge_bg, badge_tx=badge_x + 21.0, badge_text=badge_text, pct=frac*100.0,
            label_tx=label_text_x, anchor=text_anchor, header_sub=header_sub, label=escape(label)
        ));

        angle = end_angle;
    }

    Ok(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="600" viewBox="0 0 760 660" id="{chart_id}" role="img">
    <defs>
        <style>
            @import url('https://fonts.googleapis.com/css2?family=Lexend:wght@400;700;850;900&amp;display=swap');
            #{chart_id} {{ font-family: 'Lexend', ui-sans-serif, system-ui, sans-serif; }}
            #{chart_id} text {{ font-family: 'Lexend', ui-sans-serif, system-ui, sans-serif; }}
            #{chart_id} .reveal {{ opacity: 0; transform-box: fill-box; transform-origin: center; animation: orbitReveal_{chart_id} 720ms cubic-bezier(.18,.9,.24,1.12) forwards; }}
            #{chart_id} .label-line {{ stroke-dasharray: 100; stroke-dashoffset: 100; animation: labelLineReveal_{chart_id} 800ms ease forwards; opacity: 1; }}
            #{chart_id} .label-badge {{ opacity: 0; animation: labelBadgeReveal_{chart_id} 600ms cubic-bezier(.18,.9,.24,1.12) forwards; }}
            @keyframes orbitReveal_{chart_id} {{ from {{ opacity: 0; transform: scale(.92) rotate(-5deg); }} to {{ opacity: 1; transform: scale(1) rotate(0deg); }} }}
            #{chart_id} .slice-shell {{ transform-box: fill-box; transform-origin: center; transition: transform 260ms cubic-bezier(.2,.9,.2,1), filter 260ms ease; cursor: pointer; }}
            #{chart_id} .slice-shell:hover {{ transform: scale(1.035); filter: url(#donut_lift_{chart_id}); }}
            #{chart_id} .legend-item {{ transition: transform 200ms ease, opacity 200ms ease; cursor: pointer; }}
            #{chart_id} .legend-item:hover {{ transform: translateY(-1px); opacity: 0.94; }}
            @keyframes labelLineReveal_{chart_id} {{ from {{ stroke-dashoffset: 100; opacity: 0; }} to {{ stroke-dashoffset: 0; opacity: 1; }} }}
            @keyframes labelBadgeReveal_{chart_id} {{ from {{ opacity: 0; transform: translateX({badge_anim_x}); }} to {{ opacity: 1; transform: translateX(0); }} }}
        </style>
        {defs}
    </defs>
    <rect width="760" height="660" rx="28" fill="url(#donut_bg_{chart_id})"/>
    <rect width="760" height="660" rx="28" fill="url(#donut_ticks_{chart_id})" opacity="0.50"/>
    <rect width="760" height="660" rx="28" fill="url(#donut_halo_{chart_id})"/>
    <text x="56" y="50" fill="{header_prefix}" font-size="10" font-weight="900" letter-spacing="2.2">PIE CHART</text>
    <text x="56" y="74" fill="{header_title}" font-size="28" font-weight="900" letter-spacing="-0.6">{title_esc}</text>
    <rect x="56" y="88" width="64" height="4" rx="2" fill="#a37acc"/>
    <rect x="128" y="88" width="18" height="4" rx="2" fill="#84cc16"/>
    <text x="56" y="116" fill="{header_sub}" font-size="12" font-weight="700">{subtitle_esc}</text>
    
    <g transform="translate(604 48)">
        <rect width="104" height="46" rx="12" fill="{total_box_bg}" stroke="{total_box_stroke}" stroke-width="1"/>
        <text x="18" y="17" fill="{total_prefix}" font-size="9" font-weight="900" letter-spacing="1.8">TOTAL</text>
        <text x="18" y="35" fill="{total_val}" font-size="17" font-weight="900">{total}</text>
    </g>

    <g transform="translate(0 0)">
        <circle cx="{cx}" cy="{cy}" r="168" fill="{inner_fill}" opacity="0.42" filter="url(#donut_lift_{chart_id})"/>
        <circle cx="{cx}" cy="{cy}" r="154" fill="none" stroke="{tick_color}" stroke-width="1.1" stroke-dasharray="2 8"/>
        <circle cx="{cx}" cy="{cy}" r="106" fill="{inner_fill}" stroke="{tick_color}" stroke-width="1"/>
        <g transform="translate({cx} {cy})">
            {segments_html}
        </g>
        <circle cx="{cx}" cy="{cy}" r="74" fill="{inner_fill}" stroke="{tick_color}" stroke-width="1.4"/>
        <circle cx="{cx}" cy="{cy}" r="58" fill="none" stroke="{tick_color}" stroke-width="0.8" opacity="0.72"/>
        <text x="{cx}" y="294" text-anchor="middle" fill="{center_label}" font-size="11" font-weight="900" letter-spacing="1.4">TOTAL</text>
        <text x="{cx}" y="324" text-anchor="middle" fill="{center_text}" font-size="34" font-weight="900" letter-spacing="-1.2">{total}</text>
        <text x="{cx}" y="346" text-anchor="middle" fill="{center_label}" font-size="12" font-weight="700">{segments_count} segments</text>
    </g>

    <g class="external-labels" pointer-events="none">
        {external_labels}
    </g>

    <g transform="translate(80, 480)" filter="url(#donut_lift_{chart_id})">
        <rect x="0" y="0" width="600" height="164" rx="18" fill="url(#legend_surface_{chart_id})" stroke="url(#legend_stroke_{chart_id})" stroke-width="1.2"/>
        <text x="32" y="28" fill="{header_sub}" font-size="10" font-weight="900" letter-spacing="1.5">LEGEND</text>
        <text x="568" y="28" text-anchor="end" fill="{legend_val_header}" font-size="10" font-weight="900" letter-spacing="0.9">VALUES / SHARE</text>
        <line x1="32" y1="40" x2="568" y2="40" stroke="{legend_line}" stroke-opacity="0.70"/>
        {legend_html}
    </g>
</svg>"##,
        chart_id=chart_id, title_esc=escape(title), subtitle_esc=escape(subtitle),
        header_prefix=header_prefix, header_title=header_title, header_sub=header_sub,
        total_box_bg=total_box_bg, total_box_stroke=total_box_stroke, total_prefix=total_prefix, total_val=total_val,
        total=total, cx=cx, cy=cy, inner_fill=inner_fill, center_text=center_text,
        tick_color=tick_color, segments_html=segments_html, center_label=center_label,
        segments_count=data.points.len(), external_labels=external_labels,
        legend_val_header=legend_val_header, legend_line=legend_line, legend_html=legend_html,
        badge_anim_x=badge_anim_x, defs=defs
    ))
}

fn render_v1(data: &crate::common::kv::KvBody, total: f64, _controls: &HashMap<String, String>) -> Result<String, String> {
    let chart_id = format!("id_{}", Uuid::new_v4());
    let cfg = &data.config;
    let title = cfg.get("title").map(String::as_str).unwrap_or("Pie Chart");
    let desc = format!("Pie Chart with {} segments.", data.points.len());

    let is_dark = _controls.get("useDark").map(|s| s.as_str()) == Some("true")
        || cfg.get("theme").map(|s| s.as_str()) == Some("dark");

    let colors = [
        ("#71A5F8", "#3B82F6", "#306AC9"),
        ("#AB89F8", "#8B5CF6", "#714BC9"),
        ("#57BC7C", "#16A34A", "#12853C"),
        ("#E39D4B", "#D97706", "#B16104"),
        ("#E56262", "#DC2626", "#B41F1F"),
    ];

    let (bg_s1, bg_s2, bg_s3) = if is_dark { ("#0F172A", "#121b32", "#041317") } else { ("#cccccc", "#FFFFFF", "#cccccc") };
    let glow_a_op = if is_dark { "0.26" } else { "0.16" };
    let glow_b_op = if is_dark { "0.17" } else { "0.12" };
    let vignette_op = if is_dark { "0.38" } else { "0.10" };
    let sonar_fill = if is_dark { "#F9FAFB" } else { "#111827" };
    let sonar_op = if is_dark { "0.10" } else { "0.12" };
    let grid_stroke = if is_dark { "#F9FAFB" } else { "#111827" };
    let grid_op = if is_dark { "0.035" } else { "0.055" };
    let header_prefix = if is_dark { "#9CA3AF" } else { "#6B7280" };
    let header_title = if is_dark { "#F9FAFB" } else { "#111827" };
    let card_bg = if is_dark { "#121b32" } else { "rgba(255, 255, 255, 0.8)" };
    let card_label = if is_dark { "#9CA3AF" } else { "#6B7280" };
    let card_value = if is_dark { "#F9FAFB" } else { "#111827" };
    let pulse_mid_stroke = if is_dark { "#F9FAFB" } else { "#111827" };
    let center_fill = if is_dark { "#0F172A" } else { "#FFFFFF" };
    let label_line_color = if is_dark { "#9CA3AF" } else { "#6B7280" };
    let badge_bg = if is_dark { "#06191E" } else { "#172033" };
    let badge_pct = if is_dark { "#F7FBFF" } else { "#FFFFFF" };
    let badge_label = if is_dark { "#F9FAFB" } else { "#111827" };
    let badge_anim_x = if is_dark { "-10px" } else { "10px" };

    let cx = 300.0;
    let cy = 310.0;
    let r = 154.0;

    let mut gradient_defs = String::new();
    let mut slices_html = String::new();
    let mut labels_html = String::new();
    let mut angle = -PI / 2.0;

    for (i, (label, value)) in data.points.iter().enumerate() {
        let (c_light, c_main, c_dark) = colors[i % colors.len()];
        let frac = value / total;
        let sweep = frac * 2.0 * PI;
        let end_angle = angle + sweep;

        // Gradient for slice
        gradient_defs.push_str(&format!(
            r##"<linearGradient id="slice_{chart_id}_{i}" x1="0%" y1="0%" x2="100%" y2="100%">
    <stop offset="0%" stop-color="{c_light}"/>
    <stop offset="52%" stop-color="{c_main}"/>
    <stop offset="100%" stop-color="{c_dark}"/>
</linearGradient>"##,
            chart_id = chart_id, i = i, c_light = c_light, c_main = c_main, c_dark = c_dark
        ));

        // Path coordinates
        let x1 = cx + r * angle.cos();
        let y1 = cy + r * angle.sin();
        let x2 = cx + r * end_angle.cos();
        let y2 = cy + r * end_angle.sin();
        let large_arc = if sweep > PI { 1 } else { 0 };

        let path_d = format!(
            "M {cx} {cy} L {x1:.2} {y1:.2} A {r} {r} 0 {large_arc} 1 {x2:.2} {y2:.2} Z",
            cx = cx, cy = cy, x1 = x1, y1 = y1, r = r, large_arc = large_arc, x2 = x2, y2 = y2
        );

        let delay = 0.12 + (i as f64) * 0.1;
        slices_html.push_str(&format!(
            r##"<g class="pie-segment" style="animation-delay: {delay:.2}s;">
    <g class="slice-motion">
        <path d="{path_d}" fill="url(#slice_{chart_id}_{i})" stroke="#FFFFFF" stroke-opacity="0.18" stroke-width="1.2">
            <title>{label}: {value}</title>
        </path>
        <path d="{path_d}" fill="url(#sliceGlass_{chart_id})" opacity="0.54" pointer-events="none"/>
    </g>
</g>"##,
            delay = delay, path_d = path_d, chart_id = chart_id, i = i, label = escape(label), value = value
        ));

        // Label logic
        let mid_angle = angle + sweep / 2.0;
        let lx1 = cx + r * mid_angle.cos();
        let ly1 = cy + r * mid_angle.sin();
        let lx2 = cx + (r + 25.0) * mid_angle.cos();
        let ly2 = cy + (r + 25.0) * mid_angle.sin();
        let is_right = lx2 > cx;
        let lx3 = if is_right { lx2 + 20.0 } else { lx2 - 20.0 };
        
        let label_delay = 0.8 + (i as f64) * 0.1;
        let badge_delay = 0.9 + (i as f64) * 0.1;

        let badge_x = if is_right { lx3 + 4.0 } else { lx3 - 4.0 - 45.0 };
        let text_anchor = if is_right { "start" } else { "end" };
        let label_text_x = if is_right { badge_x + 53.0 } else { badge_x - 8.0 };

        labels_html.push_str(&format!(
            r##"<path class="label-line" d="M {lx1:.1} {ly1:.1} L {lx2:.1} {ly2:.1} L {lx3:.1} {ly2:.1}" fill="none" stroke="{label_line}" stroke-width="1.2" stroke-opacity="0.4" style="animation-delay: {label_delay:.2}s;"/>
<g class="label-badge" style="animation-delay: {badge_delay:.2}s;">
    <rect x="{badge_x:.1}" y="{badge_y:.1}" width="45.0" height="20" rx="10" fill="{badge_bg}" opacity="0.9"/>
    <text x="{badge_text_x:.1}" y="{badge_y_mid:.1}" text-anchor="middle" dominant-baseline="middle" fill="{badge_pct}" style="fill: {badge_pct} !important;" font-size="11" font-weight="900">{pct:.1}%</text>
    <text x="{label_text_x:.1}" y="{badge_y_mid:.1}" text-anchor="{text_anchor}" dominant-baseline="middle" fill="{badge_label}" style="fill: {badge_label} !important;" font-size="12" font-weight="700">{label}</text>
</g>"##,
            lx1 = lx1, ly1 = ly1, lx2 = lx2, ly2 = ly2, lx3 = lx3, label_line = label_line_color,
            label_delay = label_delay, badge_delay = badge_delay, badge_x = badge_x, badge_y = ly2 - 10.0,
            badge_bg = badge_bg, badge_text_x = badge_x + 22.5, badge_pct = badge_pct,
            badge_y_mid = ly2, pct = frac * 100.0, label_text_x = label_text_x, text_anchor = text_anchor,
            badge_label = badge_label, label = escape(label)
        ));

        angle = end_angle;
    }

    Ok(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="600" viewBox="0 0 600 600" id="{chart_id}" role="img" aria-labelledby="title_{chart_id} desc_{chart_id}">
    <title id="title_{chart_id}">{title_esc}</title>
    <desc id="desc_{chart_id}">{desc_esc}</desc>
    <defs>
        <style>
            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;700;900&amp;display=swap');
            #{chart_id} text {{ font-family: 'Inter', system-ui, sans-serif; }}
            @keyframes titleReveal_{chart_id} {{ from {{ opacity: 0; transform: translateY(-8px); }} to {{ opacity: 1; transform: translateY(0); }} }}
            @keyframes pieReveal_{chart_id} {{ from {{ opacity: 0; transform: scale(0.84) rotate(-4deg); }} to {{ opacity: 1; transform: scale(1) rotate(0deg); }} }}
            @keyframes legendReveal_{chart_id} {{ from {{ opacity: 0; transform: translateY(14px); }} to {{ opacity: 1; transform: translateY(0); }} }}
            @keyframes pulseRing_{chart_id} {{ 0%, 100% {{ opacity: 0.16; stroke-width: 1; }} 50% {{ opacity: 0.36; stroke-width: 1.6; }} }}
            @keyframes labelLineReveal_{chart_id} {{ from {{ stroke-dashoffset: 100; opacity: 0; }} to {{ stroke-dashoffset: 0; opacity: 1; }} }}
            @keyframes labelBadgeReveal_{chart_id} {{ from {{ opacity: 0; transform: translateX({badge_anim_x}); }} to {{ opacity: 1; transform: translateX(0); }} }}
            #{chart_id} .header-motion {{ animation: titleReveal_{chart_id} 520ms cubic-bezier(0.22, 1, 0.36, 1) both; }}
            #{chart_id} .pie-segment {{ opacity: 0; transform-box: fill-box; transform-origin: center; animation: pieReveal_{chart_id} 680ms cubic-bezier(0.22, 1, 0.36, 1) both; }}
            #{chart_id} .slice-motion {{ transform-box: fill-box; transform-origin: center; transition: transform 280ms cubic-bezier(0.22, 1, 0.36, 1), filter 280ms ease; cursor: pointer; }}
            #{chart_id} .slice-motion:hover {{ transform: scale(1.045); filter: url(#sliceGlow_{chart_id}); }}
            #{chart_id} .legend-motion {{ animation: legendReveal_{chart_id} 620ms cubic-bezier(0.22, 1, 0.36, 1) 780ms both; opacity: 1; }}
            #{chart_id} .pulse-ring {{ animation: pulseRing_{chart_id} 3.8s ease-in-out infinite; }}
            #{chart_id} .label-line {{ stroke-dasharray: 100; stroke-dashoffset: 100; animation: labelLineReveal_{chart_id} 800ms ease forwards; }}
            #{chart_id} .label-badge {{ opacity: 0; animation: labelBadgeReveal_{chart_id} 600ms cubic-bezier(0.22, 1, 0.36, 1) forwards; }}
        </style>
        <linearGradient id="bgSurface_{chart_id}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="{bg_s1}"/><stop offset="46%" stop-color="{bg_s2}"/><stop offset="100%" stop-color="{bg_s3}"/>
        </linearGradient>
        <radialGradient id="bgGlowA_{chart_id}" cx="18%" cy="10%" r="70%">
            <stop offset="0%" stop-color="#3B82F6" stop-opacity="{glow_a_op}"/><stop offset="100%" stop-color="#3B82F6" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="bgGlowB_{chart_id}" cx="84%" cy="22%" r="58%">
            <stop offset="0%" stop-color="#DC2626" stop-opacity="{glow_b_op}"/><stop offset="100%" stop-color="#DC2626" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="vignette_{chart_id}" cx="50%" cy="48%" r="78%">
            <stop offset="0%" stop-color="#000000" stop-opacity="0"/><stop offset="100%" stop-color="#000000" stop-opacity="{vignette_op}"/>
        </radialGradient>
        <pattern id="sonarDots_{chart_id}" x="0" y="0" width="24" height="24" patternUnits="userSpaceOnUse">
            <circle cx="2" cy="2" r="1" fill="{sonar_fill}" opacity="{sonar_op}"/>
        </pattern>
        <pattern id="fineGrid_{chart_id}" x="0" y="0" width="48" height="48" patternUnits="userSpaceOnUse">
            <path d="M48 0 H0 V48" fill="none" stroke="{grid_stroke}" stroke-opacity="{grid_op}" stroke-width="1"/>
        </pattern>
        <linearGradient id="sliceGlass_{chart_id}" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.45"/>
            <stop offset="50%" stop-color="#FFFFFF" stop-opacity="0.1"/>
            <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0.05"/>
        </linearGradient>
        <filter id="sliceShadow_{chart_id}" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur in="SourceAlpha" stdDeviation="3"/><feOffset dx="0" dy="2" result="offsetblur"/>
            <feComponentTransfer><feFuncA type="linear" slope="0.3"/></feComponentTransfer>
            <feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge>
        </filter>
        <filter id="sliceGlow_{chart_id}">
            <feGaussianBlur stdDeviation="4" result="blur"/><feComposite in="SourceGraphic" in2="blur" operator="over"/>
        </filter>
        {gradient_defs}
    </defs>
    <rect width="600" height="600" rx="18" fill="url(#bgSurface_{chart_id})"/>
    <rect width="600" height="600" rx="18" fill="url(#bgGlowA_{chart_id})"/>
    <rect width="600" height="600" rx="18" fill="url(#bgGlowB_{chart_id})"/>
    <rect width="600" height="600" rx="18" fill="url(#fineGrid_{chart_id})"/>
    <rect width="600" height="600" rx="18" fill="url(#sonarDots_{chart_id})"/>
    <rect width="600" height="600" rx="18" fill="url(#vignette_{chart_id})"/>
    
    <g transform="translate(40, 44)">
        <g class="header-motion">
            <text x="0" y="0" fill="{header_prefix}" style="fill: {header_prefix} !important;" font-size="10" font-weight="800" letter-spacing="2.4">PIE CHART</text>
            <text x="0" y="34" fill="{header_title}" style="fill: {header_title} !important;" font-size="28" font-weight="900">{title_esc}</text>
            <rect x="0" y="47" width="74" height="5" rx="2.5" fill="#3B82F6"/>
            <rect x="82" y="47" width="22" height="5" rx="2.5" fill="#DC2626"/>
        </g>
    </g>

    <g transform="translate(448, 36)">
        <g class="legend-motion">
            <rect width="112" height="46" rx="14" fill="{card_bg}" stroke="#3B82F6" stroke-opacity="0.28"/>
            <text x="16" y="18" fill="{card_label}" style="fill: {card_label} !important;" font-size="9" font-weight="900" letter-spacing="1.5">TOTAL</text>
            <text x="16" y="36" fill="{card_value}" style="fill: {card_value} !important;" font-size="18" font-weight="900">{total}</text>
        </g>
    </g>

    <g opacity="0.7">
        <circle class="pulse-ring" cx="{cx}" cy="{cy}" r="180" fill="none" stroke="#3B82F6"/>
        <circle cx="{cx}" cy="{cy}" r="128" fill="none" stroke="{pulse_mid_stroke}" stroke-opacity="0.045"/>
        <circle cx="{cx}" cy="{cy}" r="204" fill="none" stroke="#DC2626" stroke-opacity="0.035"/>
    </g>

    <g filter="url(#sliceShadow_{chart_id})">
        {slices_html}
    </g>

    <circle cx="{cx}" cy="{cy}" r="10" fill="{center_fill}" stroke="#3B82F6" stroke-opacity="0.55" stroke-width="1.2"/>
    <circle cx="{cx}" cy="{cy}" r="4" fill="#DC2626" opacity="0.95"/>

    <g class="external-labels" pointer-events="none">
        {labels_html}
    </g>
</svg>"##,
        chart_id = chart_id, title_esc = escape(title), desc_esc = escape(&desc), total = total,
        cx = cx, cy = cy, slices_html = slices_html, labels_html = labels_html,
        gradient_defs = gradient_defs, bg_s1 = bg_s1, bg_s2 = bg_s2, bg_s3 = bg_s3,
        glow_a_op = glow_a_op, glow_b_op = glow_b_op, vignette_op = vignette_op,
        sonar_fill = sonar_fill, sonar_op = sonar_op, grid_stroke = grid_stroke,
        grid_op = grid_op, header_prefix = header_prefix, header_title = header_title,
        card_bg = card_bg, card_label = card_label, card_value = card_value,
        pulse_mid_stroke = pulse_mid_stroke, center_fill = center_fill,
        badge_anim_x = badge_anim_x
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_v1() {
        let body = "---- title=Test Pie visualVersion=1 --- A | 30.0 B | 70.0 ----";
        let result = render(body, &HashMap::new());
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("id=\"id_"));
        assert!(svg.contains("Test Pie"));
        assert!(svg.contains("30.0%"));
        assert!(svg.contains("70.0%"));
        assert!(svg.contains("PIE CHART"));
        assert!(svg.contains("stop-color=\"#cccccc\"")); // bgSurface start (light)
        assert!(svg.contains("translateX(10px)"));
    }

    #[test]
    fn test_render_v1_dark() {
        let mut controls = HashMap::new();
        controls.insert("useDark".to_string(), "true".to_string());
        let body = "---- title=Dark Pie visualVersion=1 --- A | 30.0 ----";
        let result = render(body, &controls);
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("stop-color=\"#0F172A\"")); // bgSurface start
        assert!(svg.contains("stop-opacity=\"0.26\"")); // bgGlowA opacity
        assert!(svg.contains("fill=\"#F9FAFB\"")); // sonarDots fill
        assert!(svg.contains("legend-motion"));
        assert!(svg.contains("translateX(-10px)"));
    }

    #[test]
    fn test_render_donut() {
        let body = "---- title=Donut Chart shape=donut --- A | 30.0 B | 70.0 ----";
        let result = render(body, &HashMap::new());
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("donut_bg_"));
        assert!(svg.contains("Donut Chart"));
        assert!(svg.contains("30.0%"));
        assert!(svg.contains("70.0%"));
        assert!(svg.contains("PIE CHART"));
    }

    #[test]
    fn test_render_donut_dark() {
        let mut controls = HashMap::new();
        controls.insert("useDark".to_string(), "true".to_string());
        let body = "---- title=Dark Donut shape=donut --- A | 30.0 ----";
        let result = render(body, &controls);
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("stop-color=\"#08111d\"")); // donut_bg_ dark start
        assert!(svg.contains("fill=\"#0b1220\"")); // inner circle fill
        assert!(svg.contains("translateX(-10px)"));
    }
}