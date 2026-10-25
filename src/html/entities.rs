fn entity_to_char(name: &str) -> Option<char> {
    if let Some(hex) = name.strip_prefix("#x") {
        return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
    }
    if let Some(dec) = name.strip_prefix('#') {
        return dec.parse::<u32>().ok().and_then(char::from_u32);
    }
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{a0}'),
        "copy" => Some('©'),
        _ => None,
    }
}

pub fn decode_entities(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }

        // collect the name: letters, digits and '#', peeking so we don't eat the char after it
        let mut name = String::new();
        while let Some(&next) = chars.peek() {
            if next.is_ascii_alphanumeric() || next == '#' {
                name.push(next);
                chars.next();
            } else {
                break;
            }
        }

        if chars.peek() == Some(&';') && let Some(ch) = entity_to_char(&name) {
            chars.next();      // consume the ';'
            out.push(ch);
            continue;          // done with this entity, next character
        }

        // not a valid entity: put it back literally
        out.push('&');
        out.push_str(&name);
    }

    out
}
