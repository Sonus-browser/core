use std::collections::HashMap;

#[derive(Debug)]
pub struct Rule {
    pub selector: Vec<Selector>,
    pub declarations: HashMap<String, String>,
}

enum Mode {
    Selector,
    Property,
    Value,
}

#[derive(Debug)]
pub enum Selector {
    Tag(String),
    Class(String),
    Id(String),
}

pub fn parse_selectors(text: &str) -> Vec<Selector> {
    let split_text = text.split(',');
    let mut selectors = Vec::new();
    for selector in split_text {
        let selector = selector.trim();
        if let Some(rest) = selector.strip_prefix('.') {
            selectors.push(Selector::Class(rest.to_string()));
        } else if let Some(rest) = selector.strip_prefix('#') {
            selectors.push(Selector::Id(rest.to_string()));
        } else {
            selectors.push(Selector::Tag(selector.to_string()));
        }
    }
    selectors
}

pub fn parse(css: &str) -> Vec<Rule> {
    let mut mode = Mode::Selector;
    let mut rules = Vec::new();
    let mut selector = String::new();
    let mut property = String::new();
    let mut value = String::new();
    let mut declarations: HashMap<String, String> = HashMap::new();

    for c in css.chars() {
        match mode {
            Mode::Selector => {
                if c == '{' {
                    mode = Mode::Property;
                } else {
                    selector.push(c);
                }
            }
            Mode::Property => {
                if c == ':' {
                    mode = Mode::Value;
                } else if c == '}' {

                    rules.push(Rule { selector: parse_selectors(&selector), declarations: std::mem::take(&mut declarations) });
                    selector.clear();
                    mode = Mode::Selector;
                } else {
                    property.push(c);
                }
            }
            Mode::Value => {
                if c == ';' {
                    declarations.insert(property.trim().to_string(), value.trim().to_string());
                    property.clear();
                    value.clear();
                    mode = Mode::Property;
                } else if c == '}' {
                    declarations.insert(property.trim().to_string(), value.trim().to_string());
                    rules.push(Rule { selector: parse_selectors(&selector), declarations: std::mem::take(&mut declarations) });
                    property.clear();
                    value.clear();
                    selector.clear();
                    mode = Mode::Selector;
                } else {
                    value.push(c);
                }
            }
        }
    }

    rules
}
