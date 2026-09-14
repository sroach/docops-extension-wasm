use crate::common::kv::parse_kv_header;
use crate::common::svg::escape;
use std::collections::HashMap;
use uuid::Uuid;

struct ReleaseStep {
    phase: String,
    date: String,
    status: String,
    title: String,
    details: Vec<String>,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let (config, steps) = parse_release_body(body)?;

    let use_dark = controls
        .get("useDark")
        .map(|s| s == "true")
        .unwrap_or(false)
        || config.get("theme").map(|s| s.as_str()) == Some("dark");

    let layout = config
        .get("layout")
        .map(|s| s.as_str())
        .unwrap_or("horizontal");

    if layout == "vertical" {
        return render_vertical(&config, &steps, use_dark);
    }

    let _theme_name = config.get("theme").map(|s| s.as_str()).unwrap_or("default");

    let title = config
        .get("title")
        .map(String::as_str)
        .unwrap_or("Release Strategy");
    let subtitle = config.get("subtitle").map(String::as_str).unwrap_or("");
    let chart_id = format!("release_h_{}", Uuid::new_v4().to_string().replace('-', "_"));

    let extra_class = if use_dark { " dark-mode" } else { "" };

    // SVG parameters matching gen/release-strategy-fixed.svg
    let rail_y = 258;
    let start_x = 214;
    let end_x = 1546;
    let rail_len = end_x - start_x;
    let card_y = 154;

    let mut steps_svg = String::new();
    let mut progress_x = start_x;

    // Unique IDs for gradients and filters
    let shadow_id = format!("shadow_{}", chart_id);
    let premium_bg_id = format!("pbg_{}", chart_id);
    let blue_glow_id = format!("bglow_{}", chart_id);
    let violet_glow_id = format!("vglow_{}", chart_id);
    let glass_veil_id = format!("gveil_{}", chart_id);
    let rail_track_glass_id = format!("rtg_{}", chart_id);
    let rail_fill_grad_id = format!("rfg_{}", chart_id);
    let rail_glow_grad_id = format!("rgg_{}", chart_id);
    let rail_glow_id = format!("rglow_{}", chart_id);
    let badge_glow_id = format!("bglow_f_{}", chart_id);
    let card_highlight_id = format!("chigh_{}", chart_id);
    let _glass_card_id = format!("gcard_{}", chart_id);

    // Badge gradients
    let badge_blue_id = format!("bblue_{}", chart_id);
    let badge_green_id = format!("bgreen_{}", chart_id);
    let badge_orange_id = format!("borange_{}", chart_id);
    let badge_muted_id = format!("bmuted_{}", chart_id);

    let step_count = steps.len();
    for (i, step) in steps.iter().enumerate() {
        let x = if step_count > 1 {
            start_x + (i * rail_len) / (step_count - 1)
        } else {
            start_x + rail_len / 2
        };

        let is_complete = step.status.to_lowercase() == "complete";
        let is_in_progress = step.status.to_lowercase() == "in progress"
            || step.status.to_lowercase() == "now"
            || step.status.to_lowercase() == "testing";

        if is_complete || is_in_progress {
            progress_x = x;
        }

        let is_upcoming = !is_complete && !is_in_progress;
        let card_class = if is_upcoming { "upcoming-card" } else { "" };

        let mut details_svg = String::new();
        for (j, detail) in step.details.iter().enumerate() {
            let detail_esc = escape(detail);
            details_svg.push_str(&format!(
                r##"      <g role="listitem" aria-label="{detail}"><circle cx="30" cy="{dy_c}" r="3.5" fill="var(--text-soft)" opacity="{op}" aria-hidden="true"/><text x="42" y="{dy_t}">{detail}</text></g>
"##,
                dy_c = 108 + j * 24,
                dy_t = 113 + j * 24,
                op = if j == 0 { "0.78" } else if j == 1 { "0.66" } else { "0.54" },
                detail = detail_esc
            ));
        }

        let status_name = if is_in_progress {
            "Testing"
        } else if is_complete {
            "Complete"
        } else {
            "Upcoming"
        };

        let status_chip = if is_in_progress {
            r##"      <g transform="translate(202,16)" role="status" aria-label="Status: Testing">
        <rect width="92" height="26" rx="13" fill="#bfdbfe" opacity="0.98"/>
        <text x="46" y="18" text-anchor="middle" class="chip-text" fill="#1e3a8a">Testing</text>
      </g>
      <g transform="translate(302,16)" aria-hidden="true">
        <rect width="42" height="26" rx="13" fill="var(--primary)"/>
        <text x="21" y="18" text-anchor="middle" fill="#ffffff" font-size="10.5" font-weight="900">NOW</text>
      </g>"##.to_string()
        } else if is_complete {
            r##"      <g transform="translate(246,16)" role="status" aria-label="Status: Complete">
        <rect width="86" height="26" rx="13" fill="#bbf7d0" opacity="0.98"/>
        <text x="43" y="18" text-anchor="middle" class="chip-text" fill="#14532d">Complete</text>
      </g>"##
                .to_string()
        } else {
            r##"      <g transform="translate(246,16)" role="status" aria-label="Status: Upcoming">
        <rect width="86" height="26" rx="13" fill="#e2e8f0" opacity="1"/>
        <text x="43" y="18" text-anchor="middle" class="chip-text" fill="#334155">Upcoming</text>
      </g>"##
                .to_string()
        };

        steps_svg.push_str(&format!(
            r##"  <!-- {phase} card -->
  <g transform="translate({cx}, {cy})" class="{card_class}" role="region" aria-label="Phase {phase}: {title}, Status: {status_name}">
    <rect x="0" y="0" width="360" height="222" rx="24" class="glass-card"/>
    <path d="M22 14 H338 C348 14 356 22 356 32 V44 H22Z" fill="url(#{card_high})" opacity="{high_op}" aria-hidden="true"/>
    <text x="24" y="34" class="date-text">{date}</text>
    {status_chip}
    <text x="24" y="78" class="goal-text">{title}</text>
    <g class="detail-text" role="list" aria-label="Deliverables">
{details}
    </g>
  </g>
"##,
            phase = escape(&step.phase),
            cx = x - 180,
            cy = card_y,
            card_class = card_class,
            card_high = card_highlight_id,
            high_op = if is_in_progress { "0.38" } else if is_complete { "0.32" } else { "0.24" },
            date = escape(&step.date),
            status_chip = status_chip,
            status_name = status_name,
            title = escape(&step.title),
            details = details_svg
        ));

        // Render badge (on top of card)
        let badge_fill = if step.phase.starts_with("GA") {
            format!("url(#{})", badge_muted_id)
        } else if step.phase.starts_with("RC") {
            if is_in_progress {
                format!("url(#{})", badge_blue_id)
            } else {
                format!("url(#{})", badge_muted_id)
            }
        } else if is_complete {
            format!("url(#{})", badge_green_id)
        } else {
            format!("url(#{})", badge_muted_id)
        };

        let badge_radius = if is_in_progress { 26 } else { 24 };
        let pulse_ring = if is_in_progress {
            format!(
                r##"  <circle cx="{x}" cy="{rail_y}" r="31" class="pulse-ring" aria-hidden="true"/>
"##,
                x = x,
                rail_y = rail_y
            )
        } else {
            String::new()
        };

        let filter_str = if is_upcoming {
            "none".to_string()
        } else {
            format!("url(#{})", badge_glow_id)
        };

        steps_svg.push_str(&format!(
            r##"  <!-- {phase} badge -->
{pulse}  <g transform="translate({bx}, {by})" filter="{filter}" aria-hidden="true">
    <circle cx="{r}" cy="{r}" r="{r}" fill="{fill}"/>
    <circle cx="{r}" cy="{r}" r="{r_inner}" fill="none" stroke="#ffffff" stroke-opacity="{s_op}"/>
  </g>
  <text x="{x}" y="{ty}" text-anchor="middle" class="milestone label-mono" aria-hidden="true">{phase}</text>
"##,
            phase = escape(&step.phase),
            pulse = pulse_ring,
            bx = x as i32 - badge_radius,
            by = rail_y - badge_radius,
            r = badge_radius,
            r_inner = badge_radius - 1,
            fill = badge_fill,
            filter = filter_str,
            s_op = if is_in_progress { "0.70" } else { "0.66" },
            x = x,
            ty = rail_y + 8
        ));
    }

    let svg = format!(
        r##"<svg width="880" height="260" viewBox="0 0 1760 520" xmlns="http://www.w3.org/2000/svg" id="{chart_id}" class="release-container{extra_class}" role="graphics-document document" aria-labelledby="title_{chart_id} desc_{chart_id}">
  <title id="title_{chart_id}">{title}</title>
  <desc id="desc_{chart_id}">Horizontal release strategy roadmap for {title}.</desc>
  <defs>
    <!-- Premium background gradients -->
    <linearGradient id="{pbg}" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="var(--bg-top)"/>
      <stop offset="36%" stop-color="var(--bg-mid)"/>
      <stop offset="68%" stop-color="var(--bg-low)"/>
      <stop offset="100%" stop-color="var(--bg)"/>
    </linearGradient>

    <radialGradient id="{bglow}" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#007aff" stop-opacity="0.34"/>
      <stop offset="42%" stop-color="#5ac8fa" stop-opacity="0.18"/>
      <stop offset="100%" stop-color="#007aff" stop-opacity="0"/>
    </radialGradient>

    <radialGradient id="{vglow}" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#af52de" stop-opacity="0.28"/>
      <stop offset="48%" stop-color="#bf5af2" stop-opacity="0.14"/>
      <stop offset="100%" stop-color="#bf5af2" stop-opacity="0"/>
    </radialGradient>

    <linearGradient id="{gveil}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#ffffff" stop-opacity="0.24"/>
      <stop offset="100%" stop-color="#ffffff" stop-opacity="0.04"/>
    </linearGradient>

    <!-- Rail -->
    <linearGradient id="{rtg}" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#d8e0ea" stop-opacity="0.95"/>
      <stop offset="50%" stop-color="#cbd5e1" stop-opacity="0.95"/>
      <stop offset="100%" stop-color="#d8e0ea" stop-opacity="0.95"/>
    </linearGradient>

    <linearGradient id="{rfg}" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#16a34a"/>
      <stop offset="45%" stop-color="#0891b2"/>
      <stop offset="100%" stop-color="#007aff"/>
    </linearGradient>

    <linearGradient id="{rgg}" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#16a34a" stop-opacity="0.22"/>
      <stop offset="50%" stop-color="#0891b2" stop-opacity="0.34"/>
      <stop offset="100%" stop-color="#007aff" stop-opacity="0.22"/>
    </linearGradient>

    <!-- Card and Badge Effects -->
    <linearGradient id="{chigh}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#ffffff" stop-opacity="0.8"/>
      <stop offset="100%" stop-color="#ffffff" stop-opacity="0"/>
    </linearGradient>

    <filter id="{rglow}" x="-5%" y="-220%" width="110%" height="540%">
      <feGaussianBlur stdDeviation="7" result="blur"/>
      <feMerge>
        <feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>

    <filter id="{shadow_id}" x="-20%" y="-20%" width="140%" height="150%">
      <feDropShadow dx="0" dy="20" stdDeviation="24" flood-color="#0f172a" flood-opacity="0.14"/>
    </filter>

    <filter id="{badge_glow}" x="-80%" y="-80%" width="260%" height="260%">
      <feDropShadow dx="0" dy="8" stdDeviation="10" flood-color="#007aff" flood-opacity="0.26"/>
    </filter>

    <!-- Badge gradients -->
    <linearGradient id="{bblue}" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#64d2ff"/>
      <stop offset="48%" stop-color="#007aff"/>
      <stop offset="100%" stop-color="#5856d6"/>
    </linearGradient>
    <linearGradient id="{bgreen}" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#7ee787"/>
      <stop offset="48%" stop-color="#34c759"/>
      <stop offset="100%" stop-color="#166534"/>
    </linearGradient>
    <linearGradient id="{borange}" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#ff9f0a"/>
      <stop offset="100%" stop-color="#ff7a00"/>
    </linearGradient>
    <linearGradient id="{bmuted}" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#94a3b8"/>
      <stop offset="100%" stop-color="#64748b"/>
    </linearGradient>
  </defs>

  <style>
    #{chart_id} {{
      --bg: #ffffff;
      --bg-top: #f8fbff;
      --bg-mid: #eef6ff;
      --bg-low: #f8f5ff;
      --text: #0f172a;
      --text-soft: #475569;
      --text-muted: #64748b;
      --primary: #007aff;
      --glass-bg: rgba(255, 255, 255, 0.75);
      --glass-stroke: rgba(255, 255, 255, 0.9);
    }}
    @media (prefers-color-scheme: dark) {{
      #{chart_id} {{
        --bg: #0f172a;
        --bg-top: #1e293b;
        --bg-mid: #0f172a;
        --bg-low: #1e1b4b;
        --text: #f8fafc;
        --text-soft: #cbd5e1;
        --text-muted: #94a3b8;
        --primary: #0a84ff;
        --glass-bg: rgba(30, 41, 59, 0.7);
        --glass-stroke: rgba(255, 255, 255, 0.15);
      }}
    }}
    #{chart_id}.dark-mode {{
      --bg: #0f172a;
      --bg-top: #1e293b;
      --bg-mid: #0f172a;
      --bg-low: #1e1b4b;
      --text: #f8fafc;
      --text-soft: #cbd5e1;
      --text-muted: #94a3b8;
      --primary: #0a84ff;
      --glass-bg: rgba(30, 41, 59, 0.7);
      --glass-stroke: rgba(255, 255, 255, 0.15);
    }}
    #{chart_id} text {{ font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", Inter, system-ui, sans-serif; }}
    #{chart_id} .title {{ font-size: 38px; font-weight: 850; letter-spacing: -0.04em; fill: var(--text); }}
    #{chart_id} .subtitle {{ font-size: 16px; font-weight: 600; fill: var(--text-muted); }}
    #{chart_id} .glass-card {{ fill: var(--glass-bg); stroke: var(--glass-stroke); stroke-width: 1.5; filter: url(#{shadow_id}); }}
    #{chart_id} .current-card {{ fill: var(--glass-bg); stroke: var(--primary); stroke-width: 2.2; filter: url(#{shadow_id}); }}
    #{chart_id} .milestone {{ fill: #ffffff; font-size: 14px; font-weight: 850; }}
    #{chart_id} .date-text {{ fill: var(--text-muted); font-size: 15px; font-weight: 700; }}
    #{chart_id} .chip-text {{ font-size: 11px; font-weight: 800; text-transform: uppercase; letter-spacing: 0.04em; }}
    #{chart_id} .goal-text {{ fill: var(--text); font-size: 22px; font-weight: 820; letter-spacing: -0.02em; }}
    #{chart_id} .detail-text {{ fill: var(--text-soft); font-size: 15px; font-weight: 500; }}
    #{chart_id} .label-mono {{ font-family: "SF Mono", "JetBrains Mono", monospace; }}
    #{chart_id} .upcoming-card {{ opacity: 0.72; }}
    #{chart_id} .pulse-ring {{
      fill: none; stroke: var(--primary); stroke-width: 3.5; opacity: 0.55;
      transform-box: fill-box; transform-origin: center;
      animation: pulse_{chart_id} 2.4s cubic-bezier(0.4, 0, 0.6, 1) infinite;
    }}
    @keyframes pulse_{chart_id} {{
      0%   {{ transform: scale(1);    opacity: 0.6; }}
      100% {{ transform: scale(1.4);  opacity: 0; }}
    }}
  </style>

  <rect width="1760" height="520" fill="url(#{pbg})" rx="34"/>
  
  <!-- Ambient glows -->
  <circle cx="1580" cy="110" r="420" fill="url(#{bglow})"/>
  <circle cx="180" cy="440" r="380" fill="url(#{vglow})"/>

  <rect x="28" y="28" width="1704" height="464" rx="30" fill="url(#{gveil})" stroke="#ffffff" stroke-opacity="0.2"/>

  <g transform="translate(54,64)">
    <text class="title" x="0" y="0">{title}</text>
    {subtitle_svg}
  </g>

  <!-- Horizontal timeline rail -->
  <line x1="{start_x}" y1="{rail_y}" x2="{end_x}" y2="{rail_y}"
        stroke="url(#{rtg})" stroke-width="16" stroke-linecap="round"/>

  <line x1="{start_x}" y1="{rail_y}" x2="{progress_x}" y2="{rail_y}"
        stroke="url(#{rgg})" stroke-width="22" stroke-linecap="round"
        opacity="0.46" filter="url(#{rglow})"/>

  <line x1="{start_x}" y1="{rail_y}" x2="{progress_x}" y2="{rail_y}"
        stroke="url(#{rfg})" stroke-width="8" stroke-linecap="round"/>

{steps_svg}
</svg>"##,
        chart_id = chart_id,
        pbg = premium_bg_id,
        bglow = blue_glow_id,
        vglow = violet_glow_id,
        gveil = glass_veil_id,
        rtg = rail_track_glass_id,
        rfg = rail_fill_grad_id,
        rgg = rail_glow_grad_id,
        rglow = rail_glow_id,
        badge_glow = badge_glow_id,
        chigh = card_highlight_id,
        shadow_id = shadow_id,
        bblue = badge_blue_id,
        bgreen = badge_green_id,
        borange = badge_orange_id,
        bmuted = badge_muted_id,
        title = escape(title),
        subtitle_svg = if subtitle.is_empty() {
            String::new()
        } else {
            format!(
                r##"<text class="subtitle" x="2" y="28">{}</text>"##,
                escape(subtitle)
            )
        },
        start_x = start_x,
        end_x = end_x,
        rail_y = rail_y,
        progress_x = progress_x,
        steps_svg = steps_svg,
        extra_class = extra_class
    );

    Ok(svg)
}

fn render_vertical(
    config: &HashMap<String, String>,
    steps: &[ReleaseStep],
    use_dark: bool,
) -> Result<String, String> {
    let _theme_name = config.get("theme").map(|s| s.as_str()).unwrap_or("default");

    let title = config
        .get("title")
        .map(String::as_str)
        .unwrap_or("Release Strategy");
    let subtitle = config.get("subtitle").map(String::as_str).unwrap_or("");
    let chart_id = format!("release_v_{}", Uuid::new_v4().to_string().replace('-', "_"));
    let extra_class = if use_dark { " dark-mode" } else { "" };

    let step_height = 380;
    let header_height = if subtitle.is_empty() { 160 } else { 200 };
    let total_height = header_height + steps.len() * step_height + 60;

    let mut steps_svg = String::new();

    // IDs for gradients and filters
    let shadow_id = format!("shadow_{}", chart_id);
    let badge_glow_id = format!("bglow_{}", chart_id);
    let glass_id = format!("glass_{}", chart_id);
    let glass_stroke_id = format!("gstroke_{}", chart_id);
    let detail_glass_id = format!("dglass_{}", chart_id);
    let highlight_id = format!("high_{}", chart_id);
    let spine_grad_id = format!("spine_grad_{}", chart_id);

    let blue_glow_id = format!("blue_glow_{}", chart_id);
    let violet_glow_id = format!("violet_glow_{}", chart_id);
    let cyan_glow_id = format!("cyan_glow_{}", chart_id);
    let blur_filter_id = format!("blur_{}", chart_id);

    // Badge gradients
    let blue_grad = format!("bgrad_{}", chart_id);
    let green_grad = format!("ggrad_{}", chart_id);
    let orange_grad = format!("ograd_{}", chart_id);
    let red_grad = format!("rgrad_{}", chart_id);

    // Spine logic
    let mut spine_svg = String::new();
    if !steps.is_empty() {
        let spine_x = 111;
        let spine_start_y = header_height + 18 + 31;
        let spine_end_y = header_height + (steps.len() - 1) * step_height + 18 + 31;

        spine_svg = format!(
            r##"  <!-- Timeline spine -->
  <line x1="{x}" y1="{y1}" x2="{x}" y2="{y2}" stroke="url(#{spine_grad})" stroke-width="5" stroke-linecap="round"/>
  <line x1="{x}" y1="{y1}" x2="{x}" y2="{y2}" stroke="#ffffff" stroke-width="1.4" stroke-opacity="0.4" stroke-linecap="round"/>"##,
            x = spine_x,
            y1 = spine_start_y,
            y2 = spine_end_y,
            spine_grad = spine_grad_id
        );
    }

    for (i, step) in steps.iter().enumerate() {
        let y_offset = header_height + i * step_height;

        let is_complete = step.status.to_lowercase() == "complete";
        let is_in_progress =
            step.status.to_lowercase() == "in progress" || step.status.to_lowercase() == "now";

        // Badge fill based on phase
        let badge_fill = if step.phase.starts_with("GA") {
            format!("url(#{})", red_grad)
        } else if step.phase.starts_with("RC") {
            format!("url(#{})", orange_grad)
        } else if is_complete {
            format!("url(#{})", green_grad)
        } else {
            format!("url(#{})", blue_grad)
        };

        let card_stroke_width = if is_in_progress { "2" } else { "1.5" };

        let status_chip = if is_in_progress {
            r##"      <g transform="translate(808, 20)">
        <rect width="164" height="30" rx="15" fill="#dbeafe" opacity="0.92"/>
        <text x="82" y="20" text-anchor="middle" class="rm-chip" fill="#1d4ed8">In Progress</text>
      </g>"##
                .to_string()
        } else if is_complete {
            r##"      <g transform="translate(846, 20)">
        <rect width="126" height="30" rx="15" fill="#dcfce7" opacity="0.86"/>
        <text x="63" y="20" text-anchor="middle" class="rm-chip" fill="#166534">Complete</text>
      </g>"##
                .to_string()
        } else {
            r##"      <g transform="translate(844, 20)">
        <rect width="128" height="30" rx="15" fill="#fff1f2" opacity="0.92"/>
        <text x="64" y="20" text-anchor="middle" class="rm-chip" fill="#be123c">Upcoming</text>
      </g>"##
                .to_string()
        };

        let mut details_svg = String::new();
        for (j, detail) in step.details.iter().enumerate() {
            let detail_esc = escape(detail);
            details_svg.push_str(&format!(
                r#"      <g role="listitem" aria-label="{detail}"><text x="40" y="{dy}" class="rm-details">• {detail}</text></g>
"#,
                dy = 76 + j * 26,
                detail = detail_esc
            ));
        }

        let detail_box_height = 60 + step.details.len() * 26 + 20;

        let status_str = if is_in_progress {
            "In Progress"
        } else if is_complete {
            "Complete"
        } else {
            "Upcoming"
        };

        steps_svg.push_str(&format!(
            r##"    <g transform="translate(80, {y_offset})" role="region" aria-label="Phase {phase}: {title}, Status: {status}">
      <!-- Card Header -->
      <path d="M70 26 C70 11.641 81.641 0 96 0 H984 C998.359 0 1010 11.641 1010 26 V76 C1010 90.359 998.359 102 984 102 H96 C81.641 102 70 90.359 70 76 V26Z"
            fill="url(#{glass_id})" stroke="url(#{glass_stroke_id})" stroke-width="{card_sw}" filter="url(#{shadow_id})" aria-hidden="true"/>


      <!-- Badge -->
      <g transform="translate(0,20)" filter="url(#{badge_glow_id})" aria-hidden="true">
          <rect width="62" height="62" rx="22" fill="{badge_fill}"/>
          <rect x="1" y="1" width="60" height="60" rx="21" fill="none" stroke="#ffffff" stroke-opacity="0.6"/>
          <circle cx="22" cy="18" r="18" fill="#ffffff" opacity="0.2"/>
          <text x="31" y="38" text-anchor="middle" class="rm-badge" fill="#ffffff">{phase}</text>
      </g>
      
      <!-- Goal and Date -->
      <g transform="translate(104, 33)">
          <text class="rm-goal" fill="var(--text)">{title}</text>
          <text y="30" class="rm-date" fill="var(--text-muted)">{date}</text>
      </g>

      {status_chip}
      
      <!-- Details Section -->
      <g transform="translate(70, 122)">
          <path d="M0 28 C0 12.536 12.536 0 28 0 H912 C927.464 0 940 12.536 940 28 V{db_h_v} C940 {db_h_c1} 927.464 {db_h_f} 912 {db_h_f} H28 C12.536 {db_h_f} 0 {db_h_c2} 0 {db_h_v} V28Z"
                fill="url(#{dglass_id})" stroke="url(#{glass_stroke_id})" stroke-width="1.4" aria-hidden="true"/>
          <text x="40" y="42" class="rm-detail-head" fill="var(--primary)" aria-hidden="true">Deliverables & Scope</text>
          <g role="list" aria-label="Deliverables">
{details}
          </g>
      </g>
    </g>
"##,
            y_offset = y_offset,
            glass_id = glass_id,
            glass_stroke_id = glass_stroke_id,
            shadow_id = shadow_id,
            badge_glow_id = badge_glow_id,
            card_sw = card_stroke_width,
            phase = escape(&step.phase),
            badge_fill = badge_fill,
            title = escape(&step.title),
            status = status_str,
            date = escape(&step.date),
            dglass_id = detail_glass_id,
            db_h_v = detail_box_height - 28,
            db_h_c1 = detail_box_height - 12,
            db_h_c2 = detail_box_height - 12,
            db_h_f = detail_box_height,
            details = details_svg,
            status_chip = status_chip
        ));
    }

    let svg = format!(
        r##"<svg width="900" height="{disp_h}" viewBox="0 0 1200 {total_h}" xmlns="http://www.w3.org/2000/svg" id="{chart_id}" class="release-container{extra_class}" role="graphics-document document" aria-labelledby="title_{chart_id} desc_{chart_id}">
  <title id="title_{chart_id}">{title}</title>
  <desc id="desc_{chart_id}">Release strategy roadmap for {title} showing milestones and implementation details.</desc>
  <defs>
    <!-- Background Gradients -->
    <linearGradient id="premiumBg_{chart_id}" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="var(--bg-top)"/>
        <stop offset="42%" stop-color="var(--bg-mid)"/>
        <stop offset="72%" stop-color="var(--bg-low)"/>
        <stop offset="100%" stop-color="var(--bg)"/>
    </linearGradient>

    <radialGradient id="{blue_glow}" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="#5ac8fa" stop-opacity="0.6"/>
        <stop offset="42%" stop-color="#007aff" stop-opacity="0.2"/>
        <stop offset="100%" stop-color="#007aff" stop-opacity="0"/>
    </radialGradient>

    <radialGradient id="{violet_glow}" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="#af52de" stop-opacity="0.4"/>
        <stop offset="48%" stop-color="#bf5af2" stop-opacity="0.16"/>
        <stop offset="100%" stop-color="#bf5af2" stop-opacity="0"/>
    </radialGradient>

    <radialGradient id="{cyan_glow}" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="#64d2ff" stop-opacity="0.34"/>
        <stop offset="48%" stop-color="#5ac8fa" stop-opacity="0.14"/>
        <stop offset="100%" stop-color="#5ac8fa" stop-opacity="0"/>
    </radialGradient>

    <!-- Glass Effects -->
    <linearGradient id="{glass_id}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="var(--glass-bg)" stop-opacity="0.8"/>
        <stop offset="46%" stop-color="var(--glass-bg)" stop-opacity="0.5"/>
        <stop offset="100%" stop-color="var(--glass-bg)" stop-opacity="0.3"/>
    </linearGradient>

    <linearGradient id="{dglass_id}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="var(--glass-bg)" stop-opacity="0.5"/>
        <stop offset="100%" stop-color="var(--glass-bg)" stop-opacity="0.2"/>
    </linearGradient>

    <linearGradient id="{glass_stroke_id}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="#ffffff" stop-opacity="0.95"/>
        <stop offset="55%" stop-color="#ffffff" stop-opacity="0.4"/>
        <stop offset="100%" stop-color="#94a3b8" stop-opacity="0.2"/>
    </linearGradient>

    <linearGradient id="{highlight_id}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="#ffffff" stop-opacity="0.8"/>
        <stop offset="100%" stop-color="#ffffff" stop-opacity="0"/>
    </linearGradient>

    <!-- Timeline -->
    <linearGradient id="{spine_grad}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="#007aff" stop-opacity="0.1"/>
        <stop offset="30%" stop-color="#5ac8fa" stop-opacity="0.4"/>
        <stop offset="62%" stop-color="#af52de" stop-opacity="0.2"/>
        <stop offset="100%" stop-color="#007aff" stop-opacity="0.1"/>
    </linearGradient>

    <!-- Badge gradients -->
    <linearGradient id="{bgrad}" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="#64d2ff"/>
        <stop offset="48%" stop-color="#007aff"/>
        <stop offset="100%" stop-color="#5856d6"/>
    </linearGradient>
    <linearGradient id="{ggrad}" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="#7ee787"/>
        <stop offset="48%" stop-color="#34c759"/>
        <stop offset="100%" stop-color="#0a7f3f"/>
    </linearGradient>
    <linearGradient id="{ograd}" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="#ffd60a"/>
        <stop offset="52%" stop-color="#ff9f0a"/>
        <stop offset="100%" stop-color="#ff7a00"/>
    </linearGradient>
    <linearGradient id="{rgrad}" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="#ff9f8a"/>
        <stop offset="48%" stop-color="#ff3b30"/>
        <stop offset="100%" stop-color="#d70015"/>
    </linearGradient>

    <!-- Shadows -->
    <filter id="{shadow_id}" x="-18%" y="-35%" width="136%" height="180%">
        <feDropShadow dx="0" dy="30" stdDeviation="34" flood-color="#0f172a" flood-opacity="0.12"/>
        <feDropShadow dx="0" dy="10" stdDeviation="12" flood-color="#0f172a" flood-opacity="0.08"/>
        <feDropShadow dx="0" dy="1" stdDeviation="1" flood-color="#ffffff" flood-opacity="0.8"/>
    </filter>

    <filter id="{badge_glow}" x="-60%" y="-60%" width="220%" height="220%">
        <feDropShadow dx="0" dy="14" stdDeviation="16" flood-color="#007aff" flood-opacity="0.24"/>
        <feDropShadow dx="0" dy="4" stdDeviation="6" flood-color="#0f172a" flood-opacity="0.16"/>
    </filter>

    <filter id="{blur_filter}">
        <feGaussianBlur stdDeviation="38"/>
    </filter>
  </defs>
  <style>
    #{chart_id} {{
      --bg: #ffffff;
      --bg-top: #f8fbff;
      --bg-mid: #eef6ff;
      --bg-low: #f8f5ff;
      --text: #111827;
      --text-muted: #64748b;
      --primary: #007aff;
      --glass-bg: #ffffff;
    }}
    @media (prefers-color-scheme: dark) {{
      #{chart_id} {{
        --bg: #0f172a;
        --bg-top: #1e293b;
        --bg-mid: #0f172a;
        --bg-low: #1e1b4b;
        --text: #f8fafc;
        --text-muted: #94a3b8;
        --primary: #0a84ff;
        --glass-bg: #1e293b;
      }}
    }}
    #{chart_id}.dark-mode {{
        --bg: #0f172a;
        --bg-top: #1e293b;
        --bg-mid: #0f172a;
        --bg-low: #1e1b4b;
        --text: #f8fafc;
        --text-muted: #94a3b8;
        --primary: #0a84ff;
        --glass-bg: #1e293b;
    }}
    #{chart_id} text {{ font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", Inter, system-ui, sans-serif; }}
    #{chart_id} .rm-title {{ font-size: 46px; font-weight: 850; letter-spacing: -0.05em; }}
    #{chart_id} .rm-subtitle {{ font-size: 18px; font-weight: 600; letter-spacing: -0.01em; }}
    #{chart_id} .rm-badge {{ font-family: "SF Mono", "JetBrains Mono", monospace; font-size: 21px; font-weight: 850; }}
    #{chart_id} .rm-goal {{ font-size: 25px; font-weight: 820; letter-spacing: -0.03em; }}
    #{chart_id} .rm-date {{ font-size: 16px; font-weight: 600; }}
    #{chart_id} .rm-chip {{ font-size: 12px; font-weight: 700; }}
    #{chart_id} .rm-detail-head {{ font-size: 14px; font-weight: 800; text-transform: uppercase; letter-spacing: 0.05em; }}
    #{chart_id} .rm-details {{ font-size: 16px; font-weight: 500; fill: var(--text-muted); }}
  </style>

  <rect width="1200" height="{total_h}" fill="url(#premiumBg_{chart_id})" rx="34"/>
  
  <circle cx="970" cy="80" r="470" fill="url(#{blue_glow})"/>
  <circle cx="110" cy="660" r="430" fill="url(#{violet_glow})"/>
  <circle cx="1080" cy="1530" r="520" fill="url(#{cyan_glow})"/>
  <circle cx="610" cy="1040" r="620" fill="#ffffff" opacity="0.2" filter="url(#{blur_filter})"/>

  <!-- Subtle glass veil -->
  <rect x="28" y="28" width="1144" height="{veil_h}" rx="30" fill="#ffffff" opacity="0.15" stroke="#ffffff" stroke-opacity="0.3"/>
  
  {spine}

  <g transform="translate(80, 72)">
    <text x="0" y="0" class="rm-title" fill="var(--text)">{title}</text>
    {subtitle_svg}
  </g>

{steps_svg}
</svg>"##,
        chart_id = chart_id,
        shadow_id = shadow_id,
        badge_glow = badge_glow_id,
        glass_id = glass_id,
        glass_stroke_id = glass_stroke_id,
        dglass_id = detail_glass_id,
        highlight_id = highlight_id,
        spine_grad = spine_grad_id,
        blue_glow = blue_glow_id,
        violet_glow = violet_glow_id,
        cyan_glow = cyan_glow_id,
        blur_filter = blur_filter_id,
        bgrad = blue_grad,
        ggrad = green_grad,
        ograd = orange_grad,
        rgrad = red_grad,
        total_h = total_height,
        veil_h = total_height - 56,
        disp_h = total_height * 3 / 4,
        title = escape(title),
        subtitle_svg = if subtitle.is_empty() {
            String::new()
        } else {
            format!(
                r##"<text x="2" y="34" class="rm-subtitle" fill="var(--text-muted)">{}</text>"##,
                escape(subtitle)
            )
        },
        spine = spine_svg,
        steps_svg = steps_svg,
        extra_class = extra_class
    );

    Ok(svg)
}

fn parse_release_body(body: &str) -> Result<(HashMap<String, String>, Vec<ReleaseStep>), String> {
    let trimmed = body.trim();
    if !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("body must start and end with '----'".into());
    }
    let inner = &trimmed[4..trimmed.len() - 4];
    let parts: Vec<&str> = inner.splitn(2, "---").collect();
    if parts.len() != 2 {
        return Err("missing '---' separator".into());
    }

    let config = parse_kv_header(parts[0]);
    let mut steps = Vec::new();

    for line in parts[1].lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let segments: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if segments.len() >= 3 {
            let phase = segments[0].to_string();
            let date = segments[1].to_string();
            let status = segments[2].to_string();
            let title = if segments.len() >= 4 {
                segments[3].to_string()
            } else {
                phase.clone()
            };

            let mut details = Vec::new();
            if segments.len() > 4 {
                for detail in segments[4..].iter() {
                    details.push(detail.to_string());
                }
            }

            steps.push(ReleaseStep {
                phase,
                date,
                status,
                title,
                details,
            });
        }
    }

    if steps.is_empty() {
        return Err("no release steps found".into());
    }

    Ok((config, steps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_release_body() {
        let body = r#"----
title=Test Release
---
M1 | 2023-01-15 | Complete | Planning | Step 1 | Step 2
RC1 | 2023-03-15 | In Progress | Testing | QA | Security
GA | 2023-04-15 | Upcoming | Launch
----"#;
        let (config, steps) = parse_release_body(body).unwrap();
        assert_eq!(config.get("title").unwrap(), "Test Release");
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].phase, "M1");
        assert_eq!(steps[0].status, "Complete");
        assert_eq!(steps[0].details.len(), 2);
        assert_eq!(steps[1].phase, "RC1");
        assert_eq!(steps[1].status, "In Progress");
    }

    #[test]
    fn test_render_release() {
        let body = r#"----
title=Test Release
subtitle=Horizontal Roadmap
---
M1 | 2023-01-15 | Complete | Planning | Step 1
RC1 | 2023-03-15 | In Progress | Testing
----"#;
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("Test Release"));
        assert!(svg.contains("Horizontal Roadmap"));
        assert!(svg.contains("M1"));
        assert!(svg.contains("RC1"));
        assert!(svg.contains("pbg_release_h_"));
        assert!(svg.contains("rtg_release_h_"));
        assert!(svg.contains("viewBox=\"0 0 1760 520\""));
        assert!(svg.contains("height=\"520\""));
        assert!(svg.contains("pulse-ring"));
        assert!(svg.contains("y1=\"258\""));
        assert!(svg.contains("translate("));
        assert!(svg.contains("154)"));
        assert!(svg.contains("Testing"));
        assert!(svg.contains("NOW"));
    }

    #[test]
    fn test_render_release_vertical() {
        let body = r#"----
title=Vertical Roadmap
subtitle=Premium Design
layout=vertical
---
M1 | 2023-Q1 | Complete | Phase 1 | Detail 1
M2 | 2023-Q2 | In Progress | Phase 2 | Detail 2 | Detail 3
----"#;
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("Vertical Roadmap"));
        assert!(svg.contains("Premium Design"));
        assert!(svg.contains("viewBox=\"0 0 1200 1020\"")); // 200 + 2*380 + 60 = 1020
        assert!(svg.contains("rm-badge"));
        assert!(svg.contains("rm-goal"));
        assert!(svg.contains("M1"));
        assert!(svg.contains("Phase 1"));
        assert!(svg.contains("Detail 2"));
        assert!(svg.contains("var(--primary)"));
        assert!(svg.contains("In Progress"));
        assert!(svg.contains("<title id=\"title_"));
    }
}
