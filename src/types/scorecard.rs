use std::collections::HashMap;
use crate::common::svg::escape;
use crate::common::kv::parse_kv_header;
use uuid::Uuid;

#[derive(Debug, Default)]
struct Scorecard {
    title: String,
    subtitle: String,
    theme: String,
    before: Card,
    after: Card,
}

#[derive(Debug, Default)]
struct Card {
    title: String,
    sections: Vec<Section>,
}

#[derive(Debug, Default)]
struct Section {
    title: String,
    items: Vec<Item>,
}

#[derive(Debug, Default)]
struct Item {
    title: String,
    description: String,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let scorecard = parse_scorecard(body)?;
    let use_dark = controls.get("useDark").map(|s| s == "true").unwrap_or(false)
        || scorecard.theme == "dark";
    Ok(render_svg(&scorecard, use_dark))
}

fn parse_scorecard(body: &str) -> Result<Scorecard, String> {
    let mut trimmed = body.trim();
    if trimmed.starts_with("----") {
        trimmed = trimmed[4..].trim();
    }
    if trimmed.ends_with("----") {
        trimmed = trimmed[..trimmed.len() - 4].trim();
    }

    let parts: Vec<&str> = trimmed.split("---").map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err("Empty scorecard body".into());
    }

    let mut scorecard = Scorecard::default();

    // Part 0: Main Header
    let main_header = parse_kv_header(parts[0]);
    scorecard.title = main_header.get("title").cloned().unwrap_or_default();
    scorecard.subtitle = main_header.get("subtitle").cloned().unwrap_or_default();
    scorecard.theme = main_header.get("theme").cloned().unwrap_or_default();

    let mut i = 1;
    while i < parts.len() {
        let part = parts[i];
        if part.starts_with("[before]") {
            if i + 1 >= parts.len() {
                return Err("Missing [before.items]".into());
            }
            let header = parse_kv_header(part.strip_prefix("[before]").unwrap());
            scorecard.before.title = header.get("title").cloned().unwrap_or_default();
            
            i += 1;
            let items_part = parts[i];
            if items_part.starts_with("[before.items]") {
                scorecard.before.sections = parse_sections(items_part.strip_prefix("[before.items]").unwrap());
            } else {
                return Err(format!("Expected [before.items], got: {}", items_part));
            }
        } else if part.starts_with("[after]") {
            if i + 1 >= parts.len() {
                return Err("Missing [after.items]".into());
            }
            let header = parse_kv_header(part.strip_prefix("[after]").unwrap());
            scorecard.after.title = header.get("title").cloned().unwrap_or_default();
            
            i += 1;
            let items_part = parts[i];
            if items_part.starts_with("[after.items]") {
                scorecard.after.sections = parse_sections(items_part.strip_prefix("[after.items]").unwrap());
            } else {
                return Err(format!("Expected [after.items], got: {}", items_part));
            }
        }
        i += 1;
    }

    Ok(scorecard)
}

fn parse_sections(content: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current_section = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("===") {
            if let Some(sec) = current_section.take() {
                sections.push(sec);
            }
            current_section = Some(Section {
                title: line.strip_prefix("===").unwrap().trim().to_string(),
                items: Vec::new(),
            });
        } else if let Some(ref mut sec) = current_section {
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 2 {
                sec.items.push(Item {
                    title: parts[0].to_string(),
                    description: parts[1].to_string(),
                });
            }
        }
    }

    if let Some(sec) = current_section {
        sections.push(sec);
    }

    sections
}

fn render_svg(scorecard: &Scorecard, use_dark: bool) -> String {
    let id = format!("id_{}", Uuid::new_v4().to_string().replace('-', "_"));
    let width = 1024;
    
    // Calculate heights
    let item_height = 60;
    let card_top = 208;
    
    let before_height = calculate_card_height(&scorecard.before, item_height);
    let after_height = calculate_card_height(&scorecard.after, item_height);
    let max_card_height = before_height.max(after_height).max(400); // Minimum height
    
    let total_height = card_top + max_card_height + 40;
    let extra_class = if use_dark { " dark-mode" } else { "" };

    let mut svg = format!(
        r##"<svg width="{width}" height="{total_height}" viewBox="0 0 {width} {total_height}" xmlns="http://www.w3.org/2000/svg" class="scorecard-container{extra_class}" id="{id}">
      <metadata><rdf:rdf xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:cc="http://creativecommons.org/ns#"><cc:work rdf:about=""><dc:creator>DocOps.io</dc:creator><dc:rights>MIT License</dc:rights><dc:source>https://docops.io</dc:source><dc:date>2026-08-30</dc:date></cc:work></rdf:rdf></metadata>
      <desc>Generated by DocOps.io - Licensed under MIT License</desc>
    <defs>
        <pattern id="grid_{id}" width="40" height="40" patternUnits="userSpaceOnUse"><path d="M 40 0 L 0 0 0 40" fill="none" stroke="var(--grid-stroke)" stroke-width="1"/></pattern>
        <filter id="premiumShadow_{id}" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="12" stdDeviation="12" flood-color="var(--shadow-flood)" flood-opacity="var(--shadow-op)"/></filter>
        <style>
            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@100;200;300;400;500;600;700;800;900&amp;display=swap'); 
            #{id} {{
                --bg: #FFFFFF;
                --text-primary: #111827;
                --text-secondary: #6B7280;
                --grid-stroke: #3B82F6;
                --grid-op: 0.05;
                --card-bg-start: #FFFFFF;
                --card-bg-end: #f8fafc;
                --card-stroke-op: 0.05;
                --shadow-flood: #0F172A;
                --shadow-op: 0.12;
                --header-before-bg: #64748b;
                --header-after-bg: #3B82F6;
                --header-text-op: 0.08;
                --bullet-color: #64748b;
                --check-color: #3B82F6;
                --arrow-color: #3B82F6;
            }}
            @media (prefers-color-scheme: dark) {{
                #{id} {{
                    --bg: #0f172a;
                    --text-primary: #f8fafc;
                    --text-secondary: #94a3b8;
                    --grid-stroke: #3B82F6;
                    --grid-op: 0.1;
                    --card-bg-start: #1e293b;
                    --card-bg-end: #0f172a;
                    --card-stroke-op: 0.15;
                    --shadow-flood: #000000;
                    --shadow-op: 0.4;
                    --header-before-bg: #475569;
                    --header-after-bg: #60A5FA;
                    --bullet-color: #94a3b8;
                    --check-color: #60A5FA;
                    --arrow-color: #60A5FA;
                }}
            }}
            #{id}.dark-mode {{
                --bg: #0f172a;
                --text-primary: #f8fafc;
                --text-secondary: #94a3b8;
                --grid-stroke: #3B82F6;
                --grid-op: 0.1;
                --card-bg-start: #1e293b;
                --card-bg-end: #0f172a;
                --card-stroke-op: 0.15;
                --shadow-flood: #000000;
                --shadow-op: 0.4;
                --header-before-bg: #475569;
                --header-after-bg: #60A5FA;
                --bullet-color: #94a3b8;
                --check-color: #60A5FA;
                --arrow-color: #60A5FA;
            }}
            .main-title_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; font-size: 28.0px; fill: var(--text-primary); text-transform: none; letter-spacing: -0.5px; font-weight: 800; }}
            .sec-header_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; font-size: 16.0px; letter-spacing: 0px; text-transform: none; font-weight: 700; }} 
            .item-text_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; font-size: 14px; fill: var(--text-primary); font-weight: 500; }} 
            .item-desc_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; font-size: 12px; fill: var(--text-secondary); font-weight: 400; }} 
            .meta-text_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif, monospace; font-size: 10px; fill: var(--text-secondary); opacity: 0.5; }} 
            @keyframes slideUp_{id} {{ from {{ opacity: 0; transform: translateY(30px); }} to {{ opacity: 1; transform: translateY(0); }} }} 
            .anim-panel_{id} {{ animation: slideUp_{id} 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards; opacity: 1; }} 
            .delay-1_{id} {{ animation-delay: 0.1s; }} 
            .delay-2_{id} {{ animation-delay: 0.3s; }}
        </style>
    </defs>
    <rect width="100%" height="100%" fill="var(--bg)"/>
    <rect width="100%" height="100%" fill="url(#grid_{id})" opacity="var(--grid-op)"/>
"##,
        width = width,
        total_height = total_height,
        id = id,
        extra_class = extra_class
    );

    // Title Section
    svg.push_str(&format!(
        r##"    <g transform="translate(32, 60)">
        <rect x="0" y="8" width="6" height="48" fill="var(--header-after-bg)" rx="3"/>
        {title_lines}
    </g>
"##,
        title_lines = render_title_lines(&scorecard.title, &scorecard.subtitle, &id)
    ));

    // Cards Container
    svg.push_str(&format!(r##"    <g transform="translate(32, {card_top})">"##, card_top = card_top));

    // BEFORE Card
    svg.push_str(&render_card(&scorecard.before, &id, true, 464, max_card_height, "delay-1"));

    // AFTER Card
    svg.push_str(&render_card(&scorecard.after, &id, false, 464, max_card_height, "delay-2"));

    svg.push_str("    </g>\n");

    // Arrow
    let arrow_y = card_top + (max_card_height / 2);
    svg.push_str(&format!(
        r##"    <g transform="translate(500, {arrow_y})"><path d="M0,0 L24,0 L16,-8 M24,0 L16,8" stroke="var(--arrow-color)" stroke-width="2" fill="none" stroke-linecap="round" opacity="0.8"/></g>
"##,
        arrow_y = arrow_y
    ));

    // Meta text
    svg.push_str(&format!(
        r##"    <text x="32" y="{meta_y}" class="meta-text_{id}">SCORECARD_REF: A203 // SCALE: 1.0 // THEME: PremiumTheme</text>
"##,
        meta_y = total_height - 20,
        id = id
    ));

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scorecard_with_wrappers() {
        let dsl = r#"----
title=Software Release v2.4.0 - Feature & Bug Summary
subtitle=Migration from Legacy System to Modern Architecture
---
[before]
title=BEFORE
---
[before.items]
=== S1
I1 | D1
---
[after]
title=AFTER
---
[after.items]
=== S2
I2 | D2
----"#;

        let scorecard = parse_scorecard(dsl).unwrap();
        assert_eq!(scorecard.title, "Software Release v2.4.0 - Feature & Bug Summary");
        assert_eq!(scorecard.subtitle, "Migration from Legacy System to Modern Architecture");
    }

    #[test]
    fn test_parse_scorecard() {
        let dsl = r#"title=Software Release v2.4.0 - Feature & Bug Summary
subtitle=Migration from Legacy System to Modern Architecture
---

[before]
title=BEFORE v2.4.0
---
[before.items]
=== Feature Status
Dark Mode Theme | Missing feature affecting user experience
Multi-language Support | Not available, limiting global reach
=== Known Issues
Login timeout issues | Users frequently logged out
---

[after]
title=AFTER v2.4.0
---
[after.items]
=== New Features Added
Dark Mode Theme | Implemented with user preference saving
Multi-language Support | Added 12 languages with automatic detection
=== Bugs Resolved
Login timeout issues | Session management completely rewritten
"#;

        let scorecard = parse_scorecard(dsl).unwrap();
        assert_eq!(scorecard.title, "Software Release v2.4.0 - Feature & Bug Summary");
        assert_eq!(scorecard.subtitle, "Migration from Legacy System to Modern Architecture");
        assert_eq!(scorecard.before.title, "BEFORE v2.4.0");
        assert_eq!(scorecard.before.sections.len(), 2);
        assert_eq!(scorecard.before.sections[0].title, "Feature Status");
        assert_eq!(scorecard.before.sections[0].items.len(), 2);
        assert_eq!(scorecard.before.sections[0].items[0].title, "Dark Mode Theme");

        assert_eq!(scorecard.after.title, "AFTER v2.4.0");
        assert_eq!(scorecard.after.sections.len(), 2);
        assert_eq!(scorecard.after.sections[1].title, "Bugs Resolved");
        assert_eq!(scorecard.after.sections[1].items[0].title, "Login timeout issues");
    }

    #[test]
    fn test_render_scorecard_dark() {
        let dsl = r#"title=Test
theme=dark
---
[before]
title=Before
---
[before.items]
=== S1
I1 | D1
---
[after]
title=After
---
[after.items]
=== S2
I2 | D2
"#;
        let result = render(dsl, &HashMap::new());
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("dark-mode"));
        assert!(svg.contains("var(--bg)"));
    }

    #[test]
    fn test_render_scorecard_light() {
        let dsl = r#"title=Test Title
subtitle=Test Subtitle
---
[before]
title=Before
---
[before.items]
=== S1
I1 | D1
---
[after]
title=After
---
[after.items]
=== S2
I2 | D2
"#;
        let result = render(dsl, &HashMap::new());
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("class=\"scorecard-container\""));
        assert!(!svg.contains("class=\"scorecard-container dark-mode\""));
        assert!(svg.contains("Test Title"));
        assert!(svg.contains("Test Subtitle"));
        assert!(svg.contains("var(--text-primary)"));
    }
}

fn calculate_card_height(card: &Card, item_height: i32) -> i32 {
    let mut height = 40; // Card header
    for section in &card.sections {
        height += 60; // Section gap + title
        height += section.items.len() as i32 * item_height;
    }
    height + 40 // Padding bottom
}

fn render_title_lines(title: &str, subtitle: &str, id: &str) -> String {
    let mut lines = Vec::new();
    
    // Split title if it's too long
    let title_max_chars = 60;
    if title.len() > title_max_chars {
        let words = title.split_whitespace();
        let mut current_line = String::new();
        for word in words {
            if current_line.len() + word.len() + 1 > title_max_chars {
                lines.push(current_line.clone());
                current_line = word.to_string();
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            }
        }
        lines.push(current_line);
    } else {
        lines.push(title.to_string());
    }

    let mut result = String::new();
    let mut y = 38;
    for line in lines {
        result.push_str(&format!(
            r##"<text x="24" y="{y}" class="main-title_{id}">{line}</text>"##,
            y = y, id = id, line = escape(&line)
        ));
        y += 44;
    }

    if !subtitle.is_empty() {
        result.push_str(&format!(
            r##"<text x="24" y="{y}" class="main-title_{id}">{subtitle}</text>"##,
            y = y, id = id, subtitle = escape(subtitle)
        ));
    }

    result
}

fn render_card(card: &Card, id: &str, is_before: bool, width: i32, height: i32, anim_delay: &str) -> String {
    let x_offset = if is_before { 0 } else { 528 }; // 464 + 64 gap
    let header_color_var = if is_before { "var(--header-before-bg)" } else { "var(--header-after-bg)" };
    let bullet_color_var = if is_before { "var(--bullet-color)" } else { "var(--check-color)" };
    let card_id = if is_before { "true" } else { "false" };
    
    let mut svg = format!(
        r##"      <g transform="translate({x_offset}, 0.0)">
        <g filter="url(#premiumShadow_{id})">
          <g class="anim-panel_{id} {anim_delay}_{id}">
            <linearGradient id="cardGrad_{id}_{card_id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="var(--card-bg-start)"/><stop offset="100%" stop-color="var(--card-bg-end)"/></linearGradient>
            <rect width="{width}" height="{height}" fill="url(#cardGrad_{id}_{card_id})" stroke="none" stroke-width="1.5" rx="8"/>
            <rect width="{width}" height="40" fill="{header_color_var}" fill-opacity="var(--header-text-op)" rx="8"/>
            <text x="32" y="26" class="sec-header_{id}" style="fill: {header_color_var}">{title}</text>
"##,
        x_offset = x_offset, id = id, anim_delay = anim_delay, card_id = card_id,
        width = width, height = height, header_color_var = header_color_var, title = escape(&card.title)
    );

    let mut y = 80;
    for section in &card.sections {
        svg.push_str(&format!(
            r##"            <text x="32" y="{y}" class="sec-header_{id}" style="fill: var(--text-primary)">{sec_title}</text>
"##,
            y = y, id = id, sec_title = escape(&section.title)
        ));
        y += 40;

        for item in &section.items {
            if is_before {
                svg.push_str(&format!(
                    r##"            <circle cx="40" cy="{y_bullet}" r="3" fill="{bullet_color_var}"/>
"##,
                    y_bullet = y - 4, bullet_color_var = bullet_color_var
                ));
            } else {
                svg.push_str(&format!(
                    r##"            <path d="M32,{y_check} L38,{y_check2} L48,{y_check3}" stroke="{bullet_color_var}" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
"##,
                    y_check = y - 4, y_check2 = y + 2, y_check3 = y - 8, bullet_color_var = bullet_color_var
                ));
            }

            svg.push_str(&format!(
                r##"            <text x="60" y="{y}" class="item-text_{id}">{item_title}</text>
            <text x="60" y="{y_desc}" class="item-desc_{id}">{item_desc}</text>
"##,
                y = y, id = id, item_title = escape(&item.title),
                y_desc = y + 24, item_desc = escape(&item.description)
            ));
            y += 60;
        }
        y += 20; // Gap between sections
    }

    svg.push_str(&format!(
        r##"            <rect width="{width}" height="{height}" fill="none" stroke="var(--text-primary)" stroke-opacity="var(--card-stroke-op)" stroke-width="1" rx="8"/>
          </g>
        </g>
      </g>
"##,
        width = width, height = height
    ));

    svg
}
