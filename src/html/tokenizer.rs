use super::entities::decode_entities;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Token {
    Opening {
        name: String,
        attributes: HashMap<String, String>,
    },
    Closing(String),
    Text(String),
}

enum Mode {
    TagName,
    BetweenAttrs,
    AttrName,
    ValueStart,
    QuotedValue,
    UnquotedValue,
}

fn parse_tag(tag: &str) -> (String, HashMap<String, String>) {
    let mut mode = Mode::TagName;
    let mut name = String::new();
    let mut attr_name = String::new();
    let mut attr_value = String::new();
    let mut quote = '"';
    let mut attributes = HashMap::new();

    for c in tag.chars() {
        match mode {
            Mode::TagName => {
                if c.is_whitespace() {
                    mode = Mode::BetweenAttrs;
                } else {
                    name.push(c); 
                }
            }
            Mode::BetweenAttrs => {
                if !c.is_whitespace() {
                    attr_name.push(c);
                    mode = Mode::AttrName;
                }
            }
            Mode::AttrName => {
                if c == '=' {
                    mode = Mode::ValueStart;
                } else if c.is_whitespace() {
                    attributes.insert(std::mem::take(&mut attr_name), String::new());
                    mode = Mode::BetweenAttrs;
                } else {
                    // still reading the name
                     attr_name.push(c);
                }
            }
            Mode::ValueStart => {
                if c == '"' || c == '\'' {
                    quote = c;
                    mode = Mode::QuotedValue;
                } else {
                    attr_value.push(c);
                    mode = Mode::UnquotedValue;
                }
            }
            Mode::QuotedValue => {
                if c == quote {
                    attributes.insert(std::mem::take(&mut attr_name), decode_entities(&std::mem::take(&mut attr_value)));
                    mode = Mode::BetweenAttrs;
                } else {
                    attr_value.push(c);
                }
            }
            Mode::UnquotedValue => {
                if c.is_whitespace() {
                    attributes.insert(std::mem::take(&mut attr_name), decode_entities(&std::mem::take(&mut attr_value)));
                    mode = Mode::BetweenAttrs;
                } else {
                    attr_value.push(c);
                }
            }}
    }
    if !attr_name.is_empty() {
        attributes.insert(attr_name, decode_entities(&attr_value));
    }

    (name, attributes)
}

pub fn tokenize(html: &str) -> Vec<Token> {
    let mut in_tag = false;
    let mut in_hidden_tag = false;
    let mut tag = String::new();
    let mut tokens = Vec::new();
    let mut text = String::new();

    for c in html.chars() {
        match c {
            '<' => {
                if !text.is_empty() {
                    tokens.push(Token::Text(decode_entities(&std::mem::take(&mut text))));
                }
                in_tag = true;
                tag.clear();
            }
            '>' => {
                in_tag = false;
                let name = tag.split_whitespace().next().unwrap_or("");
                if name == "style" || name == "script" {
                    in_hidden_tag = true;
                }
                if name == "/style" || name == "/script" {
                    in_hidden_tag = false;
                }
                if tag.starts_with('!') {
                    // We dont do anything
                } else if let Some(rest) = tag.strip_prefix('/') {
                    tokens.push(Token::Closing(rest.to_string()));
                } else {
                    let (name, attributes) = parse_tag(&tag);
                    tokens.push(Token::Opening { name, attributes });
                }
            }
            _ if in_tag => tag.push(c),
            _ => {
                if !in_hidden_tag {
                    text.push(c);
                };
            }
        }
    }
    if !text.is_empty() {
         tokens.push(Token::Text(decode_entities(&std::mem::take(&mut text))));
    }

    tokens
}
