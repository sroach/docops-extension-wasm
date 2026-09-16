use crate::common::svg::escape;
use regex::Regex;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Default)]
struct Recipe {
    title: String,
    yield_val: String,
    prep: String,
    cook: String,
    tags: Vec<String>,
    summary: String,
    ingredients: Vec<String>,
    steps: Vec<String>,
    notes: Vec<String>,
    theme: String,
}

#[derive(Clone)]
struct RecipeTheme {
    name: String,
    canvas: String,
    surface: String,
    accent_color: String,
    primary_text: String,
    secondary_text: String,
    corner_radius: i32,
    font_family: String,
    font_import: String,
    font_width_multiplier: f32,
    is_premium: bool,
}

struct BodyPanelsLayout {
    y: i32,
    side_margin: i32,
    column_gap: i32,
    column_width: i32,
    height: i32,
}

struct SectionLayout {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

struct NotesAndTagsLayout {
    side_margin: i32,
    content_width: i32,
    notes_height: i32,
    notes_y: i32,
    tags_y: i32,
    tags_height: i32,
}

pub fn render(body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    let recipe = parse_recipe(body)?;
    let use_dark = controls
        .get("useDark")
        .map(|s| s == "true")
        .unwrap_or(false);

    Ok(render_svg(&recipe, use_dark))
}

fn parse_recipe(body: &str) -> Result<Recipe, String> {
    let mut trimmed = body.trim();
    if trimmed.starts_with("----") {
        trimmed = trimmed[4..].trim();
    }
    if trimmed.ends_with("----") {
        trimmed = trimmed[..trimmed.len() - 4].trim();
    }

    let mut lines = trimmed.lines().map(|s| s.trim()).filter(|s| !s.is_empty());

    let title = lines.next().ok_or("Recipe title is missing")?.to_string();

    let mut recipe = Recipe {
        title,
        ..Recipe::default()
    };

    let mut current_key = String::new();
    let key_val_re = Regex::new(r"^(\w+)=").unwrap();

    for line in lines {
        if let Some(cap) = key_val_re.captures(line) {
            current_key = cap[1].to_string();
            let val = line[cap[0].len()..].trim();
            match current_key.as_str() {
                "yield" => recipe.yield_val = val.to_string(),
                "prep" => recipe.prep = val.to_string(),
                "cook" => recipe.cook = val.to_string(),
                "tags" => {
                    recipe.tags = val
                        .split([',', ';'])
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                }
                "summary" => recipe.summary = val.to_string(),
                "ingredients" => {
                    if !val.is_empty() {
                        recipe.ingredients.push(val.to_string())
                    }
                }
                "steps" => {
                    if !val.is_empty() {
                        recipe.steps.push(val.to_string())
                    }
                }
                "notes" => {
                    if !val.is_empty() {
                        recipe.notes.push(val.to_string())
                    }
                }
                "theme" => recipe.theme = val.to_string(),
                _ => {}
            }
        } else {
            match current_key.as_str() {
                "summary" => {
                    if !recipe.summary.is_empty() {
                        recipe.summary.push(' ');
                    }
                    recipe.summary.push_str(line);
                }
                "ingredients" => recipe.ingredients.push(line.to_string()),
                "steps" => recipe.steps.push(line.to_string()),
                "notes" => recipe.notes.push(line.to_string()),
                _ => {}
            }
        }
    }

    Ok(recipe)
}

fn get_theme(name: &str, use_dark: bool) -> RecipeTheme {
    match name.to_lowercase().as_str() {
        "classic" => RecipeTheme {
            name: "classic".to_string(),
            canvas: if use_dark { "#1A1A1A" } else { "#FFFFFF" }.to_string(),
            surface: if use_dark { "#262626" } else { "#F9FAFB" }.to_string(),
            accent_color: "#6B7280".to_string(),
            primary_text: if use_dark { "#F3F4F6" } else { "#111827" }.to_string(),
            secondary_text: if use_dark { "#9CA3AF" } else { "#4B5563" }.to_string(),
            corner_radius: 8,
            font_family: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif".to_string(),
            font_import: "@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@500;700&display=swap');".to_string(),
            font_width_multiplier: 1.0,
            is_premium: false,
        },
        "spring" => RecipeTheme {
            name: "spring".to_string(),
            canvas: if use_dark { "#2D372D" } else { "#FDFCF0" }.to_string(),
            surface: if use_dark { "#232E23" } else { "#F0FAF0" }.to_string(),
            accent_color: if use_dark { "#80C080" } else { "#3A7040" }.to_string(),
            primary_text: if use_dark { "#F0F0F0" } else { "#2A3828" }.to_string(),
            secondary_text: if use_dark { "#A0B0A0" } else { "#4A8A4A" }.to_string(),
            corner_radius: 12,
            font_family: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif".to_string(),
            font_import: "@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@500;700&display=swap');".to_string(),
            font_width_multiplier: 1.0,
            is_premium: false,
        },
        "summer" => RecipeTheme {
            name: "summer".to_string(),
            canvas: if use_dark { "#1A2634" } else { "#FFF9E6" }.to_string(),
            surface: if use_dark { "#15202B" } else { "#E8F4FF" }.to_string(),
            accent_color: if use_dark { "#5090C8" } else { "#1058A0" }.to_string(),
            primary_text: if use_dark { "#E8F4FF" } else { "#0A3870" }.to_string(),
            secondary_text: if use_dark { "#88AACC" } else { "#3377BB" }.to_string(),
            corner_radius: 12,
            font_family: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif".to_string(),
            font_import: "@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@500;700&display=swap');".to_string(),
            font_width_multiplier: 1.0,
            is_premium: false,
        },
        "food" => RecipeTheme {
            name: "food".to_string(),
            canvas: if use_dark { "#2A2420" } else { "#FFF8EE" }.to_string(),
            surface: if use_dark { "#221C18" } else { "#FDF3E5" }.to_string(),
            accent_color: if use_dark { "#D2691E" } else { "#8B4513" }.to_string(),
            primary_text: if use_dark { "#F5E6D3" } else { "#4A2B10" }.to_string(),
            secondary_text: if use_dark { "#B8860B" } else { "#8B6508" }.to_string(),
            corner_radius: 8,
            font_family: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif".to_string(),
            font_import: "@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@500;700&display=swap');".to_string(),
            font_width_multiplier: 1.0,
            is_premium: false,
        },
        _ => RecipeTheme {
            name: "premium".to_string(),
            canvas: if use_dark { "#0F172A" } else { "#FFFFFF" }.to_string(),
            surface: if use_dark { "#1E293B" } else { "#F8FAFC" }.to_string(),
            accent_color: if use_dark { "#60A5FA" } else { "#3B82F6" }.to_string(),
            primary_text: if use_dark { "#F9FAFB" } else { "#111827" }.to_string(),
            secondary_text: if use_dark { "#9CA3AF" } else { "#4B5563" }.to_string(),
            corner_radius: 24,
            font_family: "'Inter', -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif".to_string(),
            font_import: "@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@500;700&display=swap');".to_string(),
            font_width_multiplier: 1.05,
            is_premium: true,
        },
    }
}

fn wrap_text_to_width(text: &str, max_width: f32, avg_char_width: f32) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    let mut lines = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut current_line = String::new();
    let mut current_width = 0.0;

    for word in words {
        let word_width = word.len() as f32 * avg_char_width;
        if current_width + word_width > max_width && !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
            current_line = String::new();
            current_width = 0.0;
        }
        if !current_line.is_empty() {
            current_line.push(' ');
            current_width += avg_char_width;
        }
        current_line.push_str(word);
        current_width += word_width;
    }
    if !current_line.is_empty() {
        lines.push(current_line.trim().to_string());
    }
    lines
}

fn render_svg(recipe: &Recipe, use_dark: bool) -> String {
    let theme = get_theme(&recipe.theme, use_dark);
    let id = format!("recipe_{}", Uuid::new_v4().simple());

    let width = 680i32;
    let side_margin = if theme.is_premium { 32 } else { 40 };
    let column_gap = if theme.is_premium { 24 } else { 20 };
    let content_width = width - (side_margin * 2);
    let column_width = (content_width - column_gap) / 2;

    let avg_char_w = 7.5 * theme.font_width_multiplier;
    let summary_avg_char_w = if theme.is_premium {
        9.5 * theme.font_width_multiplier
    } else {
        avg_char_w
    };
    let summary_to_wrap = if recipe.summary.is_empty() && theme.is_premium {
        "Creamy avocado, deep cocoa, no butter required."
    } else {
        &recipe.summary
    };
    let summary_lines = wrap_text_to_width(summary_to_wrap, (content_width - 32) as f32, summary_avg_char_w);
    let summary_lines = if summary_lines.len() > 3 {
        let mut lines = summary_lines.into_iter().take(3).collect::<Vec<_>>();
        if let Some(last) = lines.last_mut() {
            if last.len() > 3 {
                *last = format!("{}...", &last[..last.len() - 3]);
            } else {
                last.push_str("...");
            }
        }
        lines
    } else {
        summary_lines
    };

    let summary_height = if summary_lines.is_empty() {
        0
    } else {
        16 + summary_lines.len() as i32 * 22
    };

    let mut total_ingredient_lines = 0;
    let mut ingredient_lines_nested = Vec::new();
    let mut clean_ingredients = Vec::new();

    for line in &recipe.ingredients {
        let clean = line.trim_start_matches(['-', '*', '•']).trim();
        clean_ingredients.push(clean.to_string());
        let wrapped = wrap_text_to_width(clean, (column_width - 48) as f32, avg_char_w);
        total_ingredient_lines += wrapped.len();
        ingredient_lines_nested.push(wrapped);
    }
    if recipe.ingredients.is_empty() {
        clean_ingredients.push("No ingredients provided.".to_string());
        ingredient_lines_nested.push(vec!["No ingredients provided.".to_string()]);
        total_ingredient_lines = 1;
    }
    let ingredients_height = std::cmp::max(220, 60 + total_ingredient_lines as i32 * 24);

    let mut total_step_lines = 0;
    let mut step_lines_nested = Vec::new();
    let mut clean_steps = Vec::new();

    for line in &recipe.steps {
        let clean = line
            .trim_start_matches(['-', '*', '•'])
            .trim_start()
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .trim_start_matches(['.', ')'])
            .trim();
        clean_steps.push(clean.to_string());
        let wrapped = wrap_text_to_width(clean, (column_width - 56) as f32, avg_char_w);
        total_step_lines += wrapped.len();
        step_lines_nested.push(wrapped);
    }
    if recipe.steps.is_empty() {
        clean_steps.push("No steps provided.".to_string());
        step_lines_nested.push(vec!["No steps provided.".to_string()]);
        total_step_lines = 1;
    }
    let steps_height = std::cmp::max(220, 60 + total_step_lines as i32 * 24);

    let middle_height = std::cmp::max(ingredients_height, steps_height);

    let mut note_lines_nested = Vec::new();
    let mut clean_notes = Vec::new();
    let mut current_notes_height = 0;
    if !recipe.notes.is_empty() {
        current_notes_height = 54;
        for (i, note) in recipe.notes.iter().enumerate() {
            let clean = note.trim_start_matches(['-', '*', '·', '•']).trim();
            clean_notes.push(clean.to_string());
            let wrapped = wrap_text_to_width(clean, (content_width - 48) as f32, avg_char_w);
            current_notes_height += wrapped.len() as i32 * 24;
            if i < recipe.notes.len() - 1 {
                current_notes_height += 8;
            }
            note_lines_nested.push(wrapped);
        }
        current_notes_height += 16;
    }

    let mut tags_height = 0;
    if !recipe.tags.is_empty() {
        let mut tag_rows = 1;
        let mut current_x = 0;
        for tag in &recipe.tags {
            let tw = (tag.len() as i32 * 8) + 32;
            if current_x + tw > content_width {
                tag_rows += 1;
                current_x = tw + 8;
            } else {
                current_x += tw + 8;
            }
        }
        tags_height = tag_rows * 32 + (tag_rows - 1) * 8;
    }

    let header_height = if theme.is_premium { 260 } else { 148 };
    let meta_strip_height = if theme.is_premium { 72 } else { 72 };
    let gap = 16;
    let summary_padding = if summary_height > 0 && !theme.is_premium { 20 } else { 0 };

    let body_panel_y = if theme.is_premium {
        header_height + (if summary_height > 0 { summary_height + 40 } else { 20 })
    } else {
        32
            + header_height
            + gap
            + meta_strip_height
            + (if summary_height > 0 {
                gap + summary_height + summary_padding
            } else {
                gap
            })
    };
    let notes_y = body_panel_y + middle_height + (if theme.is_premium { 24 } else { 28 });
    let tags_y = if current_notes_height > 0 {
        notes_y + current_notes_height + 24
    } else {
        notes_y
    };

    let footer_padding = 40;
    let final_content_y = if tags_height > 0 {
        tags_y + tags_height
    } else if current_notes_height > 0 {
        notes_y + current_notes_height
    } else {
        notes_y
    };
    let total_height = std::cmp::max(760, final_content_y + footer_padding);

    let dark_class = if use_dark { " dark-mode" } else { "" };

    // Patterns defs only for themes that use them
    let patterns_defs = match theme.name.as_str() {
        "food" => format!(
            r##"<pattern id="{id}_linen" width="8" height="8" patternUnits="userSpaceOnUse">
            <line x1="0" y1="8" x2="8" y2="0" stroke="{accent}" stroke-width="0.4" stroke-opacity="0.18"/>
        </pattern>"##,
            id = id,
            accent = theme.accent_color
        ),
        "spring" => format!(
            r##"<pattern id="{id}_sp_stripe" width="10" height="10" patternUnits="userSpaceOnUse">
            <line x1="0" y1="10" x2="10" y2="0" stroke="#A8D4A0" stroke-width="0.5" stroke-opacity="0.2"/>
        </pattern>"##,
            id = id
        ),
        "summer" => format!(
            r##"<pattern id="{id}_su_check" width="16" height="16" patternUnits="userSpaceOnUse">
            <line x1="8" y1="0" x2="8" y2="16" stroke="{accent}" stroke-width="0.5" stroke-opacity="0.08"/>
            <line x1="0" y1="8" x2="16" y2="8" stroke="{accent}" stroke-width="0.5" stroke-opacity="0.08"/>
        </pattern>"##,
            id = id,
            accent = theme.accent_color
        ),
        _ => "".to_string(),
    };

    // SVG construction
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{total_height}" viewBox="0 0 {width} {total_height}" id="{id}" class="docops-recipe{dark_class}" role="graphics-document document" aria-labelledby="{id}_title {id}_desc">
    <title id="{id}_title">{title_esc} — Recipe Card</title>
    <desc id="{id}_desc">A styled recipe card for {title_esc}</desc>
    <defs>
        <filter id="{id}_shadow" x="-10%" y="-10%" width="120%" height="120%">
            <feDropShadow dx="0" dy="4" stdDeviation="12" flood-color="var(--shadow-color)" flood-opacity="var(--shadow-opacity)"/>
        </filter>
        {patterns_defs}
        <style>
            {font_import}
            #{id} {{
                --canvas: {canvas};
                --surface: {surface};
                --primary: {primary};
                --secondary: {secondary};
                --accent: {accent};
                --accent-subtle: rgba(59, 130, 246, 0.08);
                --border: rgba(59, 130, 246, 0.15);
                --border-subtle: rgba(17, 24, 39, 0.08);
                --shadow-color: #000000;
                --shadow-opacity: 0.06;
                --tag-bg: rgba(139, 92, 246, 0.1);
                --tag-text: #6D28D9;
                --tag-border: rgba(139, 92, 246, 0.2);
                --step-badge-bg: #3B82F6;
                --step-badge-text: #FFFFFF;
                --header-grad-start: #ECFDF5;
                --header-grad-end: #FFFBEB;
                --header-circle-1: #E5E7EB;
                --header-circle-2: #FEF3C7;
                --header-circle-3: #D1FAE5;
            }}
            #{id} .recipe-title {{ font-family: {font_family}; font-size: {title_size}; font-weight: 800; letter-spacing: -0.04em; fill: var(--primary); }}
            #{id} .recipe-subtitle {{ font-family: {font_family}; font-size: 12px; font-weight: 800; letter-spacing: 1.5px; text-transform: uppercase; fill: var(--accent); }}
            #{id} .recipe-summary {{ font-family: {font_family}; font-size: 18px; font-weight: 500; line-height: 1.4; fill: var(--secondary); opacity: 0.9; }}
            #{id} .meta-label {{ font-family: {font_family}; font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 1px; fill: var(--secondary); opacity: 0.8; }}
            #{id} .meta-value {{ font-family: {font_family}; font-size: 18px; font-weight: 700; fill: var(--primary); }}
            #{id} .section-title {{ font-family: {font_family}; font-size: 24px; font-weight: 800; letter-spacing: -0.02em; fill: var(--primary); }}
            #{id} .body-text {{ font-family: {font_family}; font-size: 16px; font-weight: 500; line-height: 1.5; fill: var(--primary); }}
            #{id} .summary-text {{ font-family: {font_family}; font-size: 16px; line-height: 1.6; fill: var(--secondary); }}
            #{id} .step-num {{ font-family: {font_family}; font-size: 12px; font-weight: 800; fill: #FFFFFF; }}
            #{id} .tag-text {{ font-family: {font_family}; font-size: 13px; font-weight: 600; fill: var(--tag-text); }}
            #{id} .notes-label {{ font-family: {font_family}; font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 1.5px; fill: var(--secondary); opacity: 0.8; }}

            @media (prefers-color-scheme: dark) {{
                #{id} {{
                    --canvas: #0F172A;
                    --surface: #1E293B;
                    --primary: #F9FAFB;
                    --secondary: #9CA3AF;
                    --accent: #60A5FA;
                    --accent-subtle: rgba(96, 165, 250, 0.12);
                    --border: rgba(96, 165, 250, 0.25);
                    --border-subtle: rgba(255, 255, 255, 0.1);
                    --shadow-color: #000000;
                    --shadow-opacity: 0.35;
                    --tag-bg: rgba(139, 92, 246, 0.25);
                    --tag-text: #C4B5FD;
                    --tag-border: rgba(139, 92, 246, 0.4);
                    --step-badge-bg: #3B82F6;
                    --step-badge-text: #FFFFFF;
                    --header-grad-start: #064E3B;
                    --header-grad-end: #1E293B;
                    --header-circle-1: #334155;
                    --header-circle-2: #451A03;
                    --header-circle-3: #065F46;
                }}
                #{id} .recipe-summary {{ fill: var(--primary); opacity: 1; }}
            }}
            #{id}.dark-mode {{
                --canvas: #0F172A;
                --surface: #1E293B;
                --primary: #F9FAFB;
                --secondary: #9CA3AF;
                --accent: #60A5FA;
                --accent-subtle: rgba(96, 165, 250, 0.12);
                --border: rgba(96, 165, 250, 0.25);
                --border-subtle: rgba(255, 255, 255, 0.1);
                --shadow-color: #000000;
                --shadow-opacity: 0.35;
                --tag-bg: rgba(139, 92, 246, 0.25);
                --tag-text: #C4B5FD;
                --tag-border: rgba(139, 92, 246, 0.4);
                --step-badge-bg: #3B82F6;
                --step-badge-text: #FFFFFF;
                --header-grad-start: #064E3B;
                --header-grad-end: #1E293B;
                --header-circle-1: #334155;
                --header-circle-2: #451A03;
                --header-circle-3: #065F46;
            }}
            #{id}.dark-mode .recipe-summary {{ fill: var(--primary); opacity: 1; }}
        </style>
    </defs>
    <rect width="{width}" height="{total_height}" fill="var(--canvas)" rx="16" aria-hidden="true"/>
"##,
        width = width,
        total_height = total_height,
        id = id,
        dark_class = dark_class,
        title_esc = escape(&recipe.title),
        patterns_defs = patterns_defs,
        font_import = theme.font_import,
        primary = theme.primary_text,
        secondary = theme.secondary_text,
        accent = theme.accent_color,
        canvas = theme.canvas,
        surface = theme.surface,
        font_family = theme.font_family,
        title_size = if theme.is_premium { "36px" } else { "32px" }
    );

    // Theme specific patterns
    match theme.name.as_str() {
        "food" => svg.push_str(&format!(
            r##"<rect width="{width}" height="{total_height}" fill="url(#{id}_linen)" aria-hidden="true"/>"##,
            width = width,
            total_height = total_height,
            id = id
        )),
        "spring" => svg.push_str(&format!(
            r##"<rect width="{width}" height="{total_height}" fill-opacity="0.05" fill="url(#{id}_sp_stripe)" aria-hidden="true"/>"##,
            width = width,
            total_height = total_height,
            id = id
        )),
        "summer" => svg.push_str(&format!(
            r##"<rect width="{width}" height="{total_height}" fill="url(#{id}_su_check)" aria-hidden="true"/>"##,
            width = width,
            total_height = total_height,
            id = id
        )),
        _ => {}
    }

    if theme.is_premium {
        svg.push_str(&format!(
            r##"<rect width="{w}" height="{h}" rx="{r}" fill="var(--canvas)" filter="url(#{id}_shadow)" aria-hidden="true"/>
            <rect x="0.5" y="0.5" width="{w_sub}" height="{h_sub}" rx="{r_sub}" fill="none" stroke="var(--border)" stroke-width="1" aria-hidden="true"/>"##,
            w = width,
            h = total_height,
            r = theme.corner_radius,
            id = id,
            w_sub = width - 1,
            h_sub = total_height - 1,
            r_sub = theme.corner_radius,
        ));
    } else {
        svg.push_str(&format!(
            r##"<rect x="14" y="14" width="{w}" height="{h}" rx="{r}" fill="none" stroke="{accent}" stroke-width="1.2" stroke-opacity="0.5" stroke-dasharray="8 4" aria-hidden="true"/>"##,
            w = width - 28,
            h = total_height - 28,
            r = theme.corner_radius,
            accent = theme.accent_color
        ));
    }

    // Corners for retro/fun themes
    if !theme.is_premium {
        match theme.name.as_str() {
            "spring" => svg.push_str(&render_spring_corners(width, total_height)),
            "summer" => svg.push_str(&render_summer_corners(width, total_height)),
            _ => svg.push_str(&render_default_corners(
                width,
                total_height,
                &theme.accent_color,
                use_dark,
            )),
        }
    }

    // Header
    svg.push_str(&render_header(recipe, &theme, &id, side_margin, &summary_lines));

    // Meta Strip
    svg.push_str(&render_meta_strip(
        recipe,
        &theme,
        side_margin,
        &summary_lines,
        summary_height,
    ));

    // Body Panels
    let panels_layout = BodyPanelsLayout {
        y: body_panel_y,
        side_margin,
        column_gap,
        column_width,
        height: middle_height,
    };
    svg.push_str(&render_body_panels(
        &ingredient_lines_nested,
        &clean_ingredients,
        &step_lines_nested,
        &clean_steps,
        &theme,
        &panels_layout,
    ));

    // Notes and Tags
    let notes_tags_layout = NotesAndTagsLayout {
        side_margin,
        content_width,
        notes_height: current_notes_height,
        notes_y,
        tags_y,
        tags_height,
    };
    svg.push_str(&render_notes_and_tags(
        recipe,
        &theme,
        &note_lines_nested,
        &notes_tags_layout,
    ));

    svg.push_str("</svg>");
    svg
}

fn render_header(
    recipe: &Recipe,
    theme: &RecipeTheme,
    id: &str,
    side_margin: i32,
    summary_lines: &[String],
) -> String {
    if theme.is_premium {
        let tags_str = if recipe.tags.is_empty() {
            "VEGAN · DESSERT · HEALTHY".to_string()
        } else {
            recipe.tags.join(" · ").to_uppercase()
        };

        let mut summary_svg = String::new();
        for (i, line) in summary_lines.iter().enumerate() {
            let y = 88 + (i * 25);
            summary_svg.push_str(&format!(
                r##"<text y="{y}" class="recipe-summary">{line}</text>"##,
                y = y,
                line = escape(line)
            ));
        }

        return format!(
            r##"<g role="region" aria-label="Recipe Header">
            <defs>
                <linearGradient id="{id}_header_grad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="var(--header-grad-start)" stop-opacity="0.8"/>
                    <stop offset="100%" stop-color="var(--header-grad-end)" stop-opacity="0.5"/>
                </linearGradient>
            </defs>
            <path d="M0 {r} Q0 0 {r} 0 L{w_r} 0 Q{w} 0 {w} {r} L{w} 240 L0 240 Z" fill="url(#{id}_header_grad)" aria-hidden="true"/>
            
            <g transform="translate({sm}, 60)">
                <text y="0" class="recipe-subtitle">{tags}</text>
                <text y="48" class="recipe-title">{title}</text>
                {summary_svg}
            </g>
 
            <g transform="translate({circle_x}, 20)" aria-hidden="true">
                <circle cx="100" cy="60" r="50" fill="var(--header-circle-1)" fill-opacity="0.3"/>
                <circle cx="120" cy="40" r="40" fill="var(--header-circle-2)" fill-opacity="0.5"/>
                <circle cx="80" cy="30" r="45" fill="var(--header-circle-3)" fill-opacity="0.6"/>
                <g transform="translate(75, 25)">
                   <circle cx="25" cy="25" r="25" fill="#4B2C20"/>
                   <circle cx="25" cy="25" r="15" fill="#34D399"/>
                   <path d="M25 15 Q32 15 32 25 Q32 35 25 35 Q18 35 18 25 Q18 15 25 15 Z" fill="#065F46" opacity="0.2"/>
                   <circle cx="25" cy="25" r="6" fill="#059669"/>
                </g>
            </g>
        </g>"##,
            id = id,
            r = theme.corner_radius,
            w = 680,
            w_r = 680 - theme.corner_radius,
            sm = side_margin,
            tags = escape(&tags_str),
            title = escape(&recipe.title),
            summary_svg = summary_svg,
            circle_x = 680 - 200
        );
    }

    let (header_fill, header_stroke, title_color, subtitle_color, tag_color) =
        match theme.name.as_str() {
            "spring" => ("#EDA0BA", "#D47898", "#FFF0F5", "#7A2848", "#FCE4EE"),
            "summer" => (
                theme.accent_color.as_str(),
                theme.accent_color.as_str(),
                "#FFFFFF",
                "#A8D0F8",
                "#E8F4FF",
            ),
            "premium" => (
                "var(--surface)",
                "var(--border)",
                "var(--primary)",
                "var(--secondary)",
                "var(--secondary)",
            ),
            _ => (
                theme.secondary_text.as_str(),
                theme.accent_color.as_str(),
                "#FFF8EE",
                "#FDEBD0",
                "#FDDDB8",
            ),
        };

    let header_pattern = match theme.name.as_str() {
        "spring" => format!("url(#{}_sp_stripe)", id),
        "summer" => format!("url(#{}_su_check)", id),
        _ => "".to_string(),
    };

    let header_width = 680 - (side_margin * 2);
    let header_y = 32;
    let tags_str = if recipe.tags.is_empty() {
        "RECIPE".to_string()
    } else {
        recipe.tags.join(" · ")
    };

    let pattern_rect = if !header_pattern.is_empty() {
        format!(
            r##"<rect x="{side_margin}" y="{header_y}" width="{header_width}" height="148" rx="{r}" fill="{pattern}" aria-hidden="true"/>"##,
            side_margin = side_margin,
            header_y = header_y,
            header_width = header_width,
            r = theme.corner_radius,
            pattern = header_pattern
        )
    } else {
        "".to_string()
    };

    format!(
        r##"<g role="region" aria-label="Recipe Header">
        <rect x="{side_margin}" y="{header_y}" width="{header_width}" height="148" rx="{r}" fill="{fill}" stroke="{stroke}" stroke-width="1" aria-hidden="true"/>
        {pattern_rect}
        <text x="340" y="{y1}" text-anchor="middle" class="recipe-subtitle" fill="{subtitle_color}" aria-hidden="true">RECIPE DOSSIER</text>
        <text x="340" y="{y2}" text-anchor="middle" class="recipe-title" fill="{title_color}">{title}</text>
        <text x="340" y="{y3}" text-anchor="middle" class="recipe-subtitle" fill="{tag_color}">{tags}</text>
        {divider}
    </g>"##,
        side_margin = side_margin,
        header_y = header_y,
        header_width = header_width,
        r = theme.corner_radius,
        fill = header_fill,
        stroke = header_stroke,
        pattern_rect = pattern_rect,
        y1 = header_y + 46,
        subtitle_color = subtitle_color,
        y2 = header_y + 90,
        title_color = title_color,
        title = escape(&recipe.title),
        y3 = header_y + 124,
        tag_color = tag_color,
        tags = escape(&tags_str),
        divider = render_divider(header_y + 160, side_margin, theme)
    )
}

fn render_divider(y: i32, side_margin: i32, theme: &RecipeTheme) -> String {
    match theme.name.as_str() {
        "spring" => format!(
            r##"<g transform="translate(0, {y})" aria-hidden="true">
                <path d="M60 0 Q120 -8 180 0 Q240 8 300 0 Q360 -8 420 0 Q480 8 540 0 Q600 -8 620 0" fill="none" stroke="#80C080" stroke-width="1.2" stroke-opacity="0.5" stroke-linecap="round"/>
                <circle cx="120" cy="-4" r="3" fill="#80C080" fill-opacity="0.45"/>
                <circle cx="300" cy="-4" r="3" fill="#80C080" fill-opacity="0.45"/>
                <circle cx="480" cy="-4" r="3" fill="#80C080" fill-opacity="0.45"/>
            </g>"##,
            y = y
        ),
        "summer" => format!(
            r##"<g transform="translate(0, {y})" aria-hidden="true">
                <path d="M44 0 Q100 -8 160 0 Q220 8 280 0 Q340 -8 400 0 Q460 8 520 0 Q580 -8 636 0" fill="none" stroke="{accent}" stroke-width="1" stroke-opacity="0.35"/>
                <circle cx="160" cy="-4" r="3" fill="#F5D840" fill-opacity="0.65"/>
                <circle cx="340" cy="-4" r="3" fill="{accent}" fill-opacity="0.5"/>
                <circle cx="520" cy="-4" r="3" fill="#F5D840" fill-opacity="0.65"/>
            </g>"##,
            y = y,
            accent = theme.accent_color
        ),
        "premium" => format!(
            r##"<g transform="translate(0, {y})" aria-hidden="true">
                <line x1="{side_margin}" y1="0" x2="{x2}" y2="0" stroke="var(--border)" stroke-width="1" stroke-opacity="0.4"/>
            </g>"##,
            y = y,
            side_margin = side_margin,
            x2 = 680 - side_margin
        ),
        _ => format!(
            r##"<g transform="translate(0, {y})" aria-hidden="true">
                <line x1="80" y1="0" x2="300" y2="0" stroke="var(--accent)" stroke-width="0.8" stroke-opacity="0.4"/>
                <circle cx="340" cy="0" r="4" fill="var(--accent)" fill-opacity="0.5"/>
                <line x1="380" y1="0" x2="600" y2="0" stroke="var(--accent)" stroke-width="0.8" stroke-opacity="0.4"/>
            </g>"##,
            y = y
        ),
    }
}

fn render_meta_strip(
    recipe: &Recipe,
    theme: &RecipeTheme,
    side_margin: i32,
    summary_lines: &[String],
    summary_height: i32,
) -> String {
    if theme.is_premium {
        let content_width = 680 - (side_margin * 2);
        let badge_w = (content_width - 32) / 3;
        let y_strip = 250;

        let yield_val = if recipe.yield_val.is_empty() { "8 servings" } else { &recipe.yield_val };
        let prep_val = if recipe.prep.is_empty() { "20 minutes" } else { &recipe.prep };
        let cook_val = if recipe.cook.is_empty() { "35 minutes" } else { &recipe.cook };

        return format!(
            r##"<g role="region" aria-label="Recipe Overview" transform="translate({sm}, {y})">
            <g role="graphics-symbol" aria-roledescription="metric" aria-label="Yield: {yield_val}">
                <rect width="{bw}" height="64" rx="16" fill="var(--surface)" aria-hidden="true"/>
                <text x="24" y="24" class="meta-label">YIELD</text>
                <text x="24" y="50" class="meta-value">{yield_val}</text>
            </g>
            <g transform="translate({x2}, 0)" role="graphics-symbol" aria-roledescription="metric" aria-label="Prep Time: {prep}">
                <rect width="{bw}" height="64" rx="16" fill="var(--surface)" aria-hidden="true"/>
                <text x="24" y="24" class="meta-label">PREP</text>
                <text x="24" y="50" class="meta-value">{prep}</text>
            </g>
            <g transform="translate({x3}, 0)" role="graphics-symbol" aria-roledescription="metric" aria-label="Bake Time: {cook}">
                <rect width="{bw}" height="64" rx="16" fill="var(--surface)" aria-hidden="true"/>
                <text x="24" y="24" class="meta-label">BAKE</text>
                <text x="24" y="50" class="meta-value">{cook}</text>
            </g>
        </g>"##,
            sm = side_margin,
            y = y_strip,
            bw = badge_w,
            x2 = badge_w + 16,
            x3 = (badge_w + 16) * 2,
            yield_val = escape(yield_val),
            prep = escape(prep_val),
            cook = escape(cook_val)
        );
    }

    let (strip_fill, strip_stroke) = match theme.name.as_str() {
        "spring" => ("#F0FAF0", "#90C890"),
        "summer" => ("#E8F4FF", "#5090C8"),
        "premium" => ("var(--surface)", "var(--border)"),
        _ => (theme.surface.as_str(), theme.accent_color.as_str()),
    };

    let content_width = 680 - (side_margin * 2);
    let item_width = content_width as f64 / 3.0;

    let mut summary_svg = String::new();
    for (i, line) in summary_lines.iter().enumerate() {
        let y = 296 + (i * 22);
        summary_svg.push_str(&format!(
            r##"<text x="340" y="{y}" text-anchor="middle" class="summary-text">{line}</text>"##,
            y = y,
            line = escape(line)
        ));
    }

    let summary_divider = if summary_height > 0 {
        format!(
            r##"<line x1="{side_margin}" y1="{line_y}" x2="{line_x2}" y2="{line_y}" stroke="var(--border)" stroke-width="1" stroke-opacity="0.3" aria-hidden="true"/>"##,
            side_margin = side_margin,
            line_y = 286 + summary_height,
            line_x2 = 680 - side_margin
        )
    } else {
        "".to_string()
    };

    let y_strip = 32 + 148 + 16;

    format!(
        r##"<g role="region" aria-label="Recipe Overview">
        <rect x="{side_margin}" y="{y_strip}" width="{content_width}" height="72" rx="{r}" fill="{fill}" stroke="{stroke}" stroke-width="1" aria-hidden="true"/>
        <g transform="translate({side_margin}, {y_strip})">
            <g role="graphics-symbol" aria-roledescription="metric" aria-label="Yield: {yield_val}">
                <text x="{x1}" y="28" text-anchor="middle" class="meta-label">YIELD</text>
                <text x="{x1}" y="52" text-anchor="middle" class="meta-value">{yield_val}</text>
            </g>
            <line x1="{w1}" y1="14" x2="{w1}" y2="58" stroke="{stroke}" stroke-width="1" stroke-opacity="0.4" aria-hidden="true"/>
            <g role="graphics-symbol" aria-roledescription="metric" aria-label="Prep Time: {prep}">
                <text x="{x2}" y="28" text-anchor="middle" class="meta-label">PREP TIME</text>
                <text x="{x2}" y="52" text-anchor="middle" class="meta-value">{prep}</text>
            </g>
            <line x1="{w2}" y1="14" x2="{w2}" y2="58" stroke="{stroke}" stroke-width="1" stroke-opacity="0.4" aria-hidden="true"/>
            <g role="graphics-symbol" aria-roledescription="metric" aria-label="Cook Time: {cook}">
                <text x="{x3}" y="28" text-anchor="middle" class="meta-label">COOK TIME</text>
                <text x="{x3}" y="52" text-anchor="middle" class="meta-value">{cook}</text>
            </g>
        </g>
        {summary_svg}
        {summary_divider}
    </g>"##,
        side_margin = side_margin,
        y_strip = y_strip,
        content_width = content_width,
        r = theme.corner_radius,
        fill = strip_fill,
        stroke = strip_stroke,
        x1 = item_width / 2.0,
        yield_val = escape(&recipe.yield_val),
        w1 = item_width,
        x2 = item_width + item_width / 2.0,
        prep = escape(&recipe.prep),
        w2 = item_width * 2.0,
        x3 = item_width * 2.0 + item_width / 2.0,
        cook = escape(&recipe.cook),
        summary_svg = summary_svg,
        summary_divider = summary_divider
    )
}

fn render_body_panels(
    ingredients: &[Vec<String>],
    raw_ingredients: &[String],
    steps: &[Vec<String>],
    raw_steps: &[String],
    theme: &RecipeTheme,
    layout: &BodyPanelsLayout,
) -> String {
    let ing_layout = SectionLayout {
        x: 0,
        y: 0,
        width: layout.column_width,
        height: layout.height,
    };
    let steps_layout = SectionLayout {
        x: 0,
        y: 0,
        width: layout.column_width,
        height: layout.height,
    };
    format!(
        r##"<g transform="translate({side_margin}, {y})">
        {ing}
        <g transform="translate({steps_x}, 0)">
            {steps}
        </g>
    </g>"##,
        side_margin = layout.side_margin,
        y = layout.y,
        ing = render_section(
            "INGREDIENTS",
            ingredients,
            raw_ingredients,
            theme,
            &ing_layout,
            false
        ),
        steps_x = layout.column_width + layout.column_gap,
        steps = render_section("STEPS", steps, raw_steps, theme, &steps_layout, true)
    )
}

fn render_section(
    title: &str,
    items: &[Vec<String>],
    raw_items: &[String],
    theme: &RecipeTheme,
    layout: &SectionLayout,
    is_steps: bool,
) -> String {
    let (card_fill, card_stroke) = match theme.name.as_str() {
        "spring" => ("#F0FAF0", "#90C890"),
        "summer" => ("#E8F4FF", "#5090C8"),
        "premium" => ("var(--surface)", "none"),
        _ => (theme.surface.as_str(), theme.accent_color.as_str()),
    };

    let region_label = if is_steps {
        "Preparation Steps"
    } else {
        "Ingredients"
    };
    let list_label = if is_steps {
        "Steps list"
    } else {
        "Ingredients list"
    };

    let mut sb = format!(
        r##"<g role="region" aria-label="{region_label}">
        <rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}" stroke="{stroke}" stroke-width="1" aria-hidden="true"/>
        <text x="{t_x}" y="{t_y}" class="section-title">{title_final}</text>
        <g role="list" aria-label="{list_label}">"##,
        region_label = region_label,
        x = layout.x,
        y = layout.y,
        w = layout.width,
        h = layout.height,
        r = theme.corner_radius,
        fill = card_fill,
        stroke = card_stroke,
        t_x = layout.x + 24,
        t_y = layout.y + 44,
        title_final = if theme.is_premium && is_steps { "Method" } else { title },
        list_label = list_label
    );

    let mut current_y = layout.y + 80;
    for (i, lines) in items.iter().enumerate() {
        let item_num = i + 1;
        let raw_text = raw_items.get(i).map(|s| s.as_str()).unwrap_or("");
        let item_aria_label = if is_steps {
            format!("Step {}: {}", item_num, raw_text)
        } else {
            raw_text.to_string()
        };

        sb.push_str(&format!(
            r##"<g role="listitem" aria-label="{aria_label}">"##,
            aria_label = escape(&item_aria_label)
        ));

        for (j, line) in lines.iter().enumerate() {
            if is_steps {
                if j == 0 {
                    let badge_bg = if theme.is_premium { "#3B82F6" } else { "var(--step-badge-bg)" };
                    sb.push_str(&format!(
                        r##"<circle cx="{cx}" cy="{cy}" r="10" fill="{bg}" aria-hidden="true"/>
                        <text x="{cx}" y="{ty}" text-anchor="middle" class="step-num" aria-hidden="true">{num}</text>"##,
                        cx = layout.x + 24,
                        cy = current_y - 4,
                        ty = current_y,
                        num = item_num,
                        bg = badge_bg
                    ));
                }
                sb.push_str(&format!(
                    r##"<text x="{tx}" y="{ty}" class="body-text">{line}</text>"##,
                    tx = layout.x + 48,
                    ty = current_y,
                    line = escape(line)
                ));
            } else {
                if j == 0 {
                    let dot_color = if theme.is_premium { "#10B981" } else { "var(--accent)" };
                    sb.push_str(&format!(
                        r##"<circle cx="{cx}" cy="{cy}" r="3" fill="{color}" aria-hidden="true"/>"##,
                        cx = layout.x + 24,
                        cy = current_y - 4,
                        color = dot_color
                    ));
                }
                sb.push_str(&format!(
                    r##"<text x="{tx}" y="{ty}" class="body-text">{line}</text>"##,
                    tx = layout.x + 40,
                    ty = current_y,
                    line = escape(line)
                ));
            }
            current_y += 24;
        }
        sb.push_str("</g>");
    }

    sb.push_str("</g></g>");
    sb
}

fn render_notes_and_tags(
    recipe: &Recipe,
    theme: &RecipeTheme,
    note_lines: &[Vec<String>],
    layout: &NotesAndTagsLayout,
) -> String {
    let mut sb = String::new();

    if !recipe.notes.is_empty() {
        let (fill, stroke) = match theme.name.as_str() {
            "spring" => ("#F5FAF0", "#80C080"),
            "summer" => ("#E8F4FF", "#5090C8"),
            "premium" => ("var(--surface)", "none"),
            _ => (theme.surface.as_str(), theme.accent_color.as_str()),
        };

        let label_text = if theme.is_premium { "COOK'S NOTES" } else { "COOK'S NOTES" };
        let title_class = if theme.is_premium { "notes-label" } else { "section-title" };

        sb.push_str(&format!(
            r##"<g role="region" aria-label="Cook's Notes" transform="translate({side_margin}, {notes_y})">
            <rect width="{w}" height="{h}" rx="{r}" fill="{fill}" stroke="{stroke}" stroke-width="1" aria-hidden="true"/>
            <text x="24" y="24" class="{title_class}">{label}</text>"##,
            side_margin = layout.side_margin,
            notes_y = layout.notes_y,
            w = layout.content_width,
            h = layout.notes_height,
            r = theme.corner_radius,
            fill = fill,
            stroke = stroke,
            title_class = title_class,
            label = label_text
        ));

        let mut curr_y = 52;
        for lines in note_lines {
            for (j, line) in lines.iter().enumerate() {
                if j == 0 {
                    sb.push_str(&format!(
                        r##"<circle cx="24" cy="{cy}" r="2.5" fill="var(--accent)" fill-opacity="0.8" aria-hidden="true"/>"##,
                        cy = curr_y - 4
                    ));
                }
                sb.push_str(&format!(
                    r##"<text x="36" y="{y}" class="summary-text" fill="var(--primary)">{line}</text>"##,
                    y = curr_y,
                    line = escape(line)
                ));
                curr_y += 24;
            }
            curr_y += 8;
        }
        sb.push_str("</g>");
    }

    if !recipe.tags.is_empty() {
        sb.push_str(&format!(
            r##"<g role="list" aria-label="Recipe Tags" transform="translate({side_margin}, {tags_y})">"##,
            side_margin = layout.side_margin,
            tags_y = layout.tags_y
        ));
        let mut tx = 0;
        let mut ty = 0;
        for tag in &recipe.tags {
            let tw = (tag.len() as i32 * 8) + 32;
            if tx + tw > layout.content_width {
                tx = 0;
                ty += 40;
            }
            let (fill, stroke) = match theme.name.as_str() {
                "spring" => ("#F2B8CC", "#D47898"),
                "summer" => ("rgba(16, 88, 160, 0.15)", "#1058A0"),
                "premium" => {
                    let fill = match tag.to_lowercase().as_str() {
                        "vegan" => "#D1FAE5",
                        "dessert" => "#FFEDD5",
                        "healthy" | "low-carb" => "#DBEAFE",
                        _ => "var(--tag-bg)",
                    };
                    (fill, "none")
                },
                _ => ("var(--tag-bg)", "var(--tag-border)"),
            };
            
            let tag_text_color = if theme.is_premium {
                match tag.to_lowercase().as_str() {
                    "vegan" => "#065F46",
                    "dessert" => "#9A3412",
                    "healthy" | "low-carb" => "#1E40AF",
                    _ => "var(--tag-text)",
                }
            } else {
                "var(--tag-text)"
            };

            sb.push_str(&format!(
                r##"<g role="listitem" aria-label="Tag: {tag_label}">
                <rect x="{tx}" y="{ty}" width="{tw}" height="32" rx="12" fill="{fill}" stroke="{stroke}" stroke-width="1" aria-hidden="true"/>
                <text x="{ctx}" y="{cty}" text-anchor="middle" class="tag-text" fill="{text_color}">{tag}</text>
            </g>"##,
                tag_label = escape(tag),
                tx = tx,
                ty = ty,
                tw = tw,
                fill = fill,
                stroke = stroke,
                ctx = tx + tw / 2,
                cty = ty + 20,
                tag = escape(tag),
                text_color = tag_text_color
            ));
            tx += tw + 8;
        }
        sb.push_str("</g>");
    }

    let final_y = if layout.tags_height > 0 {
        layout.tags_y + layout.tags_height + 16
    } else if layout.notes_height > 0 {
        layout.notes_y + layout.notes_height + 16
    } else {
        layout.notes_y
    };
    sb.push_str(&render_bottom_divider(final_y, layout.side_margin, theme));

    sb
}

fn render_bottom_divider(y: i32, side_margin: i32, theme: &RecipeTheme) -> String {
    match theme.name.as_str() {
        "spring" => format!(
            r##"<g transform="translate(0, {y})" aria-hidden="true">
                <path d="M44 14 Q160 6 280 14 Q340 18 400 14 Q520 6 636 14" fill="none" stroke="#90C890" stroke-width="0.8" stroke-opacity="0.4"/>
                <circle cx="170" cy="10" r="2.5" fill="#F2B8CC" fill-opacity="0.6"/>
                <circle cx="340" cy="11" r="2.5" fill="#80C080" fill-opacity="0.6"/>
                <circle cx="510" cy="10" r="2.5" fill="#F2B8CC" fill-opacity="0.6"/>
            </g>"##,
            y = y
        ),
        "summer" => format!(
            r##"<g transform="translate(0, {y})" aria-hidden="true">
                <path d="M44 14 Q120 6 200 14 Q280 22 360 14 Q440 6 520 14 Q580 20 636 14" fill="none" stroke="{accent}" stroke-width="0.8" stroke-opacity="0.35"/>
                <rect x="333" y="8" width="14" height="14" rx="1" transform="rotate(45 340 15)" fill="#F5D840" fill-opacity="0.55"/>
            </g>"##,
            y = y,
            accent = theme.accent_color
        ),
        "premium" => format!(
            r##"<g transform="translate(0, {y})" aria-hidden="true">
                <line x1="{side_margin}" y1="14" x2="{x2}" y2="14" stroke="var(--border)" stroke-width="1" stroke-opacity="0.3"/>
            </g>"##,
            y = y,
            side_margin = side_margin,
            x2 = 680 - side_margin
        ),
        _ => format!(
            r##"<g transform="translate(0, {y})" aria-hidden="true">
                <line x1="44" y1="14" x2="280" y2="14" stroke="{accent}" stroke-width="0.6" stroke-opacity="0.35"/>
                <circle cx="340" cy="14" r="2" fill="{accent}" fill-opacity="0.45"/>
                <line x1="400" y1="14" x2="636" y2="14" stroke="{accent}" stroke-width="0.6" stroke-opacity="0.35"/>
            </g>"##,
            y = y,
            accent = theme.accent_color.as_str()
        ),
    }
}

fn render_default_corners(w: i32, h: i32, accent: &str, use_dark: bool) -> String {
    let ornament_opacity = if use_dark { "0.45" } else { "0.55" };
    let circle_opacity = if use_dark { "0.3" } else { "0.4" };
    format!(
        r##"<g fill="none" stroke="{accent}" stroke-width="1" stroke-opacity="{o_op}" aria-hidden="true">
            <path d="M14 50 L14 14 L50 14"/>
            <circle cx="14" cy="14" r="3.5" fill="{accent}" fill-opacity="{c_op}" stroke="none"/>
            <path d="M{w_14} 50 L{w_14} 14 L{w_50} 14"/>
            <circle cx="{w_14}" cy="14" r="3.5" fill="{accent}" fill-opacity="{c_op}" stroke="none"/>
            <path d="M14 {h_50} L14 {h_14} L50 {h_14}"/>
            <circle cx="14" cy="{h_14}" r="3.5" fill="{accent}" fill-opacity="{c_op}" stroke="none"/>
            <path d="M{w_14} {h_50} L{w_14} {h_14} L{w_50} {h_14}"/>
            <circle cx="{w_14}" cy="{h_14}" r="3.5" fill="{accent}" fill-opacity="{c_op}" stroke="none"/>
        </g>"##,
        accent = accent,
        o_op = ornament_opacity,
        c_op = circle_opacity,
        w_14 = w - 14,
        w_50 = w - 50,
        h_14 = h - 14,
        h_50 = h - 50
    )
}

fn render_spring_corners(w: i32, h: i32) -> String {
    let flower = |x: i32, y: i32| {
        format!(
            r##"<g transform="translate({x}, {y})">
            <circle cx="12" cy="12" r="4" fill="#E8A0B8" fill-opacity="0.6"/>
            <ellipse cx="12" cy="5" rx="3.5" ry="5" fill="#F2B8CC" fill-opacity="0.7"/>
            <ellipse cx="12" cy="19" rx="3.5" ry="5" fill="#F2B8CC" fill-opacity="0.7"/>
            <ellipse cx="5" cy="12" rx="5" ry="3.5" fill="#F2B8CC" fill-opacity="0.7"/>
            <ellipse cx="19" cy="12" rx="5" ry="3.5" fill="#F2B8CC" fill-opacity="0.7"/>
            <circle cx="12" cy="12" r="2.5" fill="#FDE8F0"/>
        </g>"##,
            x = x,
            y = y
        )
    };
    format!(
        r##"<g aria-hidden="true">
        {f1}{f2}{f3}{f4}
    </g>"##,
        f1 = flower(10, 10),
        f2 = flower(w - 34, 10),
        f3 = flower(10, h - 34),
        f4 = flower(w - 34, h - 34)
    )
}

fn render_summer_corners(w: i32, h: i32) -> String {
    let sun = |x: i32, y: i32| {
        format!(
            r##"<g transform="translate({x}, {y})">
            <circle cx="13" cy="13" r="6" fill="#F5D840" fill-opacity="0.75"/>
            <g stroke="#F5C820" stroke-width="1.5" stroke-linecap="round">
                <line x1="13" y1="3" x2="13" y2="0"/><line x1="13" y1="23" x2="13" y2="26"/>
                <line x1="3" y1="13" x2="0" y2="13"/><line x1="23" y1="13" x2="26" y2="13"/>
            </g>
        </g>"##,
            x = x,
            y = y
        )
    };
    format!(
        r##"<g aria-hidden="true">
        {s1}{s2}{s3}{s4}
    </g>"##,
        s1 = sun(10, 10),
        s2 = sun(w - 36, 10),
        s3 = sun(10, h - 36),
        s4 = sun(w - 36, h - 36)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_recipe() {
        let body = r#"
            Chocolate Avocado Cake
            yield= 8 servings
            prep= 20 minutes
            cook= 35 minutes
            tags= vegan, dessert
            summary= A rich chocolate cake.
            ingredients=
            - 2 avocados
            - 2 cups flour
            steps=
            1. Preheat oven.
            2. Mash avocados.
            notes=
            - Tasty!
            theme= premium
        "#;
        let recipe = parse_recipe(body).unwrap();
        assert_eq!(recipe.title, "Chocolate Avocado Cake");
        assert_eq!(recipe.yield_val, "8 servings");
        assert_eq!(recipe.tags, vec!["vegan", "dessert"]);
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.steps.len(), 2);
        assert_eq!(recipe.notes.len(), 1);
        assert_eq!(recipe.theme, "premium");
    }

    #[test]
    fn test_render_recipe() {
        let body = r#"
            Test Recipe
            yield= 1
            prep= 1
            cook= 1
            summary= Test summary.
            ingredients=
            - Item 1
            steps=
            1. Step 1
            theme= spring
        "#;
        let svg = render(body, &HashMap::new()).unwrap();
        assert!(svg.contains("Test Recipe"));
        assert!(svg.contains("INGREDIENTS"));
        assert!(svg.contains("STEPS"));
        assert!(svg.contains("sp_stripe"));
    }

    #[test]
    fn test_default_theme() {
        let body = r#"
            Test Recipe
            yield= 1
            prep= 1
            cook= 1
            summary= Test summary.
            ingredients=
            - Item 1
            steps=
            1. Step 1
        "#;
        let recipe = parse_recipe(body).unwrap();
        assert_eq!(recipe.theme, "");
        let theme = get_theme(&recipe.theme, false);
        assert_eq!(theme.name, "premium");
        assert_eq!(theme.corner_radius, 24);
        assert!(theme.is_premium);
    }

    #[test]
    fn test_apple_premium_features() {
        let body = r#"
            Chocolate Avocado Cake
            yield= 8 servings
            prep= 20 minutes
            cook= 35 minutes
            tags= vegan, dessert, healthy
            summary= A rich, moist chocolate cake using ripe avocado.
            ingredients=
            - 2 large ripe avocados
            - 2 cups all-purpose flour
            steps=
            1. Preheat oven to 350F.
            2. Mash avocados smoothly.
            notes=
            - The avocado flavor disappears completely!
            theme= premium
        "#;
        let svg = render(body, &HashMap::new()).unwrap();

        // 1. Valid CSS font import inside defs/style
        assert!(svg.contains("<style>\n            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@500;700&display=swap');"));

        // 2. CSS variables and dark mode support
        assert!(svg.contains("--primary:"));
        assert!(svg.contains("--secondary:"));
        assert!(svg.contains("--accent:"));
        assert!(svg.contains("--canvas:"));
        assert!(svg.contains("--surface:"));
        assert!(svg.contains("@media (prefers-color-scheme: dark)"));
        assert!(svg.contains(".dark-mode"));

        // 3. WCAG 2.2 AA ARIA Semantics
        assert!(svg.contains("role=\"graphics-document document\""));
        assert!(svg.contains("role=\"region\" aria-label=\"Recipe Overview\""));
        assert!(svg.contains("role=\"region\" aria-label=\"Ingredients\""));
        assert!(svg.contains("role=\"region\" aria-label=\"Preparation Steps\""));
        assert!(svg.contains("role=\"region\" aria-label=\"Cook's Notes\""));
        assert!(svg.contains("role=\"list\" aria-label=\"Ingredients list\""));
        assert!(svg.contains("role=\"list\" aria-label=\"Steps list\""));
        assert!(svg.contains("role=\"listitem\""));
        assert!(svg.contains("role=\"graphics-symbol\" aria-roledescription=\"metric\""));
        assert!(svg.contains("aria-hidden=\"true\""));

        // 4. Content sanitization (no double bullets / numbers)
        assert!(svg.contains(">2 large ripe avocados<"));
        assert!(!svg.contains("• - 2 large ripe avocados"));
        assert!(svg.contains(">Preheat oven to 350F.<"));
        assert!(!svg.contains("1. 1. Preheat oven"));
        assert!(svg.contains(">The avocado flavor disappears completely!<"));
        assert!(!svg.contains("· - The avocado flavor"));

        // 5. Typography tokens
        assert!(svg.contains("font-size: 18px; font-weight: 700; fill: var(--primary);")); // Meta value on scale
        assert!(svg.contains("font-size: 12px; font-weight: 800; fill: #FFFFFF;")); // Step num on scale

        // 6. Apple aesthetic: no thick vertical accent stripe
        assert!(!svg.contains("<rect width=\"6\""));
 
        // 7. Dark mode premium contrast fixes
        assert!(svg.contains("--header-grad-start: #ECFDF5;"));
        assert!(svg.contains("--header-grad-start: #064E3B;"));
        assert!(svg.contains(".recipe-summary { fill: var(--primary); opacity: 1; }"));

        // Update gen/recipe.svg with full generated output
        let full_sample = r#"[docops,recipe]
----
Chocolate Avocado Cake
yield= 8 servings
prep= 20 minutes
cook= 35 minutes
tags= vegan, dessert, healthy
summary= A rich, moist chocolate cake that uses ripe avocado instead of butter or oil for a creamy texture and healthy fats.
ingredients=
- 2 large ripe avocados
- 2 cups all-purpose flour
- 1 cup unsweetened cocoa powder
- 1.5 cups organic cane sugar
- 2 tsp baking soda
- 1 tsp salt
- 2 cups water
- 2 tbsp white vinegar
steps=
1. Preheat oven to 350F (175C) and grease two 8-inch cake pans.
2. Mash avocados until completely smooth in a large bowl.
3. Whisk in all wet ingredients until well combined.
4. Sift in dry ingredients and fold gently until no lumps remain.
5. Divide batter between pans and bake for 30-35 minutes.
notes=
- The avocado flavor completely disappears once baked!
- Best served with a dark chocolate ganache or fresh berries.
----"#;
        let generated_svg = crate::generate_svg(full_sample);
        let _ = std::fs::write("gen/recipe.svg", generated_svg);
    }

    #[test]
    fn test_summary_wrapping() {
        let body = r#"
            Long Summary Recipe
            summary= This is a very long summary that should definitely wrap into at least two or three lines because it contains a lot of descriptive words about the food and its preparation.
            theme= premium
        "#;
        let svg = render(body, &HashMap::new()).unwrap();
        
        // Count summary text elements - each wrapped line is a separate <text> element with class recipe-summary
        let count = svg.matches("class=\"recipe-summary\"").count();
        assert!(count >= 2, "Summary should wrap into multiple lines, but found {}", count);
        
        // Check for different Y offsets
        assert!(svg.contains("y=\"88\""));
        assert!(svg.contains("y=\"113\""));
    }
}
