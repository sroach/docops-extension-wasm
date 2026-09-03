use regex::Regex;
use std::collections::HashMap;

/// Result of parsing the `---- key=value ... --- Label | value ... ----`
/// grammar. Bar charts and pie charts share this exact body syntax and only
/// differ in how they *render* the parsed data — so they both call
/// `parse_kv_body` instead of each having their own copy of this logic.
pub struct KvBody {
    pub config: HashMap<String, String>,
    pub points: Vec<(String, f64)>,
}

pub fn parse_kv_body(body: &str) -> Result<KvBody, String> {
    let trimmed = body.trim();

    if trimmed.len() < 8 {
        return Err("body too short — expected '---- ... ----'".into());
    }
    if !trimmed.starts_with("----") || !trimmed.ends_with("----") {
        return Err("body must start and end with '----'".into());
    }

    let inner = &trimmed[4..trimmed.len() - 4];
    let parts: Vec<&str> = inner.splitn(2, "---").collect();
    if parts.len() != 2 {
        return Err("missing '---' separator between header and data".into());
    }
    let (header_str, data_str) = (parts[0], parts[1]);

    // key=value pairs; each value runs from its '=' to the start of the
    // next "word=" match (or end of string). regex crate has no lookahead,
    // so this is done as two passes over match positions rather than one
    // clever pattern.
    let key_re = Regex::new(r"(\w+)=").unwrap();
    let key_matches: Vec<(String, usize, usize)> = key_re
        .captures_iter(header_str)
        .map(|cap| {
            let m = cap.get(0).unwrap();
            (cap[1].to_string(), m.start(), m.end())
        })
        .collect();

    let mut config = HashMap::new();
    for (i, (key, _start, value_start)) in key_matches.iter().enumerate() {
        let value_end = key_matches
            .get(i + 1)
            .map(|next| next.1)
            .unwrap_or(header_str.len());
        let value = header_str[*value_start..value_end].trim().to_string();
        config.insert(key.clone(), value);
    }

    // Label | value pairs; label runs up to " | ", value is a number token.
    let point_re = Regex::new(r"([A-Za-z][A-Za-z0-9 ]*?)\s*\|\s*(-?\d+(?:\.\d+)?)").unwrap();
    let mut points = Vec::new();
    for cap in point_re.captures_iter(data_str) {
        let label = cap[1].trim().to_string();
        let value: f64 = cap[2]
            .parse()
            .map_err(|_| "bad number in data".to_string())?;
        points.push((label, value));
    }

    if points.is_empty() {
        return Err("no data points found".into());
    }

    Ok(KvBody { config, points })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_header_and_points() {
        let input = "---- title=Monthly Sales theme=premium --- Jan | 10.0 Feb | 20.0 ----";
        let kv = parse_kv_body(input).unwrap();
        assert_eq!(kv.config.get("title").unwrap(), "Monthly Sales");
        assert_eq!(kv.config.get("theme").unwrap(), "premium");
        assert_eq!(
            kv.points,
            vec![("Jan".to_string(), 10.0), ("Feb".to_string(), 20.0)]
        );
    }

    #[test]
    fn rejects_missing_separator() {
        let input = "---- title=X no data here ----";
        assert!(parse_kv_body(input).is_err());
    }
}