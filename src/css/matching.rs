use super::parser::Selector;
use crate::html::tree::Node;

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

