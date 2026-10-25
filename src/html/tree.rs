use super::tokenizer::Token;

#[derive(Debug)]
pub enum Node {
    Element { name: String, children: Vec<Node> },
    Text(String),
}

fn add_child(stack: &mut Vec<Node>, child: Node) {
    if let Some(Node::Element { children, .. }) = stack.last_mut() {
        children.push(child);
    }
}

pub fn build(tokens: Vec<Token>) -> Node {
    // start with an invisible root, so there's always something "below"
    let mut stack = vec![Node::Element { name: "document".to_string(), children: Vec::new() }];

    for token in tokens {
        match token {
            Token::Opening(name) => {
                stack.push(Node::Element { name, children: Vec::new() });
            }
            Token::Text(text) => {
                add_child(&mut stack, Node::Text(text))
            }
            Token::Closing(_) => {
                let finished = stack.pop().unwrap();
                add_child(&mut stack, finished);
            }
        }
    }

    stack.pop().unwrap()
}
