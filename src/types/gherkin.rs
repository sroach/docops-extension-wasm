use std::collections::HashMap;
use crate::common::svg::escape;
use uuid::Uuid;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
enum StepType {
    Given,
    When,
    Then,
    And,
    But,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
    Passing,
    Failing,
    Pending,
    Skipped,
}

struct Step {
    step_type: StepType,
    text: String,
    #[allow(dead_code)]
    status: Status,
}

struct Examples {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

struct Scenario {
    title: String,
    steps: Vec<Step>,
    status: Status,
    #[allow(dead_code)]
    outline: bool,
    examples: Option<Examples>,
}

struct GherkinSpec {
    feature: String,
    scenarios: Vec<Scenario>,
    theme: String,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let spec = parse_gherkin(body)?;
    Ok(render_svg(&spec, controls))
}

fn parse_gherkin(body: &str) -> Result<GherkinSpec, String> {
    let trimmed = body.trim();
    if !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("Gherkin body must be wrapped in '---- ... ----'".into());
    }
    let inner = trimmed[4..trimmed.len() - 4].trim();

    let (config_part, gherkin_part) = if let Some((c, g)) = inner.split_once("---") {
        (c.trim(), g.trim())
    } else {
        ("", inner)
    };

    let mut theme = "premium".to_string();
    for line in config_part.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == "theme" {
                theme = v.trim().to_string();
            }
        }
    }

    let mut feature_title = "Feature".to_string();
    let mut scenarios = Vec::new();
    
    let mut current_scenario_title: Option<String> = None;
    let mut current_steps = Vec::new();
    let mut current_outline = false;
    let mut in_examples = false;
    let mut example_headers: Option<Vec<String>> = None;
    let mut example_rows = Vec::new();

    let feature_regex = Regex::new(r"(?i)^Feature\s*:\s*(.*)$").unwrap();
    let scenario_regex = Regex::new(r"(?i)^Scenario(?: Outline)?\s*:\s*(.*)$").unwrap();
    let examples_regex = Regex::new(r"(?i)^Examples\s*:\s*$").unwrap();
    let step_regex = Regex::new(r"(?i)^(Given|When|Then|And|But)\s*:?\s*(.*)$").unwrap();

    let status_regex = Regex::new(r"^\[(PASSING|FAILING|PENDING|SKIPPED)\]\s*(.*)$").unwrap();

    let mut flush_scenario = |title: Option<String>, steps: Vec<Step>, mut status: Status, outline: bool, headers: Option<Vec<String>>, rows: Vec<Vec<String>>| {
        if let Some(mut t) = title {
            if let Some(caps) = status_regex.captures(&t) {
                status = match caps.get(1).unwrap().as_str() {
                    "PASSING" => Status::Passing,
                    "FAILING" => Status::Failing,
                    "PENDING" => Status::Pending,
                    "SKIPPED" => Status::Skipped,
                    _ => status,
                };
                t = caps.get(2).unwrap().as_str().to_string();
            }

            let examples = if outline && headers.is_some() {
                Some(Examples {
                    headers: headers.unwrap(),
                    rows: rows,
                })
            } else {
                None
            };
            scenarios.push(Scenario {
                title: t,
                steps: steps,
                status: status,
                outline: outline,
                examples: examples,
            });
        }
    };

    let mut current_scenario_status = Status::Passing;

    for line in gherkin_part.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        if let Some(caps) = feature_regex.captures(line) {
            feature_title = caps.get(1).unwrap().as_str().trim().to_string();
        } else if let Some(caps) = scenario_regex.captures(line) {
            flush_scenario(
                current_scenario_title.take(),
                std::mem::take(&mut current_steps),
                current_scenario_status,
                current_outline,
                example_headers.take(),
                std::mem::take(&mut example_rows),
            );
            current_outline = line.to_lowercase().starts_with("scenario outline");
            current_scenario_title = Some(caps.get(1).unwrap().as_str().trim().to_string());
            current_scenario_status = Status::Passing;
            in_examples = false;
        } else if examples_regex.is_match(line) {
            in_examples = true;
        } else if in_examples && line.starts_with('|') {
            let cells: Vec<String> = line.trim_matches('|').split('|').map(|s| s.trim().to_string()).collect();
            if example_headers.is_none() {
                example_headers = Some(cells);
            } else {
                example_rows.push(cells);
            }
        } else if let Some(caps) = step_regex.captures(line) {
            let keyword = caps.get(1).unwrap().as_str().to_lowercase();
            let mut text = caps.get(2).unwrap().as_str().trim().to_string();
            let mut step_status = Status::Passing;
            
            if let Some(s_caps) = status_regex.captures(&text) {
                step_status = match s_caps.get(1).unwrap().as_str() {
                    "PASSING" => Status::Passing,
                    "FAILING" => Status::Failing,
                    "PENDING" => Status::Pending,
                    "SKIPPED" => Status::Skipped,
                    _ => Status::Passing,
                };
                text = s_caps.get(2).unwrap().as_str().to_string();
            }

            let step_type = match keyword.as_str() {
                "given" => StepType::Given,
                "when" => StepType::When,
                "then" => StepType::Then,
                "and" => StepType::And,
                "but" => StepType::But,
                _ => StepType::Given,
            };
            current_steps.push(Step {
                step_type: step_type,
                text: text,
                status: step_status,
            });
        }
    }

    flush_scenario(
        current_scenario_title.take(),
        current_steps,
        current_scenario_status,
        current_outline,
        example_headers,
        example_rows,
    );

    Ok(GherkinSpec {
        feature: feature_title,
        scenarios: scenarios,
        theme: theme,
    })
}

fn wrap_text(text: &str, max_width_px: i32, font_size_px: i32) -> Vec<String> {
    if text.is_empty() || max_width_px <= 0 {
        return vec![text.to_string()];
    }
    let avg_char_width = font_size_px as f32 * 0.6;
    let max_chars = (max_width_px as f32 / avg_char_width).max(1.0) as usize;
    
    let words = text.split_whitespace();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else {
            if current_line.len() + 1 + word.len() <= max_chars {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

fn render_svg(spec: &GherkinSpec, controls: &HashMap<String, String>) -> String {
    let _theme = &spec.theme;
    let use_dark = controls.get("useDark").map(|s| s == "true").unwrap_or(false);
    let id = Uuid::new_v4().simple().to_string()[..8].to_string();
    let id_full = format!("gherkin_{}", id);
    
    let canvas_width = 800;
    let padding = 40;
    let inner_width = canvas_width - (padding * 2);
    
    // Feature Header Height
    let feature_lines = wrap_text(&format!("FEATURE: {}", spec.feature), inner_width - 160, 27);
    let feature_line_height = 32;
    let feature_bg_height = (feature_lines.len() as i32 * feature_line_height + 64).max(96);

    let mut y_offset = padding + feature_bg_height + 30;
    let mut scenarios_svg = String::new();

    for (idx, scenario) in spec.scenarios.iter().enumerate() {
        let (scenario_svg, scenario_height) = render_scenario(scenario, y_offset, idx + 1, inner_width, use_dark, &id);
        scenarios_svg.push_str(&scenario_svg);
        y_offset += scenario_height + 40;
    }

    let total_height = y_offset + 20;
    let extra_class = if use_dark { " dark-mode" } else { "" };

    format!(
        r##"<svg width="{canvas_width}" height="{total_height}" viewBox="0 0 {canvas_width} {total_height}" xmlns="http://www.w3.org/2000/svg" id="{id_full}" class="gherkin-container{extra_class}">
    <defs>
        <style>
            #{id_full} {{
                --bg: #F8FAFC;
                --surface: #FFFFFF;
                --text: #0F172A;
                --text-soft: #64748B;
                --blue: #007AFF;
                --green: #34C759;
                --orange: #FF9500;
                --accent: #4361ee;
                --card-sheen-0: #FFFFFF;
                --card-sheen-1: #F8FAFC;
                --bg-gradient-0: #F8FAFC;
                --bg-gradient-1: #F9FAFB;
                --shadow-color: #0F172A;
                --shadow-opacity: 0.11;
                --border: rgba(148, 163, 184, 0.24);
                --table-header-bg: #f8fafc;
                --table-row-bg: #ffffff;
                --table-border: #4361ee;
            }}
            text {{ font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", Inter, system-ui, sans-serif; }}
            .eyebrow {{ font-size: 11px; font-weight: 700; letter-spacing: 0.12em; fill: var(--blue); }}
            .feature-title {{ font-size: 27px; font-weight: 800; letter-spacing: -0.03em; fill: var(--text); }}
            .scenario-title {{ font-size: 14px; font-weight: 760; letter-spacing: 0.035em; fill: var(--text); }}
            .scenario-index {{ font-size: 11px; font-weight: 800; fill: #FFFFFF; }}
            .status-text {{ font-size: 10px; font-weight: 800; letter-spacing: 0.06em; }}
            .step-keyword {{ font-size: 14px; font-weight: 780; }}
            .step-text {{ font-size: 14px; font-weight: 450; fill: var(--text); }}
            .step-subtle {{ stroke: var(--text-soft); stroke-width: 1.4; stroke-linecap: round; stroke-dasharray: 2 7; opacity: 0.3; }}
            .card-outline {{ stroke: var(--border); stroke-width: 1; }}
            .table-header {{ font-size: 11px; font-weight: bold; fill: var(--text); }}
            .table-cell {{ font-size: 11px; fill: var(--text); }}
            
            @media (prefers-color-scheme: dark) {{
                #{id_full} {{
                    --bg: #0F172A;
                    --surface: #1E293B;
                    --text: #F8FAFC;
                    --card-sheen-0: #1E293B;
                    --card-sheen-1: #0F172A;
                    --bg-gradient-0: #0F172A;
                    --bg-gradient-1: #1E293B;
                    --shadow-color: #000000;
                    --shadow-opacity: 0.3;
                    --border: rgba(255, 255, 255, 0.1);
                    --table-header-bg: #1e293b;
                    --table-row-bg: #0f172a;
                    --table-border: #334155;
                }}
            }}
            #{id_full}.dark-mode {{
                --bg: #0F172A;
                --surface: #1E293B;
                --text: #F8FAFC;
                --card-sheen-0: #1E293B;
                --card-sheen-1: #0F172A;
                --bg-gradient-0: #0F172A;
                --bg-gradient-1: #1E293B;
                --shadow-color: #000000;
                --shadow-opacity: 0.3;
                --border: rgba(255, 255, 255, 0.1);
                --table-header-bg: #1e293b;
                --table-row-bg: #0f172a;
                --table-border: #334155;
            }}
        </style>
        <linearGradient id="bgGradient_{id}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--bg-gradient-0)"/>
            <stop offset="100%" stop-color="var(--bg-gradient-1)"/>
        </linearGradient>
        <linearGradient id="premiumAccent_{id}" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="#007AFF"/>
            <stop offset="100%" stop-color="#34C759"/>
        </linearGradient>
        <linearGradient id="cardSheen_{id}" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--card-sheen-0)" stop-opacity="0.96"/>
            <stop offset="100%" stop-color="var(--card-sheen-1)" stop-opacity="0.84"/>
        </linearGradient>
        <filter id="softCardShadow_{id}" x="-20%" y="-20%" width="140%" height="150%">
            <feDropShadow dx="0" dy="18" stdDeviation="22" flood-color="var(--shadow-color)" flood-opacity="var(--shadow-opacity)"/>
        </filter>
    </defs>
    <rect width="{canvas_width}" height="{total_height}" fill="url(#bgGradient_{id})" rx="26"/>
    
    <!-- Feature Header -->
    <g transform="translate(40, 40)" filter="url(#softCardShadow_{id})">
        <rect width="{inner_width}" height="{feature_bg_height}" rx="24" fill="url(#cardSheen_{id})" class="card-outline"/>
        <g transform="translate(24, 22)">
            <rect width="44" height="44" rx="14" fill="url(#premiumAccent_{id})"/>
            <path d="M17 24.5 L22 29.5 L31 18.5" fill="none" stroke="#FFFFFF" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
        </g>
        <text x="84" y="38" class="eyebrow">FEATURE</text>
        {feature_text_svg}
    </g>

    {scenarios_svg}
</svg>"##,
        canvas_width = canvas_width,
        total_height = total_height,
        inner_width = inner_width,
        feature_bg_height = feature_bg_height,
        id = id,
        id_full = id_full,
        extra_class = extra_class,
        feature_text_svg = feature_lines.iter().enumerate().map(|(i, line)| {
            format!(r##"<text x="84" y="{}" class="feature-title">{}</text>"##, 72 + (i as i32 * feature_line_height), escape(line))
        }).collect::<Vec<_>>().join("\n"),
        scenarios_svg = scenarios_svg
    )
}

fn render_scenario(scenario: &Scenario, y: i32, index: usize, width: i32, use_dark: bool, id: &str) -> (String, i32) {
    let title_lines = wrap_text(&format!("SCENARIO: {}", scenario.title.to_uppercase()), width - 180, 14);
    let title_line_height = 18;
    
    // Steps Calculation
    let mut step_data = Vec::new();
    for step in &scenario.steps {
        let lines = wrap_text(&step.text, width - 120, 14);
        let h = (lines.len() as i32 * 22).max(28);
        step_data.push((step, lines, h));
    }
    
    let total_steps_height = if !step_data.is_empty() {
        step_data.iter().map(|(_, _, h)| h + 12).sum::<i32>() - 12
    } else {
        0
    };

    let examples_height = if let Some(ex) = &scenario.examples {
        30 + (ex.rows.len() as i32 * 25) + 20
    } else {
        0
    };

    let scenario_height = 88 + total_steps_height + examples_height + 20;

    let mut sb = String::new();
    sb.push_str(&format!(r##"<g transform="translate(40, {y})" filter="url(#softCardShadow_{id})">"##, y = y, id = id));
    sb.push_str(&format!(r##"<rect width="{width}" height="{scenario_height}" rx="26" fill="url(#cardSheen_{id})" class="card-outline"/>"##, width = width, scenario_height = scenario_height, id = id));
    
    // Index Badge
    let idx_str = if index < 10 { format!("0{}", index) } else { index.to_string() };
    sb.push_str(&format!(r##"
        <g transform="translate(24, 24)">
            <rect width="34" height="34" rx="12" fill="url(#premiumAccent_{id})"/>
            <text x="17" y="22" class="scenario-index" text-anchor="middle">{idx_str}</text>
        </g>
    "##, id = id, idx_str = idx_str));

    // Title
    for (idx, line) in title_lines.iter().enumerate() {
        sb.push_str(&format!(r##"<text x="72" y="{}" class="scenario-title">{}</text>"##, 46 + (idx as i32 * title_line_height), escape(line)));
    }

    // Status Badge
    let status_colors = get_status_colors(scenario.status, use_dark);
    sb.push_str(&format!(r##"
        <g transform="translate({status_x}, 24)">
            <rect width="78" height="28" rx="14" fill="{bg}" stroke="{stroke}" stroke-width="1"/>
            <circle cx="17" cy="14" r="5" fill="{text_c}"/>
            <text x="48" y="18" class="status-text" text-anchor="middle" style="fill: {text_c}">{status_name}</text>
        </g>
    "##, status_x = width - 102, bg = status_colors.0, stroke = status_colors.1, text_c = status_colors.2, status_name = format!("{:?}", scenario.status).to_uppercase()));

    // Steps
    let steps_group_y = 88;
    sb.push_str(&format!(r##"<g transform="translate(35, {steps_group_y})">"##, steps_group_y = steps_group_y));
    if step_data.len() > 1 {
        sb.push_str(&format!(r##"<path d="M0 16 L0 {}" class="step-subtle"/>"##, total_steps_height - 16));
    }

    let mut current_step_y = 0;
    for (step, lines, h) in step_data {
        let style = get_step_style(step.step_type, use_dark);
        sb.push_str(&format!(r##"<g transform="translate(0, {current_step_y})">"##, current_step_y = current_step_y));
        sb.push_str(&format!(r##"<rect x="-14" y="-14" width="28" height="28" rx="10" fill="{}" stroke="{}"/>"##, style.bg, style.stroke));
        if style.is_stroke {
            sb.push_str(&format!(r##"<path d="{}" fill="none" stroke="{}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>"##, style.icon_path, style.icon_color));
        } else {
            sb.push_str(&format!(r##"<path d="{}" fill="{}"/>"##, style.icon_path, style.icon_color));
        }

        let keyword = format!("{:?}", step.step_type);
        for (l_idx, line) in lines.iter().enumerate() {
            let content = if l_idx == 0 {
                format!(r##"<tspan class="step-keyword" fill="{}">{}</tspan> {}"##, style.icon_color, keyword, escape(line))
            } else {
                escape(line)
            };
            sb.push_str(&format!(r##"<text x="26" y="5" dy="{}" class="step-text">{}</text>"##, l_idx as i32 * 22, content));
        }
        sb.push_str("</g>");
        current_step_y += h + 12;
    }
    sb.push_str("</g>");

    // Examples
    if let Some(ex) = &scenario.examples {
        sb.push_str(&render_examples(ex, width, steps_group_y + current_step_y + 10, use_dark));
    }

    sb.push_str("</g>");
    (sb, scenario_height)
}

fn get_status_colors(status: Status, use_dark: bool) -> (String, String, String) {
    match status {
        Status::Passing => {
            if use_dark { ("#064e3b".into(), "#059669".into(), "#34d399".into()) }
            else { ("#dcfce7".into(), "#16a34a".into(), "#15803d".into()) }
        },
        Status::Failing => {
            if use_dark { ("#450a0a".into(), "#dc2626".into(), "#f87171".into()) }
            else { ("#fee2e2".into(), "#dc2626".into(), "#b91c1c".into()) }
        },
        _ => {
            if use_dark { ("#1e293b".into(), "#475569".into(), "#94a3b8".into()) }
            else { ("#f1f5f9".into(), "#94a3b8".into(), "#475569".into()) }
        }
    }
}

struct StepStyle {
    bg: String,
    stroke: String,
    icon_color: String,
    icon_path: String,
    is_stroke: bool,
}

fn get_step_style(st: StepType, use_dark: bool) -> StepStyle {
    match st {
        StepType::Given => StepStyle { 
            bg: if use_dark { "#1e293b".into() } else { "#EEF6FF".into() }, 
            stroke: if use_dark { "#3b82f6".into() } else { "#BFDBFE".into() }, 
            icon_color: "#007AFF".into(), 
            icon_path: "M-5 0 L-1 4 L7 -6".into(), 
            is_stroke: true 
        },
        StepType::When => StepStyle { 
            bg: if use_dark { "#451a03".into() } else { "#FFF7ED".into() }, 
            stroke: if use_dark { "#d97706".into() } else { "#FED7AA".into() }, 
            icon_color: "#FF9500".into(), 
            icon_path: "M-2 -8 L6 0 L1 0 L3 8 L-6 -1 L-1 -1Z".into(), 
            is_stroke: false 
        },
        StepType::Then => StepStyle { 
            bg: if use_dark { "#064e3b".into() } else { "#ECFDF5".into() }, 
            stroke: if use_dark { "#059669".into() } else { "#BBF7D0".into() }, 
            icon_color: "#34C759".into(), 
            icon_path: "M-6 0 L-1 5 L8 -6".into(), 
            is_stroke: true 
        },
        _ => StepStyle { 
            bg: if use_dark { "#1e293b".into() } else { "#F8FAFC".into() }, 
            stroke: if use_dark { "#475569".into() } else { "#CBD5E1".into() }, 
            icon_color: "#64748B".into(), 
            icon_path: "M-5 0 L5 0 M0 -5 L0 5".into(), 
            is_stroke: true 
        },
    }
}

fn render_examples(examples: &Examples, width: i32, y: i32, _use_dark: bool) -> String {
    let cell_width = (width - 60) / (examples.headers.len() as i32).max(1);
    let row_height = 25;
    let mut sb = String::new();
    sb.push_str(&format!(r##"<g transform="translate(45, {y})">"##, y = y));
    sb.push_str(r##"<text x="0" y="-10" font-size="10" font-weight="bold" fill="var(--accent)" style="text-transform: uppercase; letter-spacing: 0.05em;">EXAMPLES:</text>"##);
    
    // Headers
    for (i, h) in examples.headers.iter().enumerate() {
        sb.push_str(&format!(r##"
            <rect x="{x}" y="0" width="{cell_width}" height="{row_height}" fill="var(--table-header-bg)" stroke="var(--table-border)" stroke-width="1"/>
            <text x="{text_x}" y="17" class="table-header" text-anchor="middle">{text}</text>
        "##, x = i as i32 * cell_width, cell_width = cell_width, row_height = row_height, text_x = i as i32 * cell_width + cell_width / 2, text = escape(h)));
    }
    
    // Rows
    for (r_idx, row) in examples.rows.iter().enumerate() {
        let row_y = (r_idx + 1) as i32 * row_height;
        for (c_idx, cell) in row.iter().enumerate() {
            sb.push_str(&format!(r##"
                <rect x="{x}" y="{row_y}" width="{cell_width}" height="{row_height}" fill="var(--table-row-bg)" stroke="var(--table-border)" stroke-width="0.5"/>
                <text x="{text_x}" y="{text_y}" class="table-cell" text-anchor="middle">{text}</text>
            "##, x = c_idx as i32 * cell_width, row_y = row_y, cell_width = cell_width, text_x = c_idx as i32 * cell_width + cell_width / 2, text_y = row_y + 17, text = escape(cell)));
        }
    }
    sb.push_str("</g>");
    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gherkin() {
        let body = r##"----
theme=premium
---
Feature: User Authentication
  Scenario: Successful Login
    Given the user is on the login page
    When they enter valid credentials
    Then they should be redirected to dashboard
----"##;
        let spec = parse_gherkin(body).unwrap();
        assert_eq!(spec.feature, "User Authentication");
        assert_eq!(spec.scenarios.len(), 1);
        assert_eq!(spec.scenarios[0].title, "Successful Login");
        assert_eq!(spec.scenarios[0].steps.len(), 3);
        assert_eq!(spec.scenarios[0].steps[0].step_type, StepType::Given);
    }

    #[test]
    fn test_parse_outline() {
        let body = r##"----
---
Scenario Outline: eating
  Given there are <start> cucumbers
  Examples:
    | start |
    |  12   |
----"##;
        let spec = parse_gherkin(body).unwrap();
        assert_eq!(spec.scenarios.len(), 1);
        assert!(spec.scenarios[0].outline);
        assert!(spec.scenarios[0].examples.is_some());
    }

    #[test]
    fn test_dark_mode_rendering() {
        let body = r##"----
theme=premium
---
Feature: Dark Mode
  Scenario Outline: Check Dark Mode
    Given dark mode is <status>
    Examples:
      | status  |
      | enabled |
----"##;
        let mut controls = HashMap::new();
        controls.insert("useDark".to_string(), "true".to_string());
        
        let svg = render(body, &controls).unwrap();
        
        // Check for dark mode class
        assert!(svg.contains("class=\"gherkin-container dark-mode\""));
        // Check for CSS variable overrides in dark-mode class
        assert!(svg.contains("--bg: #0F172A;"));
        // Check for gradient referencing CSS variables
        assert!(svg.contains("stop-color=\"var(--bg-gradient-0)\""));
        // Check for dynamic status colors
        assert!(svg.contains("fill=\"#064e3b\"")); // Status Passing BG in dark mode
        
        // Check Examples colors
        assert!(svg.contains("fill=\"var(--table-header-bg)\""));
        assert!(svg.contains("fill=\"var(--table-row-bg)\""));
        assert!(svg.contains("class=\"table-header\""));
        assert!(svg.contains("class=\"table-cell\""));
        assert!(!svg.contains("fill=\"var(--text)\"")); // Should be in CSS, not as attribute on text
    }
}
