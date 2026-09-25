use crate::common::svg::escape;
use crate::common::kv::parse_kv_header;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct TimelineEntry {
    date: String,
    text: String,
    category: Option<String>,
    #[allow(dead_code)]
    color: Option<String>,
}

#[derive(Debug)]
struct Timeline {
    title: String,
    subtitle: String,
    #[allow(dead_code)]
    theme: String,
    entries: Vec<TimelineEntry>,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let timeline = parse_timeline(body)?;
    Ok(render_svg(&timeline, controls))
}

fn parse_timeline(body: &str) -> Result<Timeline, String> {
    let trimmed = body.trim();
    if !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("Timeline body must be wrapped in '---- ... ----'".into());
    }
    let inner = &trimmed[4..trimmed.len() - 4].trim();
    
    // We handle two formats: 
    // 1. With '---' separator for header config
    // 2. Just entries
    let (header_part, entries_part) = if let Some((h, e)) = inner.split_once("---") {
        (h.trim(), e.trim())
    } else {
        ("", inner.trim())
    };

    let config = parse_kv_header(header_part);
    
    let mut entries = Vec::new();
    let mut current_date = String::new();
    let mut current_text = Vec::new();
    let mut current_category = None;
    let mut current_color = None;

    for line in entries_part.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(date_val) = line.strip_prefix("date=") {
            if !current_date.is_empty() {
                entries.push(TimelineEntry {
                    date: current_date,
                    text: current_text.join(" "),
                    category: current_category,
                    color: current_color,
                });
                current_text.clear();
                current_category = None;
                current_color = None;
            }
            current_date = date_val.trim().to_string();
        } else if let Some(text_val) = line.strip_prefix("text=") {
            current_text.push(text_val.trim().to_string());
        } else if let Some(cat_val) = line.strip_prefix("category=") {
            current_category = Some(cat_val.trim().to_string());
        } else if let Some(col_val) = line.strip_prefix("color=") {
            current_color = Some(col_val.trim().to_string());
        } else if !current_date.is_empty() {
            // Append to current text if no prefix
            current_text.push(line.to_string());
        }
    }

    // Last entry
    if !current_date.is_empty() {
        entries.push(TimelineEntry {
            date: current_date,
            text: current_text.join(" "),
            category: current_category,
            color: current_color,
        });
    }

    if entries.is_empty() {
        return Err("No timeline entries found".into());
    }

    Ok(Timeline {
        title: config.get("title").cloned().unwrap_or_else(|| "Timeline".into()),
        subtitle: config.get("subtitle").cloned().unwrap_or_default(),
        theme: config.get("theme").cloned().unwrap_or_else(|| "premium".into()),
        entries,
    })
}

fn render_svg(timeline: &Timeline, _controls: &HashMap<String, String>) -> String {
    let id = format!("timeline_{}", Uuid::new_v4().to_string().replace('-', "_"));
    
    // Layout parameters
    let width = 800;
    let card_w = 288;
    let padding_top = 180;
    let padding_bottom = 80;
    let mid_x = width / 2;
    
    // First pass: calculate y positions and card heights
    let mut entry_data = Vec::new();
    for entry in &timeline.entries {
        let wrapped_text = wrap_text(&entry.text, 244.0, 7.8);
        let text_lines_count = wrapped_text.len();
        
        let mut text_lines_svg = String::new();
        for (idx, line) in wrapped_text.iter().enumerate() {
            text_lines_svg.push_str(&format!(
                r##"<text x="22" y="{y}" class="body">{text}</text>"##,
                y = 62 + (idx * 20),
                text = escape(line)
            ));
        }

        let last_text_y = 62 + (text_lines_count.saturating_sub(1) * 20);
        
        let category_svg = if let Some(cat) = &entry.category {
            format!(r##"<text x="22" y="{y}" class="category">{cat}</text>"##, 
                y = last_text_y + 20,
                cat = escape(&cat.to_uppercase()))
        } else {
            "".to_string()
        };

        let card_h = if entry.category.is_some() {
            last_text_y + 30
        } else {
            last_text_y + 22
        };
        let card_h = card_h.max(100);
        
        entry_data.push((text_lines_svg, category_svg, card_h));
    }
    
    let mut y_positions = Vec::new();
    let mut current_y_cursor = padding_top;
    for (_, _, card_h) in &entry_data {
        let y = current_y_cursor + (card_h / 2);
        y_positions.push(y);
        current_y_cursor = y + (card_h / 2) + 30; // Gap between cards
    }
    
    let total_height = current_y_cursor.max(padding_top + padding_bottom) + padding_bottom;
    
    let mut entries_svg = String::new();
    
    let spine_start_y = y_positions.first().map(|y| y - 50).unwrap_or(170);
    let spine_end_y = y_positions.last().map(|y| y + 40).unwrap_or(1024);

    for (i, entry) in timeline.entries.iter().enumerate() {
        let y = y_positions[i];
        let (text_lines_svg, category_svg, card_h) = &entry_data[i];
        let is_left = i % 2 == 0;
        
        let card_x = if is_left {
            mid_x - card_w - 40
        } else {
            mid_x + 40
        };
        
        let connector_x1 = if is_left { mid_x - 40 } else { mid_x + 40 };
        let connector_x2 = mid_x;
        
        entries_svg.push_str(&format!(
            r##"    <g class="timeline-entry" role="graphics-symbol" aria-roledescription="event" tabindex="0" aria-label="{date}: {text_raw}">
        <line x1="{c_x1}" y1="{y_mid}" x2="{c_x2}" y2="{y_mid}" class="connector" aria-hidden="true" />
        <circle cx="{mid_x}" cy="{y_mid}" r="18" class="node-ring" filter="url(#{id}_nodeGlow)" aria-hidden="true" />
        <circle cx="{mid_x}" cy="{y_mid}" r="5.5" class="node-core" aria-hidden="true" />
        
        <g transform="translate({card_x}, {card_y})">
            <rect width="{card_w}" height="{card_h}" rx="24" class="card" aria-hidden="true" />
            <rect x="1" y="1" width="{card_w_inner}" height="{card_h_inner}" rx="23" class="card-highlight" aria-hidden="true" />
            <text x="22" y="34" class="date">{date}</text>
            {text_lines_svg}
            {category_svg}
        </g>
    </g>
"##,
            c_x1 = connector_x1,
            c_x2 = connector_x2,
            y_mid = y,
            mid_x = mid_x,
            id = id,
            card_x = card_x,
            card_y = y - (card_h / 2),
            card_w = card_w,
            card_w_inner = card_w - 2,
            card_h = card_h,
            card_h_inner = card_h - 2,
            date = escape(&entry.date),
            text_lines_svg = text_lines_svg,
            category_svg = category_svg,
            text_raw = escape(&entry.text)
        ));
    }
    
    format!(
        r##"<svg viewBox="0 0 {width} {height}" width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg" role="graphics-document document" aria-labelledby="{id}_title {id}_desc" id="{id}">
    <title id="{id}_title">{title}</title>
    <desc id="{id}_desc">{subtitle}</desc>
    <defs>
        <radialGradient id="{id}_bgGlowBlue" cx="18%" cy="10%" r="65%">
            <stop offset="0%" stop-color="#60A5FA" stop-opacity="0.38"/>
            <stop offset="42%" stop-color="#3B82F6" stop-opacity="0.12"/>
            <stop offset="100%" stop-color="#0F172A" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="{id}_bgGlowViolet" cx="86%" cy="30%" r="60%">
            <stop offset="0%" stop-color="#A78BFA" stop-opacity="0.32"/>
            <stop offset="48%" stop-color="#8B5CF6" stop-opacity="0.11"/>
            <stop offset="100%" stop-color="#0F172A" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="{id}_bgGlowCyan" cx="70%" cy="90%" r="55%">
            <stop offset="0%" stop-color="#22D3EE" stop-opacity="0.24"/>
            <stop offset="56%" stop-color="#06B6D4" stop-opacity="0.09"/>
            <stop offset="100%" stop-color="#0F172A" stop-opacity="0"/>
        </radialGradient>
        <linearGradient id="{id}_spineGradient" x1="{mid_x}" y1="{spine_start_y}" x2="{mid_x}" y2="{spine_end_y}" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stop-color="#93C5FD" stop-opacity="0.28"/>
            <stop offset="15%" stop-color="#60A5FA" stop-opacity="0.86"/>
            <stop offset="45%" stop-color="#22D3EE" stop-opacity="0.82"/>
            <stop offset="70%" stop-color="#A78BFA" stop-opacity="0.82"/>
            <stop offset="100%" stop-color="#93C5FD" stop-opacity="0.24"/>
        </linearGradient>
        <linearGradient id="{id}_glassCard" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.24"/>
            <stop offset="45%" stop-color="#FFFFFF" stop-opacity="0.11"/>
            <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0.06"/>
        </linearGradient>
        <linearGradient id="{id}_glassCardLight" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.96"/>
            <stop offset="100%" stop-color="#EFF6FF" stop-opacity="0.78"/>
        </linearGradient>
        <filter id="{id}_softShadow" x="-25%" y="-35%" width="150%" height="180%">
            <feDropShadow dx="0" dy="18" stdDeviation="18" flood-color="#020617" flood-opacity="0.28"/>
            <feDropShadow dx="0" dy="1" stdDeviation="1" flood-color="#FFFFFF" flood-opacity="0.10"/>
        </filter>
        <filter id="{id}_nodeGlow" x="-90%" y="-90%" width="280%" height="280%">
            <feGaussianBlur stdDeviation="6" result="blur"/>
            <feColorMatrix in="blur" type="matrix" values="0 0 0 0 0.231 0 0 0 0 0.510 0 0 0 0 0.965 0 0 0 0.75 0"/>
            <feMerge>
                <feMergeNode/>
                <feMergeNode in="SourceGraphic"/>
            </feMerge>
        </filter>
        <style>
            #{id} {{
                --bg-canvas: #080D18;
                --text-main: #F8FAFC;
                --text-soft: #CBD5E1;
                --text-muted: #94A3B8;
                --card-fill: url(#{id}_glassCard);
                --card-border: rgba(255, 255, 255, 0.22);
                --accent: #3B82F6;
                --accent-soft: #93C5FD;
                --focus: #60A5FA;
                background-color: var(--bg-canvas);
                font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "Inter", "Segoe UI", Roboto, sans-serif;
            }}
            @media (prefers-color-scheme: light) {{
                #{id} {{
                    --bg-canvas: #F8FAFC;
                    --text-main: #0F172A;
                    --text-soft: #475569;
                    --text-muted: #64748B;
                    --card-fill: url(#{id}_glassCardLight);
                    --card-border: rgba(59, 130, 246, 0.18);
                    --accent: #2563EB;
                    --accent-soft: #3B82F6;
                    --focus: #2563EB;
                }}
            }}
            #{id} .title {{ font-size: 38px; font-weight: 800; letter-spacing: -0.055em; fill: var(--text-main); }}
            #{id} .subtitle {{ font-size: 15px; font-weight: 560; letter-spacing: -0.01em; fill: var(--text-soft); }}
            #{id} .card {{ fill: var(--card-fill); stroke: var(--card-border); stroke-width: 1; filter: url(#{id}_softShadow); }}
            #{id} .card-highlight {{ fill: none; stroke: rgba(255, 255, 255, 0.34); stroke-width: 1; }}
            #{id} .date {{ font-size: 13px; font-weight: 780; letter-spacing: -0.01em; fill: var(--accent-soft); }}
            #{id} .body {{ font-size: 14px; font-weight: 520; letter-spacing: -0.012em; fill: var(--text-main); }}
            #{id} .category {{ font-size: 10px; font-weight: 800; letter-spacing: 0.13em; fill: var(--text-muted); }}
            #{id} .connector {{ stroke: var(--accent-soft); stroke-width: 1.5; stroke-linecap: round; opacity: 0.42; }}
            #{id} .node-ring {{ fill: rgba(59, 130, 246, 0.16); stroke: rgba(147, 197, 253, 0.48); stroke-width: 1; }}
            #{id} .node-core {{ fill: #F8FAFC; }}
            #{id} .timeline-entry:focus {{ outline: none; }}
            #{id} .timeline-entry:focus-visible .card {{ stroke: var(--focus); stroke-width: 2.5; }}
            #{id} .timeline-entry:focus-visible .node-ring {{ stroke: var(--focus); stroke-width: 3; }}
        </style>
    </defs>
    
    <rect width="{width}" height="{height}" fill="var(--bg-canvas)" />
    
    <!-- Background Glows -->
    <rect width="{width}" height="{height}" fill="url(#{id}_bgGlowBlue)" aria-hidden="true" />
    <rect width="{width}" height="{height}" fill="url(#{id}_bgGlowViolet)" aria-hidden="true" />
    <rect width="{width}" height="{height}" fill="url(#{id}_bgGlowCyan)" aria-hidden="true" />

    <text x="50" y="70" class="title">{title}</text>
    <text x="50" y="95" class="subtitle">{subtitle}</text>
    
    <!-- Spine -->
    <line x1="{mid_x}" y1="{spine_start_y}" x2="{mid_x}" y2="{spine_end_y}" stroke="url(#{id}_spineGradient)" stroke-width="6" stroke-linecap="round" aria-hidden="true" />
    
    {entries_svg}
</svg>"##,
        id = id,
        width = width,
        height = total_height,
        title = escape(&timeline.title),
        subtitle = escape(&timeline.subtitle),
        mid_x = mid_x,
        spine_start_y = spine_start_y,
        spine_end_y = spine_end_y,
        entries_svg = entries_svg
    )
}

fn wrap_text(text: &str, max_width: f32, avg_char_w: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0.0;

    for word in text.split_whitespace() {
        let word_width = word.len() as f32 * avg_char_w;
        if current_width + word_width > max_width && !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
            current_line = String::new();
            current_width = 0.0;
        }
        current_line.push_str(word);
        current_line.push(' ');
        current_width += word_width + avg_char_w;
    }
    if !current_line.is_empty() {
        lines.push(current_line.trim().to_string());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timeline() {
        let body = r#"----
date= 1891
text= Mailbox, invented by Phillip Downing
category= Communication

date= July 23rd, 2023
text= DocOps extension Server releases Timeline Maker
category= Software
color= #FF0000
----"#;
        let timeline = parse_timeline(body).unwrap();
        assert_eq!(timeline.entries.len(), 2);
        assert_eq!(timeline.entries[0].date, "1891");
        assert_eq!(timeline.entries[0].category.as_deref(), Some("Communication"));
        assert_eq!(timeline.entries[1].color.as_deref(), Some("#FF0000"));
        assert!(timeline.entries[1].text.contains("DocOps extension Server"));
    }

    #[test]
    fn test_render_timeline_output() {
        let body = r#"----
title= Notable Inventions
subtitle= Brief History of Everyday Items
---
date= 1891
text= Mailbox, invented by Phillip Downing
category= Communication

date= 1885
text= First gasoline-powered automobile by Karl Benz
category= Transport

date= 1928
text= Penicillin discovered by Alexander Fleming
category= Medicine

date= 1989
text= World Wide Web proposed by Tim Berners-Lee
category= Communication
----"#;
        let svg = render(body, &HashMap::new()).unwrap();
        let _ = std::fs::write("gen/timeline.svg", &svg);
        assert!(svg.contains("Notable Inventions"));
        assert!(svg.contains("Phillip Downing"));
        assert!(svg.contains("filter"));
    }

    #[test]
    fn test_long_content_rendering() {
        let body = r#"----
title= Space Exploration
subtitle= Key Milestones
---
date= April 12, 1961
text= Yuri Gagarin becomes the first human to journey into outer space

date= July 20, 1969
text= Neil Armstrong and Buzz Aldrin become the first humans to land on the Moon

date= April 12, 1981
text= First launch of Space Shuttle Columbia

date= November 20, 1998
text= Launch of the first module of the International Space Station

date= February 6, 2018
text= SpaceX launches Falcon Heavy, sending a Tesla Roadster into space

date= May 30, 2020
text= SpaceX Crew Dragon Demo2 becomes the first private spacecraft to carry humans to the ISS
category= Private Spaceflight
----"#;
        let svg = render(body, &HashMap::new()).unwrap();
        let _ = std::fs::write("gen/space_timeline.svg", &svg);
        
        // Check for full text content that was previously truncated
        assert!(svg.contains("journey into outer space"));
        assert!(svg.contains("land on the Moon"));
        assert!(svg.contains("Tesla Roadster into space"));
        assert!(svg.contains("carry humans to the ISS"));
    }
}
