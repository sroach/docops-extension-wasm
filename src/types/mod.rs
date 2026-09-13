use std::collections::HashMap;
pub mod badge;
pub mod bar_chart;
pub mod pie_chart;
pub mod adr;
pub mod combination_chart;
pub mod line_chart;
pub mod scorecard;
pub mod button;
pub mod quadrant_chart;
pub mod gherkin;

pub mod metrics_card;


/// The one place that knows every supported visualization type. Adding a
/// new type is: write `types/whatever.rs` with a `render(body, controls)`
/// function matching this signature, then add one arm here.
pub fn render(viz_type: &str, body: &str, controls: &HashMap<String, String>) -> Result<String, String> {
    match viz_type {
        "bar" | "barchart" => bar_chart::render(body, controls),
        "pieslice" | "piechart" | "pie" => pie_chart::render(body, controls),
        "line" | "linechart" => line_chart::render(body, controls),
        "combination" | "combo" => combination_chart::render(body, controls),
        "badge" => badge::render(body, controls),
        "adr" => adr::render(body, controls),
        "scorecard" | "score" => scorecard::render(body, controls),
        "button" => button::render(body, controls),
        "quadrant" | "magic" => quadrant_chart::render(body, controls),
        "gherkin" => gherkin::render(body, controls),
        "metrics" | "metricscard" => metrics_card::render(body, controls),
        other => Err(format!(
            "unknown visualization type '{other}' — expected one of: bar, pieslice, badge, adr, gherkin, scorecard, button, quadrant, metrics"
        )),
    }
}