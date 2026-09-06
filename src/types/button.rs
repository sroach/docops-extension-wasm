use crate::common::kv::parse_kv_header;
use crate::common::svg::{brighten_color, darken_color, determine_text_color, escape, get_rgb, wrap_text};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Button {
    label: String,
    link: String,
    button_type: String,
    description: String,
    color: String,
    active: bool,
    enabled: bool,
    #[allow(dead_code)]
    date: String,
}

struct ButtonConfig {
    shape: String,
    columns: usize,
    scale: f64,
    use_dark: bool,
    new_win: bool,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let trimmed = body.trim();
    if !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("button body must be wrapped in '---- ... ----'".into());
    }
    let inner = trimmed[4..trimmed.len() - 4].trim();

    let parts: Vec<&str> = inner.splitn(2, "---").collect();
    let (header_str, data_str) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        ("", parts[0])
    };

    let mut config_map = parse_kv_header(header_str);
    for (k, v) in controls {
        config_map.insert(k.clone(), v.clone());
    }

    let config = ButtonConfig {
        shape: config_map.get("shape").cloned().unwrap_or_else(|| "regular".to_string()),
        columns: config_map.get("columns").and_then(|s| s.parse().ok()).unwrap_or(3),
        scale: config_map.get("scale").and_then(|s| s.parse().ok()).unwrap_or(1.0),
        use_dark: config_map.get("useDark").map(|s| s == "true").unwrap_or(false) || config_map.get("theme").map(|s| s == "dark").unwrap_or(false),
        new_win: config_map.get("newWin").map(|s| s == "true").unwrap_or(true),
    };

    let mut buttons = Vec::new();
    for line in data_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if fields.len() < 2 {
            continue;
        }

        buttons.push(Button {
            label: fields.get(0).unwrap_or(&"").to_string(),
            link: fields.get(1).unwrap_or(&"").to_string(),
            button_type: fields.get(2).unwrap_or(&"").to_string(),
            description: fields.get(3).unwrap_or(&"").to_string(),
            color: fields.get(4).unwrap_or(&"").to_string(),
            active: fields.get(5).map(|&s| s == "true").unwrap_or(false),
            enabled: fields.get(6).map(|&s| s != "false").unwrap_or(true),
            date: fields.get(7).unwrap_or(&"").to_string(),
        });
    }

    if buttons.is_empty() {
        return Err("no buttons found in block".into());
    }

    let id = Uuid::new_v4().simple().to_string()[..8].to_string();

    match config.shape.to_lowercase().as_str() {
        "large" => Ok(render_large(&buttons, &config, &id)),
        "hex" => Ok(render_hex(&buttons, &config, &id)),
        "pill" => Ok(render_pill(&buttons, &config, &id)),
        "circle" => Ok(render_circle(&buttons, &config, &id)),
        "round" => Ok(render_round(&buttons, &config, &id)),
        "rectangle" => Ok(render_rectangle(&buttons, &config, &id)),
        _ => Ok(render_regular(&buttons, &config, &id)),
    }
}

fn get_styles(id: &str, shape: &str) -> String {
    let mut extra = String::new();
    match shape {
        "large" => {
            extra.push_str(r##"
            .card { cursor: pointer; transition: transform 0.28s ease, opacity 0.28s ease; }
            .card:hover { transform: translateY(-8px); }
            .eyebrow { font-size: 11px; font-weight: 800; letter-spacing: 2.4px; text-transform: uppercase; }
            .title { font-size: 28px; font-weight: 850; letter-spacing: -1.1px; fill: var(--text); text-transform: uppercase; }
            .description { font-size: 14px; font-weight: 500; fill: var(--text); fill-opacity: 0.7; }
            .meta { font-size: 11px; font-weight: 750; letter-spacing: 0.8px; fill: var(--text); fill-opacity: 0.6; }
            .badgeText { font-size: 10px; font-weight: 850; letter-spacing: 1.5px; fill: #ffffff; }
            .headerGlyph { font-size: 54px; font-weight: 900; fill: #ffffff; opacity: 0.95; }
            .headerSubtle { font-size: 13px; font-weight: 750; letter-spacing: 2px; fill: #ffffff; opacity: 0.78; }
            "##);
        },
        "hex" => {
            extra.push_str(r##"
            .hex-button { cursor: pointer; transition: transform 360ms cubic-bezier(0.22, 1, 0.36, 1), opacity 260ms ease; transform-box: fill-box; transform-origin: center; }
            .hex-button:hover { transform: scale(1.045); }
            .hex-label { font-size: 18px; font-weight: 850; letter-spacing: -0.25px; fill: #ffffff; text-transform: uppercase; }
            .hex-type { font-size: 12px; font-weight: 850; letter-spacing: 2.8px; fill: #ffffff; opacity: 0.68; text-transform: uppercase; }
            .hex-caption { font-size: 10px; font-weight: 750; letter-spacing: 1.2px; fill: #ffffff; opacity: 0.56; text-transform: uppercase; }
            .hex-icon { font-size: 34px; font-weight: 900; fill: #ffffff; opacity: 0.92; }
            .hex-ring { transition: stroke-opacity 0.3s ease; }
            .hex-button:hover .hex-ring { stroke-opacity: 0.9; }
            .outer-halo { transition: opacity 260ms ease; }
            .hex-button:hover .outer-halo { opacity: 0.28; }
            "##);
        },
        "circle" => {
            extra.push_str(&format!(r##"
            [id='btn_{id}'] .orb-body, [id='btn_{id}'] .orb-glass, [id='btn_{id}'] .tech-ring {{
                transition: all 0.4s cubic-bezier(0.22, 1, 0.36, 1);
            }}
            [id='btn_{id}'] .button-hover:hover .orb-body, [id='btn_{id}'] .button-hover:hover .orb-glass {{
                cx: 56;
                cy: 56;
            }}
            [id='btn_{id}'] .tech-ring {{
                stroke-dasharray: 365;
                stroke-dashoffset: 365;
            }}
            [id='btn_{id}'] .button-hover:hover .tech-ring {{
                stroke-dashoffset: 0;
                opacity: 0.8;
            }}
            [id='btn_{id}'] .moving-text {{
                transition: transform 0.4s cubic-bezier(0.22, 1, 0.36, 1);
            }}
            [id='btn_{id}'] .button-hover:hover .moving-text {{
                transform: translate(-4px, -4px);
            }}
            "##, id=id));
        },
        "round" => {
            extra.push_str(&format!(r##"
            [id='btn_{id}'] .moving-group {{
                transition: transform 0.4s cubic-bezier(0.22, 1, 0.36, 1);
            }}
            [id='btn_{id}'] .glow-ring {{
                stroke-dasharray: 400;
                stroke-dashoffset: 400;
                transition: stroke-dashoffset 0.6s ease, opacity 0.4s ease;
            }}
            [id='btn_{id}'] .button-hover:hover .glow-ring {{
                stroke-dashoffset: 0;
                opacity: 0.8;
            }}
            [id='btn_{id}'] .button-hover:hover .moving-group {{
                transform: translate(-4px, -4px);
            }}
            "##, id=id));
        },
        "rectangle" => {
            extra.push_str(r##"
            .button-hover { transition: all 0.3s ease; }
            .button-hover:hover { transform: translateY(-3px); }
            .link-chip { transition: fill-opacity 0.2s ease; }
            .link-chip:hover { fill-opacity: 0.2; }
            "##);
        },
        _ => {
            extra.push_str(r##"
            .button-hover { transition: transform 0.2s ease; }
            .button-hover:hover { transform: translateY(-2px); }
            "##);
        }
    }
    format!(r##"
        <style>
            #btn_{id} {{
                font-family: 'Inter', ui-sans-serif, system-ui, -apple-system, sans-serif;
                --bg: #f8fafc;
                --surface: #ffffff;
                --text: #0f172a;
                --accent: #2563eb;
                --active-opacity: 0.12;
                --shadow-opacity: 0.1;
                --glass-opacity: 0.92;
                --glass-stop: 0.72;
                --border-color: #ffffff;
            }}

            @media (prefers-color-scheme: dark) {{
                #btn_{id} {{
                    --bg: #0f172a;
                    --surface: #1e293b;
                    --text: #f8fafc;
                    --accent: #3b82f6;
                    --active-opacity: 0.15;
                    --shadow-opacity: 0.3;
                    --glass-opacity: 0.1;
                    --glass-stop: 0.05;
                    --border-color: #334155;
                }}
            }}

            #btn_{id}.dark-mode {{
                --bg: #0f172a;
                --surface: #1e293b;
                --text: #f8fafc;
                --accent: #3b82f6;
                --active-opacity: 0.15;
                --shadow-opacity: 0.3;
                --glass-opacity: 0.1;
                --glass-stop: 0.05;
                --border-color: #334155;
            }}
            .button-hover {{ cursor: pointer; }}
            .button-hover:focus {{ outline: none; }}
            {extra}
        </style>
    "##, id=id, extra=extra)
}

fn compute_title_font_size(label: &str) -> i32 {
    let len = label.trim().len();
    if len > 35 { 20 }
    else if len > 25 { 24 }
    else if len > 15 { 28 }
    else { 31 }
}

fn render_regular(buttons: &[Button], config: &ButtonConfig, id: &str) -> String {
    let button_width = 295.0;
    let button_height = 80.0;
    let button_padding = 10.0;
    let start_x = 20.0;
    let start_y = 20.0;

    let rows: Vec<&[Button]> = buttons.chunks(config.columns).collect();
    let max_in_row = if rows.is_empty() { 0 } else { rows.iter().map(|r| r.len()).max().unwrap_or(0) };
    
    let width = (start_x + max_in_row as f64 * button_width + (max_in_row as f64) * button_padding + 10.0) * config.scale;
    let height = (start_y + rows.len() as f64 * button_height + (rows.len() as f64) * button_padding + 10.0) * config.scale;

    let svg_width = format!("{:.1}", width / 1.77);
    let svg_height = format!("{:.1}", height / 1.77);

    let mut elements = String::new();
    let win = if config.new_win { "_blank" } else { "_top" };

    for (r_idx, row) in rows.iter().enumerate() {
        let y = start_y + r_idx as f64 * (button_height + button_padding);
        for (c_idx, btn) in row.iter().enumerate() {
            let x = start_x + c_idx as f64 * (button_width + button_padding);
            let accent = if btn.color.is_empty() { "var(--accent)".to_string() } else { escape(&btn.color) };
            let opacity = if btn.enabled { "1.0" } else { "0.55" };
            let interaction = if btn.enabled {
                format!(r##"onclick="window.open('{}', '{}')" style="cursor: pointer;" "##, btn.link, win)
            } else {
                r##"style="cursor: not-allowed;" "##.to_string()
            };

            let active_state = if btn.active {
                let r = 10.0;
                format!(r##"
                    <rect x="2" y="2" width="{}" height="{}" rx="{}" fill="{}" opacity="var(--active-opacity)"/>
                    <rect x="4" y="{}" width="{}" height="2" fill="{}"/>
                "##, button_width - 4.0, button_height - 4.0, r, accent, button_height - 2.0, button_width - 8.0, accent)
            } else {
                "".to_string()
            };

            elements.push_str(&format!(r##"
            <g transform="translate({x}, {y})">
                <g class="button-hover" role="button" tabindex="0" {interaction} opacity="{opacity}">
                    <title>{label}</title>
                    <rect x="0" y="0" width="{bw}" height="{bh}" rx="12" fill="var(--surface)" stroke="{accent}" stroke-opacity="0.2" stroke-width="1" filter="url(#cardShadow_{id})"/>
                    <path d="M0 12 A12 12 0 0 1 4 8.5 V71.5 A12 12 0 0 1 0 68 Z" fill="{accent}" fill-opacity="0.8"/>
                    {active_state}
                    <text x="{cx}" y="{cy}" text-anchor="middle" fill="var(--text)" style="font-size: 14px; font-weight: 600;">{label}</text>
                </g>
            </g>"##,
                x=x, y=y, bw=button_width, bh=button_height, id=id, label=escape(&btn.label),
                accent=accent,
                active_state=active_state,
                cx=button_width/2.0, cy=button_height/2.0 + 5.0,
                interaction=interaction,
                opacity=opacity
            ));
        }
    }

    let extra_class = if config.use_dark { " dark-mode" } else { "" };

    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_width}" height="{svg_height}" viewBox="0 0 {width} {height}" xmlns:xlink="http://www.w3.org/1999/xlink" id="btn_{id}" class="button-container{extra_class}">
    <defs>
        <filter id="cardShadow_{id}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="4" stdDeviation="6" flood-opacity="var(--shadow-opacity)"/>
        </filter>
        {styles}
    </defs>
    <rect width="100%" height="100%" fill="var(--bg)" rx="12"/>
    <g transform="scale({scale})">
        {elements}
    </g>
</svg>"##, svg_width=svg_width, svg_height=svg_height, width=width, height=height, id=id, elements=elements, scale=config.scale, styles=get_styles(id, "regular"), extra_class=extra_class)
}

fn render_large(buttons: &[Button], config: &ButtonConfig, id: &str) -> String {
    let button_width = 320.0;
    let button_height = 440.0;
    let button_spacing = 20.0;
    let row_spacing = 20.0;
    let start_x = 30.0;
    let start_y = 30.0;

    let rows: Vec<&[Button]> = buttons.chunks(config.columns).collect();
    let max_in_row = if rows.is_empty() { 0 } else { rows.iter().map(|r| r.len()).max().unwrap_or(0) };

    let width = (start_x + max_in_row as f64 * (button_width + button_spacing) + 20.0) * config.scale;
    let height = (start_y + rows.len() as f64 * (button_height + row_spacing) + 20.0) * config.scale;

    let svg_width = format!("{:.1}", width / 1.77);
    let svg_height = format!("{:.1}", height / 1.77);

    let mut elements = String::new();
    let mut gradients = String::new();
    let win = if config.new_win { "_blank" } else { "_top" };

    for (r_idx, row) in rows.iter().enumerate() {
        let y = start_y + r_idx as f64 * (button_height + row_spacing);
        for (c_idx, btn) in row.iter().enumerate() {
            let x = start_x + c_idx as f64 * (button_width + button_spacing);
            let accent = if btn.color.is_empty() { "var(--accent)".to_string() } else { escape(&btn.color) };
            let gradient_id = format!("btn_{}_{}_grad", id, r_idx * config.columns + c_idx);
            
            let darker = darken_color(if btn.color.is_empty() { "#2563eb" } else { &btn.color }, 0.4);
            gradients.push_str(&format!(r##"
        <linearGradient id="{grad_id}" x1="0%" y1="0%" x2="1" y2="1">
            <stop offset="0%" stop-color="{accent}"/>
            <stop offset="50%" stop-color="{accent}" stop-opacity="0.8"/>
            <stop offset="100%" stop-color="{darker}"/>
        </linearGradient>"##, grad_id=gradient_id, accent=accent, darker=darker));

            let type_text = if btn.button_type.is_empty() { "COMPONENT" } else { &btn.button_type.to_uppercase() };
            let badge_width = (type_text.len() * 8 + 24).max(82);
            
            let title_lines = wrap_text(&btn.label, 16);
            let mut title_svg = String::new();
            for (idx, line) in title_lines.iter().take(2).enumerate() {
                let dy = if idx == 0 { "0" } else { "1.1em" };
                title_svg.push_str(&format!(r##"<tspan x="0" dy="{}">{}</tspan>"##, dy, escape(line)));
            }
            let title_font_size = compute_title_font_size(&btn.label);

            let desc_lines = wrap_text(&btn.description, 35);
            let mut desc_svg = String::new();
            if !desc_lines.is_empty() {
                desc_svg.push_str(r##"<text x="0" y="82" class="description">"##);
                for (idx, line) in desc_lines.iter().take(3).enumerate() {
                    let dy = if idx == 0 { "0" } else { "20" };
                    desc_svg.push_str(&format!(r##"<tspan x="0" dy="{}">{}</tspan>"##, dy, escape(line)));
                }
                desc_svg.push_str("</text>");
            }

            let glyph = if btn.label.len() >= 2 { &btn.label[0..2] } else if !btn.label.is_empty() { &btn.label[0..1] } else { "BT" };

            elements.push_str(&format!(r##"
            <g transform="translate({x}, {y})">
                <g class="card" role="button" tabindex="0" onclick="window.open('{link}', '{win}')">
                    <title>{label} — {type_text}</title>
                    <rect x="0" y="0" width="{bw}" height="{bh}" rx="28" fill="url(#glassCard_{id})" filter="url(#premiumShadow_{id})"/>
                    <rect x="0.75" y="0.75" width="{bw_inner}" height="{bh_inner}" rx="27.25" fill="none" stroke="var(--border-color)" stroke-opacity="0.75"/>
                    
                    <g clip-path="url(#cardClip_{id})">
                        <rect x="0" y="0" width="{bw}" height="188" fill="url(#{grad_id})"/>
                        <circle cx="258" cy="22" r="95" fill="#ffffff" opacity="0.14" filter="url(#softGlow_{id})"/>
                        <circle cx="42" cy="178" r="92" fill="#ffffff" opacity="0.12"/>
                        <path d="M-10 156 C66 112, 112 174, 180 128 S278 64, 350 118 L350 200 L-10 200 Z" fill="#ffffff" opacity="0.12"/>
                        <path d="M0 0 L{bw} 0 L{bw} 86 C244 124, 184 52, 112 94 C66 120, 28 112, 0 96 Z" fill="url(#shine_{id})"/>
                    </g>
                    
                    <rect x="24" y="24" width="{badge_width}" height="28" rx="14" fill="#ffffff" fill-opacity="0.2"/>
                    <text x="{badge_center}" y="43" text-anchor="middle" class="badgeText">{type_text}</text>
                    
                    <text x="28" y="119" class="headerGlyph">{glyph_escaped}</text>
                    <text x="29" y="147" class="headerSubtle">DOC OPS</text>
                    
                    <g transform="translate(28, 220)">
                        <text x="0" y="0" class="eyebrow" fill="{accent}">{type_text}</text>
                        <text x="0" y="42" class="title" font-size="{title_font_size}">{title_svg}</text>
                        {desc_svg}
                        
                        <rect x="0" y="146" width="264" height="1.5" rx="0.75" fill="var(--text)" fill-opacity="0.1"/>
                        <rect x="0" y="146" width="94" height="1.5" rx="0.75" fill="{accent}"/>
                        
                        <circle cx="10" cy="188" r="5" fill="{accent}"/>
                        <text x="24" y="192" class="meta">OPEN RESOURCE</text>
                        <path d="M251 184 L265 184 M260 179 L265 184 L260 189" fill="none" stroke="var(--text)" stroke-opacity="0.4" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                    </g>
                </g>
            </g>"##,
                x=x, y=y, bw=button_width, bh=button_height, id=id, label=escape(&btn.label),
                bw_inner=button_width - 1.5, bh_inner=button_height - 1.5,
                link=btn.link, win=win, grad_id=gradient_id, accent=accent,
                badge_width=badge_width, badge_center=24.0 + badge_width as f64 / 2.0,
                type_text=escape(type_text),
                glyph_escaped=escape(glyph),
                title_font_size=title_font_size,
                title_svg=title_svg,
                desc_svg=desc_svg
            ));
        }
    }

    let extra_class = if config.use_dark { " dark-mode" } else { "" };

    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_width}" height="{svg_height}" viewBox="0 0 {width} {height}" xmlns:xlink="http://www.w3.org/1999/xlink" id="btn_{id}" class="button-container{extra_class}">
    <defs>
        <linearGradient id="glassCard_{id}" x1="0%" y1="0%" x2="0%" y2="1">
            <stop offset="0%" stop-color="var(--surface)" stop-opacity="var(--glass-opacity)"/>
            <stop offset="100%" stop-color="var(--surface)" stop-opacity="var(--glass-stop)"/>
        </linearGradient>
        <linearGradient id="shine_{id}" x1="0%" y1="0%" x2="1" y2="1">
            <stop offset="0%" stop-color="#ffffff" stop-opacity="0.55"/>
            <stop offset="45%" stop-color="#ffffff" stop-opacity="0.08"/>
            <stop offset="100%" stop-color="#ffffff" stop-opacity="0"/>
        </linearGradient>
        <filter id="premiumShadow_{id}" x="-20%" y="-20%" width="140%" height="150%">
            <feDropShadow dx="0" dy="18" stdDeviation="18" flood-color="#0f172a" flood-opacity="var(--shadow-opacity)"/>
            <feDropShadow dx="0" dy="4" stdDeviation="5" flood-color="#0f172a" flood-opacity="0.08"/>
        </filter>
        <filter id="softGlow_{id}" x="-40%" y="-40%" width="180%" height="180%">
            <feGaussianBlur stdDeviation="18" result="blur"/>
            <feColorMatrix in="blur" type="matrix" values="1 0 0 0 0 0 1 0 0 0 0 0 1 0 0 0 0 0 0.55 0"/>
        </filter>
        <clipPath id="cardClip_{id}">
            <rect x="0" y="0" width="{bw}" height="{bh}" rx="28"/>
        </clipPath>
        {gradients}
        {styles}
    </defs>
    <rect width="100%" height="100%" fill="var(--bg)" rx="12"/>
    <g transform="scale({scale})">
        {elements}
    </g>
</svg>"##, svg_width=svg_width, svg_height=svg_height, width=width, height=height, id=id, bw=button_width, bh=button_height, gradients=gradients, elements=elements, scale=config.scale, styles=get_styles(id, "large"), extra_class=extra_class)
}

fn render_hex(buttons: &[Button], config: &ButtonConfig, id: &str) -> String {
    let button_width = 295.0;
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut row_count = 0;
    
    for btn in buttons {
        let cols_this_row = if row_count % 2 == 0 { config.columns } else { (config.columns as isize - 1).max(1) as usize };
        current_row.push(btn.clone());
        if current_row.len() == cols_this_row {
            rows.push(current_row);
            current_row = Vec::new();
            row_count += 1;
        }
    }
    if !current_row.is_empty() {
        rows.push(current_row);
    }

    let width = (config.columns as f64 * button_width + 80.0) * config.scale;
    let height = (rows.len() as f64 * 255.0 + 130.0) * config.scale;

    let svg_width = format!("{:.1}", width / 1.77);
    let svg_height = format!("{:.1}", height / 1.77);

    let mut elements = String::new();
    let mut gradients = String::new();
    let win = if config.new_win { "_blank" } else { "_top" };

    for (r_idx, row) in rows.iter().enumerate() {
        let start_x = if r_idx % 2 == 0 { 38.5 } else { 186.0 };
        let y = 20.0 + r_idx as f64 * 255.0;
        
        for (c_idx, btn) in row.iter().enumerate() {
            let x = start_x + c_idx as f64 * button_width;
            let accent_raw = if btn.color.is_empty() { "#2563eb" } else { &btn.color };
            let accent = escape(accent_raw);
            let button_idx = r_idx * config.columns + c_idx;
            let orb_id = format!("orb_{}_{}", id, button_idx);
            let glow_id = format!("glow_{}_{}", id, button_idx);
            
            let (r, g, b) = get_rgb(accent_raw);
            
            gradients.push_str(&format!(r##"
        <radialGradient id="{orb_id}" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stop-color="{accent}" stop-opacity="0.48"/>
            <stop offset="100%" stop-color="{accent}" stop-opacity="0"/>
        </radialGradient>
        <filter id="{glow_id}" x="-40%" y="-40%" width="180%" height="180%">
            <feGaussianBlur stdDeviation="20"/>
        </filter>"##, orb_id=orb_id, glow_id=glow_id, accent=accent));

            let type_text = btn.button_type.to_uppercase();
            let desc_text = btn.description.to_uppercase();
            
            let accent_dark = darken_color(accent_raw, 0.4);
            let grad_id = format!("hexGrad_{}_{}", id, button_idx);
            gradients.push_str(&format!(r##"
        <linearGradient id="{grad_id}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="{accent}"/>
            <stop offset="100%" stop-color="{accent_dark}"/>
        </linearGradient>"##, grad_id=grad_id, accent=accent, accent_dark=accent_dark));

            let glyph = if btn.label.len() >= 1 { &btn.label[0..1] } else { "B" };

            elements.push_str(&format!(r##"
            <g transform="translate({x}, {y})">
                <g class="hex-button" onclick="window.open('{link}', '{win}')" role="button" tabindex="0">
                    <title>{label}</title>
                    <circle class="outer-halo" cx="149" cy="170" r="136" fill="url(#{orb_id})" opacity="0.16" filter="url(#{glow_id})"/>
                    <polygon points="291,254 149,336 7,254 7,90 149,8 291,90" 
                             fill="url(#{grad_id})" 
                             filter="url(#premiumHexShadow_{id})"/>
                    <polygon points="291,254 149,336 7,254 7,90 149,8 291,90" 
                             fill="url(#hexGlass_{id})"/>
                    <polygon class="hex-ring"
                             points="291,254 149,336 7,254 7,90 149,8 291,90" 
                             fill="none" 
                             stroke="{accent}" 
                             stroke-opacity="0.56"
                             stroke-width="2"/>
                    <polygon points="149,18 276,96 149,173 22,96" 
                             fill="url(#hexShine_{id})" 
                             pointer-events="none"/>
                    <path d="M51 254 L149 311 L247 254" fill="none" stroke="#ffffff" stroke-opacity="0.16" stroke-width="2"/>
                    
                    <circle cx="149" cy="121" r="34" fill="#ffffff" fill-opacity="0.14"/>
                    <text x="149" y="133" text-anchor="middle" class="hex-icon">{glyph_escaped}</text>
                    
                    <text x="149" y="189" text-anchor="middle" class="hex-label">{label}</text>
                    <text x="149" y="216" text-anchor="middle" class="hex-type">{type_text}</text>
                    <text x="149" y="243" text-anchor="middle" class="hex-caption">{desc_text}</text>
                </g>
            </g>"##,
                x=x, y=y, id=id, label=escape(&btn.label), link=btn.link, win=win,
                orb_id=orb_id, glow_id=glow_id, accent=accent, grad_id=grad_id,
                glyph_escaped=escape(glyph), type_text=escape(&type_text), desc_text=escape(&desc_text)
            ));
        }
    }

    let extra_class = if config.use_dark { " dark-mode" } else { "" };

    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_width}" height="{svg_height}" viewBox="0 0 {width} {height}" xmlns:xlink="http://www.w3.org/1999/xlink" id="btn_{id}" class="button-container{extra_class}">
    <defs>
        <linearGradient id="premiumBg_{id}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--bg)"/>
            <stop offset="100%" stop-color="var(--surface)" stop-opacity="0.5"/>
        </linearGradient>
        <filter id="premiumHexShadow_{id}" x="-30%" y="-30%" width="160%" height="170%">
            <feDropShadow dx="0" dy="18" stdDeviation="18" flood-color="#0f172a" flood-opacity="var(--shadow-opacity)"/>
            <feDropShadow dx="0" dy="4" stdDeviation="5" flood-color="#0f172a" flood-opacity="0.10"/>
        </filter>
        <linearGradient id="hexGlass_{id}" x1="0%" y1="0%" x2="0%" y2="1">
            <stop offset="0%" stop-color="#ffffff" stop-opacity="0.34"/>
            <stop offset="55%" stop-color="#ffffff" stop-opacity="0.08"/>
            <stop offset="100%" stop-color="#ffffff" stop-opacity="0"/>
        </linearGradient>
        <linearGradient id="hexShine_{id}" x1="0%" y1="0%" x2="1" y2="1">
            <stop offset="0%" stop-color="#ffffff" stop-opacity="0.72"/>
            <stop offset="36%" stop-color="#ffffff" stop-opacity="0.16"/>
            <stop offset="100%" stop-color="#ffffff" stop-opacity="0"/>
        </linearGradient>
        {gradients}
        {styles}
    </defs>
    <rect width="100%" height="100%" fill="url(#premiumBg_{id})" rx="12"/>
    <g transform="scale({scale})">
        {elements}
    </g>
</svg>"##, svg_width=svg_width, svg_height=svg_height, width=width, height=height, id=id, gradients=gradients, elements=elements, scale=config.scale, styles=get_styles(id, "hex"), extra_class=extra_class)
}

fn render_pill(buttons: &[Button], config: &ButtonConfig, id: &str) -> String {
    let button_width = 300.0;
    let button_height = 56.0;
    let button_padding = 12.0;
    let start_x = 20.0;
    let start_y = 26.0;

    let rows: Vec<&[Button]> = buttons.chunks(config.columns).collect();
    let max_in_row = if rows.is_empty() { 0 } else { rows.iter().map(|r| r.len()).max().unwrap_or(0) };

    let width = (start_x + max_in_row as f64 * (button_width + button_padding) + 20.0) * config.scale;
    let height = (start_y + rows.len() as f64 * (button_height + button_padding) + 20.0) * config.scale;

    let svg_width = format!("{:.1}", width / 1.77);
    let svg_height = format!("{:.1}", height / 1.77);

    let mut elements = String::new();
    let mut gradients = String::new();
    let win = if config.new_win { "_blank" } else { "_top" };

    gradients.push_str(r##"
        <linearGradient id="topshineGrad" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="#ffffff" stop-opacity="0.4" />
            <stop offset="100%" stop-color="#ffffff" stop-opacity="0" />
        </linearGradient>"##);

    for (r_idx, row) in rows.iter().enumerate() {
        let y = start_y + r_idx as f64 * (button_height + button_padding);
        for (c_idx, btn) in row.iter().enumerate() {
            let x = start_x + c_idx as f64 * (button_width + button_padding);
            let accent = if btn.color.is_empty() { "var(--accent)".to_string() } else { escape(&btn.color) };
            let gradient_id = format!("pillGrad_{}_{}", id, elements.len());
            let text_color = if btn.color.is_empty() { "var(--text)".to_string() } else { determine_text_color(&btn.color).to_string() };

            gradients.push_str(&format!(r##"
        <linearGradient id="{grad_id}" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="{accent}" />
            <stop offset="100%" stop-color="{accent}" stop-opacity="0.7" />
        </linearGradient>"##, grad_id=gradient_id, accent=accent));

            elements.push_str(&format!(r##"
            <g transform="translate({x}, {y})">
                <g class="button-hover" role="button" tabindex="0" onclick="window.open('{link}', '{win}')">
                    <title>{label}</title>
                    <rect x="0" y="0" width="{bw}" height="{bh}" rx="26" ry="26" fill="url(#{grad_id})" filter="url(#cardShadow_{id})" />
                    <rect x="10" y="5" width="280" height="25" rx="24" ry="24" fill="url(#topshineGrad)" fill-opacity="0.15"/>
                    <rect x="20" y="44" width="260" height="7" rx="24" ry="24" fill="#ffffff" fill-opacity="0.1"/>
                    <text x="150" y="33" text-anchor="middle" fill="{text_color}" style="font-weight: 700; font-size: 14px;">{label}</text>
                </g>
            </g>"##,
                x=x, y=y, bw=button_width, bh=button_height, id=id, label=escape(&btn.label),
                link=btn.link, win=win, grad_id=gradient_id, text_color=text_color
            ));
        }
    }

    let extra_class = if config.use_dark { " dark-mode" } else { "" };

    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_width}" height="{svg_height}" viewBox="0 0 {width} {height}" xmlns:xlink="http://www.w3.org/1999/xlink" id="btn_{id}" class="button-container{extra_class}">
    <defs>
        {gradients}
        <filter id="cardShadow_{id}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="4" stdDeviation="6" flood-opacity="var(--shadow-opacity)"/>
        </filter>
        {styles}
    </defs>
    <rect width="100%" height="100%" fill="var(--bg)" rx="12"/>
    <g transform="scale({scale})">
        {elements}
    </g>
</svg>"##, svg_width=svg_width, svg_height=svg_height, width=width, height=height, id=id, gradients=gradients, elements=elements, scale=config.scale, styles=get_styles(id, "pill"), extra_class=extra_class)
}

fn render_circle(buttons: &[Button], config: &ButtonConfig, id: &str) -> String {
    let circle_size = 130.0;
    let start_x = 35.0;
    let start_y = 35.0;

    let rows: Vec<&[Button]> = buttons.chunks(config.columns).collect();
    let max_in_row = if rows.is_empty() { 0 } else { rows.iter().map(|r| r.len()).max().unwrap_or(0) };

    let width = (start_x + max_in_row as f64 * circle_size + 30.0) * config.scale;
    let height = (start_y + rows.len() as f64 * circle_size + 30.0) * config.scale;

    let svg_width = format!("{:.1}", width / 1.77);
    let svg_height = format!("{:.1}", height / 1.77);

    let mut elements = String::new();
    let mut gradients = String::new();
    let win = if config.new_win { "_blank" } else { "_top" };

    gradients.push_str(r##"
        <linearGradient id="glassReflectionCircle" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="white" stop-opacity="0.2"/>
            <stop offset="100%" stop-color="white" stop-opacity="0"/>
        </linearGradient>"##);

    for (r_idx, row) in rows.iter().enumerate() {
        let y = start_y + r_idx as f64 * circle_size;
        for (c_idx, btn) in row.iter().enumerate() {
            let x = start_x + c_idx as f64 * circle_size;
            let accent = if btn.color.is_empty() { "var(--accent)".to_string() } else { escape(&btn.color) };
            let button_idx = r_idx * config.columns + c_idx;
            let gradient_id = format!("circleGrad_{}_{}", id, button_idx);
            let text_color = if btn.color.is_empty() { "var(--text)".to_string() } else { determine_text_color(&btn.color).to_string() };

            let base_color = if btn.color.is_empty() { "#3b82f6" } else { &btn.color };
            let darker = darken_color(base_color, 0.3);
            let lighter = brighten_color(base_color, 0.2);

            gradients.push_str(&format!(r##"
        <radialGradient id="{grad_id}" cx="30%" cy="25%" r="80%">
            <stop offset="0%" style="stop-color:{lighter};stop-opacity:1" />
            <stop offset="50%" style="stop-color:{base};stop-opacity:1" />
            <stop offset="100%" style="stop-color:{darker};stop-opacity:1" />
        </radialGradient>"##, grad_id=gradient_id, lighter=lighter, base=escape(base_color), darker=darker));

            let lines = wrap_text(&btn.label, 12);
            let mut title_svg = String::new();
            let dy_values = match lines.len() {
                2 => vec![-6, 12],
                3 => vec![-12, 12, 12],
                4 => vec![-18, 12, 12, 12],
                _ => vec![0],
            };
            for (i, line) in lines.iter().take(4).enumerate() {
                title_svg.push_str(&format!(r##"<tspan x="60" dy="{}">{}</tspan>"##, dy_values[i], escape(line)));
            }

            elements.push_str(&format!(r##"
            <g transform="translate({x}, {y})">
                <g role="button" tabindex="0" class="button-hover circle-group" onclick="window.open('{link}', '{win}')">
                    <title>{desc}</title>
                    <circle r="50" cx="60" cy="60" fill="black" opacity="0.15" filter="url(#cardShadow_{id})"/>
                    <circle class="tech-ring" r="58" cx="60" cy="60" fill="none" stroke="{accent}" stroke-width="2" stroke-opacity="0.3"/>
                    <circle class="orb-body" r="50" cx="60" cy="60" fill="url(#{grad_id})" stroke="{accent}" stroke-width="1.5" stroke-opacity="0.2"/>
                    <circle class="orb-glass" r="46" cx="60" cy="60" fill="url(#glassReflectionCircle)" pointer-events="none"/>
                    
                    <g class="moving-text">
                        <path d="M 25 35 L 25 25 L 35 25" fill="none" stroke="{accent}" stroke-width="2.5" opacity="0.8"/>
                        <text x="60" y="60" text-anchor="middle" dominant-baseline="central" fill="{text_color}" style="font-family: 'Lexend', sans-serif; font-weight: 800; font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; pointer-events: none;">
                            {title_svg}
                        </text>
                        <text x="60" y="78" text-anchor="middle" fill="{text_color}" fill-opacity="0.6" style="font-size: 7px; font-weight: 400;">ID: 0x{hex_id}</text>
                    </g>
                </g>
            </g>"##,
                x=x, y=y, id=id, desc=escape(&btn.description),
                link=btn.link, win=win, grad_id=gradient_id, accent=accent, text_color=text_color,
                title_svg=title_svg, hex_id=format!("{:x}", button_idx + 100)
            ));
        }
    }

    let extra_class = if config.use_dark { " dark-mode" } else { "" };

    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_width}" height="{svg_height}" viewBox="0 0 {width} {height}" xmlns:xlink="http://www.w3.org/1999/xlink" id="btn_{id}" class="button-container{extra_class}">
    <defs>
        {gradients}
        <filter id="cardShadow_{id}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="4" stdDeviation="6" flood-opacity="var(--shadow-opacity)"/>
        </filter>
        {styles}
    </defs>
    <rect width="100%" height="100%" fill="var(--bg)" rx="12"/>
    <g transform="scale({scale})">
        {elements}
    </g>
</svg>"##, svg_width=svg_width, svg_height=svg_height, width=width, height=height, id=id, gradients=gradients, elements=elements, scale=config.scale, styles=get_styles(id, "circle"), extra_class=extra_class)
}

fn render_round(buttons: &[Button], config: &ButtonConfig, id: &str) -> String {
    let orb_spacing = 140.0;
    let start_x = 80.0;
    let start_y = 80.0;

    let rows: Vec<&[Button]> = buttons.chunks(config.columns).collect();
    let max_in_row = if rows.is_empty() { 0 } else { rows.iter().map(|r| r.len()).max().unwrap_or(0) };

    let width = (start_x + (max_in_row as f64 - 1.0) * orb_spacing + 80.0) * config.scale;
    let height = (start_y + (rows.len() as f64 - 1.0) * orb_spacing + 80.0) * config.scale;

    let svg_width = format!("{:.1}", width / 1.77);
    let svg_height = format!("{:.1}", height / 1.77);

    let mut elements = String::new();
    let mut gradients = String::new();
    let win = if config.new_win { "_blank" } else { "_top" };

    gradients.push_str(r##"
        <linearGradient id="glassReflectionRound" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="white" stop-opacity="0.4"/>
            <stop offset="50%" stop-color="white" stop-opacity="0.05"/>
            <stop offset="100%" stop-color="white" stop-opacity="0"/>
        </linearGradient>"##);

    for (r_idx, row) in rows.iter().enumerate() {
        let y = start_y + r_idx as f64 * orb_spacing;
        for (c_idx, btn) in row.iter().enumerate() {
            let x = start_x + c_idx as f64 * orb_spacing;
            let accent = if btn.color.is_empty() { "var(--accent)".to_string() } else { escape(&btn.color) };
            let button_idx = r_idx * config.columns + c_idx;
            let gradient_id = format!("roundGrad_{}_{}", id, button_idx);
            let text_color = if btn.color.is_empty() { "var(--text)".to_string() } else { determine_text_color(&btn.color).to_string() };

            let base_color = if btn.color.is_empty() { "#3b82f6" } else { &btn.color };
            let darker = darken_color(base_color, 0.4);

            gradients.push_str(&format!(r##"
        <radialGradient id="{grad_id}" cx="35%" cy="30%" r="65%">
            <stop offset="0%" style="stop-color:{base};stop-opacity:1" />
            <stop offset="100%" style="stop-color:{darker};stop-opacity:1" />
        </radialGradient>"##, grad_id=gradient_id, base=escape(base_color), darker=darker));

            let lines = wrap_text(&btn.label, 15);
            let line_y = if lines.is_empty() { 0 } else { (lines.len() as i32) * -6 };
            let mut title_svg = String::new();
            for (i, line) in lines.iter().enumerate() {
                let dy = if i == 0 { "0" } else { "12" };
                title_svg.push_str(&format!(r##"<tspan x="0" dy="{}">{}</tspan>"##, dy, escape(line)));
            }

            elements.push_str(&format!(r##"
            <g transform="translate({x}, {y})">
                <g role="button" tabindex="0" class="button-hover orb-group" onclick="window.open('{link}', '{win}')">
                    <title>{label}</title>
                    <circle r="62" cx="0" cy="0" fill="none" stroke="{accent}" stroke-width="2" class="glow-ring" stroke-opacity="0.3"/>
                    
                    <g class="moving-group">
                        <circle r="55" cx="0" cy="0" fill="black" opacity="0.15" filter="url(#cardShadow_{id})"/>
                        <circle r="55" cx="0" cy="0" fill="url(#{grad_id})" stroke="{accent}" stroke-width="1.5" stroke-opacity="0.2"/>
                        <circle r="50" cx="0" cy="-2" fill="url(#glassReflectionRound)" pointer-events="none"/>
                        <circle r="4" cx="-18" cy="-18" fill="white" fill-opacity="0.4" pointer-events="none"/>
                    </g>
                    
                    <g class="moving-group">
                        <text x="0" y="{line_y}" text-anchor="middle" fill="{text_color}" style="font-weight: 800; font-size: 12px; text-transform: uppercase; letter-spacing: 0.05em; pointer-events: none;">
                            {title_svg}
                        </text>
                    </g>
                </g>
            </g>"##,
                x=x, y=y, id=id, label=escape(&btn.label), link=btn.link, win=win, 
                grad_id=gradient_id, accent=accent, text_color=text_color, line_y=line_y, title_svg=title_svg
            ));
        }
    }

    let extra_class = if config.use_dark { " dark-mode" } else { "" };

    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_width}" height="{svg_height}" viewBox="0 0 {width} {height}" xmlns:xlink="http://www.w3.org/1999/xlink" id="btn_{id}" class="button-container{extra_class}">
    <defs>
        {gradients}
        <filter id="cardShadow_{id}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="8" stdDeviation="12" flood-opacity="var(--shadow-opacity)"/>
        </filter>
        {styles}
    </defs>
    <rect width="100%" height="100%" fill="var(--bg)" rx="12"/>
    <g transform="scale({scale})">
        {elements}
    </g>
</svg>"##, svg_width=svg_width, svg_height=svg_height, width=width, height=height, id=id, gradients=gradients, elements=elements, scale=config.scale, styles=get_styles(id, "round"), extra_class=extra_class)
}

fn render_rectangle(buttons: &[Button], config: &ButtonConfig, id: &str) -> String {
    let card_width = 300.0;
    let card_height = 100.0;
    let card_padding = 15.0;
    let start_x = 22.5;
    let start_y = 22.5;

    let rows: Vec<&[Button]> = buttons.chunks(config.columns).collect();
    let max_in_row = if rows.is_empty() { 0 } else { rows.iter().map(|r| r.len()).max().unwrap_or(0) };

    let width = (start_x + max_in_row as f64 * (card_width + card_padding) + 10.0) * config.scale;
    let height = (start_y + rows.len() as f64 * (card_height + card_padding) + 10.0) * config.scale;

    let svg_width = format!("{:.1}", width / 1.77);
    let svg_height = format!("{:.1}", height / 1.77);

    let mut elements = String::new();
    let win = if config.new_win { "_blank" } else { "_top" };

    for (r_idx, row) in rows.iter().enumerate() {
        let y = start_y + r_idx as f64 * (card_height + card_padding);
        for (c_idx, btn) in row.iter().enumerate() {
            let x = start_x + c_idx as f64 * (card_width + card_padding);
            let accent = if btn.color.is_empty() { "var(--accent)".to_string() } else { escape(&btn.color) };
            
            let label = escape(&btn.label);
            let desc = if btn.description.is_empty() { &btn.label } else { &btn.description };

            elements.push_str(&format!(r##"
            <g transform="translate({x}, {y})">
                <g class="button-hover" role="button" tabindex="0" onclick="window.open('{link}', '{win}')">
                    <title>{desc}</title>
                    <rect x="0" y="0" width="{bw}" height="{bh}" rx="12" ry="12" fill="var(--surface)" stroke="var(--accent)" stroke-opacity="0.15" stroke-width="1" filter="url(#cardShadow_{id})"/>
                    <rect x="0" y="20" width="4" height="60" fill="{accent}" rx="2"/>
                    <text x="20" y="38" fill="var(--text)" style="font-size: 18px; font-weight: 800;">{label}</text>
                    <text x="20" y="62" fill="var(--text)" fill-opacity="0.6" style="font-size: 11px;">
                        {desc_wrapped}
                    </text>
                </g>
            </g>"##,
                x=x, y=y, bw=card_width, bh=card_height, id=id, label=label,
                link=btn.link, win=win, accent=accent, desc=escape(desc),
                desc_wrapped=wrap_rectangle_desc(&btn.description)
            ));
        }
    }

    let extra_class = if config.use_dark { " dark-mode" } else { "" };

    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_width}" height="{svg_height}" viewBox="0 0 {width} {height}" xmlns:xlink="http://www.w3.org/1999/xlink" id="btn_{id}" class="button-container{extra_class}">
    <defs>
        <filter id="cardShadow_{id}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="4" stdDeviation="6" flood-opacity="var(--shadow-opacity)"/>
        </filter>
        {styles}
    </defs>
    <rect width="100%" height="100%" fill="var(--bg)" rx="12"/>
    <g transform="scale({scale})">
        {elements}
    </g>
</svg>"##, svg_width=svg_width, svg_height=svg_height, width=width, height=height, id=id, elements=elements, scale=config.scale, styles=get_styles(id, "rectangle"), extra_class=extra_class)
}

fn wrap_rectangle_desc(text: &str) -> String {
    let lines = wrap_text(text, 45);
    let mut result = String::new();
    for (i, line) in lines.iter().take(3).enumerate() {
        result.push_str(&format!(r##"<tspan x="20" dy="{}">{}</tspan>"##, if i == 0 { 0 } else { 14 }, escape(line)));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_regular() {
        let body = "----
---
Google | https://google.com
GitHub | https://github.com
----";
        let result = render(body, &HashMap::new()).unwrap();
        assert!(result.contains("Google"));
        assert!(result.contains("GitHub"));
        assert!(result.contains("onclick=\"window.open('https://google.com', '_blank')\""));
    }

    #[test]
    fn test_render_large() {
        let body = "----
shape=large
---
Large Button | https://example.com | Type A | A large button description | #ff0000 | 2024-09-06
----";
        let result = render(body, &HashMap::new()).unwrap();
        assert!(result.contains("Large Button"));
        assert!(result.contains("TYPE A"));
        assert!(result.contains("A large button description"));
        assert!(result.contains("#ff0000"));
    }
    
    #[test]
    fn test_render_pill() {
        let body = "----
shape=pill
---
Pill Button | https://example.com | Pill
----";
        let result = render(body, &HashMap::new()).unwrap();
        assert!(result.contains("Pill Button"));
        assert!(result.contains("rx=\"26\" ry=\"26\""));
        assert!(result.contains("topshineGrad"));
    }

    #[test]
    fn test_render_circle() {
        let body = "----
shape=circle
---
Circle | https://example.com | Circle
----";
        let result = render(body, &HashMap::new()).unwrap();
        assert!(result.contains("Circle"));
        assert!(result.contains("class=\"tech-ring\""));
        assert!(result.contains("class=\"orb-body\""));
    }

    #[test]
    fn test_render_round() {
        let body = "----
shape=round
---
Round Button | https://example.com | Round
----";
        let result = render(body, &HashMap::new()).unwrap();
        assert!(result.contains("Round Button"));
        assert!(result.contains("class=\"glow-ring\""));
        assert!(result.contains("class=\"moving-group\""));
    }

    #[test]
    fn test_render_large_wrap() {
        let body = "----
shape=large
---
Rust | https://rust-lang.org | Language | A language empowering everyone to build reliable and efficient software. | #dea584 | 2024-09-06
----";
        let result = render(body, &HashMap::new()).unwrap();
        // The description should be wrapped
        assert!(result.contains("<tspan x=\"0\" dy=\"0\">A language empowering everyone to</tspan>"));
        assert!(result.contains("<tspan x=\"0\" dy=\"20\">build reliable and efficient</tspan>"));
        assert!(result.contains("<tspan x=\"0\" dy=\"20\">software.</tspan>"));
    }

    #[test]
    fn test_render_hex() {
        let body = "----
shape=hex
---
Hex Button | https://example.com | HexType
----";
        let result = render(body, &HashMap::new()).unwrap();
        assert!(result.contains("Hex Button"));
        assert!(result.contains("HEXTYPE"));
        assert!(result.contains("polygon points="));
    }

    #[test]
    fn test_render_rectangle() {
        let body = "----
shape=rectangle
---
Rect Button | https://example.com | Rect | This is a rectangle button description | #ff00ff
----";
        let result = render(body, &HashMap::new()).unwrap();
        assert!(result.contains("Rect Button"));
        assert!(result.contains("This is a rectangle button description"));
        assert!(result.contains("width=\"300\" height=\"100\""));
        assert!(result.contains("width=\"4\" height=\"60\""));
    }
}
