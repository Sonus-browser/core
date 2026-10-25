use super::tokenizer::Token;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Node {
    Element {
        name: String,
        attributes: HashMap<String, String>,
        children: Vec<Node>,
    },
    Text(String),
}

fn is_void(name: &str) -> bool {
    matches!(
        name,
        "meta"
            | "link"
            | "br"
            | "img"
            | "input"
            | "hr"
            | "area"
            | "base"
            | "col"
            | "embed"
            | "source"
            | "track"
            | "wbr"
    )
}

fn add_child(stack: &mut Vec<Node>, child: Node) {
    if let Some(Node::Element { children, .. }) = stack.last_mut() {
        children.push(child);
    }
}

fn close_top(stack: &mut Vec<Node>) {
    let finished = stack.pop().unwrap();
    add_child(stack, finished);
}

pub fn build(tokens: Vec<Token>) -> Node {
    // start with an invisible root, so there's always something "below"
    let mut stack = vec![Node::Element {
        name: "document".to_string(),
        attributes: HashMap::new(),
        children: Vec::new(),
    }];

    for token in tokens {
        match token {
            Token::Opening { name, attributes } => {
                if !is_void(&name) {
                    stack.push(Node::Element {
                        name,
                        attributes,
                        children: Vec::new(),
                    });
                } else {
                    add_child(
                        &mut stack,
                        Node::Element {
                            name,
                            attributes,
                            children: Vec::new(),
                        },
                    );
                }
            }
            Token::Text(text) => add_child(&mut stack, Node::Text(text)),
            Token::Closing(_) => {
                if stack.len() > 1 {
                    close_top(&mut stack);
                }
            }
        }
    }
    while stack.len() > 1 {
        close_top(&mut stack);
    }

    stack.pop().unwrap()
}

pub fn collect_text(node: &Node, out: &mut String) {
    match node {
        Node::Text(text) => {
            out.push_str(text);
            out.push(' ');
        }
        Node::Element {
            name,
            attributes,
            children,
        } => {
            if matches!(name.as_str(), "head" | "script" | "style") {
                return;
            }
            for child in children {
                collect_text(child, out);
            }
        }
    }
}

pub fn collect_css(node: &Node, out: &mut String) {
    match node {
        Node::Text(_) => {}
        Node::Element {
            name,
            children,
            ..
        } => {
            if name == "style" {
                for child in children {
                    collect_text(child, out);
                }
            } else {
                for child in children {
                    collect_css(child, out);
                }
            }
        }
    }
}
