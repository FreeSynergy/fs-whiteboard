//! A small well-formedness check for the SVG the core writes: tags nest, attributes are quoted,
//! entities are the five predefined ones or numeric, and `<` never appears inside text or values.

/// `Ok` with the number of elements if `s` is well-formed, otherwise what is wrong.
pub fn check(s: &str) -> Result<usize, String> {
    let b = s.as_bytes();
    let (mut i, mut stack, mut count) = (0, Vec::<String>::new(), 0);
    while i < b.len() {
        if b[i] != b'<' {
            let end = s[i..].find('<').map_or(b.len(), |n| i + n);
            entities(&s[i..end])?;
            i = end;
            continue;
        }
        let end = s[i..].find('>').ok_or("unclosed tag")? + i;
        let tag = &s[i + 1..end];
        if tag.starts_with('?') {
        } else if let Some(name) = tag.strip_prefix('/') {
            if stack.pop().as_deref() != Some(name.trim()) {
                return Err(format!("mismatched close tag {name}"));
            }
        } else {
            let selfclose = tag.ends_with('/');
            let body = tag.trim_end_matches('/');
            let name = body.split_whitespace().next().ok_or("empty tag")?;
            attributes(&body[name.len()..])?;
            count += 1;
            if !selfclose {
                stack.push(name.to_string());
            }
        }
        i = end + 1;
        // a `>` inside an attribute value would have cut the tag short; the attribute check catches it
    }
    if stack.is_empty() {
        Ok(count)
    } else {
        Err(format!("unclosed: {stack:?}"))
    }
}

fn entities(text: &str) -> Result<(), String> {
    if text.contains('>') && text.contains("]]>") {
        return Err("]]> in text".into());
    }
    let mut rest = text;
    while let Some(p) = rest.find('&') {
        let semi = rest[p..].find(';').ok_or("bare &")? + p;
        let e = &rest[p + 1..semi];
        let ok = matches!(e, "amp" | "lt" | "gt" | "quot" | "apos")
            || e.strip_prefix('#').is_some_and(|n| !n.is_empty());
        if !ok {
            return Err(format!("unknown entity &{e};"));
        }
        rest = &rest[semi + 1..];
    }
    Ok(())
}

fn attributes(mut s: &str) -> Result<(), String> {
    let mut seen: Vec<String> = Vec::new();
    s = s.trim();
    while !s.is_empty() {
        let eq = s
            .find('=')
            .ok_or_else(|| format!("attribute without value: {s}"))?;
        let name = s[..eq].trim();
        if name.is_empty() || name.contains(char::is_whitespace) {
            return Err(format!("bad attribute name {name:?}"));
        }
        if seen.iter().any(|n| n == name) {
            return Err(format!("duplicate attribute {name}"));
        }
        seen.push(name.to_string());
        let rest = s[eq + 1..].trim_start();
        let quote = rest
            .chars()
            .next()
            .filter(|c| *c == '"')
            .ok_or("unquoted value")?;
        let close = rest[1..].find(quote).ok_or("unclosed value")? + 1;
        let value = &rest[1..close];
        if value.contains('<') {
            return Err("< in attribute value".into());
        }
        entities(value)?;
        s = rest[close + 1..].trim_start();
    }
    Ok(())
}
