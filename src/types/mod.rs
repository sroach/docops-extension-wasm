use std::collections::HashMap;
pub mod badge;
pub mod bar_chart;
pub mod pie_chart;
pub mod adr;
pub mod combination_chart;
pub mod line_chart;


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
        other => Err(format!(
            "unknown visualization type '{other}' — expected one of: bar, pieslice, badge, adr, gherkin, scorecard"
        )),
    }
}