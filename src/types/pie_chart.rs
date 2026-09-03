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
}