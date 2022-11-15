use nom::{multi::many0, combinator::{map, opt}, branch::alt, sequence::terminated};

use crate::{ Element, Parser, keywords::*, Node };


pub type Body = Vec<Element>;

impl Parser for Body {
    fn parse(input: &str) -> nom::IResult<&str, Self> {

        // { ... }
        fn parse_block(input: &str) -> nom::IResult<&str, Body> {
            let (input, _) = KeywordCurlyOpen::parse(input)?;

            let (input, nodes) = many0(terminated(
                Element::parse_trim,
                opt(KeywordComma::parse_trim),
            ))(input)?;

            let (input, _) = KeywordCurlyClose::parse_trim(input)?;

            Ok((input, nodes))
        }
        
        // div
        // "abcdefg"
        // ;
        // {}
        alt((
            // { ... }
            parse_block,
            // ;
            map(KeywordNone::parse, |_| 
                 // create new empty vector
                 Body::new()),
            // "hello"
            // the same as { "hello" }
            map(String::parse, |s| {
                vec![Element::Text(s)]
            }),

            // div
            // e.g. directly a node as first child.
            map(Node::parse, |n| vec![Element::Node(n)]),
        ))(input)
    }

    fn parse_trim(input: &str) -> nom::IResult<&str, Self> {
        let input = input.trim_start();

        // cut out commments
        if input.starts_with("//") {
            if let Some(index) = input.find('\n') {
                let index = index + 1;
                return Self::parse_trim(&input[index..]);
            }
            return Self::parse_trim("");
        }
        /* cut out comments */
        if input.starts_with("/*") {
            if let Some(index) = input.find("*/") {
                let index = index + 2;
                return Self::parse_trim(&input[index..]);
            }
            // It's allowed to simply cut off all remaining content without closing */
            return Self::parse_trim("");
        }

        Self::parse(input)
    }

    fn from_s(s: &str) -> Self {
        Self::parse_trim(s).unwrap().1
    }
}
