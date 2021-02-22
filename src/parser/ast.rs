#[derive(Debug)]
pub struct Dialogue<'input> {
    pub nodes: Vec<Node<'input>>,
}

#[derive(Debug)]
pub struct Node<'input> {
    pub headers: Vec<Header<'input>>,
    pub body: Body,
}

#[derive(Debug)]
pub struct Header<'input> {
    pub key: &'input str,
    pub value: &'input str,
}

#[derive(Debug)]
pub struct Body {
    pub statements: Vec<String>,
}

impl Body {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}
