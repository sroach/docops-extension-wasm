use std::collections::HashMap;

/// The parsed `[docops, <type>, key=value, ...]` header plus whatever raw
/// text follows it. What that raw text *means* is entirely up to the
/// per-type parser in `types/` — the envelope doesn't assume any grammar
/// for the body beyond "it's the rest of the input."
pub struct Envelope<'a> {
    pub viz_type: String,
    pub controls: HashMap<String, String>,
    pub body: &'a str,
}

/// Parses the shared outer syntax:
///   [docops, <type>, key=value, key=value, ...] <body>
///
/// Examples this must handle:
///   [docops,pieslice, controls=false,useDark=true] ---- ... ----
///   [docops,badge] ---- ... ----
pub fn parse_envelope(input: &str) -> Result<Envelope<'_>, String> {
    let trimmed = input.trim();

    if !trimmed.starts_with('[') {
        return Err("expected input to start with '[docops, <type>, ...]'".into());
    }
    let close = trimmed.find(']').ok_or("missing closing ']' in header")?;
    let header = &trimmed[1..close];
    let body = trimmed[close + 1..].trim();

    if body.is_empty() {
        return Err("missing body after '[...]' header".into());
    }

    let mut fields = header.split(',').map(str::trim);

    let namespace = fields.next().unwrap_or("");
    if namespace != "docops" {
        return Err(format!("expected 'docops' namespace, got '{namespace}'"));
    }

    let viz_type = fields
        .next()
        .filter(|s| !s.is_empty())
        .ok_or("missing visualization type, e.g. '[docops,badge]'")?
        .to_string();

    let mut controls = HashMap::new();
    for field in fields {
        if field.is_empty() {
            continue;
        }
        if let Some((k, v)) = field.split_once('=') {
            controls.insert(k.trim().to_string(), v.trim().to_string());
        }
        // Bare flags with no '=' are silently ignored for now; extend here
        // if you need boolean flags like `[docops,badge,compact]`.
    }

    Ok(Envelope {
        viz_type,
        controls,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_type_and_controls() {
        let input = "[docops,pieslice, controls=false,useDark=true] ---- body ----";
        let env = parse_envelope(input).unwrap();
        assert_eq!(env.viz_type, "pieslice");
        assert_eq!(env.controls.get("controls").unwrap(), "false");
        assert_eq!(env.controls.get("useDark").unwrap(), "true");
        assert_eq!(env.body, "---- body ----");
    }

    #[test]
    fn parses_type_with_no_controls() {
        let input = "[docops,badge] ---- body ----";
        let env = parse_envelope(input).unwrap();
        assert_eq!(env.viz_type, "badge");
        assert!(env.controls.is_empty());
    }

    #[test]
    fn rejects_wrong_namespace() {
        let input = "[notdocops,badge] ---- body ----";
        assert!(parse_envelope(input).is_err());
    }
}