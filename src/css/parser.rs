use std::collections::HashMap;

#[derive(Debug)]
pub struct Rule {
    pub selector: String,
    pub declarations: HashMap<String, String>,
}

enum Mode {
    Selector,
    Property,
    Value,
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
                    rules.push(Rule { selector: selector.trim().to_string(), declarations: std::mem::take(&mut declarations) });
                    selector.clear();
                    mode = Mode::Selector;
                } else {
                    property.push(c);
                }
            }
            Mode::Value => {
                /* TODO: ';' → store declaration, Property; '}' → store both, Selector; else collect */
                if c == ';' {
                    declarations.insert(property.trim().to_string(), value.trim().to_string());
                    property.clear();
                    value.clear();
                    mode = Mode::Property;
                } else if c == '}' {
                    declarations.insert(property.trim().to_string(), value.trim().to_string());
                    rules.push(Rule { selector: selector.trim().to_string(), declarations: std::mem::take(&mut declarations) });
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
