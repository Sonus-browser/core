use super::parser::{Rule, Selector};
use crate::html::tree::Node;
use std::collections::HashMap;

pub fn is_inherited(property: &str) -> bool {
    matches!(
        property,
        "color"
        | "font"
        | "font-family"
        | "font-size"
        | "font-style"
        | "font-weight"
        | "line-height"
        | "text-align"
        | "visibility"
        | "white-space"
        | "color-scheme"
    )
}

pub fn matches(selector: &Selector, node: &Node) -> bool {
    let Node::Element { name, attributes, .. } = node else {
        return false;   // it's a Text node: never matches
    };

    match selector {
        Selector::Tag(tag) => {
            name == tag
        }
        Selector::Class(class) => attributes
            .get("class")
            .is_some_and(|classes| classes.split_whitespace().any(|c| c == class)),
        Selector::Id(id) => attributes.get("id") == Some(id)
    }
}

fn specificity(selector: &Selector) -> u32 {
    match selector {
        Selector::Tag(..) => 1,
        Selector::Class(..) => 10,
        Selector::Id(..) => 100,
    }
}

pub fn apply_styles(node: &mut Node, rules: &[Rule], parent_styles: &HashMap<String, String>) {
    if let Node::Text(_) = node {
        return;
    }

    let mut matching: Vec<(u32, &Rule)> = Vec::new();

    for rule in rules {
        let mut best: Option<u32> = None;

        for selector in &rule.selector {
            if matches(selector, node) {
                let value = specificity(selector);
                if Some(value) > best {
                    best = Some(value);
                }

            }
        }

        if let Some(value) = best {
            matching.push((value, rule));
        }
    }

    matching.sort_by_key(|(spec, _)| *spec);

    if let Node::Element { styles, children, .. } = node {
        for (property, value) in parent_styles {
            if is_inherited(property) {
                styles.insert(property.clone(), value.clone());
            }
        }
        for (_, rule) in &matching {
            for (property, value) in &rule.declarations {
                styles.insert(property.clone(), value.clone());
            }
        }

        for child in children {
            apply_styles(child, rules, styles);
        }
    }
}
