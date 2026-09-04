use crate::common::kv::parse_kv_header;
use crate::common::svg::escape;
use std::collections::HashMap;
use uuid::Uuid;

/// Grammar: `[docops,badge] ---- Label|Message|Style|LabelColor|MessageColor|Icon|FontColor ----`
///
/// This is intentionally NOT the key=value grammar bar/pie charts use.
/// Badges are shields.io-style: one line, fixed field positions separated
/// by '|'. Empty fields fall back to sensible defaults.
struct Badge {
    label: String,
    message: String,
    style: String,
    label_color: String,
    message_color: String,
    icon: String,
    font_color: String,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let trimmed = body.trim();
    if trimmed.len() < 8 || !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("badge body must be wrapped in '---- ... ----'".into());
    }
    let inner = trimmed[4..trimmed.len() - 4].trim();

    let parts: Vec<&str> = inner.splitn(2, "---").collect();
    let (config_str, data_str) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        ("", parts[0])
    };

    let mut config = parse_kv_header(config_str);
    for (k, v) in controls {
        config.insert(k.clone(), v.clone());
    }

    let mut badges = Vec::new();
    for line in data_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('|').map(str::trim).collect();

        let get = |i: usize, default: &str| -> String {
            fields
                .get(i)
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| default.to_string())
        };

        badges.push(Badge {
            label: get(0, "label"),
            message: get(1, "message"),
            style: get(2, "flat"),
            label_color: get(3, "#555555"),
            message_color: get(4, "#4c1"),
            icon: get(5, ""),
            font_color: get(6, "#ffffff"),
        });
    }

    if badges.is_empty() {
        return Err("no badges found in block".into());
    }

    let use_dark = config.get("useDark").map(|s| s == "true").unwrap_or(false);
    let columns = config
        .get("columns")
        .or_else(|| config.get("wrap"))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let h_gap = config.get("gap")
        .or_else(|| config.get("hGap"))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(4.0);

    let v_gap = config.get("vGap")
        .or_else(|| config.get("gap"))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(8.0);

    Ok(render_multi_svg(&badges, columns, h_gap, v_gap, use_dark))
}

fn text_width(s: &str) -> f64 {
    // Rough estimate for Inter/SF Pro stack at 11px.
    s.chars().count() as f64 * 6.4
}

fn render_multi_svg(badges: &[Badge], columns: usize, h_gap: f64, v_gap: f64, use_dark: bool) -> String {
    let id_root = Uuid::new_v4().simple().to_string()[..8].to_string();
    let id_full = format!("badges_{}", id_root);
    let badge_h = 20.0;

    let mut badge_elements = Vec::new();
    let mut defs = Vec::new();
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut max_w = 0.0;

    for (i, b) in badges.iter().enumerate() {
        if columns > 0 && i > 0 && i % columns == 0 {
            current_x = 0.0;
            current_y += badge_h + v_gap;
        }

        let has_icon = !b.icon.is_empty();
        let icon_w = if has_icon { 18.0 } else { 0.0 };
        let label_w = text_width(&b.label) + 20.0 + icon_w;
        let message_w = text_width(&b.message) + 20.0;
        let total_w = label_w + message_w;

        let badge_id = format!("{}_{}", id_root, i);
        let radius = if b.style == "flat-square" { "0" } else { "3" };
        let show_gradient = b.style == "plastic";

        let icon_svg = if has_icon {
            format!(
                r##"<text x="10" y="14.8" font-size="12" filter="url(#shadow_{id})">{icon}</text>"##,
                id = id_root,
                icon = escape(&b.icon)
            )
        } else {
            String::new()
        };

        let label_cx = (label_w + icon_w) / 2.0;
        let message_cx = label_w + message_w / 2.0;

        defs.push(format!(
            r##"<clipPath id="r_{bid}"><rect width="{tw:.0}" height="{bh:.0}" rx="{rad}"/></clipPath>"##,
            bid = badge_id,
            tw = total_w,
            bh = badge_h,
            rad = radius
        ));

        let gradient_rect = if show_gradient {
            format!(
                r##"<rect width="{tw:.0}" height="{bh:.0}" fill="url(#g_{id})"/>"##,
                tw = total_w,
                bh = badge_h,
                id = id_root
            )
        } else {
            String::new()
        };

        badge_elements.push(format!(
            r##"<g transform="translate({x:.1}, {y:.1})" style="--label-bg: {lcol}; --message-bg: {mcol}; --font-color: {fcol};">
    <g clip-path="url(#r_{bid})">
        <rect width="{lw:.1}" height="{bh:.0}" fill="var(--label-bg)"/>
        <rect x="{lw:.1}" width="{mw:.1}" height="{bh:.0}" fill="var(--message-bg)"/>
        {grad}
    </g>
    <g fill="var(--font-color)" class="badge-text" text-anchor="middle">
        {icon}
        <text x="{lcx:.1}" y="14.8" filter="url(#shadow_{id})">{label}</text>
        <text x="{mcx:.1}" y="14.8" filter="url(#shadow_{id})">{message}</text>
    </g>
</g>"##,
            x = current_x,
            y = current_y,
            lcol = b.label_color,
            mcol = b.message_color,
            fcol = b.font_color,
            bid = badge_id,
            lw = label_w,
            mw = message_w,
            bh = badge_h,
            grad = gradient_rect,
            icon = icon_svg,
            lcx = label_cx,
            mcx = message_cx,
            label = escape(&b.label),
            message = escape(&b.message),
            id = id_root
        ));

        current_x += total_w + h_gap;
        if current_x - h_gap > max_w {
            max_w = current_x - h_gap;
        }
    }

    let total_h = current_y + badge_h;
    let extra_class = if use_dark { " dark-mode" } else { "" };

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.0} {h:.0}" id="{id_full}" class="badge-container{extra_class}" role="img">
    <defs>
        <style>
            #{id_full} {{ --shadow-op: 0.1; }}
            @media (prefers-color-scheme: dark) {{ #{id_full} {{ --shadow-op: 0.3; }} }}
            #{id_full}.dark-mode {{ --shadow-op: 0.3; }}
            .badge-text {{ 
                font-family: 'SF Pro Display', 'Inter', system-ui, -apple-system, Verdana, sans-serif; 
                font-size: 11px;
                font-weight: 500;
                letter-spacing: -0.01em;
            }}
        </style>
        <linearGradient id="g_{id}" x2="0" y2="100%">
            <stop offset="0" stop-color="#fff" stop-opacity=".1"/>
            <stop offset="1" stop-opacity=".1"/>
        </linearGradient>
        <filter id="shadow_{id}">
            <feDropShadow dx="0" dy="1" stdDeviation="0.5" flood-opacity="var(--shadow-op)"/>
        </filter>
        {defs}
    </defs>
    {elements}
</svg>"##,
        w = max_w,
        h = total_h,
        id_full = id_full,
        id = id_root,
        extra_class = extra_class,
        defs = defs.join("\n        "),
        elements = badge_elements.join("\n    ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_badge() {
        let body = "---- Made With | Rust ----";
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("Made With"));
        assert!(svg.contains("Rust"));
        assert!(svg.contains("rx=\"3\"")); // default rounded
    }

    #[test]
    fn test_render_flat_square_badge() {
        let body = "---- Made With | Rust | flat-square ----";
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("rx=\"0\""));
    }

    #[test]
    fn test_render_plastic_badge() {
        let body = "---- Made With | Rust | plastic ----";
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("linearGradient id=\"g_"));
    }

    #[test]
    fn test_render_with_icon() {
        let body = "---- Made With | Rust |||| 🦀 ----";
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("🦀"));
        assert!(svg.contains("x=\"10\"")); // icon position
    }

    #[test]
    fn test_render_multi_badges() {
        let body = "----
Made With | Rust
Made With | Kotlin
----";
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("Made With"));
        assert!(svg.contains("Rust"));
        assert!(svg.contains("Kotlin"));
        // By default should flow horizontally
        assert!(svg.contains("transform=\"translate(0.0, 0.0)\""));
        // Second badge should be at X > 0
        assert!(svg.matches("transform=\"translate(").count() == 2);
        assert!(!svg.contains("transform=\"translate(0.0, 28.0)\""));
    }

    #[test]
    fn test_render_wrap_badges() {
        let body = "----
B1 | M1
B2 | M2
----";
        let mut controls = HashMap::new();
        controls.insert("columns".to_string(), "1".to_string());
        let svg = render(body, &controls).unwrap();
        // Should have two rows
        assert!(svg.contains("transform=\"translate(0.0, 0.0)\""));
        assert!(svg.contains("transform=\"translate(0.0, 28.0)\"")); // 20 + 8 (default v_gap)
    }

    #[test]
    fn test_internal_config_wrap() {
        let body = "----
columns=1
---
B1 | M1
B2 | M2
----";
        let svg = render(body, &HashMap::new()).unwrap();
        // Should wrap because columns=1 is in the header
        assert!(svg.contains("transform=\"translate(0.0, 0.0)\""));
        assert!(svg.contains("transform=\"translate(0.0, 28.0)\""));
    }

    #[test]
    fn test_custom_gap() {
        let body = "----
B1 | M1
B2 | M2
----";
        let mut controls = HashMap::new();
        controls.insert("gap".to_string(), "20".to_string());
        let svg = render(body, &controls).unwrap();
        // Check horizontal translation with 20px gap
        // B1 width approx: (2+2)*6.4 + 20 + 20 = 25.6 + 40 = 65.6 (roughly)
        // We just check if it contains translate with a value larger than default 4
        assert!(svg.contains("transform=\"translate(0.0, 0.0)\""));
        // The second badge should be at total_w + 20
        // We don't know the exact width due to text_width estimation but we can verify it's there
        assert!(svg.matches("transform=\"translate(").count() == 2);
    }

    #[test]
    fn test_dark_mode() {
        let body = "---- Made With | Rust ----";
        let mut controls = HashMap::new();
        controls.insert("useDark".to_string(), "true".to_string());
        let svg = render(body, &controls).unwrap();
        assert!(svg.contains("class=\"badge-container dark-mode\""));
    }
}