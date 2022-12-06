use crate::{Node, Parser};
use nom::{branch::alt, combinator::map};

/// Represents an element in the DOM tree.
/// Might be an (Html)Node, might be some Text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    Node(Node),
    Text(String),
}

impl Default for Element {
    fn default() -> Self {
        Element::Node(Node::default())
    }
}

impl Parser for Element {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        alt((
            map(Node::parse, Element::Node),
            map(String::parse, Element::Text),
        ))(input)
    }
}
