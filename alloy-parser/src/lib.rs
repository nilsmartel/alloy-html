pub struct Node {
    kind: String,
    idsAndClasses: Vec<IdOrClass>,
    attributes: Vec<Attribute>,
    body: Vec<Node>,
}

pub enum IdOrClass {
    Id(String),
    Class(String),
}

pub struct Attribute {
    key: String,
    value: String,
}
