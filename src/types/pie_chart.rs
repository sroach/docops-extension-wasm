use crate::common::kv::parse_kv_body;
use crate::common::svg::{escape, theme};
use std::collections::HashMap;
use std::f64::consts::PI;

/// Grammar: `[docops,pieslice] ---- title=... theme=... --- Label | value ... ----`
/// (identical body syntax to bar_chart — see common::kv). Reusing the same
/// parser is exactly why splitting "parse" from "render" per type pays off:
/// two visualization types, one grammar, zero duplicated parsing code.
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