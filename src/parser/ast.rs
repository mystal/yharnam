#[derive(Debug)]
pub struct Dialogue {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    pub headers: Vec<Header>,
    pub body: Body,
}

#[derive(Debug)]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Body {
    pub statements: Vec<String>,
}
