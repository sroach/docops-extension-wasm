use crate::common::svg::escape;
use std::collections::HashMap;

/// Grammar: `[docops,badge] ---- Label|Message|Style|LabelColor|MessageColor|Icon|FontColor ----`
///
/// This is intentionally NOT the key=value grammar bar/pie charts use.
/// Badges are shields.io-style: one line, fixed field positions separated
/// by '|'. Empty fields fall back to sensible defaults, so
/// `Made With|Kotlin||#06133b|#6fc441|<kotlin>|#fcfcfc` works even though
/// field 2 (style) is blank.
struct Badge {
    label: String,
    message: String,
    // field index 2 ("style": flat/plastic/etc) is parsed but not yet used
    // by the renderer below — hook it up here when you add style variants.
    label_color: String,
    message_color: String,
    // field index 5 ("icon", e.g. "<kotlin>") is parsed but not yet
    // rendered — wire up an icon registry (name -> embedded SVG path data)
    // and splice it in next to the label text when you're ready.
    font_color: String,
}

pub fn render(body: &str, _controls: &HashMap<String, String>) -> Result<String, String> {
    let trimmed = body.trim();
    if trimmed.len() < 8 || !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("badge body must be wrapped in '---- ... ----'".into());
    }
    let inner = trimmed[4..trimmed.len() - 4].trim();
    let fields: Vec<&str> = inner.split('|').map(str::trim).collect();

    let get = |i: usize, default: &str| -> String {
        fields
            .get(i)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default.to_string())
    };

    let badge = Badge {
        label: get(0, "label"),
        message: get(1, "message"),
        label_color: get(3, "#555555"),
        message_color: get(4, "#4c1"),
        font_color: get(6, "#ffffff"),
    };

    Ok(render_svg(&badge))
}

fn text_width(s: &str) -> f64 {
    // Rough monospace-ish estimate (px per char at 11px Verdana). Good
    // enough for a badge; swap for real font metrics if you need pixel
    // accuracy — wasm has no access to system font metrics on its own.
    s.chars().count() as f64 * 6.5
}

fn render_svg(b: &Badge) -> String {
    let label_w = text_width(&b.label) + 20.0;
    let message_w = text_width(&b.message) + 20.0;
    let total_w = label_w + message_w;
    let height = 20.0;

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_w:.0}" height="{height:.0}" viewBox="0 0 {total_w:.0} {height:.0}">
  <rect width="{label_w:.1}" height="{height:.0}" fill="{label_color}"/>
  <rect x="{label_w:.1}" width="{message_w:.1}" height="{height:.0}" fill="{message_color}"/>
  <text x="{label_cx:.1}" y="14" text-anchor="middle" font-size="11" font-family="Verdana,sans-serif" fill="{font_color}">{label}</text>
  <text x="{message_cx:.1}" y="14" text-anchor="middle" font-size="11" font-family="Verdana,sans-serif" fill="{font_color}">{message}</text>
</svg>"##,
        total_w = total_w,
        height = height,
        label_w = label_w,
        message_w = message_w,
        label_color = b.label_color,
        message_color = b.message_color,
        label_cx = label_w / 2.0,
        message_cx = label_w + message_w / 2.0,
        font_color = b.font_color,
        label = escape(&b.label),
        message = escape(&b.message),
    )
}