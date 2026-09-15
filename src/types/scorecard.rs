use crate::common::kv::parse_kv_header;
use crate::common::svg::escape;
use std::collections::HashMap;
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
    let use_dark = controls
        .get("useDark")
        .map(|s| s == "true")
        .unwrap_or(false)
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
                scorecard.before.sections =
                    parse_sections(items_part.strip_prefix("[before.items]").unwrap());
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
                scorecard.after.sections =
                    parse_sections(items_part.strip_prefix("[after.items]").unwrap());
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
    let title_esc = escape(&scorecard.title);
    let desc_text = if !scorecard.subtitle.is_empty() {
        format!("{} - {}", scorecard.title, scorecard.subtitle)
    } else {
        format!(
            "Scorecard comparison between {} and {}",
            scorecard.before.title, scorecard.after.title
        )
    };
    let desc_esc = escape(&desc_text);

    let mut svg = format!(
        r##"<svg width="{width}" height="{total_height}" viewBox="0 0 {width} {total_height}" xmlns="http://www.w3.org/2000/svg" class="scorecard-container{extra_class}" id="{id}" role="graphics-document document" aria-labelledby="{id}_title {id}_desc">
      <title id="{id}_title">{title_esc}</title>
      <desc id="{id}_desc">{desc_esc}</desc>
      <metadata><rdf:rdf xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:cc="http://creativecommons.org/ns#"><cc:work rdf:about=""><dc:creator>DocOps.io</dc:creator><dc:rights>MIT License</dc:rights><dc:source>https://docops.io</dc:source><dc:date>2026-08-30</dc:date></cc:work></rdf:rdf></metadata>
    <defs>
        <radialGradient id="ambientBlue_{id}" cx="15%" cy="10%" r="55%">
            <stop offset="0%" stop-color="#BFDBFE" stop-opacity="0.45"/>
            <stop offset="60%" stop-color="#DBEAFE" stop-opacity="0.12"/>
            <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="ambientViolet_{id}" cx="85%" cy="12%" r="50%">
            <stop offset="0%" stop-color="#DDD6FE" stop-opacity="0.35"/>
            <stop offset="60%" stop-color="#EDE9FE" stop-opacity="0.10"/>
            <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0"/>
        </radialGradient>
        <filter id="premiumShadow_{id}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="12" stdDeviation="16" flood-color="var(--shadow-flood)" flood-opacity="var(--shadow-op)"/>
            <feDropShadow dx="0" dy="2" stdDeviation="4" flood-color="var(--shadow-flood)" flood-opacity="var(--shadow-op-2)"/>
        </filter>
        <style>
            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&amp;family=JetBrains+Mono:wght@400;500;600&amp;display=swap'); 
            #{id} {{
                --bg: #FFFFFF;
                --text-primary: #111827;
                --text-secondary: #4B5563;
                --text-tertiary: #6B7280;
                --card-bg-start: rgba(255, 255, 255, 0.95);
                --card-bg-end: rgba(248, 250, 252, 0.90);
                --card-stroke: rgba(229, 231, 235, 0.85);
                --card-inner-glow: rgba(255, 255, 255, 0.80);
                --ambient-op: 1.0;
                --shadow-flood: #0F172A;
                --shadow-op: 0.08;
                --shadow-op-2: 0.04;
                --header-before-bg: #64748b;
                --header-after-bg: #3B82F6;
                --header-text-op: 0.08;
                --bullet-color: #64748b;
                --check-color: #3B82F6;
                --arrow-color: #3B82F6;
            }}
            @media (prefers-color-scheme: dark) {{
                #{id} {{
                    --bg: #0b1220;
                    --text-primary: #f8fafc;
                    --text-secondary: #94a3b8;
                    --text-tertiary: #64748b;
                    --card-bg-start: rgba(23, 32, 47, 0.85);
                    --card-bg-end: rgba(17, 26, 40, 0.90);
                    --card-stroke: rgba(51, 65, 85, 0.70);
                    --card-inner-glow: rgba(255, 255, 255, 0.05);
                    --ambient-op: 0.25;
                    --shadow-flood: #000000;
                    --shadow-op: 0.40;
                    --shadow-op-2: 0.20;
                    --header-before-bg: #475569;
                    --header-after-bg: #60A5FA;
                    --bullet-color: #94a3b8;
                    --check-color: #60A5FA;
                    --arrow-color: #60A5FA;
                }}
            }}
            #{id}.dark-mode {{
                --bg: #0b1220;
                --text-primary: #f8fafc;
                --text-secondary: #94a3b8;
                --text-tertiary: #64748b;
                --card-bg-start: rgba(23, 32, 47, 0.85);
                --card-bg-end: rgba(17, 26, 40, 0.90);
                --card-stroke: rgba(51, 65, 85, 0.70);
                --card-inner-glow: rgba(255, 255, 255, 0.05);
                --ambient-op: 0.25;
                --shadow-flood: #000000;
                --shadow-op: 0.40;
                --shadow-op-2: 0.20;
                --header-before-bg: #475569;
                --header-after-bg: #60A5FA;
                --bullet-color: #94a3b8;
                --check-color: #60A5FA;
                --arrow-color: #60A5FA;
            }}
            .main-title_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; font-size: 30px; fill: var(--text-primary); text-transform: none; letter-spacing: -0.025em; font-weight: 700; }}
            .main-subtitle_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; font-size: 16px; fill: var(--text-secondary); text-transform: none; font-weight: 400; }}
            .sec-header_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; font-size: 16px; letter-spacing: 0px; text-transform: none; font-weight: 600; }} 
            .item-text_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; font-size: 14px; fill: var(--text-primary); font-weight: 600; }} 
            .item-desc_{id} {{ font-family: 'Inter', ui-sans-serif, system-ui, sans-serif; font-size: 12px; fill: var(--text-secondary); font-weight: 400; }} 
            .meta-text_{id} {{ font-family: 'JetBrains Mono', ui-monospace, monospace; font-size: 12px; fill: var(--text-tertiary); }} 
            @keyframes slideUp_{id} {{ from {{ opacity: 0; transform: translateY(20px); }} to {{ opacity: 1; transform: translateY(0); }} }} 
            .anim-panel_{id} {{ animation: slideUp_{id} 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards; opacity: 1; }} 
            .delay-1_{id} {{ animation-delay: 0.1s; }} 
            .delay-2_{id} {{ animation-delay: 0.25s; }}
            @media (prefers-reduced-motion: reduce) {{
                .anim-panel_{id} {{ animation: none; opacity: 1; transform: none; }}
            }}
        </style>
    </defs>
    <rect width="100%" height="100%" fill="var(--bg)" aria-hidden="true"/>
    <rect width="100%" height="100%" fill="url(#ambientBlue_{id})" opacity="var(--ambient-op)" aria-hidden="true"/>
    <rect width="100%" height="100%" fill="url(#ambientViolet_{id})" opacity="var(--ambient-op)" aria-hidden="true"/>
"##,
        width = width,
        total_height = total_height,
        id = id,
        title_esc = title_esc,
        desc_esc = desc_esc,
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
    svg.push_str(&format!(
        r##"    <g transform="translate(32, {card_top})">"##,
        card_top = card_top
    ));

    // BEFORE Card
    svg.push_str(&render_card(
        &scorecard.before,
        &id,
        true,
        464,
        max_card_height,
        "delay-1",
    ));

    // AFTER Card
    svg.push_str(&render_card(
        &scorecard.after,
        &id,
        false,
        464,
        max_card_height,
        "delay-2",
    ));

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
        r##"    <text x="32" y="{meta_y}" class="meta-text_{id}">SCORECARD_REF: A203 · THEME: Premium · SCALE: 1.0</text>
"##,
        meta_y = total_height - 20,
        id = id
    ));

    svg.push_str("</svg>");
    svg
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
            y = y,
            id = id,
            line = escape(&line)
        ));
        y += 44;
    }

    if !subtitle.is_empty() {
        result.push_str(&format!(
            r##"<text x="24" y="{y}" class="main-subtitle_{id}">{subtitle}</text>"##,
            y = y,
            id = id,
            subtitle = escape(subtitle)
        ));
    }

    result
}

fn render_card(
    card: &Card,
    id: &str,
    is_before: bool,
    width: i32,
    height: i32,
    anim_delay: &str,
) -> String {
    let x_offset = if is_before { 0 } else { 528 }; // 464 + 64 gap
    let header_color_var = if is_before {
        "var(--header-before-bg)"
    } else {
        "var(--header-after-bg)"
    };
    let bullet_color_var = if is_before {
        "var(--bullet-color)"
    } else {
        "var(--check-color)"
    };
    let card_id = if is_before { "true" } else { "false" };
    let card_title = escape(&card.title);

    let mut svg = format!(
        r##"      <g transform="translate({x_offset}, 0.0)" role="region" aria-label="{title}">
        <g filter="url(#premiumShadow_{id})">
          <g class="anim-panel_{id} {anim_delay}_{id}">
            <linearGradient id="cardGrad_{id}_{card_id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="var(--card-bg-start)"/><stop offset="100%" stop-color="var(--card-bg-end)"/></linearGradient>
            <rect width="{width}" height="{height}" fill="url(#cardGrad_{id}_{card_id})" stroke="var(--card-stroke)" stroke-width="1" rx="12" aria-hidden="true"/>
            <rect x="0.5" y="0.5" width="{inner_w}" height="{inner_h}" rx="11.5" fill="none" stroke="var(--card-inner-glow)" stroke-width="1" aria-hidden="true"/>
            <rect width="{width}" height="40" fill="{header_color_var}" fill-opacity="var(--header-text-op)" rx="12" aria-hidden="true"/>
            <rect y="28" width="{width}" height="12" fill="{header_color_var}" fill-opacity="var(--header-text-op)" aria-hidden="true"/>
            <text x="32" y="26" class="sec-header_{id}" style="fill: {header_color_var}">{title}</text>
"##,
        x_offset = x_offset,
        id = id,
        anim_delay = anim_delay,
        card_id = card_id,
        width = width,
        height = height,
        inner_w = width - 1,
        inner_h = height - 1,
        header_color_var = header_color_var,
        title = card_title
    );

    let mut y = 80;
    for section in &card.sections {
        let sec_title_esc = escape(&section.title);
        svg.push_str(&format!(
            r##"            <g role="list" aria-label="{sec_title}">
            <text x="32" y="{y}" class="sec-header_{id}" style="fill: var(--text-primary)">{sec_title}</text>
"##,
            y = y, id = id, sec_title = sec_title_esc
        ));
        y += 40;

        for item in &section.items {
            let item_title = escape(&item.title);
            let item_desc = escape(&item.description);

            svg.push_str(&format!(
                r##"            <g role="listitem" aria-label="{item_title}: {item_desc}" tabindex="0">"##,
                item_title = item_title,
                item_desc = item_desc
            ));

            if is_before {
                svg.push_str(&format!(
                    r##"            <circle cx="40" cy="{y_bullet}" r="3" fill="{bullet_color_var}" aria-hidden="true"/>
"##,
                    y_bullet = y - 4, bullet_color_var = bullet_color_var
                ));
            } else {
                svg.push_str(&format!(
                    r##"            <path d="M32,{y_check} L38,{y_check2} L48,{y_check3}" stroke="{bullet_color_var}" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"/>
"##,
                    y_check = y - 4, y_check2 = y + 2, y_check3 = y - 8, bullet_color_var = bullet_color_var
                ));
            }

            svg.push_str(&format!(
                r##"            <text x="60" y="{y}" class="item-text_{id}">{item_title}</text>
            <text x="60" y="{y_desc}" class="item-desc_{id}">{item_desc}</text>
            </g>
"##,
                y = y,
                id = id,
                item_title = item_title,
                y_desc = y + 24,
                item_desc = item_desc
            ));
            y += 60;
        }
        svg.push_str("            </g>\n");
        y += 20; // Gap between sections
    }

    svg.push_str(
        r##"          </g>
        </g>
      </g>
"##,
    );

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
        assert_eq!(
            scorecard.title,
            "Software Release v2.4.0 - Feature & Bug Summary"
        );
        assert_eq!(
            scorecard.subtitle,
            "Migration from Legacy System to Modern Architecture"
        );
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
        assert_eq!(
            scorecard.title,
            "Software Release v2.4.0 - Feature & Bug Summary"
        );
        assert_eq!(
            scorecard.subtitle,
            "Migration from Legacy System to Modern Architecture"
        );
        assert_eq!(scorecard.before.title, "BEFORE v2.4.0");
        assert_eq!(scorecard.before.sections.len(), 2);
        assert_eq!(scorecard.before.sections[0].title, "Feature Status");
        assert_eq!(scorecard.before.sections[0].items.len(), 2);
        assert_eq!(
            scorecard.before.sections[0].items[0].title,
            "Dark Mode Theme"
        );

        assert_eq!(scorecard.after.title, "AFTER v2.4.0");
        assert_eq!(scorecard.after.sections.len(), 2);
        assert_eq!(scorecard.after.sections[1].title, "Bugs Resolved");
        assert_eq!(
            scorecard.after.sections[1].items[0].title,
            "Login timeout issues"
        );
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
