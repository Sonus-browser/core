#[derive(Debug)]
pub enum Token {
    Opening(String),
    Closing(String),
    Text(String),
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
                   tokens.push(Token::Text(std::mem::take(&mut text)));
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
                    tokens.push(Token::Opening(name.to_string()));
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
        tokens.push(Token::Text(std::mem::take(&mut text)));
    }

    tokens
}
