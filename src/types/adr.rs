use std::collections::HashMap;
use crate::common::svg::escape;
use uuid::Uuid;

struct Participant {
    name: String,
    role: String,
    email: String,
    color: String,
    emoji: Option<String>,
}

struct Reference {
    url: String,
    title: String,
}

struct Adr {
    title: String,
    status: String,
    date: String,
    context: Vec<String>,
    decision: Vec<String>,
    consequences: Vec<String>,
    participants: Vec<Participant>,
    references: Vec<Reference>,
}

enum Fragment {
    Text(String),
    Link { url: String, title: String },
}

fn parse_fragments(input: &str) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let mut current_pos = 0;

    while let Some(start_idx) = input[current_pos..].find("[[") {
        let absolute_start = current_pos + start_idx;
        if absolute_start > current_pos {
            fragments.push(Fragment::Text(input[current_pos..absolute_start].to_string()));
        }

        if let Some(end_idx) = input[absolute_start..].find("]]") {
            let absolute_end = absolute_start + end_idx + 2;
            let inner = &input[absolute_start + 2..absolute_start + end_idx];
            
            if let Some((url, title)) = inner.trim().split_once(' ') {
                fragments.push(Fragment::Link {
                    url: url.trim().to_string(),
                    title: title.trim().to_string(),
                });
            } else {
                fragments.push(Fragment::Link {
                    url: inner.trim().to_string(),
                    title: inner.trim().to_string(),
                });
            }
            current_pos = absolute_end;
        } else {
            // Unclosed [[
            break;
        }
    }

    if current_pos < input.len() {
        fragments.push(Fragment::Text(input[current_pos..].to_string()));
    }

    fragments
}

fn url_encode(s: &str) -> String {
    let mut res = String::new();
    for b in s.as_bytes() {
        match *b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => res.push(*b as char),
            b' ' => res.push('+'),
            _ => res.push_str(&format!("%{:02X}", b)),
        }
    }
    res
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let adr = parse_adr(body)?;
    Ok(render_svg(&adr, controls))
}

fn parse_adr(body: &str) -> Result<Adr, String> {
    let trimmed = body.trim();
    if !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("ADR body must be wrapped in '---- ... ----'".into());
    }
    let inner = trimmed[4..trimmed.len() - 4].trim();

    let mut adr = Adr {
        title: "Untitled ADR".to_string(),
        status: "Proposed".to_string(),
        date: "".to_string(),
        context: Vec::new(),
        decision: Vec::new(),
        consequences: Vec::new(),
        participants: Vec::new(),
        references: Vec::new(),
    };

    let mut current_key = String::new();

    for line in inner.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            current_key = key.trim().to_string();
            let val = value.trim();
            match current_key.as_str() {
                "title" => adr.title = val.to_string(),
                "status" => adr.status = val.to_string(),
                "date" => adr.date = val.to_string(),
                "context" | "decision" | "consequences" | "participants" | "references" => {
                    if !val.is_empty() {
                        add_value(&mut adr, &current_key, val);
                    }
                }
                _ => {}
            }
        } else if line.starts_with('-') && !current_key.is_empty() {
            let val = line[1..].trim();
            add_value(&mut adr, &current_key, val);
        } else if !current_key.is_empty() {
            add_value(&mut adr, &current_key, line);
        }
    }

    Ok(adr)
}

fn add_value(adr: &mut Adr, key: &str, val: &str) {
    match key {
        "context" => adr.context.push(val.to_string()),
        "decision" => adr.decision.push(val.to_string()),
        "consequences" => adr.consequences.push(val.to_string()),
        "participants" => {
            if val.contains('|') {
                let parts: Vec<&str> = val.split('|').map(|s| s.trim()).collect();
                if parts[0].to_lowercase() == "name" {
                    return;
                }
                if parts.len() >= 4 {
                    adr.participants.push(Participant {
                        name: parts[0].to_string(),
                        role: parts[1].to_string(),
                        email: parts[2].to_string(),
                        color: parts[3].to_string(),
                        emoji: if parts.len() > 4 && !parts[4].is_empty() { Some(parts[4].to_string()) } else { None },
                    });
                }
            } else {
                for part in val.split(',') {
                    let part = part.trim();
                    if part.is_empty() { continue; }
                    
                    if let Some((name, role_with_parens)) = part.split_once('(') {
                        let role = role_with_parens.trim_end_matches(')').trim();
                        adr.participants.push(Participant {
                            name: name.trim().to_string(),
                            role: role.to_string(),
                            email: "".to_string(),
                            color: "#6366F1".to_string(),
                            emoji: None,
                        });
                    } else {
                        adr.participants.push(Participant {
                            name: part.to_string(),
                            role: "".to_string(),
                            email: "".to_string(),
                            color: "#6366F1".to_string(),
                            emoji: None,
                        });
                    }
                }
            }
        }
        "references" => {
            if val.starts_with("[[") && val.ends_with("]]") {
                let inner = val[2..val.len() - 2].trim();
                if let Some((url, title)) = inner.split_once(' ') {
                    adr.references.push(Reference {
                        url: url.trim().to_string(),
                        title: title.trim().to_string(),
                    });
                } else {
                    adr.references.push(Reference {
                        url: inner.to_string(),
                        title: inner.to_string(),
                    });
                }
            } else {
                 adr.references.push(Reference {
                    url: val.to_string(),
                    title: val.to_string(),
                });
            }
        }
        _ => {}
    }
}

fn render_svg(adr: &Adr, controls: &HashMap<String, String>) -> String {
    let use_dark = controls.get("useDark").map(|s| s == "true").unwrap_or(false);
    let id = Uuid::new_v4().simple().to_string()[..8].to_string();
    let id_full = format!("adr_{}", id);
    let status_color = match adr.status.to_lowercase().as_str() {
        "proposed" => "#6366F1",
        "accepted" | "approved" | "completed" => "#10B981",
        "superseded" => "#F59E0B",
        "deprecated" => "#EF4444",
        "rejected" => "#DC2626",
        _ => "#6366F1", // Proposed, Draft, etc.
    };

    let mut y = 280.0;
    let mut sections_svg = String::new();

    // Context
    if !adr.context.is_empty() {
        sections_svg.push_str(&render_section("CONTEXT", &adr.context, status_color, y, &id));
        y += (adr.context.len() as f64 * 30.0 + 102.0).max(120.0);
    }

    // Decision
    if !adr.decision.is_empty() {
        sections_svg.push_str(&render_section("DECISION", &adr.decision, status_color, y, &id));
        y += (adr.decision.len() as f64 * 30.0 + 102.0).max(120.0);
    }

    // Consequences
    if !adr.consequences.is_empty() {
        sections_svg.push_str(&render_section("CONSEQUENCES", &adr.consequences, status_color, y, &id));
        y += (adr.consequences.len() as f64 * 30.0 + 102.0).max(120.0);
    }

    // Participants
    if !adr.participants.is_empty() {
        sections_svg.push_str(&render_participants(&adr.participants, status_color, y, &id, &adr.title));
        y += (adr.participants.len() as f64 * 80.0 + 100.0).max(160.0);
    }

    // References
    if !adr.references.is_empty() {
        sections_svg.push_str(&render_references(&adr.references, status_color, y, &id));
        y += (adr.references.len() as f64 * 40.0 + 100.0).max(140.0);
    }

    let total_height = y + 60.0;
    let extra_class = if use_dark { " dark-mode" } else { "" };

    format!(
        r##"<svg width="900" height="{total_height}" viewBox="0 0 900 {total_height}" xmlns="http://www.w3.org/2000/svg" id="{id_full}" class="adr-container{extra_class}">
    <defs>
        <style>@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&amp;display=swap');
            .apple-text {{ font-family: 'SF Pro Display', 'Inter', system-ui, -apple-system, sans-serif; }}
            .apple-mono {{ font-family: 'SF Mono', 'JetBrains Mono', monospace; }}
            .section-card {{ transition: transform 0.3s ease; }}
            .section-card:hover {{ transform: translateY(-2px); }}
            .link:hover {{ text-decoration: underline; }}
            .participant-node {{ transition: all 0.2s ease; }}
            .participant-node:hover {{ opacity: 0.7; }}
            .chat-btn:hover {{ filter: brightness(1.1); }}
            
            #{id_full} {{
                --apple-bg-start: #FFFFFF;
                --apple-bg-end: #F2F2F7;
                --apple-card-bg: #FFFFFF;
                --apple-text-primary: #000000;
                --apple-text-secondary: #8E8E93;
                --apple-text-item: #1C1C1E;
                --apple-line: rgba(0,0,0,0.1);
                --apple-link: #007AFF;
                --apple-shadow-opacity: 0.08;
            }}

            @media (prefers-color-scheme: dark) {{
                #{id_full} {{
                    --apple-bg-start: #1C1C1E;
                    --apple-bg-end: #000000;
                    --apple-card-bg: #2C2C2E;
                    --apple-text-primary: #FFFFFF;
                    --apple-text-secondary: #8E8E93;
                    --apple-text-item: #F2F2F7;
                    --apple-line: rgba(255,255,255,0.1);
                    --apple-link: #0A84FF;
                    --apple-shadow-opacity: 0.3;
                }}
            }}

            #{id_full}.dark-mode {{
                --apple-bg-start: #1C1C1E;
                --apple-bg-end: #000000;
                --apple-card-bg: #2C2C2E;
                --apple-text-primary: #FFFFFF;
                --apple-text-secondary: #8E8E93;
                --apple-text-item: #F2F2F7;
                --apple-line: rgba(255,255,255,0.1);
                --apple-link: #0A84FF;
                --apple-shadow-opacity: 0.3;
            }}
        </style>
        <filter id="appleShadow_{id}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="8" stdDeviation="12" flood-color="#000000" flood-opacity="var(--apple-shadow-opacity)"></feDropShadow>
        </filter>
        <linearGradient id="appleBg_{id}" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--apple-bg-start)"></stop>
            <stop offset="100%" stop-color="var(--apple-bg-end)"></stop>
        </linearGradient>
    </defs>
    <rect width="900" height="{total_height}" fill="url(#appleBg_{id})"></rect>
    <g transform="translate(48, 60)">
        <text x="0" y="0" class="apple-text" font-size="14" font-weight="600" fill="{status_color}" letter-spacing="0.05em">{status}</text>
        <text x="0" y="48" class="apple-text" font-size="44" font-weight="700" fill="var(--apple-text-primary)" letter-spacing="-0.02em">{title}</text>
        <text x="0" y="90" class="apple-mono" font-size="14" font-weight="500" fill="var(--apple-text-secondary)">{date} • ADR-{id_upper}</text>
        <line x1="0" y1="130" x2="800" y2="130" stroke="var(--apple-line)" stroke-width="1"></line>
    </g>
    {sections_svg}
</svg>"##,
        total_height = total_height,
        id = id,
        id_full = id_full,
        extra_class = extra_class,
        status = escape(&adr.status.to_uppercase()),
        status_color = status_color,
        title = escape(&adr.title),
        date = escape(&adr.date),
        id_upper = id.to_uppercase(),
        sections_svg = sections_svg
    )
}

fn render_section(title: &str, items: &[String], color: &str, y: f64, id: &str) -> String {
    let height = items.len() as f64 * 30.0 + 70.0;
    let mut items_svg = String::new();
    for (i, item) in items.iter().enumerate() {
        let fragments = parse_fragments(item);
        let mut line_content = String::new();
        for frag in fragments {
            match frag {
                Fragment::Text(t) => line_content.push_str(&escape(&t)),
                Fragment::Link { url, title } => {
                    line_content.push_str(&format!(
                        r##"<a href="{url}" target="_blank" class="link"><tspan fill="var(--apple-link)">{title}</tspan></a>"##,
                        url = escape(&url),
                        title = escape(&title)
                    ));
                }
            }
        }
        items_svg.push_str(&format!(
            r##"<text x="32" y="{y}" class="apple-text" font-size="17" font-weight="400" fill="var(--apple-text-item)">- {line_content}</text>"##,
            y = 64.0 + i as f64 * 30.0,
            line_content = line_content
        ));
    }

    format!(
        r##"<g transform="translate(48, {y})">
        <rect width="800" height="{height}" rx="28" fill="var(--apple-card-bg)" filter="url(#appleShadow_{id})"></rect>
        <rect width="6" height="{height}" rx="3" fill="{color}" transform="translate(-12, 0)"></rect>
        <text x="32" y="32" class="apple-text" font-size="13" font-weight="600" fill="{color}" letter-spacing="0.05em">{title}</text>
        {items_svg}
    </g>"##,
        y = y,
        height = height,
        id = id,
        color = color,
        title = title,
        items_svg = items_svg
    )
}

fn render_participants(participants: &[Participant], color: &str, y: f64, id: &str, title: &str) -> String {
    let height = participants.len() as f64 * 80.0 + 80.0;
    let mut participants_svg = String::new();
    
    let participant_emails: Vec<&str> = participants.iter()
        .map(|p| p.email.as_str())
        .filter(|e| !e.is_empty())
        .collect();

    let chat_btn = if participant_emails.len() >= 2 {
        let mailto = format!("https://teams.microsoft.com/l/chat/0/0?users={}&amp;topicName={}", 
            participant_emails.join(","), 
            escape(&url_encode(title)));
        format!(r##"<g transform="translate(640, 15)" class="chat-btn">
            <a href="{mailto}" target="_blank" style="text-decoration: none;">
                <rect width="140" height="28" rx="14" fill="#007AFF"></rect>
                <text x="70" y="18" text-anchor="middle" class="apple-text" font-size="11" font-weight="600" fill="#FFFFFF">START GROUP CHAT</text>
            </a>
        </g>"##, mailto = mailto)
    } else {
        "".to_string()
    };

    for (i, p) in participants.iter().enumerate() {
        let initials: String = p.name.split_whitespace()
            .map(|n| n.chars().next().unwrap_or(' '))
            .collect();
        let py = 60.0 + i as f64 * 80.0;
        let avatar_content = if let Some(emoji) = &p.emoji {
            format!(r##"<text x="30" y="42" text-anchor="middle" class="apple-text" font-size="30">{emoji}</text>"##, emoji = escape(emoji))
        } else {
            format!(r##"<text x="30" y="38" text-anchor="middle" class="apple-text" font-size="20" font-weight="700" fill="{p_color}">{initials}</text>"##,
                p_color = escape(&p.color),
                initials = escape(&initials))
        };

        participants_svg.push_str(&format!(
            r##"<g transform="translate(32, {py})">
            <g class="participant-node">
                <circle cx="30" cy="30" r="30" fill="{p_color}" fill-opacity="0.1"></circle>
                {avatar_content}
                <text x="80" y="24" class="apple-text" font-size="16" font-weight="600" fill="var(--apple-text-item)">{name}</text>
                <text x="80" y="44" class="apple-text" font-size="13" font-weight="400" fill="var(--apple-text-secondary)">{role}</text>
            </g>
        </g>"##,
            py = py,
            p_color = escape(&p.color),
            avatar_content = avatar_content,
            name = escape(&p.name),
            role = escape(&p.role)
        ));
    }

    format!(
        r##"<g transform="translate(48, {y})">
        <rect width="800" height="{height}" rx="28" fill="var(--apple-card-bg)" filter="url(#appleShadow_{id})"></rect>
        <text x="32" y="32" class="apple-text" font-size="13" font-weight="600" fill="{color}" letter-spacing="0.05em">PARTICIPANTS</text>
        {chat_btn}
        {participants_svg}
    </g>"##,
        y = y,
        height = height,
        id = id,
        color = color,
        chat_btn = chat_btn,
        participants_svg = participants_svg
    )
}

fn render_references(references: &[Reference], color: &str, y: f64, id: &str) -> String {
    let height = references.len() as f64 * 40.0 + 70.0;
    let mut refs_svg = String::new();
    for (i, r) in references.iter().enumerate() {
        let ry = 64.0 + i as f64 * 40.0;
        refs_svg.push_str(&format!(
            r##"<a href="{url}" target="_blank" class="link">
            <text x="32" y="{ry}" class="apple-text" font-size="15" fill="var(--apple-link)">{title}</text>
        </a>"##,
            ry = ry,
            url = escape(&r.url),
            title = escape(&r.title)
        ));
    }

    format!(
        r##"<g transform="translate(48, {y})">
        <rect width="800" height="{height}" rx="28" fill="var(--apple-card-bg)" filter="url(#appleShadow_{id})"></rect>
        <text x="32" y="32" class="apple-text" font-size="13" font-weight="600" fill="{color}" letter-spacing="0.05em">REFERENCES</text>
        {refs_svg}
    </g>"##,
        y = y,
        height = height,
        id = id,
        color = color,
        refs_svg = refs_svg
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_adr() {
        let body = r##"----
title= Adopt GraphQL for API Layer
status= Superseded
date= 2024-07-15
context=
- Our REST APIs have become complex
decision=
- We will adopt GraphQL
participants=
Alex Rivera | API Architect | alex@example.com | #4F46E5 | 👨‍💻
references=
[[https://graphql.org/ GraphQL]]
----"##;
        let adr = parse_adr(body).unwrap();
        assert_eq!(adr.title, "Adopt GraphQL for API Layer");
        assert_eq!(adr.status, "Superseded");
        assert_eq!(adr.context.len(), 1);
        assert_eq!(adr.participants.len(), 1);
        assert_eq!(adr.participants[0].emoji, Some("👨‍💻".to_string()));
        assert_eq!(adr.references.len(), 1);
    }

    #[test]
    fn test_participant_formats() {
        let body = r##"----
title= Participant Test
status= Accepted
participants= Jane Smith (Architect), John Doe (Developer)
participants=
Name | Title | email | #color | emoji
Bob Wilson | Lead | bob@example.com | #6366F1 | 🚀
----"##;
        let adr = parse_adr(body).unwrap();
        assert_eq!(adr.participants.len(), 3);
        
        assert_eq!(adr.participants[0].name, "Jane Smith");
        assert_eq!(adr.participants[0].role, "Architect");
        
        assert_eq!(adr.participants[1].name, "John Doe");
        assert_eq!(adr.participants[1].role, "Developer");
        
        assert_eq!(adr.participants[2].name, "Bob Wilson");
        assert_eq!(adr.participants[2].emoji, Some("🚀".to_string()));
    }

    #[test]
    fn test_render_adr_with_inline_links() {
        let body = r##"----
title= Link Test
status= Accepted
decision=
- Use [[https://rust-lang.org Rust]] for safety
- Multiple [[https://a.com A]] and [[https://b.com B]] links
----"##;
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("https://rust-lang.org"), "SVG should contain the link URL");
        assert!(svg.contains("Rust"), "SVG should contain the link text");
        assert!(svg.contains("https://a.com"));
        assert!(svg.contains("https://b.com"));
        assert!(svg.contains("<a"), "SVG should contain an anchor tag for the link");
        assert!(svg.contains("#10B981"), "SVG should contain Accepted status color");
    }

    #[test]
    fn test_status_colors() {
        let statuses = vec![
            ("Proposed", "#6366F1"),
            ("Accepted", "#10B981"),
            ("Superseded", "#F59E0B"),
            ("Deprecated", "#EF4444"),
            ("Rejected", "#DC2626"),
        ];
        
        for (status, color) in statuses {
            let body = format!("----\ntitle= Test\nstatus= {}\n----", status);
            let svg = render(&body, &HashMap::new()).unwrap();
            assert!(svg.contains(color), "SVG for status {} should contain color {}", status, color);
        }
    }

    #[test]
    fn test_group_chat_button_visibility() {
        // Case 1: No participants with emails -> No button
        let body1 = r##"----
title= No Emails
participants= Jane Smith (Architect), John Doe (Developer)
----"##;
        let svg1 = render(body1, &HashMap::new()).unwrap();
        assert!(!svg1.contains("START GROUP CHAT"), "Should not show button with no emails");

        // Case 2: Only one participant with email -> No button
        let body2 = r##"----
title= One Email
participants=
Name | Title | email | #color
Alex | Dev | alex@example.com | #000
----"##;
        let svg2 = render(body2, &HashMap::new()).unwrap();
        assert!(!svg2.contains("START GROUP CHAT"), "Should not show button with only one email");

        // Case 3: Two participants with emails -> Button should appear
        let body3 = r##"----
title= Two Emails
participants=
Name | Title | email | #color
Alex | Dev | alex@example.com | #000
Bob | Dev | bob@example.com | #000
----"##;
        let svg3 = render(body3, &HashMap::new()).unwrap();
        assert!(svg3.contains("START GROUP CHAT"), "Should show button with two emails");
        assert!(svg3.contains("alex@example.com,bob@example.com"), "URL should contain both emails");
    }

    #[test]
    fn test_dark_mode_support() {
        let body = "----\ntitle= Dark Mode Test\nstatus= Accepted\n----";
        
        // Default mode (should have variables and media query)
        let svg_light = render(body, &HashMap::new()).unwrap();
        assert!(svg_light.contains("--apple-bg-start: #FFFFFF"));
        assert!(svg_light.contains("@media (prefers-color-scheme: dark)"));
        assert!(svg_light.contains("class=\"adr-container\""));
        
        // Forced dark mode
        let mut controls = HashMap::new();
        controls.insert("useDark".to_string(), "true".to_string());
        let svg_dark = render(body, &controls).unwrap();
        assert!(svg_dark.contains("class=\"adr-container dark-mode\""));
    }
}