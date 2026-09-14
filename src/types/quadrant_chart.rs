use crate::common::svg::{escape, theme};
use std::collections::HashMap;
use uuid::Uuid;

struct QuadrantPoint {
    label: String,
    x: f64,
    y: f64,
    category: Option<String>,
}

struct QuadrantChart {
    title: String,
    x_axis: String,
    y_axis: String,
    leaders: String,
    challengers: String,
    visionaries: String,
    niche: String,
    points: Vec<QuadrantPoint>,
    theme: String,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let chart = parse_quadrant(body)?;
    Ok(render_svg(&chart, controls))
}

fn parse_quadrant(body: &str) -> Result<QuadrantChart, String> {
    let trimmed = body.trim();
    if !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("Quadrant body must be wrapped in '---- ... ----'".into());
    }
    let inner = &trimmed[4..trimmed.len() - 4];
    let parts: Vec<&str> = inner.splitn(2, "---").collect();
    if parts.len() != 2 {
        return Err("missing '---' separator between header and data".into());
    }
    let (header_str, data_str) = (parts[0], parts[1]);

    let mut config = HashMap::new();
    for line in header_str.lines() {
        if let Some((k, v)) = line.split_once('=') {
            config.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    let mut points = Vec::new();
    for line in data_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let tokens: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if tokens.len() >= 3 {
            let label = tokens[0].to_string();
            let x: f64 = tokens[1]
                .parse()
                .map_err(|_| format!("Invalid X value: {}", tokens[1]))?;
            let y: f64 = tokens[2]
                .parse()
                .map_err(|_| format!("Invalid Y value: {}", tokens[2]))?;
            let category = tokens.get(3).map(|s| s.to_string());
            points.push(QuadrantPoint {
                label,
                x,
                y,
                category,
            });
        }
    }

    Ok(QuadrantChart {
        title: config
            .get("title")
            .cloned()
            .unwrap_or_else(|| "Quadrant Chart".to_string()),
        x_axis: config
            .get("xAxis")
            .cloned()
            .unwrap_or_else(|| "EFFORT REQUIRED".to_string()),
        y_axis: config
            .get("yAxis")
            .cloned()
            .unwrap_or_else(|| "IMPACT LEVEL".to_string()),
        leaders: config
            .get("leaders")
            .cloned()
            .unwrap_or_else(|| "HIGH IMPACT".to_string()),
        challengers: config
            .get("challengers")
            .cloned()
            .unwrap_or_else(|| "STRATEGIC".to_string()),
        visionaries: config
            .get("visionaries")
            .cloned()
            .unwrap_or_else(|| "FILL-INS".to_string()),
        niche: config
            .get("niche")
            .cloned()
            .unwrap_or_else(|| "THANKLESS".to_string()),
        points,
        theme: config
            .get("theme")
            .cloned()
            .unwrap_or_else(|| "premium".to_string()),
    })
}

fn render_svg(chart: &QuadrantChart, controls: &HashMap<String, String>) -> String {
    let use_dark = controls
        .get("useDark")
        .map(|s| s == "true")
        .unwrap_or(false)
        || chart.theme == "dark";

    let is_premium = chart.theme == "premium";

    let t = if use_dark {
        theme("dark")
    } else {
        theme(&chart.theme)
    };
    let dark_t = theme("dark");
    let chart_id = format!("quad-{}", &Uuid::new_v4().to_string()[..8]);

    let width = 960.0;
    let height = 700.0;
    let margin = 80.0;
    let plot_w = width - 2.0 * margin;
    let plot_h = height - 2.0 * margin - 60.0; // Extra room for title
    let plot_x = margin;
    let plot_y = margin + 60.0;

    let center_x = plot_x + plot_w / 2.0;
    let center_y = plot_y + plot_h / 2.0;

    let extra_class = if use_dark { " dark-mode" } else { "" };
    let title_esc = escape(&chart.title);
    let desc_text = format!(
        "Quadrant chart with {} and {} axes, containing {} points",
        chart.x_axis,
        chart.y_axis,
        chart.points.len()
    );
    let desc_esc = escape(&desc_text);

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" font-family="Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, sans-serif" id="{chart_id}" class="quadrant-chart{extra_class}" role="graphics-document document" aria-labelledby="{chart_id}_title {chart_id}_desc">
  <title id="{chart_id}_title">{title_esc}</title>
  <desc id="{chart_id}_desc">{desc_esc}</desc>
  <defs>
    <style>
        #{chart_id} {{
            --bg: {bg};
            --text: {text};
            --axis: {axis};
            --quad-op: {quad_op};
            --quad-0: {q0};
            --quad-1: {q1};
            --quad-2: {q2};
            --quad-3: {q3};
            --palette-0: {p0};
            --palette-1: {p1};
            --palette-2: {p2};
            --palette-3: {p3};
            --palette-4: {p4};
            --palette-5: {p5};
            
            --bg-start: {bg};
            --bg-end: {bg_end};
            --card-bg: {card_bg};
            --card-stroke: {card_stroke};
            --muted: {muted};
            --grid: {grid};
            --marker-stroke: {marker_stroke};
            --shadow-opacity: {shadow_op};
        }}
        @media (prefers-color-scheme: dark) {{
            #{chart_id} {{
                --bg: {dark_bg};
                --text: {dark_text};
                --axis: {dark_axis};
                --quad-op: 0.08;
                --quad-0: {dark_q0};
                --quad-1: {dark_q1};
                --quad-2: {dark_q2};
                --quad-3: {dark_q3};
                --palette-0: {dark_p0};
                --palette-1: {dark_p1};
                --palette-2: {dark_p2};
                --palette-3: {dark_p3};
                --palette-4: {dark_p4};
                --palette-5: {dark_p5};
                
                --bg-start: #020617;
                --bg-end: #111827;
                --card-bg: rgba(15, 23, 42, 0.86);
                --card-stroke: rgba(148, 163, 184, 0.22);
                --text: #f8fafc;
                --muted: #cbd5e1;
                --axis: #64748b;
                --grid: rgba(148, 163, 184, 0.16);
                --marker-stroke: #020617;
                --shadow-opacity: 0.36;
            }}
        }}
        #{chart_id}.dark-mode {{
            --bg: {dark_bg};
            --text: {dark_text};
            --axis: {dark_axis};
            --quad-op: 0.08;
            --quad-0: {dark_q0};
            --quad-1: {dark_q1};
            --quad-2: {dark_q2};
            --quad-3: {dark_q3};
            --palette-0: {dark_p0};
            --palette-1: {dark_p1};
            --palette-2: {dark_p2};
            --palette-3: {dark_p3};
            --palette-4: {dark_p4};
            --palette-5: {dark_p5};
            
            --bg-start: #020617;
            --bg-end: #111827;
            --card-bg: rgba(15, 23, 42, 0.86);
            --card-stroke: rgba(148, 163, 184, 0.22);
            --text: #f8fafc;
            --muted: #cbd5e1;
            --axis: #64748b;
            --grid: rgba(148, 163, 184, 0.16);
            --marker-stroke: #020617;
            --shadow-opacity: 0.36;
        }}
        #{chart_id} .point-label {{
            paint-order: stroke;
            stroke: var(--card-bg);
            stroke-width: 5;
            stroke-linejoin: round;
        }}
    </style>
    <clipPath id="{chart_id}-plot-clip">
        <rect x="{plot_x}" y="{plot_y}" width="{plot_w}" height="{plot_h}" rx="22"/>
    </clipPath>
    <filter id="{chart_id}-shadow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur in="SourceAlpha" stdDeviation="2" />
      <feOffset dx="1" dy="1" result="offsetblur" />
      <feComponentTransfer>
        <feFuncA type="linear" slope="0.2" />
      </feComponentTransfer>
      <feMerge>
        <feMergeNode />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
    <filter id="{chart_id}-card-shadow" x="-12%" y="-12%" width="124%" height="124%">
        <feDropShadow dx="0" dy="18" stdDeviation="22" flood-color="#0f172a" flood-opacity="var(--shadow-opacity)"/>
    </filter>
    <filter id="{chart_id}-marker-shadow" x="-80%" y="-80%" width="260%" height="260%">
        <feDropShadow dx="0" dy="7" stdDeviation="6" flood-color="#0f172a" flood-opacity="0.22"/>
    </filter>
    <linearGradient id="{chart_id}-bg-grad" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="var(--bg-start)"/>
        <stop offset="100%" stop-color="var(--bg-end)"/>
    </linearGradient>
    <radialGradient id="{chart_id}-shine" cx="35%" cy="25%" r="70%">
        <stop offset="0%" stop-color="#ffffff" stop-opacity="0.65"/>
        <stop offset="45%" stop-color="#ffffff" stop-opacity="0.18"/>
        <stop offset="100%" stop-color="#ffffff" stop-opacity="0"/>
    </radialGradient>
  </defs>
"##,
        width = width,
        height = height,
        chart_id = chart_id,
        extra_class = if is_premium {
            format!("{} premium", extra_class)
        } else {
            extra_class.to_string()
        },
        bg = t.background,
        text = t.text,
        axis = t.axis,
        quad_op = if use_dark { "0.08" } else { "0.04" },
        q0 = t.palette[1 % t.palette.len()],
        q1 = t.palette[0 % t.palette.len()],
        q2 = t.palette[2 % t.palette.len()],
        q3 = t.palette[3 % t.palette.len()],
        p0 = t.palette[0 % t.palette.len()],
        p1 = t.palette[1 % t.palette.len()],
        p2 = t.palette[2 % t.palette.len()],
        p3 = t.palette[3 % t.palette.len()],
        p4 = t.palette[4 % t.palette.len()],
        p5 = t.palette[5 % t.palette.len()],
        dark_bg = dark_t.background,
        dark_text = dark_t.text,
        dark_axis = dark_t.axis,
        dark_q0 = dark_t.palette[1 % dark_t.palette.len()],
        dark_q1 = dark_t.palette[0 % dark_t.palette.len()],
        dark_q2 = dark_t.palette[2 % dark_t.palette.len()],
        dark_q3 = dark_t.palette[3 % dark_t.palette.len()],
        dark_p0 = dark_t.palette[0 % dark_t.palette.len()],
        dark_p1 = dark_t.palette[1 % dark_t.palette.len()],
        dark_p2 = dark_t.palette[2 % dark_t.palette.len()],
        dark_p3 = dark_t.palette[3 % dark_t.palette.len()],
        dark_p4 = dark_t.palette[4 % dark_t.palette.len()],
        dark_p5 = dark_t.palette[5 % dark_t.palette.len()],
        bg_end = if use_dark { "#111827" } else { "#eef2ff" },
        card_bg = if use_dark {
            "rgba(15, 23, 42, 0.86)"
        } else {
            "rgba(255, 255, 255, 0.88)"
        },
        card_stroke = if use_dark {
            "rgba(148, 163, 184, 0.22)"
        } else {
            "rgba(148, 163, 184, 0.28)"
        },
        muted = if use_dark { "#cbd5e1" } else { "#64748b" },
        grid = if use_dark {
            "rgba(148, 163, 184, 0.16)"
        } else {
            "rgba(148, 163, 184, 0.22)"
        },
        marker_stroke = if use_dark { "#020617" } else { "#ffffff" },
        shadow_op = if use_dark { "0.36" } else { "0.16" },
        plot_x = plot_x,
        plot_y = plot_y,
        plot_w = plot_w,
        plot_h = plot_h
    );

    if is_premium {
        svg.push_str(&format!(
            r##"  <rect width="{width}" height="{height}" fill="url(#{chart_id}-bg-grad)" aria-hidden="true"/>
  <circle cx="130" cy="95" r="120" fill="#8b5cf6" opacity="0.08" aria-hidden="true"/>
  <circle cx="835" cy="610" r="150" fill="#06b6d4" opacity="0.10" aria-hidden="true"/>
  <g filter="url(#{chart_id}-card-shadow)" aria-hidden="true">
    <rect x="{card_x}" y="{card_y}" width="{card_w}" height="{card_h}" rx="32" fill="var(--card-bg)" stroke="var(--card-stroke)" stroke-width="1.2"/>
  </g>
"##,
            width = width,
            height = height,
            chart_id = chart_id,
            card_x = plot_x - 24.0,
            card_y = plot_y - 24.0,
            card_w = plot_w + 48.0,
            card_h = plot_h + 48.0
        ));
    } else {
        svg.push_str(&format!(
            r##"  <rect width="{width}" height="{height}" fill="var(--bg)" aria-hidden="true"/>
"##,
            width = width,
            height = height
        ));
    }

    // Title
    svg.push_str(&format!(
        r##"  <text x="{x}" y="52" font-size="{font_size}" font-weight="850" fill="var(--text)" text-anchor="middle" class="title" aria-hidden="true">{title}</text>
"##,
        x = width / 2.0,
        font_size = if is_premium { 32 } else { 28 },
        title = escape(&chart.title)
    ));

    // Chart Content Group (Clipped if premium)
    let clip = if is_premium {
        format!(" clip-path=\"url(#{chart_id}-plot-clip)\"")
    } else {
        "".to_string()
    };
    svg.push_str(&format!(
        r##"  <g{clip}>
"##,
        clip = clip
    ));

    // Quadrant Backgrounds
    if is_premium {
        svg.push_str(&format!(
            r##"    <rect x="{plot_x}" y="{plot_y}" width="{half_w}" height="{half_h}" fill="var(--quad-0)" fill-opacity="1.0" role="region" aria-label="{l0}"/>
    <rect x="{center_x}" y="{plot_y}" width="{half_w}" height="{half_h}" fill="var(--quad-1)" fill-opacity="1.0" role="region" aria-label="{l1}"/>
    <rect x="{plot_x}" y="{center_y}" width="{half_w}" height="{half_h}" fill="var(--quad-2)" fill-opacity="1.0" role="region" aria-label="{l2}"/>
    <rect x="{center_x}" y="{center_y}" width="{half_w}" height="{half_h}" fill="var(--quad-3)" fill-opacity="1.0" role="region" aria-label="{l3}"/>
"##,
            plot_x = plot_x,
            plot_y = plot_y,
            center_x = center_x,
            center_y = center_y,
            half_w = plot_w / 2.0,
            half_h = plot_h / 2.0,
            l0 = escape(&chart.challengers),
            l1 = escape(&chart.leaders),
            l2 = escape(&chart.visionaries),
            l3 = escape(&chart.niche)
        ));

        // Grid lines for premium
        svg.push_str(&format!(
            r##"    <line x1="{plot_x}" y1="{grid_y1}" x2="{plot_x_end}" y2="{grid_y1}" stroke="var(--grid)" stroke-width="1" aria-hidden="true"/>
    <line x1="{plot_x}" y1="{grid_y2}" x2="{plot_x_end}" y2="{grid_y2}" stroke="var(--grid)" stroke-width="1" aria-hidden="true"/>
    <line x1="{grid_x1}" y1="{plot_y}" x2="{grid_x1}" y2="{plot_y_end}" stroke="var(--grid)" stroke-width="1" aria-hidden="true"/>
    <line x1="{grid_x2}" y1="{plot_y}" x2="{grid_x2}" y2="{plot_y_end}" stroke="var(--grid)" stroke-width="1" aria-hidden="true"/>
"##,
            plot_x = plot_x,
            plot_x_end = plot_x + plot_w,
            plot_y = plot_y,
            plot_y_end = plot_y + plot_h,
            grid_y1 = plot_y + plot_h * 0.25,
            grid_y2 = plot_y + plot_h * 0.75,
            grid_x1 = plot_x + plot_w * 0.25,
            grid_x2 = plot_x + plot_w * 0.75
        ));
    } else {
        svg.push_str(&format!(
            r##"    <rect x="{plot_x}" y="{plot_y}" width="{half_w}" height="{half_h}" fill="var(--quad-0)" fill-opacity="var(--quad-op)" role="region" aria-label="{l0}"/>
    <rect x="{center_x}" y="{plot_y}" width="{half_w}" height="{half_h}" fill="var(--quad-1)" fill-opacity="var(--quad-op)" role="region" aria-label="{l1}"/>
    <rect x="{plot_x}" y="{center_y}" width="{half_w}" height="{half_h}" fill="var(--quad-2)" fill-opacity="var(--quad-op)" role="region" aria-label="{l2}"/>
    <rect x="{center_x}" y="{center_y}" width="{half_w}" height="{half_h}" fill="var(--quad-3)" fill-opacity="var(--quad-op)" role="region" aria-label="{l3}"/>
"##,
            plot_x = plot_x,
            plot_y = plot_y,
            center_x = center_x,
            center_y = center_y,
            half_w = plot_w / 2.0,
            half_h = plot_h / 2.0,
            l0 = escape(&chart.challengers),
            l1 = escape(&chart.leaders),
            l2 = escape(&chart.visionaries),
            l3 = escape(&chart.niche)
        ));
    }

    // Axes
    svg.push_str(&format!(
        r##"    <line x1="{plot_x}" y1="{center_y}" x2="{plot_x_end}" y2="{center_y}" stroke="var(--axis)" stroke-width="{axis_w}" />
    <line x1="{center_x}" y1="{plot_y}" x2="{center_x}" y2="{plot_y_end}" stroke="var(--axis)" stroke-width="{axis_w}" />
"##,
        plot_x = plot_x,
        center_y = center_y,
        plot_x_end = plot_x + plot_w,
        center_x = center_x,
        plot_y = plot_y,
        plot_y_end = plot_y + plot_h,
        axis_w = if is_premium { "2.2" } else { "2" }
    ));

    svg.push_str("  </g>\n");

    // Plot Border for premium
    if is_premium {
        svg.push_str(&format!(
            r##"  <rect x="{plot_x}" y="{plot_y}" width="{plot_w}" height="{plot_h}" rx="22" fill="none" stroke="var(--card-stroke)" stroke-width="1.4"/>
"##,
            plot_x = plot_x,
            plot_y = plot_y,
            plot_w = plot_w,
            plot_h = plot_h
        ));
    }

    // Axis Labels
    svg.push_str(&format!(
        r##"  <text x="{x}" y="{y}" font-size="13" font-weight="800" fill="var(--muted)" text-anchor="middle" class="axis-label">{label}</text>
  <text x="{yx}" y="{yy}" font-size="13" font-weight="800" fill="var(--muted)" text-anchor="middle" class="axis-label" transform="rotate(-90, {yx}, {yy})">{ylabel}</text>
"##,
        x = center_x,
        y = plot_y + plot_h + 40.0,
        label = escape(&chart.x_axis),
        yx = plot_x - 48.0,
        yy = center_y,
        ylabel = escape(&chart.y_axis)
    ));

    // Quadrant Labels
    if is_premium {
        svg.push_str(&format!(
            r##"  <g>
    <rect x="{x0}" y="{y0}" width="110" height="28" rx="14" fill="var(--card-bg)" stroke="var(--card-stroke)"/>
    <text x="{x0_t}" y="{y0_t}" class="quad-label" font-size="11" font-weight="800" fill="var(--muted)" text-anchor="middle">{l0}</text>

    <rect x="{x1}" y="{y1}" width="102" height="28" rx="14" fill="var(--card-bg)" stroke="var(--card-stroke)"/>
    <text x="{x1_t}" y="{y1_t}" class="quad-label" font-size="11" font-weight="800" fill="var(--muted)" text-anchor="middle">{l1}</text>

    <rect x="{x2}" y="{y2}" width="84" height="28" rx="14" fill="var(--card-bg)" stroke="var(--card-stroke)"/>
    <text x="{x2_t}" y="{y2_t}" class="quad-label" font-size="11" font-weight="800" fill="var(--muted)" text-anchor="middle">{l2}</text>

    <rect x="{x3}" y="{y3}" width="108" height="28" rx="14" fill="var(--card-bg)" stroke="var(--card-stroke)"/>
    <text x="{x3_t}" y="{y3_t}" class="quad-label" font-size="11" font-weight="800" fill="var(--muted)" text-anchor="middle">{l3}</text>
  </g>
"##,
            x0 = plot_x + 16.0, y0 = plot_y + 14.0, x0_t = plot_x + 71.0, y0_t = plot_y + 33.0, l0 = escape(&chart.challengers),
            x1 = plot_x + plot_w - 118.0, y1 = plot_y + 14.0, x1_t = plot_x + plot_w - 67.0, y1_t = plot_y + 33.0, l1 = escape(&chart.leaders),
            x2 = plot_x + 16.0, y2 = plot_y + plot_h - 42.0, x2_t = plot_x + 58.0, y2_t = plot_y + plot_h - 23.0, l2 = escape(&chart.visionaries),
            x3 = plot_x + plot_w - 124.0, y3 = plot_y + plot_h - 42.0, x3_t = plot_x + plot_w - 70.0, y3_t = plot_y + plot_h - 23.0, l3 = escape(&chart.niche)
        ));
    } else {
        let quad_font_size = 12;
        svg.push_str(&format!(
            r##"  <text x="{x}" y="{y}" font-size="{size}" font-weight="700" fill="var(--axis)" text-anchor="start">{label}</text>
"##,
            x = plot_x + 10.0,
            y = plot_y + 20.0,
            size = quad_font_size,
            label = escape(&chart.challengers)
        ));
        svg.push_str(&format!(
            r##"  <text x="{x}" y="{y}" font-size="{size}" font-weight="700" fill="var(--axis)" text-anchor="end">{label}</text>
"##,
            x = plot_x + plot_w - 10.0,
            y = plot_y + 20.0,
            size = quad_font_size,
            label = escape(&chart.leaders)
        ));
        svg.push_str(&format!(
            r##"  <text x="{x}" y="{y}" font-size="{size}" font-weight="700" fill="var(--axis)" text-anchor="start">{label}</text>
"##,
            x = plot_x + 10.0,
            y = plot_y + plot_h - 10.0,
            size = quad_font_size,
            label = escape(&chart.visionaries)
        ));
        svg.push_str(&format!(
            r##"  <text x="{x}" y="{y}" font-size="{size}" font-weight="700" fill="var(--axis)" text-anchor="end">{label}</text>
"##,
            x = plot_x + plot_w - 10.0,
            y = plot_y + plot_h - 10.0,
            size = quad_font_size,
            label = escape(&chart.niche)
        ));
    }

    // Data Points
    // Important: Put markers INSIDE the clipped group if premium
    svg.push_str(&format!(
        r##"  <g{clip}>
"##,
        clip = if is_premium {
            format!(" clip-path=\"url(#{chart_id}-plot-clip)\"")
        } else {
            "".to_string()
        }
    ));

    let mut category_colors = HashMap::new();
    let mut color_idx = 0;

    for point in &chart.points {
        let px = plot_x + (point.x / 100.0) * plot_w;
        let py = plot_y + (1.0 - (point.y / 100.0)) * plot_h;

        let idx = if let Some(cat) = &point.category {
            *category_colors.entry(cat.clone()).or_insert_with(|| {
                let i = color_idx % 6;
                color_idx += 1;
                i
            })
        } else {
            0
        };
        let color_var = format!("var(--palette-{})", idx);

        if is_premium {
            svg.push_str(&format!(
                r##"    <g filter="url(#{chart_id}-marker-shadow)" role="graphics-symbol" aria-roledescription="data point" tabindex="0" aria-label="{label}: ({x:.1}, {y:.1})">
      <circle cx="{px}" cy="{py}" r="11" fill="{color_var}" stroke="var(--marker-stroke)" stroke-width="3" aria-hidden="true"/>
      <circle cx="{px}" cy="{py}" r="11" fill="url(#{chart_id}-shine)" aria-hidden="true"/>
      <text x="{px}" y="{text_y}" font-size="12" font-weight="700" fill="var(--text)" text-anchor="middle" class="point-label">{label}</text>
    </g>
"##,
                chart_id = chart_id,
                px = px,
                py = py,
                color_var = color_var,
                text_y = py + 27.0,
                label = escape(&point.label),
                x = point.x,
                y = point.y
            ));
        } else {
            svg.push_str(&format!(
                r##"    <g filter="url(#{chart_id}-shadow)" role="graphics-symbol" aria-roledescription="data point" tabindex="0" aria-label="{label}: ({x:.1}, {y:.1})">
      <circle cx="{px}" cy="{py}" r="8" fill="{color_var}" stroke="var(--bg)" stroke-width="2" aria-hidden="true"/>
      <text x="{px}" y="{text_y}" font-size="11" font-weight="600" fill="var(--text)" text-anchor="middle">{label}</text>
    </g>
"##,
                chart_id = chart_id,
                px = px,
                py = py,
                color_var = color_var,
                text_y = py + 22.0,
                label = escape(&point.label),
                x = point.x,
                y = point.y
            ));
        }
    }

    svg.push_str("  </g>\n");
    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_is_premium() {
        let body = r#"----
title=Default Theme Test
xAxis=X
yAxis=Y
---
A | 50 | 50
----"#;
        let chart = parse_quadrant(body).unwrap();
        assert_eq!(chart.theme, "premium");

        let controls = HashMap::new();
        let svg = render_svg(&chart, &controls);

        // Premium features should be present
        assert!(
            svg.contains("premium"),
            "SVG class should contain 'premium'"
        );
        assert!(
            svg.contains("card-shadow"),
            "SVG should contain premium shadow filter"
        );
        assert!(
            svg.contains("bg-grad"),
            "SVG should contain premium background gradient"
        );
    }

    #[test]
    fn test_explicit_theme_overrides_default() {
        let body = r#"----
theme=dark
title=Dark Theme Test
---
A | 50 | 50
----"#;
        let chart = parse_quadrant(body).unwrap();
        assert_eq!(chart.theme, "dark");

        let controls = HashMap::new();
        let svg = render_svg(&chart, &controls);

        assert!(
            svg.contains("dark-mode"),
            "SVG class should contain 'dark-mode'"
        );
        assert!(
            !svg.contains("premium"),
            "SVG should not contain 'premium' when dark theme is explicit"
        );
    }
}
