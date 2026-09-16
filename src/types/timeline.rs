use crate::common::svg::escape;
use crate::common::kv::parse_kv_header;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct TimelineEntry {
    date: String,
    text: String,
    category: Option<String>,
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
    let card_w = 280;
    let padding_top = 160;
    let padding_bottom = 60;
    let mid_x = width / 2;
    
    // First pass: calculate y positions and card heights
    let mut entry_data = Vec::new();
    for entry in &timeline.entries {
        let wrapped_text = wrap_text(&entry.text, (card_w - 40) as f32, 7.5);
        let text_lines_count = wrapped_text.len();
        
        let mut text_lines_svg = String::new();
        for (idx, line) in wrapped_text.iter().enumerate() {
            text_lines_svg.push_str(&format!(
                r##"<text x="20" y="{y}" class="entry-text">{text}</text>"##,
                y = 55 + (idx * 18),
                text = escape(line)
            ));
        }

        let last_text_y = 55 + (text_lines_count.saturating_sub(1) * 18);
        
        let category_y = last_text_y + 25;
        let category_svg = if let Some(cat) = &entry.category {
            format!(r##"<text x="20" y="{y}" class="entry-category" fill-opacity="0.5">{cat}</text>"##, 
                y = category_y,
                cat = escape(&cat.to_uppercase()))
        } else {
            "".to_string()
        };

        let card_h = if entry.category.is_some() {
            category_y + 20
        } else {
            last_text_y + 25
        };
        let card_h = card_h.max(100) as i32;
        
        entry_data.push((text_lines_svg, category_svg, card_h));
    }
    
    let mut y_positions = Vec::new();
    let mut current_y_cursor = padding_top as i32;
    for (_, _, card_h) in &entry_data {
        let y = current_y_cursor + (card_h / 2);
        y_positions.push(y);
        current_y_cursor = y + (card_h / 2) + 40; // 40px gap between cards
    }
    
    let total_height = current_y_cursor.max(padding_top + padding_bottom) + padding_bottom;
    
    let apple_colors = ["#3B82F6", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#06B6D4"];
    
    let mut entries_svg = String::new();
    let mut spine_segments = String::new();

    // Initial spine segment
    if !timeline.entries.is_empty() {
        let first_y = y_positions[0];
        let color = timeline.entries[0].color.clone().unwrap_or_else(|| apple_colors[0].to_string());
        spine_segments.push_str(&format!(
            r##"<line x1="{mid_x}" y1="{start_y}" x2="{mid_x}" y2="{first_y}" stroke="{color}" stroke-width="4" stroke-linecap="round" opacity="0.3" />"##,
            mid_x = mid_x, start_y = first_y - 40, first_y = first_y, color = color
        ));
    }
    
    for (i, entry) in timeline.entries.iter().enumerate() {
        let y = y_positions[i];
        let (text_lines_svg, category_svg, card_h) = &entry_data[i];
        let is_left = i % 2 == 0;
        let color = entry.color.clone().unwrap_or_else(|| apple_colors[i % apple_colors.len()].to_string());
        
        // Spine segments
        if i < timeline.entries.len() - 1 {
            let next_y = y_positions[i+1];
            spine_segments.push_str(&format!(
                r##"<line x1="{mid_x}" y1="{y}" x2="{mid_x}" y2="{next_y}" stroke="{color}" stroke-width="4" stroke-linecap="round" opacity="0.6" />"##,
                mid_x = mid_x, y = y, next_y = next_y, color = color
            ));
        } else {
            // Final spine segment
            spine_segments.push_str(&format!(
                r##"<line x1="{mid_x}" y1="{y}" x2="{mid_x}" y2="{end_y}" stroke="{color}" stroke-width="4" stroke-linecap="round" opacity="0.3" />"##,
                mid_x = mid_x, y = y, end_y = y + 40, color = color
            ));
        }

        let card_x = if is_left {
            mid_x - card_w - 40
        } else {
            mid_x + 40
        };
        
        let connector_x1 = if is_left { card_x + card_w } else { card_x };
        let connector_x2 = mid_x;
        
        entries_svg.push_str(&format!(
            r##"    <g class="timeline-entry" role="graphics-symbol" aria-roledescription="event" tabindex="0" aria-label="{date}: {text_raw}">
        <line x1="{c_x1}" y1="{y_mid}" x2="{c_x2}" y2="{y_mid}" stroke="{color}" stroke-width="2" opacity="0.4" aria-hidden="true" />
        <circle cx="{mid_x}" cy="{y_mid}" r="10" fill="{color}" filter="url(#{id}_glow)" aria-hidden="true" />
        <circle cx="{mid_x}" cy="{y_mid}" r="5" fill="white" aria-hidden="true" />
        
        <g transform="translate({card_x}, {card_y})">
            <rect width="{card_w}" height="{card_h}" rx="16" class="entry-card" aria-hidden="true" />
            <text x="20" y="32" class="entry-date" fill="{color}">{date}</text>
            {text_lines_svg}
            {category_svg}
        </g>
    </g>
"##,
            c_x1 = connector_x1,
            c_x2 = connector_x2,
            y_mid = y,
            color = color,
            mid_x = mid_x,
            id = id,
            card_x = card_x,
            card_y = y - (card_h / 2),
            card_w = card_w,
            card_h = card_h,
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
        <filter id="{id}_glow" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="4" result="blur" />
            <feComposite in="SourceGraphic" in2="blur" operator="over" />
        </filter>
        <style>
            #{id} {{
                --primary: #3B82F6;
                --text-main: #FFFFFF;
                --text-soft: #94A3B8;
                --bg-canvas: #0F172A;
                --bg-card: rgba(30, 41, 59, 0.7);
                --card-border: rgba(255, 255, 255, 0.1);
                background-color: var(--bg-canvas);
                font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            }}
            @media (prefers-color-scheme: light) {{
                #{id} {{
                    --text-main: #1E293B;
                    --text-soft: #64748B;
                    --bg-canvas: #F8FAFC;
                    --bg-card: rgba(255, 255, 255, 0.9);
                    --card-border: rgba(0, 0, 0, 0.05);
                }}
            }}
            #{id} .timeline-title {{ font-size: 32px; font-weight: 800; fill: var(--text-main); letter-spacing: -0.04em; }}
            #{id} .timeline-subtitle {{ font-size: 16px; fill: var(--text-soft); font-weight: 500; }}
            #{id} .entry-card {{ fill: var(--bg-card); stroke: var(--card-border); stroke-width: 1; }}
            #{id} .entry-date {{ font-size: 14px; font-weight: 800; letter-spacing: 0.02em; }}
            #{id} .entry-text {{ font-size: 14px; font-weight: 500; fill: var(--text-main); }}
            #{id} .entry-category {{ font-size: 10px; font-weight: 800; fill: var(--text-soft); letter-spacing: 0.1em; }}
            #{id} .timeline-entry:focus {{ outline: none; }}
            #{id} .timeline-entry:focus-visible .entry-card {{ stroke: var(--primary); stroke-width: 2; }}
        </style>
    </defs>
    
    <rect width="{width}" height="{height}" fill="var(--bg-canvas)" />
    
    <!-- Decorative background waves -->
    <g opacity="0.05" fill="none" stroke="var(--text-soft)" stroke-width="2" aria-hidden="true">
        <path d="M0 {h8} Q{w4} {h7} {w2} {h8} T{width} {h8}" />
        <path d="M0 {h6} Q{w4} {h5} {w2} {h6} T{width} {h6}" />
    </g>

    <text x="50" y="60" class="timeline-title">{title}</text>
    <text x="50" y="85" class="timeline-subtitle">{subtitle}</text>
    
    <g class="spine-container" aria-hidden="true">
        {spine_segments}
    </g>
    
    {entries_svg}
</svg>"##,
        id = id,
        width = width,
        height = total_height,
        title = escape(&timeline.title),
        subtitle = escape(&timeline.subtitle),
        spine_segments = spine_segments,
        entries_svg = entries_svg,
        w4 = width / 4,
        w2 = width / 2,
        h8 = total_height * 7 / 8,
        h7 = total_height * 6 / 8,
        h6 = total_height * 5 / 8,
        h5 = total_height * 4 / 8
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
